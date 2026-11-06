//! Les données de la carte : un résumé compact de l'état, fait pour le dessin côté navigateur.
//!
//! Il est publié sur `/api/carte.json` et aussi incorporé dans la page d'accueil (`<script type="application/json">`),
//! pour que la carte se dessine sans aller-retour réseau. Il ne contient que des faits publics.

use crate::{
    catalogue::{self, GENERIQUE},
    model::{Etat, Gravite, Quartier},
};
use serde::Serialize;

/// Version du format de ce fichier, indépendante de celle de `/api/etat.json`.
pub const VERSION: u32 = 1;

#[derive(Serialize)]
struct Carte<'a> {
    version: u32,
    genere_le: &'a str,
    endormi_jours: i64,
    niveau: u32,
    quartiers: Vec<QuartierCarte>,
    projets: Vec<ProjetCarte<'a>>,
    acteurs: Vec<ActeurCarte<'a>>,
    quetes: Vec<QueteCarte<'a>>,
}

#[derive(Serialize)]
struct ProjetCarte<'a> {
    slug: &'a str,
    nom: &'a str,
    quartier: &'static str,
    accent: &'static str,
    /// De 0 (éteint) à 4 (toutes les lumières) : le nombre de groupes de fenêtres allumés.
    lumieres: u8,
    jours_inactif: i64,
    chantier: bool,
    archive: bool,
    /// La gravité du drapeau le plus grave : `info`, `attention` ou `alerte`.
    gravite: Option<&'static str>,
    alertes: usize,
    issues: usize,
    prs: usize,
}

#[derive(Serialize)]
struct QuartierCarte {
    id: &'static str,
    titre: &'static str,
    n: usize,
}

#[derive(Serialize)]
struct ActeurCarte<'a> {
    login: &'a str,
    robot: bool,
    activite: usize,
}

#[derive(Serialize)]
struct QueteCarte<'a> {
    id: &'a str,
    accomplie: bool,
    a_traiter: &'a [String],
}

/// Les données de la carte, en JSON compact.
pub fn json(etat: &Etat, endormi_jours: i64) -> String {
    let projets = etat
        .projets
        .iter()
        .map(|p| ProjetCarte {
            slug: &p.slug,
            nom: &p.nom,
            quartier: p.quartier.id(),
            accent: catalogue::chercher(&p.depot).map_or(GENERIQUE.accent, |e| e.accent),
            lumieres: p.activite.lumieres(),
            jours_inactif: p.jours_inactif,
            chantier: p.chantier,
            archive: p.archive,
            gravite: p.gravite_max().map(|g| match g {
                Gravite::Alerte => "alerte",
                Gravite::Attention => "attention",
                Gravite::Info => "info",
            }),
            alertes: p.drapeaux.len(),
            issues: p.issues.len(),
            prs: p.prs.len(),
        })
        .collect();
    let carte = Carte {
        version: VERSION,
        genere_le: &etat.genere_le,
        endormi_jours,
        niveau: etat.progression.niveau,
        quartiers: Quartier::TOUS
            .iter()
            .map(|q| QuartierCarte {
                id: q.id(),
                titre: q.titre(),
                n: etat.projets.iter().filter(|p| p.quartier == *q).count(),
            })
            .filter(|q| q.n > 0)
            .collect(),
        projets,
        acteurs: etat
            .acteurs
            .iter()
            .map(|a| ActeurCarte {
                login: &a.login,
                robot: a.robot,
                activite: a.activite,
            })
            .collect(),
        quetes: etat
            .progression
            .quetes
            .iter()
            .map(|q| QueteCarte {
                id: q.id,
                accomplie: q.accomplie(),
                a_traiter: &q.a_traiter,
            })
            .collect(),
    };
    serde_json::to_string(&carte).unwrap_or_else(|_| "{}".into())
}

/// Le même JSON, prêt à entrer dans un `<script type="application/json">` : `</` ne peut pas y fermer la balise.
pub fn json_pour_page(etat: &Etat, endormi_jours: i64) -> String {
    json(etat, endormi_jours)
        .replace("</", "<\\/")
        .replace("<!--", "<\\!--")
}
