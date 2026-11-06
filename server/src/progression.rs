//! La progression de la ville : un niveau, de l'XP et des quêtes, tous calculés sur des faits publics.
//!
//! Une quête est un objectif de la charte à atteindre sur tous les projets (une licence partout, une CI verte partout…).
//! Chaque projet qui remplit l'objectif rapporte des points : l'XP de la ville est leur somme. Rien ne se coche à la main :
//! quand un fait change sur GitHub, la quête avance à la lecture suivante.
//!
//! L'XP est celui **de l'organisation**, jamais celui d'une personne : la charte interdit d'évaluer les gens (R27).
//! Pour ajouter une quête : une ligne dans [`DEFINITIONS`]. Voir `docs/ajouter-une-quete.md`.

use crate::model::{EtatCi, Progression, Projet, Quete};

struct Definition {
    id: &'static str,
    titre: &'static str,
    description: &'static str,
    regle: &'static str,
    points: u32,
    /// Le projet remplit-il l'objectif ?
    ok: fn(&Projet) -> bool,
}

fn drapeau(p: &Projet, code: &str) -> bool {
    p.drapeaux.iter().any(|d| d.code == code)
}

const DEFINITIONS: &[Definition] = &[
    Definition {
        id: "licences",
        titre: "Une licence partout",
        description: "Chaque dépôt public a une licence que GitHub détecte sur sa branche par défaut.",
        regle: "R20",
        points: 10,
        ok: |p| !drapeau(p, "licence_absente"),
    },
    Definition {
        id: "descriptions",
        titre: "Chaque projet se présente",
        description: "Chaque dépôt a une description d'une phrase.",
        regle: "R20",
        points: 5,
        ok: |p| !drapeau(p, "description_absente"),
    },
    Definition {
        id: "topics",
        titre: "Des étiquettes pour être trouvé",
        description: "Chaque dépôt a au moins deux topics.",
        regle: "R20",
        points: 5,
        ok: |p| !drapeau(p, "topics_insuffisants"),
    },
    Definition {
        id: "ci",
        titre: "Des feux verts partout",
        description: "La dernière intégration continue de chaque projet est verte.",
        regle: "R4",
        points: 15,
        ok: |p| p.ci.as_ref().is_some_and(|c| c.etat == EtatCi::Vert),
    },
    Definition {
        id: "jalons",
        titre: "Chaque issue a une échéance",
        description: "Aucune issue sans jalon : l'échéance est la date du jalon.",
        regle: "R46",
        points: 10,
        ok: |p| !drapeau(p, "jalon_manquant"),
    },
    Definition {
        id: "retards",
        titre: "Personne en retard",
        description: "Aucune issue dont l'échéance est dépassée.",
        regle: "R46",
        points: 10,
        ok: |p| !drapeau(p, "jalon_echu"),
    },
    Definition {
        id: "relectures",
        titre: "Les relectures n'attendent pas",
        description: "Aucune pull request humaine n'attend une relecture depuis plus d'une semaine.",
        regle: "R4",
        points: 5,
        ok: |p| !drapeau(p, "pr_en_attente"),
    },
    Definition {
        id: "dormants",
        titre: "Aucun projet oublié",
        description: "Aucun projet sans activité depuis six mois : on le relance ou on l'archive.",
        regle: "R26",
        points: 5,
        ok: |p| !drapeau(p, "dormant"),
    },
];

/// Les noms des niveaux, du plus petit au plus grand.
const TITRES: [&str; 7] = [
    "Hameau",
    "Village",
    "Bourg",
    "Ville",
    "Cité",
    "Métropole",
    "Capitale",
];
/// Part de l'XP maximal à atteindre pour entrer dans chaque niveau.
const SEUILS: [f64; 7] = [0.0, 0.15, 0.30, 0.45, 0.60, 0.75, 0.90];

