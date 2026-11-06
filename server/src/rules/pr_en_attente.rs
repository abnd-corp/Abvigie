//! Une pull request écrite par un humain attend depuis plus d'une semaine. Les robots (Dependabot) sont ignorés :
//! leur file est suivie par le nombre de PR, pas par ce drapeau.

use super::{Contexte, nombre};
use crate::model::{Drapeau, Gravite};

const SEUIL_JOURS: i64 = 7;

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    let n = c
        .prs
        .iter()
        .filter(|p| !p.robot && p.age_jours > SEUIL_JOURS)
        .count();
    (n > 0).then(|| Drapeau {
        code: "pr_en_attente",
        regle: "R4",
        gravite: Gravite::Info,
        message: format!(
            "{} attend une relecture depuis plus de {SEUIL_JOURS} jours",
            nombre(n, "pull request", "pull requests")
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::super::essais::{sain, ticket};
    use super::*;
    use crate::config::Config;

    #[test]
    fn ignore_les_robots_et_les_pr_recentes() {
        let config = Config::default();
        let mut vieille = ticket(1);
        vieille.age_jours = 20;
        let mut robot = ticket(2);
        robot.age_jours = 20;
        robot.robot = true;
        let mut recente = ticket(3);
        recente.age_jours = 2;
        let prs = [vieille, robot, recente];
        let mut c = sain(&config, &[]);
        c.prs = &prs;
        assert_eq!(
            evaluer(&c).unwrap().message,
            "1 pull request attend une relecture depuis plus de 7 jours"
        );
    }
}
