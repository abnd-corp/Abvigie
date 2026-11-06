//! Les règles : chacune regarde un projet et décide de lever, ou non, un drapeau.
//!
//! Une règle = un fichier = une fonction `evaluer(&Contexte) -> Option<Drapeau>` + ses tests.
//! Pour en ajouter une : créer le fichier, le déclarer ci-dessous et l'ajouter à [`REGLES`].
//! Voir `docs/ajouter-une-regle.md`. Le catalogue lisible est dans `docs/regles.md`.

mod ci_rouge;
mod description;
mod dormant;
mod jalon_echu;
mod jalon_manquant;
mod licence;
mod pr_en_attente;
mod topics;

use crate::{
    config::Config,
    model::{Ci, Drapeau, Ticket},
};

/// Tout ce qu'une règle peut consulter sur un projet. Les règles ne modifient rien.
pub struct Contexte<'a> {
    pub config: &'a Config,
    pub description: Option<&'a str>,
    pub licence: Option<&'a str>,
    pub topics: &'a [String],
    pub jours_inactif: i64,
    pub issues: &'a [Ticket],
    pub prs: &'a [Ticket],
    pub ci: Option<&'a Ci>,
    pub archive: bool,
}

type Regle = fn(&Contexte) -> Option<Drapeau>;

/// Toutes les règles, dans l'ordre d'affichage.
const REGLES: &[Regle] = &[
    ci_rouge::evaluer,
    licence::evaluer,
    description::evaluer,
    topics::evaluer,
    jalon_echu::evaluer,
    jalon_manquant::evaluer,
    pr_en_attente::evaluer,
    dormant::evaluer,
];

/// Applique toutes les règles. Un projet archivé ne lève aucun drapeau : il est volontairement à l'arrêt.
pub fn evaluer(contexte: &Contexte) -> Vec<Drapeau> {
    if contexte.archive {
        return Vec::new();
    }
    let mut drapeaux: Vec<Drapeau> = REGLES.iter().filter_map(|regle| regle(contexte)).collect();
    drapeaux.sort_by_key(|d| std::cmp::Reverse(d.gravite));
    drapeaux
}

/// Pluriel simple : « 1 issue », « 3 issues ».
pub(crate) fn nombre(n: usize, singulier: &str, pluriel: &str) -> String {
    format!("{n} {}", if n == 1 { singulier } else { pluriel })
}

#[cfg(test)]
pub(crate) mod essais {
    use super::*;

    /// Un contexte sain : aucune règle ne doit s'y déclencher. Les tests modifient ce qui les intéresse.
    pub fn sain<'a>(config: &'a Config, topics: &'a [String]) -> Contexte<'a> {
        Contexte {
            config,
            description: Some("Un projet"),
            licence: Some("AGPL-3.0"),
            topics,
            jours_inactif: 3,
            issues: &[],
            prs: &[],
            ci: None,
            archive: false,
        }
    }

    pub fn ticket(numero: u64) -> Ticket {
        Ticket {
            numero,
            titre: format!("Ticket {numero}"),
            url: format!("https://example.org/{numero}"),
            auteur: "humain".into(),
            robot: false,
            jalon: Some("Semaine du 9 novembre".into()),
            echeance: Some("2999-01-01".into()),
            en_retard: false,
            cree_le: "2026-11-01T00:00:00Z".into(),
            age_jours: 2,
        }
    }

    #[test]
    fn un_projet_sain_ne_leve_aucun_drapeau() {
        let config = Config::default();
        let topics = vec!["rust".to_string(), "dashboard".to_string()];
        assert!(evaluer(&sain(&config, &topics)).is_empty());
    }

    #[test]
    fn un_projet_archive_ne_leve_aucun_drapeau() {
        let config = Config::default();
        let mut c = sain(&config, &[]);
        c.licence = None;
        c.archive = true;
        assert!(evaluer(&c).is_empty());
    }
}
