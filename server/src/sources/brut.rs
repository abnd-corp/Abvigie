//! Les données telles que GitHub les renvoie, réduites aux champs utilisés.
//!
//! Les champs inconnus sont ignorés et presque tout est optionnel : un changement de l'API ne doit pas
//! casser le service. Les fichiers de `server/fixtures/` ont exactement cette forme.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Depot {
    pub name: String,
    pub html_url: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub license: Option<Licence>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub pushed_at: Option<String>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default = "vrai")]
    pub has_issues: bool,
    #[serde(default)]
    pub default_branch: Option<String>,
}

fn vrai() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct Licence {
    #[serde(default)]
    pub spdx_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ticket {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    #[serde(default)]
    pub user: Option<Utilisateur>,
    #[serde(default)]
    pub milestone: Option<Jalon>,
    #[serde(default)]
    pub created_at: Option<String>,
    /// Présent seulement pour une pull request dans la liste des issues.
    #[serde(default)]
    pub pull_request: Option<serde::de::IgnoredAny>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Utilisateur {
    pub login: String,
    #[serde(default, rename = "type")]
    pub genre: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jalon {
    pub title: String,
    #[serde(default)]
    pub due_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Execution {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub head_branch: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub conclusion: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
}

/// L'enveloppe renvoyée par `actions/runs`.
#[derive(Debug, Deserialize)]
pub struct EnveloppeExecutions {
    #[serde(default)]
    pub workflow_runs: Vec<Execution>,
}
