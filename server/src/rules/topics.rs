//! R20 : au moins deux topics par dépôt.

use super::Contexte;
use crate::model::{Drapeau, Gravite};

const MINIMUM: usize = 2;

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    (c.topics.len() < MINIMUM).then(|| Drapeau {
        code: "topics_insuffisants",
        regle: "R20",
        gravite: Gravite::Info,
        message: format!(
            "Moins de {MINIMUM} topics ({} actuellement)",
            c.topics.len()
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::super::essais::sain;
    use super::*;
    use crate::config::Config;

    #[test]
    fn deux_topics_suffisent() {
        let config = Config::default();
        let un = vec!["a".to_string()];
        assert!(evaluer(&sain(&config, &un)).is_some());
        let deux = vec!["a".to_string(), "b".to_string()];
        assert!(evaluer(&sain(&config, &deux)).is_none());
    }
}
