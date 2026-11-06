//! Un serveur HTTP/1.1 minimal et strict, volontairement sans dépendance.
//!
//! Il ne sert que des pages fabriquées à l'avance ([`Site`]), donc il peut se permettre d'être très restrictif :
//!
//! - seulement `GET` et `HEAD` ; aucun corps de requête n'est jamais lu (`Content-Length` non nul,
//!   `Transfer-Encoding` et `Expect` sont refusés, ce qui ferme la porte au « request smuggling ») ;
//! - une requête, puis la connexion est fermée (`Connection: close`), sans keep-alive à gérer ;
//! - requête limitée à 8 Kio, délai de lecture par appel et délai total, nombre de connexions limité ;
//! - aucun accès au disque : les chemins sont des clés d'une table, pas des fichiers (rien à traverser).
//!
//! Il est prévu pour tourner derrière un proxy (Traefik, Caddy) qui apporte TLS, compression et limitation de débit.

use crate::{
    dates,
    site::{Cache, Page, Site},
};
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    panic::{self, AssertUnwindSafe},
    sync::{Arc, Mutex, PoisonError, RwLock, mpsc},
    thread,
    time::{Duration, Instant},
};

const TAILLE_MAX_REQUETE: usize = 8 * 1024;
const TAILLE_MAX_CIBLE: usize = 2 * 1024;
const NOMBRE_MAX_EN_TETES: usize = 64;
const DELAI_PAR_LECTURE: Duration = Duration::from_secs(5);
const DELAI_TOTAL: Duration = Duration::from_secs(10);
const DELAI_ECRITURE: Duration = Duration::from_secs(10);
const TRAVAILLEURS: usize = 8;
const FILE_D_ATTENTE: usize = 64;

