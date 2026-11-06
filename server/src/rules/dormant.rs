//! R26 : un projet sans activité depuis six mois est revu : on le relance ou on l'archive.
//!
//! L'extinction des lumières (60 jours par défaut) est une information visuelle, pas un drapeau :
//! le drapeau n'apparaît qu'au seuil de la revue.

use super::Contexte;
use crate::model::{Drapeau, Gravite};

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    (c.jours_inactif >= c.config.revue_jours).then(|| Drapeau {
        code: "dormant",
        regle: "R26",
        gravite: Gravite::Attention,
        message: format!(
            "Sans activité depuis {} jours : à relancer ou à archiver",
            c.jours_inactif
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::super::essais::sain;
    use super::*;
    use crate::config::Config;

    #[test]
    fn se_declenche_au_seuil_de_revue() {
        let config = Config::default();
        let mut c = sain(&config, &[]);
        c.jours_inactif = 179;
        assert!(evaluer(&c).is_none());
        c.jours_inactif = 180;
        assert!(evaluer(&c).is_some());
    }
}
