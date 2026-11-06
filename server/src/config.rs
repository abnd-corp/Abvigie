//! Configuration, par ordre croissant de priorité : valeurs par défaut, fichier `abvigie.conf`, variables
//! d'environnement, arguments de la ligne de commande.
//!
//! | Fichier | Variable | Défaut | Rôle |
//! |---|---|---|---|
//! | `organisation` | `ABVIGIE_ORG` | `abnd-corp` | Organisation GitHub à observer |
//! | `adresse` | `ABVIGIE_ADRESSE` | `127.0.0.1:8080` | Adresse d'écoute |
//! | `url` | `ABVIGIE_URL` | `http://localhost:8080` | URL publique (canonical, sitemap, JSON-LD) |
//! | `rafraichir` | `ABVIGIE_RAFRAICHIR` | `6h` | Délai entre deux lectures de GitHub (`90s`, `15min`, `6h`, `1d`) |
//! | `endormi_jours` | `ABVIGIE_ENDORMI_JOURS` | `60` | Jours sans activité avant l'extinction des lumières |
//! | `revue_jours` | `ABVIGIE_REVUE_JOURS` | `180` | Jours sans activité avant la revue de projet dormant (R26) |
//! | (jamais) | `GITHUB_TOKEN` | aucun | Jeton en lecture seule (sans lui : 60 requêtes par heure) |
//!
//! Le jeton n'est **jamais** lu dans le fichier : un secret n'a pas sa place dans un fichier (R18).
//! Le fichier est `--config <fichier>`, sinon `ABVIGIE_CONFIG`, sinon `abvigie.conf` s'il existe.
//! `--fixtures <dossier>` remplace GitHub par des fichiers locaux ; `--export <dossier>` écrit le site statique.

use std::{env, fmt, fs, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    /// Le fichier de configuration lu, s'il y en a un (pour le journal de démarrage).
    pub fichier: Option<PathBuf>,
    pub organisation: String,
    pub adresse: String,
    pub url_publique: String,
    pub rafraichir_secondes: u64,
    pub endormi_jours: i64,
    pub revue_jours: i64,
    pub jeton: Option<Jeton>,
    pub fixtures: Option<PathBuf>,
    /// Mode statique : écrire le site dans ce dossier puis s'arrêter, sans serveur.
    pub exporter: Option<PathBuf>,
}

/// Le jeton GitHub. Son `Debug` ne l'affiche jamais : il ne doit apparaître dans aucun journal (R9, R18).
#[derive(Clone)]
pub struct Jeton(String);

impl Jeton {
    pub fn valeur(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Jeton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Jeton(***)")
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("organisation", &self.organisation)
            .field("adresse", &self.adresse)
            .field("url_publique", &self.url_publique)
            .field("jeton", &self.jeton)
            .field("fixtures", &self.fixtures)
            .field("exporter", &self.exporter)
            .finish_non_exhaustive()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            fichier: None,
            organisation: "abnd-corp".into(),
            adresse: "127.0.0.1:8080".into(),
            url_publique: "http://localhost:8080".into(),
            rafraichir_secondes: 6 * 3600,
            endormi_jours: 60,
            revue_jours: 180,
            jeton: None,
            fixtures: None,
            exporter: None,
        }
    }
}

