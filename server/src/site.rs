//! Le site complet, fabriqué d'un coup à chaque rafraîchissement des données.
//!
//! Toutes les pages sont calculées à l'avance et rangées dans une table : répondre à une requête revient à
//! chercher une clé, sans rendu, sans allocation ni calcul. C'est ce qui rend le service léger et prévisible.

use crate::{
    catalogue::{ENTREES, GENERIQUE},
    config::Config,
    dates,
    model::{Etat, Resume, VERSION_SCHEMA},
    render::{self, Meta, Rendu, accueil, esc, projet, seo},
};
use std::collections::HashMap;

const STYLE: &str = include_str!("../assets/style.css");
/// Les modules JavaScript de la carte, servis sous `/assets/js/<nom>.js`. L'ordre n'a pas d'importance.
const MODULES: [(&str, &str); 9] = [
    ("acteurs", include_str!("../assets/js/acteurs.js")),
    ("batiments", include_str!("../assets/js/batiments.js")),
    ("camera", include_str!("../assets/js/camera.js")),
    ("carte", include_str!("../assets/js/carte.js")),
    ("donnees", include_str!("../assets/js/donnees.js")),
    ("interface", include_str!("../assets/js/interface.js")),
    ("iso", include_str!("../assets/js/iso.js")),
    ("monde", include_str!("../assets/js/monde.js")),
    ("vignettes", include_str!("../assets/js/vignettes.js")),
];
const SANS_JS: &str = include_str!("../assets/sansjs.css");
const FAVICON: &str = include_str!("../assets/favicon.svg");
const IMAGE_PARTAGE: &[u8] = include_bytes!("../assets/og.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cache {
    /// Pages et données : fraîches une minute, puis revalidées.
    Court,
    /// Ressources versionnées dans l'URL (`?v=…`) : un an. Sans version : une heure.
    Ressource,
    /// Icône : un jour.
    Jour,
}

pub struct Page {
    pub statut: u16,
    pub type_mime: &'static str,
    pub corps: Vec<u8>,
    pub etag: String,
    pub cache: Cache,
}

pub struct Site {
    pages: HashMap<String, Page>,
    introuvable: Page,
    /// Vrai quand aucune donnée n'a pu être lue : le service répond 503 sur la page d'accueil.
    pub degrade: bool,
    /// Nombre de projets affichés.
    pub projets: usize,
    /// Vrai quand certaines données n'ont pas pu être lues (limite de GitHub, dépôt illisible…).
    pub partiel: bool,
}

/// Combien de rafraîchissements partiels d'affilée on refuse avant de les accepter quand même.
const PARTIELS_TOLERES: u32 = 3;

/// Faut-il remplacer le site courant par le nouveau ?
///
/// Un rafraîchissement **partiel** (un dépôt illisible, la limite de GitHub atteinte) ferait disparaître les alertes
/// des projets qu'on n'a pas pu lire : un tableau de bord qui affiche « rien à signaler » faute de données ment.
/// On garde donc le site précédent, s'il était complet, pendant quelques rafraîchissements ; au-delà, le problème
/// semble durable (un dépôt toujours illisible) et on accepte l'état partiel, qui dit ce qui manque.
pub fn faut_remplacer(actuel: &Site, nouveau: &Site, partiels_d_affilee: &mut u32) -> bool {
    if !nouveau.partiel {
        *partiels_d_affilee = 0;
        return true;
    }
    if actuel.degrade || actuel.partiel {
        return true; // rien de meilleur à garder
    }
    *partiels_d_affilee += 1;
    *partiels_d_affilee > PARTIELS_TOLERES
}

/// Ajoute la version à chaque import d'un module (`from './iso.js'` devient `from './iso.js?v=…'`).
///
/// Tous les modules d'une même version portent la même empreinte : une mise à jour change toutes les adresses d'un coup,
/// donc un navigateur ne mélange jamais un module ancien avec un nouveau, et le cache peut durer un an.
fn versionner(source: &str, version: &str) -> String {
    let mut sortie = String::with_capacity(source.len() + 256);
    let mut reste = source;
    while let Some(debut) = reste.find("from './") {
        let apres = debut + "from './".len();
        sortie.push_str(&reste[..apres]);
        reste = &reste[apres..];
        match reste.find('\'') {
            Some(fin) => {
                sortie.push_str(&reste[..fin]);
                sortie.push_str("?v=");
                sortie.push_str(version);
                reste = &reste[fin..];
            }
            None => break,
        }
    }
    sortie.push_str(reste);
    sortie
}

