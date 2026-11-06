//! Configuration : variables d'environnement, avec des valeurs par défaut sûres.
//!
//! | Variable | Défaut | Rôle |
//! |---|---|---|
//! | `ABVIGIE_ORG` | `abnd-corp` | Organisation GitHub à observer |
//! | `ABVIGIE_ADRESSE` | `127.0.0.1:8080` | Adresse d'écoute |
//! | `ABVIGIE_URL` | `http://localhost:8080` | URL publique (canonical, sitemap, JSON-LD) |
//! | `ABVIGIE_RAFRAICHIR` | `900` | Secondes entre deux lectures de GitHub |
//! | `ABVIGIE_ENDORMI_JOURS` | `60` | Jours sans activité avant l'extinction des lumières |
//! | `ABVIGIE_REVUE_JOURS` | `180` | Jours sans activité avant la revue de projet dormant (R26) |
//! | `GITHUB_TOKEN` | aucun | Jeton en lecture seule (sans lui : 60 requêtes par heure) |
//!
//! L'option `--fixtures <dossier>` remplace GitHub par des fichiers locaux.

use std::{env, fmt, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub organisation: String,
    pub adresse: String,
    pub url_publique: String,
    pub rafraichir_secondes: u64,
    pub endormi_jours: i64,
    pub revue_jours: i64,
    pub jeton: Option<Jeton>,
    pub fixtures: Option<PathBuf>,
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
            .finish_non_exhaustive()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            organisation: "abnd-corp".into(),
            adresse: "127.0.0.1:8080".into(),
            url_publique: "http://localhost:8080".into(),
            rafraichir_secondes: 900,
            endormi_jours: 60,
            revue_jours: 180,
            jeton: None,
            fixtures: None,
        }
    }
}

impl Config {
    /// Lit l'environnement puis les arguments de la ligne de commande.
    pub fn depuis_environnement(arguments: &[String]) -> Result<Config, String> {
        let mut c = Config::default();
        if let Some(v) = texte("ABVIGIE_ORG") {
            c.organisation = valider_organisation(v)?;
        }
        if let Some(v) = texte("ABVIGIE_ADRESSE") {
            c.adresse = v;
        }
        if let Some(v) = texte("ABVIGIE_URL") {
            c.url_publique = v.trim_end_matches('/').to_string();
        }
        c.rafraichir_secondes = nombre("ABVIGIE_RAFRAICHIR", c.rafraichir_secondes)?.max(60);
        c.endormi_jours = nombre("ABVIGIE_ENDORMI_JOURS", c.endormi_jours as u64)? as i64;
        c.revue_jours = nombre("ABVIGIE_REVUE_JOURS", c.revue_jours as u64)? as i64;
        c.jeton = texte("GITHUB_TOKEN").map(Jeton);

        let mut it = arguments.iter();
        while let Some(a) = it.next() {
            match a.as_str() {
                "--fixtures" => {
                    c.fixtures = Some(PathBuf::from(
                        it.next().ok_or("--fixtures attend un dossier")?,
                    ));
                }
                "--adresse" => c.adresse = it.next().ok_or("--adresse attend une valeur")?.clone(),
                autre => return Err(format!("argument inconnu : {autre}")),
            }
        }
        Ok(c)
    }
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

fn nombre(nom: &str, defaut: u64) -> Result<u64, String> {
    match texte(nom) {
        None => Ok(defaut),
        Some(v) => v
            .parse()
            .map_err(|_| format!("{nom} : nombre entier attendu")),
    }
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
}
