//! La ville : un sprite de bâtiments, puis une ligne d'horizon par quartier.
//!
//! Chaque bâtiment est un lien vers la page du projet. Ses fenêtres reflètent l'activité (classes `n0` à `n4`),
//! un drapeau signale les choses à régler, des pointillés un chantier à débloquer.

use super::esc;
use crate::{
    catalogue::{self, GENERIQUE},
    collect::du_quartier,
    model::{Etat, Gravite, Projet, Quartier},
};
use std::fmt::Write;

const LARGEUR_BATIMENT: i32 = 100;
const PAS: i32 = 118;
const HAUTEUR_VUE: i32 = 196;
const SOL: i32 = 150;

/// Identifiant de symbole et classe de couleur d'un projet.
pub fn identifiant(projet: &Projet) -> &'static str {
    // le slug du catalogue est 'static ; les projets inconnus partagent le bâtiment générique
    catalogue::chercher(&projet.depot)
        .map(|e| e.slug)
        .unwrap_or("generique")
}

fn dessin(projet: &Projet) -> &'static str {
    catalogue::chercher(&projet.depot)
        .map(|e| e.dessin)
        .unwrap_or(GENERIQUE.dessin)
}

/// Les dessins nécessaires à une liste de projets, une seule fois chacun.
pub fn sprite<'a>(projets: impl Iterator<Item = &'a Projet>) -> String {
    let mut deja: Vec<&str> = Vec::new();
    let mut s = String::from(
        "<svg class=\"sprite\" width=\"0\" height=\"0\" aria-hidden=\"true\" focusable=\"false\"><defs>",
    );
    for p in projets {
        let id = identifiant(p);
        if deja.contains(&id) {
            continue;
        }
        deja.push(id);
        let _ = write!(
            s,
            "<symbol id=\"b-{id}\" viewBox=\"0 0 {LARGEUR_BATIMENT} 170\">{}</symbol>",
            dessin(p).trim()
        );
    }
    s.push_str("</defs></svg>\n");
    s
}

/// Phrase qui décrit un projet pour les infobulles et les lecteurs d'écran.
pub fn resume(p: &Projet) -> String {
    let mut s = format!("{} : {}", p.nom, p.activite.libelle());
    match p.drapeaux.len() {
        0 => {}
        1 => s.push_str(", 1 alerte"),
        n => {
            let _ = write!(s, ", {n} alertes");
        }
    }
    if p.chantier {
        s.push_str(", chantier à débloquer");
    }
    s
}

pub fn classe_gravite(g: Gravite) -> &'static str {
    match g {
        Gravite::Alerte => "g-alerte",
        Gravite::Attention => "g-attention",
        Gravite::Info => "g-info",
    }
}

/// La ville entière : un bloc par quartier non vide.
pub fn ville(etat: &Etat) -> String {
    let mut h = String::new();
    for q in Quartier::TOUS {
        let projets: Vec<&Projet> = du_quartier(etat, q).collect();
        if projets.is_empty() {
            continue;
        }
        h.push_str(&quartier(q, &projets));
    }
    h
}

fn quartier(q: Quartier, projets: &[&Projet]) -> String {
    let n = projets.len() as i32;
    let largeur = n * PAS;
    let mut h = String::new();
    let _ = write!(
        h,
        "<section class=\"quartier qc{n}\" aria-labelledby=\"q-{id}\"><h3 id=\"q-{id}\">{titre} <span class=\"compte\">{n}</span></h3>",
        id = q.id(),
        titre = esc(q.titre()),
    );
    let _ = write!(
        h,
        "<svg class=\"horizon\" viewBox=\"0 0 {largeur} {HAUTEUR_VUE}\" role=\"group\" aria-label=\"Bâtiments du quartier {}\">",
        esc(q.titre())
    );
    let _ = write!(
        h,
        "<line class=\"sol\" x1=\"0\" y1=\"{SOL}\" x2=\"{largeur}\" y2=\"{SOL}\"/>"
    );
    for (i, p) in projets.iter().enumerate() {
        h.push_str(&batiment(p, i as i32 * PAS + (PAS - LARGEUR_BATIMENT) / 2));
    }
    h.push_str("</svg></section>");
    h
}

