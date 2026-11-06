//! R20 : un dépôt public a une licence. GitHub la détecte sur la branche par défaut : tant qu'elle n'y est
//! pas arrivée (promotion `dev → staging → main`), le drapeau reste levé.

use super::Contexte;
use crate::model::{Drapeau, Gravite};

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    match c.licence {
        Some(l) if l != "NOASSERTION" => None,
        _ => Some(Drapeau {
            code: "licence_absente",
            regle: "R20",
            gravite: Gravite::Alerte,
            message: "Aucune licence détectée sur la branche par défaut".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::super::essais::sain;
    use super::*;
    use crate::config::Config;

    #[test]
    fn leve_un_drapeau_sans_licence() {
        let config = Config::default();
        let mut c = sain(&config, &[]);
        c.licence = None;
        assert_eq!(evaluer(&c).unwrap().code, "licence_absente");
        c.licence = Some("NOASSERTION");
        assert!(evaluer(&c).is_some());
        c.licence = Some("MIT");
        assert!(evaluer(&c).is_none());
    }
}