/// Calcule la progression. Les projets archivés ne comptent pas : ils sont à l'arrêt volontairement.
pub fn calculer(projets: &[Projet]) -> Progression {
    let actifs: Vec<&Projet> = projets.iter().filter(|p| !p.archive).collect();
    let mut quetes = Vec::new();
    let (mut xp, mut xp_max) = (0u32, 0u32);

    for d in DEFINITIONS {
        let mut a_traiter: Vec<&Projet> = actifs.iter().copied().filter(|p| !(d.ok)(p)).collect();
        a_traiter.sort_by_key(|p| p.nom.to_lowercase());
        let fait = actifs.len() - a_traiter.len();
        xp += fait as u32 * d.points;
        xp_max += actifs.len() as u32 * d.points;
        quetes.push(Quete {
            id: d.id,
            titre: d.titre,
            description: d.description,
            regle: d.regle,
            points: d.points,
            fait,
            total: actifs.len(),
            a_traiter: a_traiter.into_iter().map(|p| p.slug.clone()).collect(),
        });
    }

    let part = if xp_max == 0 {
        0.0
    } else {
        f64::from(xp) / f64::from(xp_max)
    };
    let indice = SEUILS.iter().rposition(|&s| part >= s).unwrap_or(0);
    let seuil = |i: usize| (SEUILS[i] * f64::from(xp_max)).ceil() as u32;
    Progression {
        niveau: indice as u32 + 1,
        titre: TITRES[indice],
        xp,
        xp_max,
        xp_debut: seuil(indice),
        xp_suivant: (indice + 1 < SEUILS.len()).then(|| seuil(indice + 1)),
        quetes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Activite, Ci, Drapeau, Gravite, Quartier};

    fn projet(nom: &str) -> Projet {
        Projet {
            slug: nom.to_lowercase(),
            depot: nom.into(),
            nom: nom.into(),
            url: String::new(),
            description: Some("x".into()),
            langage: None,
            licence: Some("MIT".into()),
            topics: vec!["a".into(), "b".into()],
            quartier: Quartier::Web,
            cree_le: String::new(),
            dernier_push: String::new(),
            jours_inactif: 1,
            activite: Activite::Vif,
            archive: false,
            issues: vec![],
            prs: vec![],
            ci: Some(Ci {
                etat: EtatCi::Vert,
                workflow: "w".into(),
                branche: "main".into(),
                date: String::new(),
                url: String::new(),
            }),
            executions: vec![],
            drapeaux: vec![],
            chantier: false,
        }
    }

    fn avec_drapeau(mut p: Projet, code: &'static str) -> Projet {
        p.drapeaux.push(Drapeau {
            code,
            regle: "R20",
            gravite: Gravite::Alerte,
            message: String::new(),
        });
        p
    }

    #[test]
    fn une_ville_parfaite_est_au_dernier_niveau() {
        let g = calculer(&[projet("A"), projet("B")]);
        assert_eq!((g.niveau, g.titre), (7, "Capitale"));
        assert_eq!(g.xp, g.xp_max);
        assert!(g.xp_suivant.is_none());
        assert!(g.quetes.iter().all(Quete::accomplie));
    }

    #[test]
    fn une_ville_sans_rien_est_au_premier_niveau() {
        let mut p = projet("A");
        p.licence = None;
        p.description = None;
        p.topics.clear();
        p.ci = None;
        for code in [
            "licence_absente",
            "description_absente",
            "topics_insuffisants",
            "jalon_manquant",
            "jalon_echu",
            "pr_en_attente",
            "dormant",
        ] {
            p = avec_drapeau(p, code);
        }
        let g = calculer(&[p]);
        assert_eq!((g.niveau, g.xp), (1, 0));
        assert!(g.xp_suivant.is_some());
    }

    #[test]
    fn une_quete_avance_projet_par_projet_et_nomme_les_retardataires() {
        let sans = avec_drapeau(projet("Zeta"), "licence_absente");
        let g = calculer(&[
            projet("Alpha"),
            sans,
            avec_drapeau(projet("Beta"), "licence_absente"),
        ]);
        let q = g.quetes.iter().find(|q| q.id == "licences").unwrap();
        assert_eq!((q.fait, q.total), (1, 3));
        assert_eq!(q.a_traiter, vec!["beta", "zeta"], "triés par nom");
        assert!(!q.accomplie());
    }

    #[test]
    fn les_projets_archives_ne_comptent_pas() {
        let mut vieux = avec_drapeau(projet("Vieux"), "licence_absente");
        vieux.archive = true;
        let g = calculer(&[projet("A"), vieux]);
        assert!(g.quetes.iter().all(|q| q.total == 1));
    }

    #[test]
    fn les_niveaux_sont_croissants_et_coherents() {
        let progression = calculer(&[projet("A")]);
        assert!(progression.xp_debut <= progression.xp);
        assert_eq!(TITRES.len(), SEUILS.len());
        assert!(SEUILS.windows(2).all(|w| w[0] < w[1]));
    }
}