impl Config {
    /// Lit le fichier, puis l'environnement, puis les arguments de la ligne de commande.
    pub fn depuis_environnement(arguments: &[String]) -> Result<Config, String> {
        let mut c = Config::default();

        // le fichier de configuration : demandé, ou trouvé à côté
        let demande = arguments
            .iter()
            .position(|a| a == "--config")
            .map(|i| {
                arguments
                    .get(i + 1)
                    .map(PathBuf::from)
                    .ok_or("--config attend un fichier")
            })
            .transpose()?
            .or_else(|| texte("ABVIGIE_CONFIG").map(PathBuf::from));
        let fichier = match demande {
            Some(f) => Some(f),
            None => Some(PathBuf::from("abvigie.conf")).filter(|f| f.is_file()),
        };
        if let Some(f) = fichier {
            let contenu = fs::read_to_string(&f)
                .map_err(|e| format!("configuration {} : {e}", f.display()))?;
            for (cle, valeur) in
                lire_fichier(&contenu).map_err(|e| format!("{} : {e}", f.display()))?
            {
                c.appliquer(&cle, &valeur)
                    .map_err(|e| format!("{} : {e}", f.display()))?;
            }
            c.fichier = Some(f);
        }

        for (nom, cle) in [
            ("ABVIGIE_ORG", "organisation"),
            ("ABVIGIE_ADRESSE", "adresse"),
            ("ABVIGIE_URL", "url"),
            ("ABVIGIE_RAFRAICHIR", "rafraichir"),
            ("ABVIGIE_ENDORMI_JOURS", "endormi_jours"),
            ("ABVIGIE_REVUE_JOURS", "revue_jours"),
        ] {
            if let Some(v) = texte(nom) {
                c.appliquer(cle, &v).map_err(|e| format!("{nom} : {e}"))?;
            }
        }
        c.jeton = texte("GITHUB_TOKEN").map(Jeton);

        let mut it = arguments.iter();
        while let Some(a) = it.next() {
            match a.as_str() {
                "--fixtures" => {
                    c.fixtures = Some(PathBuf::from(
                        it.next().ok_or("--fixtures attend un dossier")?,
                    ));
                }
                "--export" => {
                    c.exporter = Some(PathBuf::from(
                        it.next().ok_or("--export attend un dossier")?,
                    ));
                }
                "--config" => {
                    it.next();
                }
                "--adresse" => c.adresse = it.next().ok_or("--adresse attend une valeur")?.clone(),
                autre => return Err(format!("argument inconnu : {autre}")),
            }
        }
        Ok(c)
    }

    /// Applique un réglage venu du fichier ou de l'environnement.
    fn appliquer(&mut self, cle: &str, valeur: &str) -> Result<(), String> {
        match cle {
            "organisation" => self.organisation = valider_organisation(valeur.to_string())?,
            "adresse" => self.adresse = valeur.to_string(),
            "url" => self.url_publique = valeur.trim_end_matches('/').to_string(),
            "rafraichir" => self.rafraichir_secondes = duree(valeur)?.max(60),
            "endormi_jours" => self.endormi_jours = entier(valeur, cle)? as i64,
            "revue_jours" => self.revue_jours = entier(valeur, cle)? as i64,
            "jeton" | "token" | "github_token" => {
                return Err(
                    "le jeton ne se met jamais dans un fichier (R18) : utiliser la variable GITHUB_TOKEN".into(),
                );
            }
            autre => return Err(format!("réglage inconnu : {autre}")),
        }
        Ok(())
    }

    /// Le dossier de l'URL publique : `/Abvigie` pour `https://abnd-corp.github.io/Abvigie`, vide pour un domaine seul.
    pub fn chemin_base(&self) -> String {
        let apres = self
            .url_publique
            .split_once("://")
            .map_or(self.url_publique.as_str(), |(_, reste)| reste);
        match apres.find('/') {
            Some(i) => apres[i..].trim_end_matches('/').to_string(),
            None => String::new(),
        }
    }
}

/// Lit un fichier `cle = valeur` : une ligne par réglage, `#` pour les commentaires (en début de ligne ou après la valeur).
fn lire_fichier(contenu: &str) -> Result<Vec<(String, String)>, String> {
    let mut sortie = Vec::new();
    for (i, ligne) in contenu.lines().enumerate() {
        let sans_commentaire = ligne.split(" #").next().unwrap_or("").trim();
        if sans_commentaire.is_empty() || sans_commentaire.starts_with('#') {
            continue;
        }
        let (cle, valeur) = sans_commentaire
            .split_once('=')
            .ok_or_else(|| format!("ligne {} : « cle = valeur » attendu", i + 1))?;
        let valeur = valeur.trim().trim_matches('"').trim();
        if valeur.is_empty() {
            return Err(format!("ligne {} : valeur vide", i + 1));
        }
        sortie.push((cle.trim().to_string(), valeur.to_string()));
    }
    Ok(sortie)
}

/// Une durée : `90s`, `15min`, `6h`, `1d`, ou un nombre de secondes seul. Renvoie des secondes.
fn duree(texte: &str) -> Result<u64, String> {
    let t = texte.trim();
    let fin = t.find(|c: char| !c.is_ascii_digit()).unwrap_or(t.len());
    let (nombre, unite) = t.split_at(fin);
    let n: u64 = nombre
        .parse()
        .map_err(|_| format!("durée attendue (90s, 15min, 6h, 1d), reçu « {t} »"))?;
    let facteur = match unite.trim() {
        "" | "s" => 1,
        "min" | "m" => 60,
        "h" => 3600,
        "j" | "d" => 86_400,
        autre => {
            return Err(format!(
                "unité de durée inconnue : « {autre} » (s, min, h, j)"
            ));
        }
    };
    n.checked_mul(facteur)
        .ok_or_else(|| "durée trop grande".into())
}

