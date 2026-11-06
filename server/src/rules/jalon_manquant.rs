//! R46 : pas d'issue sans échéance. L'échéance est la date du jalon : une issue sans jalon n'en a pas.

use super::{Contexte, nombre};
use crate::model::{Drapeau, Gravite};

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    let n = c
        .issues
        .iter()
        .filter(|t| !t.robot && t.jalon.is_none())
        .count();
    (n > 0).then(|| Drapeau {
        code: "jalon_manquant",
        regle: "R46",
        gravite: Gravite::Attention,
        message: format!(
            "{} sans jalon, donc sans échéance",
            nombre(n, "issue", "issues")
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::super::essais::{sain, ticket};
    use super::*;
    use crate::config::Config;

    #[test]
    fn compte_les_issues_humaines_sans_jalon() {
        let config = Config::default();
        let mut sans = ticket(1);
        sans.jalon = None;
        let mut robot = ticket(2);
        robot.jalon = None;
        robot.robot = true;
        let avec = ticket(3);
        let issues = [sans, robot, avec];
        let mut c = sain(&config, &[]);
        c.issues = &issues;
        let d = evaluer(&c).unwrap();
        assert_eq!(d.message, "1 issue sans jalon, donc sans échéance");
    }
}
