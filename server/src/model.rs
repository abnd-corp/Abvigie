//! Le modèle de données. Tout ce que la page affiche et tout ce que l'API publie vient d'ici.
//!
//! Le fichier `/api/etat.json` est la sérialisation de [`Etat`]. Il est versionné par [`VERSION_SCHEMA`] :
//! tout changement incompatible l'incrémente (voir `docs/donnees.md`).

use serde::Serialize;

/// Version du format de `/api/etat.json`.
pub const VERSION_SCHEMA: u32 = 1;

/// Une photographie complète de l'organisation à un instant donné.
#[derive(Debug, Clone, Serialize)]
pub struct Etat {
    pub schema: u32,
    pub genere_le: String,
    pub organisation: String,
    pub resume: Resume,
    /// Ce qui n'a pas pu être lu (limite de GitHub, dépôt illisible…). Vide quand tout va bien.
    pub avertissements: Vec<String>,
    pub projets: Vec<Projet>,
}

/// Les compteurs de l'en-tête.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Resume {
    pub projets: usize,
    pub actifs: usize,
    pub endormis: usize,
    pub alertes: usize,
    pub chantiers: usize,
}

/// Un dépôt public, vu comme un bâtiment de la ville.
#[derive(Debug, Clone, Serialize)]
pub struct Projet {
    /// Identifiant dans les URL : `/projets/<slug>`.
    pub slug: String,
    /// Nom du dépôt GitHub.
    pub depot: String,
    /// Nom affiché.
    pub nom: String,
    pub url: String,
    pub description: Option<String>,
    pub langage: Option<String>,
    pub licence: Option<String>,
    pub topics: Vec<String>,
    pub quartier: Quartier,
    pub cree_le: String,
    pub dernier_push: String,
    pub jours_inactif: i64,
    pub activite: Activite,
    pub archive: bool,
    pub issues: Vec<Ticket>,
    pub prs: Vec<Ticket>,
    pub ci: Option<Ci>,
    pub executions: Vec<Execution>,
    pub drapeaux: Vec<Drapeau>,
    /// Un travail bloqué ou en retard : le bâtiment est entouré de pointillés.
    pub chantier: bool,
}

/// Regroupement des bâtiments, d'après les domaines de la charte (section « Rôles et périmètres »).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Quartier {
    Web,
    Systemes,
    Outillage,
    Autres,
}

impl Quartier {
    pub const TOUS: [Quartier; 4] = [
        Quartier::Web,
        Quartier::Systemes,
        Quartier::Outillage,
        Quartier::Autres,
    ];

    pub fn titre(self) -> &'static str {
        match self {
            Quartier::Web => "Développement web",
            Quartier::Systemes => "Développement systèmes",
            Quartier::Outillage => "Outillage et gouvernance",
            Quartier::Autres => "Autres projets",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Quartier::Web => "web",
            Quartier::Systemes => "systemes",
            Quartier::Outillage => "outillage",
            Quartier::Autres => "autres",
        }
    }
}

/// Le niveau de lumière d'un bâtiment, déduit de la dernière activité.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Activite {
    /// Toutes les fenêtres allumées.
    Vif,
    Actif,
    Calme,
    /// Quelques fenêtres seulement.
    Faible,
    /// Plus aucune lumière : sans activité depuis le seuil d'endormissement.
    Endormi,
}

impl Activite {
    /// Niveau de lumière pour un nombre de jours sans activité. `endormi` est le seuil d'extinction (60 jours par défaut).
    pub fn depuis(jours: i64, endormi: i64) -> Activite {
        match jours {
            j if j >= endormi => Activite::Endormi,
            j if j <= 7 => Activite::Vif,
            j if j <= 21 => Activite::Actif,
            j if j <= endormi * 2 / 3 => Activite::Calme,
            _ => Activite::Faible,
        }
    }

    /// Nombre de groupes de fenêtres allumés, de 0 à 4. Le CSS (`.n0` à `.n4`) s'appuie sur ce nombre.
    pub fn lumieres(self) -> u8 {
        match self {
            Activite::Vif => 4,
            Activite::Actif => 3,
            Activite::Calme => 2,
            Activite::Faible => 1,
            Activite::Endormi => 0,
        }
    }

    pub fn libelle(self) -> &'static str {
        match self {
            Activite::Vif => "très actif",
            Activite::Actif => "actif",
            Activite::Calme => "calme",
            Activite::Faible => "peu actif",
            Activite::Endormi => "endormi",
        }
    }
}

/// Une issue ou une pull request ouverte.
#[derive(Debug, Clone, Serialize)]
pub struct Ticket {
    pub numero: u64,
    pub titre: String,
    pub url: String,
    pub auteur: String,
    /// Écrit par un robot (Dependabot, GitHub Actions…).
    pub robot: bool,
    pub jalon: Option<String>,
    /// Date d'échéance du jalon, « AAAA-MM-JJ ».
    pub echeance: Option<String>,
    pub en_retard: bool,
    pub cree_le: String,
    pub age_jours: i64,
}

/// L'état de la dernière intégration continue sur une branche principale.
#[derive(Debug, Clone, Serialize)]
pub struct Ci {
    pub etat: EtatCi,
    pub workflow: String,
    pub branche: String,
    pub date: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EtatCi {
    Vert,
    Rouge,
    EnCours,
}

/// Une exécution récente de la CI, pour le journal.
#[derive(Debug, Clone, Serialize)]
pub struct Execution {
    pub workflow: String,
    pub branche: String,
    pub etat: EtatCi,
    pub date: String,
    pub age_jours: i64,
    pub url: String,
}

/// Une chose à régler, produite par une règle (`rules/`).
#[derive(Debug, Clone, Serialize)]
pub struct Drapeau {
    /// Identifiant stable de la règle (`licence_absente`…), pour filtrer et tester.
    pub code: &'static str,
    /// Règle de la charte à l'origine du drapeau (« R20 »), ou vide.
    pub regle: &'static str,
    pub gravite: Gravite,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gravite {
    Info,
    Attention,
    Alerte,
}

impl Projet {
    /// La gravité la plus élevée parmi les drapeaux.
    pub fn gravite_max(&self) -> Option<Gravite> {
        self.drapeaux.iter().map(|d| d.gravite).max()
    }
}
