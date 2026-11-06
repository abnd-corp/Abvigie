//! La dernière intégration continue d'une branche principale est en échec.

use super::Contexte;
use crate::model::{Drapeau, EtatCi, Gravite};

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    let ci = c.ci?;
    (ci.etat == EtatCi::Rouge).then(|| Drapeau {
        code: "ci_rouge",
        regle: "R4",
        gravite: Gravite::Alerte,
        message: format!("CI en échec sur {} : {}", ci.branche, ci.workflow),
    })
}

#[cfg(test)]
mod tests {
    use super::super::essais::sain;
    use super::*;
    use crate::{config::Config, model::Ci};

    fn ci(etat: EtatCi) -> Ci {
        Ci {
            etat,
            workflow: "build".into(),
            branche: "main".into(),
            date: "2026-11-01T00:00:00Z".into(),
            url: String::new(),
        }
    }

    #[test]
    fn seul_un_echec_leve_un_drapeau() {
        let config = Config::default();
        let mut c = sain(&config, &[]);
        let rouge = ci(EtatCi::Rouge);
        c.ci = Some(&rouge);
        assert_eq!(evaluer(&c).unwrap().message, "CI en échec sur main : build");
        let vert = ci(EtatCi::Vert);
        c.ci = Some(&vert);
        assert!(evaluer(&c).is_none());
        let en_cours = ci(EtatCi::EnCours);
        c.ci = Some(&en_cours);
        assert!(evaluer(&c).is_none());
        c.ci = None;
        assert!(evaluer(&c).is_none());
    }
}
