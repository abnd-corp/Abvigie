//! R46 : une issue dont l'échéance est dépassée et qui n'est pas fermée est « en retard ».

use super::{Contexte, nombre};
use crate::model::{Drapeau, Gravite};

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    let n = c.issues.iter().filter(|t| t.en_retard).count();
    (n > 0).then(|| Drapeau {
        code: "jalon_echu",
        regle: "R46",
        gravite: Gravite::Attention,
        message: format!(
            "{} en retard (échéance du jalon dépassée)",
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
    fn compte_les_issues_en_retard() {
        let config = Config::default();
        let mut tard = ticket(1);
        tard.en_retard = true;
        let mut tard2 = ticket(2);
        tard2.en_retard = true;
        let issues = [tard, tard2, ticket(3)];
        let mut c = sain(&config, &[]);
        c.issues = &issues;
        assert_eq!(
            evaluer(&c).unwrap().message,
            "2 issues en retard (échéance du jalon dépassée)"
        );
    }
}
