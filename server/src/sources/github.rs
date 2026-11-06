//! La source GitHub : l'API REST, en lecture seule.
//!
//! - Chaque réponse est mémorisée avec son `ETag`. Au rafraîchissement suivant, on redemande avec
//!   `If-None-Match` : GitHub répond `304` et ne décompte pas la requête de la limite d'utilisation.
//! - Sans jeton, GitHub accepte 60 requêtes par heure ; avec un jeton, 5 000.
//! - Le jeton ne sort jamais de ce fichier : il n'est ni journalisé ni inclus dans les erreurs.

use super::{
    Erreur, Source,
    brut::{Depot, EnveloppeExecutions, Execution, Ticket},
};
use crate::config::Config;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::Mutex, time::Duration};

const API: &str = "https://api.github.com";
/// Plafond de pages lues pour la liste des dépôts (100 dépôts par page).
const PAGES_MAX: usize = 5;

pub struct GitHub {
    agent: ureq::Agent,
    organisation: String,
    jeton: Option<String>,
    /// URL → (ETag, corps). Protégé par un verrou : les lectures sont séquentielles, le verrou est quasi libre.
    memoire: Mutex<HashMap<String, (String, String)>>,
}

impl GitHub {
    pub fn new(config: &Config) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(20)))
            .user_agent("abvigie (+https://github.com/abnd-corp/Abvigie)")
            .build()
            .into();
        GitHub {
            agent,
            organisation: config.organisation.clone(),
            jeton: config.jeton.as_ref().map(|j| j.valeur().to_string()),
            memoire: Mutex::new(HashMap::new()),
        }
    }

    /// Lit un chemin de l'API et renvoie son corps JSON, éventuellement depuis la mémoire (réponse 304).
    fn lire_texte(&self, chemin: &str) -> Result<String, Erreur> {
        let url = format!("{API}{chemin}");
        let precedent = self.memoire.lock().ok().and_then(|m| m.get(&url).cloned());

        let mut requete = self
            .agent
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Some(jeton) = &self.jeton {
            requete = requete.header("Authorization", &format!("Bearer {jeton}"));
        }
        if let Some((etag, _)) = &precedent {
            requete = requete.header("If-None-Match", etag);
        }

        let mut reponse = requete
            .call()
            .map_err(|e| Erreur(format!("GET {chemin} : {e}")))?;
        let statut = reponse.status().as_u16();
        match statut {
            304 => precedent
                .map(|(_, corps)| corps)
                .ok_or_else(|| Erreur(format!("GET {chemin} : 304 sans mémoire"))),
            200 => {
                let etag = reponse
                    .headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                let corps = reponse
                    .body_mut()
                    .read_to_string()
                    .map_err(|e| Erreur(format!("GET {chemin} : lecture du corps : {e}")))?;
                if let (Some(etag), Ok(mut m)) = (etag, self.memoire.lock()) {
                    m.insert(url, (etag, corps.clone()));
                }
                Ok(corps)
            }
            403 | 429 => Err(Erreur(format!(
                "GET {chemin} : limite d'utilisation de GitHub atteinte (HTTP {statut}) ; renseigner GITHUB_TOKEN"
            ))),
            autre => Err(Erreur(format!("GET {chemin} : HTTP {autre}"))),
        }
    }

    fn lire<T: DeserializeOwned>(&self, chemin: &str) -> Result<T, Erreur> {
        let texte = self.lire_texte(chemin)?;
        serde_json::from_str(&texte)
            .map_err(|e| Erreur(format!("GET {chemin} : JSON inattendu : {e}")))
    }
}

impl Source for GitHub {
    fn depots(&self) -> Result<Vec<Depot>, Erreur> {
        let mut tous = Vec::new();
        for page in 1..=PAGES_MAX {
            let chemin = format!(
                "/orgs/{}/repos?type=public&per_page=100&page={page}",
                self.organisation
            );
            let lot: Vec<Depot> = self.lire(&chemin)?;
            let fin = lot.len() < 100;
            tous.extend(lot);
            if fin {
                break;
            }
        }
        Ok(tous)
    }

    fn tickets(&self, depot: &Depot) -> Result<Vec<Ticket>, Erreur> {
        // Quand les issues sont désactivées, seul l'endpoint des pull requests répond.
        let ressource = if depot.has_issues { "issues" } else { "pulls" };
        let chemin = format!(
            "/repos/{}/{}/{ressource}?state=open&per_page=100",
            self.organisation, depot.name
        );
        self.lire(&chemin)
    }

    fn executions(&self, depot: &Depot) -> Result<Vec<Execution>, Erreur> {
        let chemin = format!(
            "/repos/{}/{}/actions/runs?per_page=15",
            self.organisation, depot.name
        );
        let e: EnveloppeExecutions = self.lire(&chemin)?;
        Ok(e.workflow_runs)
    }
}
