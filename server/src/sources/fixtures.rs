//! Une source qui lit des fichiers locaux au lieu de GitHub : développement hors ligne, tests, démonstration.
//!
//! Dossier attendu (voir `scripts/capturer-fixtures.sh`, qui le remplit depuis GitHub) :
//! `depots.json`, puis pour chaque dépôt `<nom>.tickets.json` et `<nom>.executions.json`.

use super::{
    Erreur, Source,
    brut::{Depot, EnveloppeExecutions, Execution, Ticket},
};
use serde::de::DeserializeOwned;
use std::{fs, path::PathBuf};

pub struct Fixtures {
    dossier: PathBuf,
}

impl Fixtures {
    pub fn new(dossier: PathBuf) -> Self {
        Fixtures { dossier }
    }

    fn lire<T: DeserializeOwned>(&self, fichier: &str) -> Result<T, Erreur> {
        let chemin = self.dossier.join(fichier);
        let texte = fs::read_to_string(&chemin)
            .map_err(|e| Erreur(format!("{} : {e}", chemin.display())))?;
        serde_json::from_str(&texte).map_err(|e| Erreur(format!("{} : {e}", chemin.display())))
    }
}

impl Source for Fixtures {
    fn depots(&self) -> Result<Vec<Depot>, Erreur> {
        self.lire("depots.json")
    }

    fn tickets(&self, depot: &Depot) -> Result<Vec<Ticket>, Erreur> {
        self.lire(&format!("{}.tickets.json", nom_sur(&depot.name)))
    }

    fn instant(&self) -> Option<i64> {
        let texte = fs::read_to_string(self.dossier.join("capture.txt")).ok()?;
        crate::dates::analyser(texte.trim())
    }

    fn executions(&self, depot: &Depot) -> Result<Vec<Execution>, Erreur> {
        let e: EnveloppeExecutions =
            self.lire(&format!("{}.executions.json", nom_sur(&depot.name)))?;
        Ok(e.workflow_runs)
    }
}

/// Un nom de dépôt ne contient que des lettres, chiffres, `-`, `_` et `.`. Tout le reste est remplacé :
/// un nom venu de `depots.json` ne peut pas sortir du dossier.
fn nom_sur(nom: &str) -> String {
    nom.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::nom_sur;

    #[test]
    fn un_nom_ne_peut_pas_sortir_du_dossier() {
        assert_eq!(nom_sur("Abcom"), "Abcom");
        assert_eq!(nom_sur("../../etc/passwd"), "______etc_passwd");
        assert_eq!(nom_sur("a.b"), "a_b");
    }
}
