//! R20 : chaque dépôt a une description d'une phrase.

use super::Contexte;
use crate::model::{Drapeau, Gravite};

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    let vide = c.description.map(|d| d.trim().is_empty()).unwrap_or(true);
    vide.then(|| Drapeau {
        code: "description_absente",
        regle: "R20",
        gravite: Gravite::Attention,
        message: "Pas de description".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::essais::sain;
    use super::*;
    use crate::config::Config;

    #[test]
    fn leve_un_drapeau_sans_description_ou_avec_des_espaces() {
        let config = Config::default();
        let mut c = sain(&config, &[]);
        c.description = None;
        assert!(evaluer(&c).is_some());
        c.description = Some("   ");
        assert!(evaluer(&c).is_some());
        c.description = Some("Quelque chose");
        assert!(evaluer(&c).is_none());
    }
}