/// Empreinte FNV-1a 64 bits : sert d'`ETag` et de version de ressource. Pas de cryptographie ici.
fn empreinte(octets: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for o in octets {
        h ^= u64::from(*o);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn page(statut: u16, type_mime: &'static str, corps: impl Into<Vec<u8>>, cache: Cache) -> Page {
    let corps = corps.into();
    let etag = format!("\"{}\"", empreinte(&corps));
    Page {
        statut,
        type_mime,
        corps,
        etag,
        cache,
    }
}

const HTML: &str = "text/html; charset=utf-8";
const TEXTE: &str = "text/plain; charset=utf-8";

/// Feuille de style : le fichier écrit à la main, plus une règle de couleur par bâtiment (voir `catalogue.rs`).
fn feuille_de_style() -> String {
    let mut css = String::from(STYLE);
    for e in ENTREES.iter().chain([&GENERIQUE]) {
        let slug = if e.slug.is_empty() {
            "generique"
        } else {
            e.slug
        };
        css.push_str(&format!(".v-{slug}{{--ac:{}}}\n", e.accent));
    }
    css
}

impl Site {
    pub fn construire(etat: &Etat, config: &Config, maintenant: i64) -> Site {
        let css = feuille_de_style();
        let (version_css, version_js) = (
            empreinte(css.as_bytes())[..8].to_string(),
            empreinte(
                MODULES
                    .iter()
                    .map(|(_, s)| *s)
                    .collect::<String>()
                    .as_bytes(),
            )[..8]
                .to_string(),
        );
        let r = Rendu {
            config,
            etat,
            version_css: &version_css,
            version_js: &version_js,
        };

        let mut pages = HashMap::new();
        let mut ajouter = |chemin: &str, p: Page| {
            pages.insert(chemin.to_string(), p);
        };

        ajouter("/", page(200, HTML, accueil::page(&r), Cache::Court));
        for p in &etat.projets {
            ajouter(
                &format!("/projets/{}", p.slug),
                page(200, HTML, projet::page(&r, p), Cache::Court),
            );
            let fragment = projet::fiche(p, 2);
            ajouter(
                &format!("/projets/{}\u{0}partiel", p.slug),
                page(200, HTML, fragment, Cache::Court),
            );
        }
        ajouter(
            "/api/etat.json",
            page(
                200,
                "application/json; charset=utf-8",
                serde_json::to_vec(etat).unwrap_or_default(),
                Cache::Court,
            ),
        );
        ajouter(
            "/sitemap.xml",
            page(
                200,
                "application/xml; charset=utf-8",
                seo::plan_du_site(&r),
                Cache::Court,
            ),
        );
        ajouter(
            "/robots.txt",
            page(200, TEXTE, seo::robots(&r), Cache::Jour),
        );
        ajouter(
            "/llms.txt",
            page(
                200,
                "text/markdown; charset=utf-8",
                seo::llms(&r),
                Cache::Court,
            ),
        );
        ajouter(
            "/.well-known/security.txt",
            page(200, TEXTE, seo::security_txt(&r, maintenant), Cache::Jour),
        );
        let sante = format!(
            "{{\"ok\":true,\"genere_le\":\"{}\",\"projets\":{}}}",
            esc(&etat.genere_le),
            etat.resume.projets
        );
        ajouter(
            "/sante",
            page(200, "application/json; charset=utf-8", sante, Cache::Court),
        );

        ajouter(
            "/assets/style.css",
            page(200, "text/css; charset=utf-8", css, Cache::Ressource),
        );
        for (nom, source) in MODULES {
            ajouter(
                &format!("/assets/js/{nom}.js"),
                page(
                    200,
                    "text/javascript; charset=utf-8",
                    versionner(source, &version_js),
                    Cache::Ressource,
                ),
            );
        }
        ajouter(
            "/assets/sansjs.css",
            page(200, "text/css; charset=utf-8", SANS_JS, Cache::Ressource),
        );
        ajouter(
            "/api/carte.json",
            page(
                200,
                "application/json; charset=utf-8",
                render::carte::json(etat, config.endormi_jours),
                Cache::Court,
            ),
        );
        ajouter(
            "/assets/og.png",
            page(200, "image/png", IMAGE_PARTAGE, Cache::Jour),
        );
        ajouter(
            "/favicon.svg",
            page(200, "image/svg+xml", FAVICON, Cache::Jour),
        );

        let corps_404 = "<main id=\"contenu\" class=\"vide\"><h1>Page introuvable</h1><p>Cette page n'existe pas. <a href=\"/\">Retour à la ville</a>.</p></main>";
        let meta = Meta {
            titre: "Page introuvable · Abvigie",
            description: "Page introuvable.",
            chemin: "/",
            json_ld: "",
            indexable: false,
            entree: "vignettes",
            classe: "fiche-page",
        };
        let introuvable = page(
            404,
            HTML,
            render::document(&r, &meta, corps_404),
            Cache::Court,
        );

        Site {
            pages,
            introuvable,
            degrade: false,
            projets: etat.projets.len(),
            partiel: !etat.avertissements.is_empty(),
        }
    }

    /// Un site sans données : la page d'accueil explique la situation et répond 503. Le service reste en vie
    /// et se rétablit tout seul au prochain rafraîchissement réussi.
    pub fn degrade(config: &Config, raison: &str, maintenant: i64) -> Site {
        let etat = Etat {
            schema: VERSION_SCHEMA,
            genere_le: dates::iso(maintenant),
            organisation: config.organisation.clone(),
            resume: Resume::default(),
            avertissements: vec![format!(
                "Les données de GitHub ne sont pas encore disponibles ({raison})"
            )],
            progression: crate::model::Progression::vide(),
            acteurs: Vec::new(),
            projets: Vec::new(),
        };
        let mut site = Site::construire(&etat, config, maintenant);
        if let Some(accueil) = site.pages.get_mut("/") {
            accueil.statut = 503;
        }
        if let Some(sante) = site.pages.get_mut("/sante") {
            sante.statut = 503;
        }
        site.degrade = true;
        site.partiel = true;
        site
    }

    /// La page d'un chemin, ou la page 404. `partiel` demande le fragment du panneau latéral.
    pub fn chercher(&self, chemin: &str, partiel: bool) -> &Page {
        let cle = if partiel {
            format!("{chemin}\u{0}partiel")
        } else {
            chemin.to_string()
        };
        self.pages.get(&cle).unwrap_or(&self.introuvable)
    }

    #[cfg(test)]
    pub fn chemins(&self) -> Vec<&str> {
        self.pages.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_import_recoit_la_version() {
        let source = "import { a } from './iso.js';\nimport { b, c } from './monde.js';\nconst x = 'pas un import';\n";
        let v = versionner(source, "abc123");
        assert!(v.contains("from './iso.js?v=abc123';"));
        assert!(v.contains("from './monde.js?v=abc123';"));
        assert!(v.contains("const x = 'pas un import';"));
        assert_eq!(versionner("rien à faire", "v"), "rien à faire");
    }

    fn site(partiel: bool, degrade: bool) -> Site {
        let mut s = Site::construire(
            &Etat {
                schema: VERSION_SCHEMA,
                genere_le: "2026-11-06T12:00:00Z".into(),
                organisation: "x".into(),
                resume: Resume::default(),
                avertissements: if partiel {
                    vec!["x".into()]
                } else {
                    Vec::new()
                },
                progression: crate::model::Progression::vide(),
                acteurs: Vec::new(),
                projets: Vec::new(),
            },
            &Config::default(),
            0,
        );
        s.degrade = degrade;
        s
    }

    #[test]
    fn un_site_complet_remplace_toujours_l_ancien() {
        let mut n = 2;
        assert!(faut_remplacer(
            &site(false, false),
            &site(false, false),
            &mut n
        ));
        assert_eq!(n, 0, "le compteur repart à zéro");
    }

    #[test]
    fn un_rafraichissement_partiel_ne_remplace_pas_un_site_complet_tout_de_suite() {
        let (complet, partiel) = (site(false, false), site(true, false));
        let mut n = 0;
        for _ in 0..PARTIELS_TOLERES {
            assert!(!faut_remplacer(&complet, &partiel, &mut n));
        }
        // le problème dure : on finit par accepter l'état partiel, qui dit ce qui manque
        assert!(faut_remplacer(&complet, &partiel, &mut n));
    }

    #[test]
    fn on_accepte_du_partiel_quand_il_n_y_a_rien_de_mieux() {
        let mut n = 0;
        assert!(faut_remplacer(
            &site(true, false),
            &site(true, false),
            &mut n
        ));
        assert!(faut_remplacer(
            &site(true, true),
            &site(true, false),
            &mut n
        ));
    }
}