/// « 6 h », « 15 min »… pour le journal.
pub fn duree_lisible(secondes: u64) -> String {
    match secondes {
        s if s % 86_400 == 0 => format!("{} j", s / 86_400),
        s if s % 3_600 == 0 => format!("{} h", s / 3_600),
        s if s % 60 == 0 => format!("{} min", s / 60),
        s => format!("{s} s"),
    }
}

fn entier(valeur: &str, nom: &str) -> Result<u64, String> {
    valeur
        .trim()
        .parse()
        .map_err(|_| format!("{nom} : nombre entier attendu"))
}

/// Un nom d'organisation GitHub : lettres, chiffres et tirets seulement. Il finit dans une URL.
fn valider_organisation(v: String) -> Result<String, String> {
    if !v.is_empty() && v.len() <= 39 && v.bytes().all(|o| o.is_ascii_alphanumeric() || o == b'-') {
        Ok(v)
    } else {
        Err("ABVIGIE_ORG : lettres, chiffres et tirets seulement".into())
    }
}

fn texte(nom: &str) -> Option<String> {
    env::var(nom)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_jeton_n_apparait_jamais_dans_le_debug() {
        let c = Config {
            jeton: Some(Jeton("ghp_secret".into())),
            ..Config::default()
        };
        let affichage = format!("{c:?}");
        assert!(!affichage.contains("ghp_secret"));
        assert!(affichage.contains("***"));
    }

    #[test]
    fn refuse_une_organisation_suspecte() {
        assert!(valider_organisation("abnd-corp".into()).is_ok());
        for mauvais in ["", "a/b", "a b", "a?x=1", "../x", "é"] {
            assert!(valider_organisation(mauvais.into()).is_err(), "{mauvais}");
        }
    }

    #[test]
    fn arguments_inconnus_refuses() {
        assert!(Config::depuis_environnement(&["--nimporte".into()]).is_err());
        let c = Config::depuis_environnement(&["--fixtures".into(), "x".into()]).unwrap();
        assert_eq!(c.fixtures, Some(PathBuf::from("x")));
    }

    #[test]
    fn les_durees_se_lisent_avec_leur_unite() {
        assert_eq!(duree("90s"), Ok(90));
        assert_eq!(duree("15min"), Ok(900));
        assert_eq!(duree("6h"), Ok(21_600));
        assert_eq!(duree("1d"), Ok(86_400));
        assert_eq!(duree("900"), Ok(900), "un nombre seul reste des secondes");
        assert!(duree("six heures").is_err());
        assert!(duree("5 semaines").is_err());
    }

    #[test]
    fn le_fichier_se_lit_avec_ses_commentaires() {
        let lu = lire_fichier("# titre\n\norganisation = abnd-corp\nrafraichir = 15min # pour être à jour\nurl = \"https://x.org\"\n").unwrap();
        assert_eq!(
            lu,
            vec![
                ("organisation".into(), "abnd-corp".into()),
                ("rafraichir".into(), "15min".into()),
                ("url".into(), "https://x.org".into()),
            ]
        );
        assert!(lire_fichier("n'importe quoi").is_err());
        assert!(lire_fichier("url =").is_err());
    }

    #[test]
    fn le_fichier_refuse_un_jeton_et_les_fautes_de_frappe() {
        let mut c = Config::default();
        let e = c.appliquer("jeton", "ghp_x").unwrap_err();
        assert!(e.contains("R18") && !e.contains("ghp_x"));
        assert!(c.appliquer("rafrechir", "6h").is_err());
        c.appliquer("rafraichir", "15min").unwrap();
        assert_eq!(c.rafraichir_secondes, 900);
        c.appliquer("rafraichir", "10s").unwrap();
        assert_eq!(c.rafraichir_secondes, 60, "jamais moins d'une minute");
    }

    #[test]
    fn le_delai_par_defaut_est_de_six_heures() {
        assert_eq!(Config::default().rafraichir_secondes, 21_600);
    }

    #[test]
    fn le_chemin_de_base_vient_de_l_url_publique() {
        let avec = |u: &str| Config {
            url_publique: u.into(),
            ..Config::default()
        };
        assert_eq!(
            avec("https://abnd-corp.github.io/Abvigie").chemin_base(),
            "/Abvigie"
        );
        assert_eq!(avec("https://abvigie.exemple.org").chemin_base(), "");
        assert_eq!(avec("http://localhost:8080").chemin_base(), "");
    }
}
