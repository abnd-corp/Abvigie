//! Les sources de données. Une source sait lire trois choses : les dépôts, leurs tickets et leurs exécutions de CI.
//!
//! Pour en ajouter une (par exemple `inventaire`, à l'étape 2) : implémenter [`Source`] dans un nouveau fichier.
//! Voir `docs/ajouter-une-source.md`.

pub mod brut;
pub mod fixtures;
pub mod github;

use brut::{Depot, Execution, Ticket};
use std::fmt;

#[derive(Debug)]
pub struct Erreur(pub String);

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Erreur {}

pub trait Source: Send + Sync {
    /// Les dépôts publics de l'organisation.
    fn depots(&self) -> Result<Vec<Depot>, Erreur>;
    /// Les issues et pull requests ouvertes d'un dépôt.
    fn tickets(&self, depot: &Depot) -> Result<Vec<Ticket>, Erreur>;
    /// Les dernières exécutions de la CI d'un dépôt.
    fn executions(&self, depot: &Depot) -> Result<Vec<Execution>, Erreur>;
    /// L'instant auquel les données ont été prises, quand il n'est pas « maintenant » : une capture figée raisonne à sa
    /// date (sinon, quelques mois plus tard, tous ses projets paraîtraient endormis).
    fn instant(&self) -> Option<i64> {
        None
    }
}