/// Politique de sécurité du contenu : rien ne vient d'ailleurs, aucun script ni style en ligne.
const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src 'self'; \
                   base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Methode {
    Get,
    Head,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Requete {
    pub methode: Methode,
    pub chemin: String,
    /// `?partial=1` : le fragment du panneau latéral au lieu de la page complète.
    pub partiel: bool,
    /// Une ressource demandée avec `?v=…` peut être mise en cache un an.
    pub versionne: bool,
    pub si_none_match: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Refus {
    /// 400
    Invalide,
    /// 405
    MethodeNonAutorisee,
    /// 431
    TropLong,
}

/// Lit l'en-tête d'une requête (tout ce qui précède la ligne vide) et refuse tout ce qui sort du cadre.
pub fn analyser(tete: &[u8]) -> Result<Requete, Refus> {
    if tete.len() > TAILLE_MAX_REQUETE {
        return Err(Refus::TropLong);
    }
    // ASCII imprimable, plus les retours à la ligne : pas de NUL, pas d'octet de contrôle, pas d'UTF-8 déguisé
    if !tete
        .iter()
        .all(|&o| o == b'\r' || o == b'\n' || (0x20..0x7f).contains(&o) || o == b'\t')
    {
        return Err(Refus::Invalide);
    }
    let texte = std::str::from_utf8(tete).map_err(|_| Refus::Invalide)?;
    let mut lignes = texte.split("\r\n");
    let premiere = lignes.next().ok_or(Refus::Invalide)?;
    if premiere.contains(['\r', '\n']) {
        return Err(Refus::Invalide);
    }

    let mut parties = premiere.split(' ');
    let (methode, cible, version) = match (
        parties.next(),
        parties.next(),
        parties.next(),
        parties.next(),
    ) {
        (Some(m), Some(c), Some(v), None) => (m, c, v),
        _ => return Err(Refus::Invalide),
    };
    if version != "HTTP/1.1" && version != "HTTP/1.0" {
        return Err(Refus::Invalide);
    }
    let methode = match methode {
        "GET" => Methode::Get,
        "HEAD" => Methode::Head,
        m if !m.is_empty() && m.bytes().all(|o| o.is_ascii_uppercase()) => {
            return Err(Refus::MethodeNonAutorisee);
        }
        _ => return Err(Refus::Invalide),
    };
    if cible.len() > TAILLE_MAX_CIBLE {
        return Err(Refus::TropLong);
    }
    if !cible.starts_with('/')
        || cible.starts_with("//")
        || cible.contains('#')
        || cible.bytes().any(|o| !(0x21..0x7f).contains(&o))
    {
        return Err(Refus::Invalide);
    }
    let (chemin, requete) = cible.split_once('?').unwrap_or((cible, ""));

    let mut si_none_match = None;
    let mut compte = 0;
    for ligne in lignes {
        compte += 1;
        if compte > NOMBRE_MAX_EN_TETES
            || ligne.contains(['\r', '\n'])
            || ligne.starts_with([' ', '\t'])
        {
            return Err(Refus::Invalide);
        }
        let (nom, valeur) = ligne.split_once(':').ok_or(Refus::Invalide)?;
        if nom.is_empty() || !nom.bytes().all(|o| o.is_ascii_alphanumeric() || o == b'-') {
            return Err(Refus::Invalide);
        }
        let valeur = valeur.trim();
        match nom.to_ascii_lowercase().as_str() {
            // aucun corps : toute annonce de corps est refusée plutôt que lue
            "content-length" if valeur != "0" => return Err(Refus::Invalide),
            "transfer-encoding" | "expect" | "upgrade" => return Err(Refus::Invalide),
            "if-none-match" if valeur.len() <= 512 => si_none_match = Some(valeur.to_string()),
            _ => {}
        }
    }

    Ok(Requete {
        methode,
        chemin: chemin.to_string(),
        partiel: requete == "partial=1",
        versionne: requete.split('&').any(|p| p.starts_with("v=")),
        si_none_match,
    })
}

/// `If-None-Match` correspond-il à l'`ETag` de la page ? (`*`, listes et préfixe `W/` compris.)
fn correspond(si_none_match: &str, etag: &str) -> bool {
    si_none_match
        .split(',')
        .map(str::trim)
        .any(|c| c == "*" || c.strip_prefix("W/").unwrap_or(c) == etag)
}

fn libelle(statut: u16) -> &'static str {
    match statut {
        200 => "OK",
        304 => "Not Modified",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

fn cache_control(page: &Page, versionne: bool) -> &'static str {
    match (page.cache, versionne) {
        (Cache::Court, _) => "public, max-age=60, stale-while-revalidate=300",
        (Cache::Ressource, true) => "public, max-age=31536000, immutable",
        (Cache::Ressource, false) => "public, max-age=3600",
        (Cache::Jour, _) => "public, max-age=86400",
    }
}

/// Fabrique la réponse complète (en-têtes et corps) à une requête valide.
pub fn repondre(requete: &Requete, site: &Site, maintenant: i64) -> Vec<u8> {
    let page = site.chercher(&requete.chemin, requete.partiel);
    let sans_changement = page.statut == 200
        && requete
            .si_none_match
            .as_deref()
            .is_some_and(|v| correspond(v, &page.etag));
    let statut = if sans_changement { 304 } else { page.statut };

    let mut tete = String::with_capacity(768);
    tete.push_str(&format!("HTTP/1.1 {statut} {}\r\n", libelle(statut)));
    entetes_communs(&mut tete, maintenant);
    if requete.chemin.starts_with("/api/") {
        tete.push_str(
            "Access-Control-Allow-Origin: *\r\nCross-Origin-Resource-Policy: cross-origin\r\n",
        );
    }
    tete.push_str(&format!(
        "ETag: {}\r\nCache-Control: {}\r\n",
        page.etag,
        cache_control(page, requete.versionne)
    ));
    if statut != 304 {
        tete.push_str(&format!(
            "Content-Type: {}\r\nContent-Length: {}\r\n",
            page.type_mime,
            page.corps.len()
        ));
    }
    tete.push_str("\r\n");

    let mut reponse = tete.into_bytes();
    if statut != 304 && requete.methode == Methode::Get {
        reponse.extend_from_slice(&page.corps);
    }
    reponse
}

/// Réponse à une requête refusée.
pub fn refuser(refus: &Refus, maintenant: i64) -> Vec<u8> {
    let (statut, texte) = match refus {
        Refus::Invalide => (400, "Requête invalide.\n"),
        Refus::MethodeNonAutorisee => (405, "Seuls GET et HEAD sont acceptés.\n"),
        Refus::TropLong => (431, "Requête trop longue.\n"),
    };
    erreur_simple(statut, texte, maintenant)
}

fn erreur_simple(statut: u16, texte: &str, maintenant: i64) -> Vec<u8> {
    let mut tete = format!("HTTP/1.1 {statut} {}\r\n", libelle(statut));
    entetes_communs(&mut tete, maintenant);
    if statut == 405 {
        tete.push_str("Allow: GET, HEAD\r\n");
    }
    tete.push_str(&format!(
        "Content-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{texte}",
        texte.len()
    ));
    tete.into_bytes()
}

fn entetes_communs(tete: &mut String, maintenant: i64) {
    tete.push_str(&format!("Date: {}\r\n", dates::http(maintenant)));
    tete.push_str("Connection: close\r\n");
    tete.push_str(&format!("Content-Security-Policy: {CSP}\r\n"));
    tete.push_str("X-Content-Type-Options: nosniff\r\n");
    tete.push_str("X-Frame-Options: DENY\r\n");
    tete.push_str("Referrer-Policy: strict-origin-when-cross-origin\r\n");
    tete.push_str("Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=(), usb=(), interest-cohort=()\r\n");
    tete.push_str("Cross-Origin-Opener-Policy: same-origin\r\n");
    tete.push_str("Strict-Transport-Security: max-age=31536000; includeSubDomains\r\n");
}

enum Echec {
    Refus(Refus),
    Abandon,
}

/// Lit jusqu'à la ligne vide qui termine l'en-tête, sans jamais dépasser la taille ni le délai permis.
fn lire_tete(flux: &mut TcpStream) -> Result<Vec<u8>, Echec> {
    let debut = Instant::now();
    let mut donnees = Vec::with_capacity(1024);
    let mut morceau = [0u8; 1024];
    loop {
        if let Some(pos) = donnees.windows(4).position(|f| f == b"\r\n\r\n") {
            donnees.truncate(pos);
            return Ok(donnees);
        }
        if donnees.len() >= TAILLE_MAX_REQUETE {
            return Err(Echec::Refus(Refus::TropLong));
        }
        if debut.elapsed() > DELAI_TOTAL {
            return Err(Echec::Abandon);
        }
        match flux.read(&mut morceau) {
            Ok(0) | Err(_) => return Err(Echec::Abandon),
            Ok(n) => donnees.extend_from_slice(&morceau[..n]),
        }
    }
}

fn traiter(mut flux: TcpStream, site: &RwLock<Arc<Site>>) {
    let _ = flux.set_read_timeout(Some(DELAI_PAR_LECTURE));
    let _ = flux.set_write_timeout(Some(DELAI_ECRITURE));
    let _ = flux.set_nodelay(true);

    let maintenant = dates::maintenant();
    let reponse = panic::catch_unwind(AssertUnwindSafe(|| match lire_tete(&mut flux) {
        Ok(tete) => match analyser(&tete) {
            Ok(requete) => {
                let site = Arc::clone(&site.read().unwrap_or_else(PoisonError::into_inner));
                Some(repondre(&requete, &site, maintenant))
            }
            Err(refus) => Some(refuser(&refus, maintenant)),
        },
        Err(Echec::Refus(refus)) => Some(refuser(&refus, maintenant)),
        Err(Echec::Abandon) => None,
    }));
    let reponse = match reponse {
        Ok(Some(r)) => r,
        Ok(None) => return,
        Err(_) => {
            eprintln!(
                "{} erreur interne pendant une requête (ignorée)",
                dates::iso(maintenant)
            );
            erreur_simple(500, "Erreur interne.\n", maintenant)
        }
    };
    let _ = flux.write_all(&reponse);
    let _ = flux.flush();
    let _ = flux.shutdown(std::net::Shutdown::Both);
}

/// Écoute et sert, pour toujours. Un nombre fixe de travailleurs lit une file bornée : au-delà, les connexions
/// sont refusées plutôt que d'enfler la mémoire.
pub fn servir(ecouteur: TcpListener, site: Arc<RwLock<Arc<Site>>>) -> io::Result<()> {
    let (envoi, reception) = mpsc::sync_channel::<TcpStream>(FILE_D_ATTENTE);
    let reception = Arc::new(Mutex::new(reception));
    for _ in 0..TRAVAILLEURS {
        let (reception, site) = (Arc::clone(&reception), Arc::clone(&site));
        thread::Builder::new()
            .name("travailleur".into())
            .spawn(move || {
                loop {
                    let flux = reception
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .recv();
                    match flux {
                        Ok(flux) => traiter(flux, &site),
                        Err(_) => return,
                    }
                }
            })?;
    }
    for flux in ecouteur.incoming() {
        match flux {
            // file pleine : on laisse tomber la connexion (le proxy en amont réessaiera)
            Ok(flux) => drop(envoi.try_send(flux)),
            Err(e) => eprintln!(
                "{} connexion refusée par le système : {e}",
                dates::iso(dates::maintenant())
            ),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collect, config::Config, sources::fixtures::Fixtures};
    use std::{net::TcpListener, path::PathBuf};

    fn requete(texte: &str) -> Result<Requete, Refus> {
        analyser(texte.as_bytes())
    }

    #[test]
    fn accepte_un_get_simple() {
        let r =
            requete("GET /projets/abcom?partial=1 HTTP/1.1\r\nHost: x\r\nIf-None-Match: \"abc\"")
                .unwrap();
        assert_eq!(r.methode, Methode::Get);
        assert_eq!(r.chemin, "/projets/abcom");
        assert!(r.partiel && !r.versionne);
        assert_eq!(r.si_none_match.as_deref(), Some("\"abc\""));
        assert!(
            requete("GET /assets/app.js?v=1a2b HTTP/1.0")
                .unwrap()
                .versionne
        );
        assert_eq!(requete("HEAD / HTTP/1.1").unwrap().methode, Methode::Head);
    }

    #[test]
    fn refuse_les_methodes_qui_ecrivent() {
        for m in [
            "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "TRACE", "CONNECT",
        ] {
            assert_eq!(
                requete(&format!("{m} / HTTP/1.1")),
                Err(Refus::MethodeNonAutorisee),
                "{m}"
            );
        }
        assert_eq!(requete("get / HTTP/1.1"), Err(Refus::Invalide));
    }

    #[test]
    fn refuse_tout_ce_qui_annonce_un_corps_request_smuggling() {
        assert_eq!(
            requete("GET / HTTP/1.1\r\nContent-Length: 5"),
            Err(Refus::Invalide)
        );
        assert_eq!(
            requete("GET / HTTP/1.1\r\nTransfer-Encoding: chunked"),
            Err(Refus::Invalide)
        );
        assert_eq!(
            requete("GET / HTTP/1.1\r\ntransfer-encoding : chunked"),
            Err(Refus::Invalide)
        );
        assert_eq!(
            requete("GET / HTTP/1.1\r\nExpect: 100-continue"),
            Err(Refus::Invalide)
        );
        assert_eq!(
            requete("GET / HTTP/1.1\r\nUpgrade: websocket"),
            Err(Refus::Invalide)
        );
        assert!(requete("GET / HTTP/1.1\r\nContent-Length: 0").is_ok());
    }

    #[test]
    fn refuse_les_requetes_mal_formees() {
        for mauvais in [
            "",
            "GET",
            "GET /",
            "GET / HTTP/2",
            "GET / HTTP/1.1 extra",
            "GET  / HTTP/1.1",
            "GET /a b HTTP/1.1",
            "GET //evil.com/x HTTP/1.1",
            "GET /#frag HTTP/1.1",
            "GET http://x/ HTTP/1.1",
            "GET / HTTP/1.1\r\nSansDeuxPoints",
            "GET / HTTP/1.1\r\n suite-de-ligne: x",
            "GET / HTTP/1.1\r\nBad Name: x",
            "GET /\u{0}x HTTP/1.1",
            "GET /é HTTP/1.1",
            "GET / HTTP/1.1\nHost: x",
        ] {
            assert!(
                requete(mauvais).is_err(),
                "devrait être refusé : {mauvais:?}"
            );
        }
    }

    #[test]
    fn limite_la_taille_de_la_cible_et_des_en_tetes() {
        let longue = format!("GET /{} HTTP/1.1", "a".repeat(TAILLE_MAX_CIBLE));
        assert_eq!(requete(&longue), Err(Refus::TropLong));
        let trop = format!(
            "GET / HTTP/1.1{}",
            "\r\nX: y".repeat(NOMBRE_MAX_EN_TETES + 1)
        );
        assert_eq!(requete(&trop), Err(Refus::Invalide));
    }

    #[test]
    fn if_none_match_gere_les_listes_et_les_validateurs_faibles() {
        assert!(correspond("\"a\"", "\"a\""));
        assert!(correspond("W/\"a\"", "\"a\""));
        assert!(correspond("\"b\", \"a\"", "\"a\""));
        assert!(correspond("*", "\"a\""));
        assert!(!correspond("\"b\"", "\"a\""));
    }

    fn site_de_test() -> Site {
        let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
        let config = Config::default();
        let maintenant = dates::analyser("2026-11-06T12:00:00Z").unwrap();
        let etat = collect::construire(&Fixtures::new(dossier), &config, maintenant)
            .expect("fixtures lisibles");
        Site::construire(&etat, &config, maintenant)
    }

    fn envoyer(adresse: std::net::SocketAddr, texte: &[u8]) -> String {
        let mut flux = TcpStream::connect(adresse).unwrap();
        flux.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        flux.write_all(texte).unwrap();
        let mut reponse = Vec::new();
        let _ = flux.read_to_end(&mut reponse);
        String::from_utf8_lossy(&reponse).into_owned()
    }

    #[test]
    fn serveur_complet_de_bout_en_bout() {
        let ecouteur = TcpListener::bind("127.0.0.1:0").unwrap();
        let adresse = ecouteur.local_addr().unwrap();
        let site = Arc::new(RwLock::new(Arc::new(site_de_test())));
        thread::spawn(move || {
            let _ = servir(ecouteur, site);
        });

        let accueil = envoyer(adresse, b"GET / HTTP/1.1\r\nHost: x\r\n\r\n");
        assert!(
            accueil.starts_with("HTTP/1.1 200 OK\r\n"),
            "{}",
            &accueil[..80.min(accueil.len())]
        );
        assert!(accueil.contains("Content-Security-Policy: default-src 'none'"));
        assert!(accueil.contains("X-Content-Type-Options: nosniff"));
        assert!(accueil.contains("<title>Abvigie"));
        let etag = accueil
            .lines()
            .find_map(|l| l.strip_prefix("ETag: "))
            .expect("ETag")
            .to_string();

        let sans_changement = envoyer(
            adresse,
            format!("GET / HTTP/1.1\r\nIf-None-Match: {etag}\r\n\r\n").as_bytes(),
        );
        assert!(sans_changement.starts_with("HTTP/1.1 304 Not Modified\r\n"));
        assert!(!sans_changement.contains("<html"));

        let tete = envoyer(adresse, b"HEAD / HTTP/1.1\r\n\r\n");
        assert!(tete.starts_with("HTTP/1.1 200 OK") && !tete.contains("<html"));

        assert!(
            envoyer(adresse, b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\nabcd")
                .starts_with("HTTP/1.1 405")
        );
        assert!(
            envoyer(adresse, b"GET / HTTP/1.1\r\nContent-Length: 4\r\n\r\nabcd")
                .starts_with("HTTP/1.1 400")
        );
        assert!(
            envoyer(adresse, b"GET /n-existe-pas HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 404")
        );
        assert!(
            envoyer(adresse, b"GET /../../etc/passwd HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 404")
        );
        assert!(envoyer(adresse, b"GET /sante HTTP/1.1\r\n\r\n").contains("\"ok\":true"));
        assert!(
            envoyer(adresse, b"GET /api/etat.json HTTP/1.1\r\n\r\n")
                .contains("Access-Control-Allow-Origin: *")
        );
        // une requête qui n'en finit pas est abandonnée sans réponse ni blocage
        let gros = format!(
            "GET / HTTP/1.1\r\nX: {}\r\n\r\n",
            "a".repeat(TAILLE_MAX_REQUETE)
        );
        assert!(envoyer(adresse, gros.as_bytes()).starts_with("HTTP/1.1 431"));
    }

    /// Le budget de poids de `docs/performance.md`. La page grandit avec le nombre de projets (environ 1,5 Ko chacun).
    #[test]
    fn la_page_reste_dans_son_budget_de_poids() {
        let site = site_de_test();
        let taille = |chemin: &str| site.chercher(chemin, false).corps.len();
        let projets = site.projets;
        assert!(
            taille("/") <= 14_000 + 1_500 * projets,
            "accueil : {} octets pour {projets} projets",
            taille("/")
        );
        assert!(
            taille("/assets/style.css") <= 20_000,
            "style.css : {} octets",
            taille("/assets/style.css")
        );
        let js: usize = site
            .chemins()
            .into_iter()
            .filter(|c| c.starts_with("/assets/js/"))
            .map(taille)
            .sum();
        assert!(js <= 110_000, "JavaScript : {js} octets");
        assert!(
            taille("/api/carte.json") <= 800 * projets + 1_000,
            "carte.json : {} octets",
            taille("/api/carte.json")
        );
        assert!(taille("/favicon.svg") <= 1_000);
        assert!(
            taille("/assets/og.png") <= 120_000,
            "og.png : {} octets",
            taille("/assets/og.png")
        );
    }

    /// Chaque module JavaScript importé existe bel et bien dans ce que le serveur sert : un import cassé ne se verrait
    /// qu'au chargement de la page, dans le navigateur.
    #[test]
    fn tous_les_imports_javascript_sont_servis() {
        let site = site_de_test();
        let chemins = site.chemins();
        let mut vus = 0;
        for c in chemins.iter().filter(|c| c.starts_with("/assets/js/")) {
            let source = String::from_utf8_lossy(&site.chercher(c, false).corps).into_owned();
            for ligne in source
                .lines()
                .filter(|l| l.trim_start().starts_with("import ") || l.contains(" from './"))
            {
                if let Some(debut) = ligne.find("from './") {
                    let nom = &ligne[debut + 8..];
                    let nom = &nom[..nom.find('\'').unwrap_or(nom.len())];
                    let nom = nom.split('?').next().unwrap_or(nom);
                    assert!(ligne.contains("?v="), "{c} : import sans version : {ligne}");
                    assert!(
                        chemins.contains(&format!("/assets/js/{nom}").as_str()),
                        "{c} importe {nom}, qui n'est pas servi"
                    );
                    vus += 1;
                }
            }
        }
        assert!(
            vus >= 10,
            "trop peu d'imports trouvés ({vus}) : le test ne vérifie plus rien"
        );
    }

    #[test]
    fn la_carte_publie_la_progression_et_les_quetes() {
        let site = site_de_test();
        let carte: serde_json::Value =
            serde_json::from_slice(&site.chercher("/api/carte.json", false).corps).unwrap();
        assert_eq!(carte["version"], 1);
        assert!(carte["projets"].as_array().unwrap().len() >= 10);
        assert!(!carte["quetes"].as_array().unwrap().is_empty());
        assert!((1..=7).contains(&carte["niveau"].as_u64().unwrap()));
        // ni jeton ni donnée privée : seulement des champs prévus
        let texte = carte.to_string();
        assert!(!texte.contains("ghp_") && !texte.contains("Bearer"));
    }

    #[test]
    fn aucune_ressource_externe_dans_les_pages() {
        let site = site_de_test();
        for chemin in site.chemins() {
            let page = site.chercher(chemin, false);
            if !page.type_mime.starts_with("text/html") {
                continue;
            }
            let html = String::from_utf8_lossy(&page.corps);
            for interdit in [
                "src=\"http",
                "src='http",
                "@import",
                "<iframe",
                "<object",
                "<embed",
                "<form",
                "onclick=",
                "onerror=",
                "javascript:",
                " style=\"",
            ] {
                assert!(!html.contains(interdit), "{chemin} contient {interdit:?}");
            }
            assert!(
                !html.contains("rel=\"stylesheet\" href=\"http"),
                "{chemin} charge une feuille de style externe"
            );
        }
    }

    #[test]
    fn toutes_les_pages_attendues_existent() {
        let site = site_de_test();
        let chemins = site.chemins();
        for attendu in [
            "/",
            "/sitemap.xml",
            "/robots.txt",
            "/llms.txt",
            "/api/etat.json",
            "/sante",
            "/assets/style.css",
            "/assets/js/carte.js",
            "/assets/js/monde.js",
            "/assets/sansjs.css",
            "/api/carte.json",
            "/favicon.svg",
            "/.well-known/security.txt",
        ] {
            assert!(chemins.contains(&attendu), "page manquante : {attendu}");
        }
        assert!(chemins.iter().any(|c| c.starts_with("/projets/")));
        assert_eq!(site.chercher("/introuvable", false).statut, 404);
    }
}