fn batiment(p: &Projet, x: i32) -> String {
    let id = identifiant(p);
    let mut h = String::new();
    let _ = write!(
        h,
        "<a class=\"bat v-{id} n{}\" href=\"/projets/{}\" data-projet aria-label=\"{}\"><title>{}</title>",
        p.activite.lumieres(),
        esc(&p.slug),
        esc(&resume(p)),
        esc(&infobulle(p)),
    );
    let _ = write!(
        h,
        "<rect class=\"focus\" x=\"{}\" y=\"2\" width=\"{}\" height=\"{}\" rx=\"4\"/>",
        x - 4,
        LARGEUR_BATIMENT + 8,
        SOL + 20
    );
    if p.chantier {
        let _ = write!(
            h,
            "<rect class=\"chantier\" x=\"{}\" y=\"6\" width=\"{}\" height=\"{}\" rx=\"3\"/>",
            x - 1,
            LARGEUR_BATIMENT + 2,
            SOL - 6
        );
    }
    let _ = write!(
        h,
        "<use href=\"#b-{id}\" x=\"{x}\" y=\"0\" width=\"{LARGEUR_BATIMENT}\" height=\"170\"/>"
    );
    if let Some(g) = p.gravite_max() {
        // un fanion planté à côté du bâtiment, à mi-hauteur : lisible quelle que soit la forme du bâtiment
        let (mat, y) = (x + LARGEUR_BATIMENT - 4, SOL - 64);
        let _ = write!(
            h,
            "<g class=\"alerte {}\"><path class=\"mat\" d=\"M{mat} {SOL}V{y}\"/><path class=\"fanion\" d=\"M{mat} {y}h16l-5 6 5 6h-16z\"/></g>",
            classe_gravite(g)
        );
    }
    let _ = write!(
        h,
        "<text class=\"nom\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
        x + LARGEUR_BATIMENT / 2,
        HAUTEUR_VUE - 8,
        esc(&p.nom)
    );
    h.push_str("</a>");
    h
}

fn infobulle(p: &Projet) -> String {
    let mut s = resume(p);
    let _ = write!(
        s,
        " · {} · {} issue(s) · {} PR",
        crate::dates::il_y_a(p.jours_inactif),
        p.issues.len(),
        p.prs.len()
    );
    for d in &p.drapeaux {
        let _ = write!(s, "\n• {}", d.message);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalogue::ENTREES,
        model::{Activite, Drapeau},
    };

    pub fn projet_exemple(depot: &str) -> Projet {
        Projet {
            slug: depot.to_lowercase(),
            depot: depot.into(),
            nom: depot.into(),
            url: "https://github.com/abnd-corp/x".into(),
            description: Some("Un <b>projet</b>".into()),
            langage: Some("Rust".into()),
            licence: Some("AGPL-3.0".into()),
            topics: vec!["a".into(), "b".into()],
            quartier: Quartier::Web,
            cree_le: "2026-01-01T00:00:00Z".into(),
            dernier_push: "2026-11-01T00:00:00Z".into(),
            jours_inactif: 5,
            activite: Activite::Vif,
            archive: false,
            issues: vec![],
            prs: vec![],
            ci: None,
            executions: vec![],
            drapeaux: vec![],
            chantier: false,
        }
    }

    #[test]
    fn le_sprite_ne_repete_pas_un_dessin() {
        let a = projet_exemple("Abcom");
        let b = projet_exemple("Abcom");
        let c = projet_exemple("Inconnu");
        let s = sprite([&a, &b, &c].into_iter());
        assert_eq!(s.matches("id=\"b-abcom\"").count(), 1);
        assert_eq!(s.matches("id=\"b-generique\"").count(), 1);
    }

    #[test]
    fn un_projet_avec_une_alerte_a_un_fanion_et_un_chantier_des_pointilles() {
        let mut p = projet_exemple("Abcom");
        p.chantier = true;
        p.drapeaux.push(Drapeau {
            code: "x",
            regle: "R20",
            gravite: Gravite::Alerte,
            message: "m".into(),
        });
        let h = batiment(&p, 9);
        assert!(h.contains("g-alerte") && h.contains("class=\"chantier\""));
        assert!(h.contains("href=\"/projets/abcom\""));
    }

    #[test]
    fn le_nom_d_un_projet_ne_peut_pas_injecter_de_html() {
        let mut p = projet_exemple("X");
        p.nom = "<script>alert(1)</script>".into();
        let h = batiment(&p, 0);
        assert!(!h.contains("<script"));
    }

    #[test]
    fn chaque_entree_du_catalogue_a_un_identifiant_stable() {
        for e in ENTREES {
            let p = projet_exemple(e.depot);
            assert_eq!(identifiant(&p), e.slug);
        }
    }
}
