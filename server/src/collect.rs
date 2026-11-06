//! La collecte : transforme ce que renvoie une [`Source`] en [`Etat`], prêt à être affiché.
//!
//! C'est ici, et seulement ici, que les données brutes de GitHub deviennent des notions de la charte
//! (activité, jalon en retard, CI rouge…). Le reste du code ne connaît pas GitHub.

use crate::{
    catalogue::{self, GENERIQUE},
    config::Config,
    dates,
    model::{
        Activite, Ci, Drapeau, Etat, EtatCi, Execution, Projet, Quartier, Resume, Ticket,
        VERSION_SCHEMA,
    },
    rules::{self, Contexte},
    sources::{
        Erreur, Source,
        brut::{self, Depot},
    },
};

/// Nombre d'exécutions gardées pour le journal d'un projet.
const JOURNAL: usize = 8;
const BRANCHES_PRINCIPALES: [&str; 3] = ["dev", "staging", "main"];

pub fn construire(source: &dyn Source, config: &Config, maintenant: i64) -> Result<Etat, Erreur> {
    let mut avertissements = Vec::new();
    let mut projets = Vec::new();

    for depot in source.depots()?.into_iter().filter(|d| nom_sur(&d.name)) {
        let tickets = source.tickets(&depot).unwrap_or_else(|e| {
            avertissements.push(format!("{} : tickets illisibles ({e})", depot.name));
            Vec::new()
        });
        let executions = source.executions(&depot).unwrap_or_else(|e| {
            avertissements.push(format!("{} : CI illisible ({e})", depot.name));
            Vec::new()
        });
        projets.push(projet(&depot, tickets, executions, config, maintenant));
    }

    projets.sort_by(|a, b| {
        a.quartier
            .cmp(&b.quartier)
            .then_with(|| a.nom.to_lowercase().cmp(&b.nom.to_lowercase()))
    });
    let resume = resumer(&projets);
    Ok(Etat {
        schema: VERSION_SCHEMA,
        genere_le: dates::iso(maintenant),
        organisation: config.organisation.clone(),
        resume,
        avertissements,
        projets,
    })
}

/// Un nom de dépôt sert dans des URL et des noms de fichiers : on n'accepte que les caractères que GitHub autorise.
fn nom_sur(nom: &str) -> bool {
    !nom.is_empty()
        && nom
            .bytes()
            .all(|o| o.is_ascii_alphanumeric() || matches!(o, b'-' | b'_' | b'.'))
}

fn projet(
    depot: &Depot,
    tickets: Vec<brut::Ticket>,
    executions: Vec<brut::Execution>,
    config: &Config,
    maintenant: i64,
) -> Projet {
    let entree = catalogue::chercher(&depot.name);
    let (slug, nom, quartier) = match entree {
        Some(e) => (e.slug.to_string(), e.nom.to_string(), e.quartier),
        None => (
            depot.name.to_lowercase(),
            depot.name.clone(),
            GENERIQUE.quartier,
        ),
    };

    let dernier_push = depot
        .pushed_at
        .clone()
        .or_else(|| depot.created_at.clone())
        .unwrap_or_default();
    let jours_inactif = dates::analyser(&dernier_push)
        .map(|t| dates::jours_ecoules(t, maintenant))
        .unwrap_or(0);
    let activite = Activite::depuis(jours_inactif, config.endormi_jours);

    let (mut issues, mut prs) = (Vec::new(), Vec::new());
    for t in tickets {
        let est_pr = t.pull_request.is_some() || !depot.has_issues;
        let ticket = ticket(t, est_pr, maintenant);
        if est_pr {
            prs.push(ticket)
        } else {
            issues.push(ticket)
        }
    }
    issues.sort_by_key(|t| t.age_jours);
    prs.sort_by_key(|t| t.age_jours);

    let journal = journal(&executions, maintenant);
    let ci = etat_ci(&executions, depot.default_branch.as_deref());

    let licence = depot.license.as_ref().and_then(|l| l.spdx_id.clone());
    let contexte = Contexte {
        config,
        description: depot.description.as_deref(),
        licence: licence.as_deref(),
        topics: &depot.topics,
        jours_inactif,
        issues: &issues,
        prs: &prs,
        ci: ci.as_ref(),
        archive: depot.archived,
    };
    let drapeaux: Vec<Drapeau> = rules::evaluer(&contexte);
    let chantier = drapeaux
        .iter()
        .any(|d| matches!(d.code, "jalon_echu" | "pr_en_attente"));

    Projet {
        slug,
        depot: depot.name.clone(),
        nom: nom.clone(),
        url: depot.html_url.clone(),
        description: depot
            .description
            .as_deref()
            .map(|d| sans_le_nom(d, &nom))
            .filter(|d| !d.is_empty()),
        langage: depot.language.clone(),
        licence,
        topics: depot.topics.clone(),
        quartier,
        cree_le: depot.created_at.clone().unwrap_or_default(),
        dernier_push,
        jours_inactif,
        activite,
        archive: depot.archived,
        issues,
        prs,
        ci,
        executions: journal,
        drapeaux,
        chantier,
    }
}

/// Retire le nom du projet au début de sa description (« Abcom: serverless… » devient « serverless… »),
/// puisque le nom est déjà affiché juste au-dessus.
fn sans_le_nom(description: &str, nom: &str) -> String {
    let d = description.trim();
    let debut = d.get(..nom.len()).filter(|p| p.eq_ignore_ascii_case(nom));
    let reste = match debut {
        Some(_) => d[nom.len()..].trim_start(),
        None => return d.to_string(),
    };
    let reste = match reste.strip_prefix([':', '\u{2014}', '-']) {
        Some(r) => r.trim_start(),
        None => return d.to_string(),
    };
    let mut lettres = reste.chars();
    match lettres.next() {
        Some(c) => c.to_uppercase().chain(lettres).collect(),
        None => d.to_string(),
    }
}

fn ticket(t: brut::Ticket, est_pr: bool, maintenant: i64) -> Ticket {
    let (auteur, robot) = match &t.user {
        Some(u) => (
            u.login.clone(),
            u.genre.as_deref() == Some("Bot") || u.login.ends_with("[bot]"),
        ),
        None => ("inconnu".to_string(), false),
    };
    let echeance_secondes = t
        .milestone
        .as_ref()
        .and_then(|j| j.due_on.as_deref())
        .and_then(dates::analyser);
    // Le jalon est échu quand sa date est strictement avant le début d'aujourd'hui.
    let debut_du_jour = maintenant - maintenant.rem_euclid(86_400);
    let en_retard = !est_pr && echeance_secondes.is_some_and(|d| d < debut_du_jour);
    let cree = t.created_at.unwrap_or_default();
    Ticket {
        numero: t.number,
        titre: t.title,
        url: t.html_url,
        auteur,
        robot,
        jalon: t.milestone.as_ref().map(|j| j.title.clone()),
        echeance: echeance_secondes.map(|s| dates::iso(s)[..10].to_string()),
        en_retard,
        age_jours: dates::analyser(&cree)
            .map(|d| dates::jours_ecoules(d, maintenant))
            .unwrap_or(0),
        cree_le: cree,
    }
}

fn etat_depuis(e: &brut::Execution) -> Option<EtatCi> {
    match (e.status.as_deref(), e.conclusion.as_deref()) {
        (Some("completed"), Some("success")) => Some(EtatCi::Vert),
        (Some("completed"), Some("failure" | "timed_out" | "startup_failure")) => {
            Some(EtatCi::Rouge)
        }
        (Some("in_progress" | "queued" | "waiting" | "pending"), _) => Some(EtatCi::EnCours),
        // annulée, ignorée, neutre : ne dit rien de la santé du code
        _ => None,
    }
}

fn journal(executions: &[brut::Execution], maintenant: i64) -> Vec<Execution> {
    executions
        .iter()
        .filter_map(|e| {
            let etat = etat_depuis(e)?;
            let date = e.created_at.clone().unwrap_or_default();
            Some(Execution {
                workflow: e.name.clone().unwrap_or_else(|| "workflow".into()),
                branche: e.head_branch.clone().unwrap_or_default(),
                etat,
                age_jours: dates::analyser(&date)
                    .map(|d| dates::jours_ecoules(d, maintenant))
                    .unwrap_or(0),
                date,
                url: e.html_url.clone().unwrap_or_default(),
            })
        })
        .take(JOURNAL)
        .collect()
}

/// La CI « officielle » d'un projet : la plus récente exécution sur une branche principale, hors tâches automatiques
/// de GitHub (mises à jour Dependabot, exécutions dynamiques).
fn etat_ci(executions: &[brut::Execution], defaut: Option<&str>) -> Option<Ci> {
    executions
        .iter()
        .filter(|e| {
            e.event.as_deref() != Some("dynamic") && e.name.as_deref() != Some("Dependabot Updates")
        })
        .filter(|e| {
            let b = e.head_branch.as_deref().unwrap_or_default();
            BRANCHES_PRINCIPALES.contains(&b) || Some(b) == defaut
        })
        .find_map(|e| {
            Some(Ci {
                etat: etat_depuis(e)?,
                workflow: e.name.clone().unwrap_or_default(),
                branche: e.head_branch.clone().unwrap_or_default(),
                date: e.created_at.clone().unwrap_or_default(),
                url: e.html_url.clone().unwrap_or_default(),
            })
        })
}

fn resumer(projets: &[Projet]) -> Resume {
    Resume {
        projets: projets.len(),
        actifs: projets
            .iter()
            .filter(|p| p.activite != Activite::Endormi)
            .count(),
        endormis: projets
            .iter()
            .filter(|p| p.activite == Activite::Endormi)
            .count(),
        alertes: projets.iter().map(|p| p.drapeaux.len()).sum(),
        chantiers: projets.iter().filter(|p| p.chantier).count(),
    }
}

/// Pour les pages : les projets d'un quartier, dans l'ordre.
pub fn du_quartier(etat: &Etat, quartier: Quartier) -> impl Iterator<Item = &Projet> {
    etat.projets.iter().filter(move |p| p.quartier == quartier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::brut::{Execution as ExecBrute, Jalon, Ticket as TicketBrut, Utilisateur};

    fn exec(nom: &str, branche: &str, statut: &str, conclusion: Option<&str>) -> ExecBrute {
        ExecBrute {
            name: Some(nom.into()),
            event: Some("push".into()),
            head_branch: Some(branche.into()),
            status: Some(statut.into()),
            conclusion: conclusion.map(Into::into),
            created_at: Some("2026-11-01T00:00:00Z".into()),
            html_url: Some("https://example.org/run".into()),
        }
    }

    #[test]
    fn la_ci_ignore_dependabot_les_branches_de_travail_et_les_annulations() {
        let runs = [
            exec("Dependabot Updates", "main", "completed", Some("failure")),
            exec("build", "feature/x", "completed", Some("failure")),
            exec("build", "main", "completed", Some("cancelled")),
            exec("build", "dev", "completed", Some("failure")),
            exec("build", "main", "completed", Some("success")),
        ];
        let ci = etat_ci(&runs, Some("main")).unwrap();
        assert_eq!(ci.etat, EtatCi::Rouge);
        assert_eq!(ci.branche, "dev");
    }

    fn brut_ticket(jalon_du: Option<&str>, robot: bool) -> TicketBrut {
        TicketBrut {
            number: 7,
            title: "x".into(),
            html_url: "https://example.org/7".into(),
            user: Some(Utilisateur {
                login: if robot {
                    "dependabot[bot]".into()
                } else {
                    "hugo".into()
                },
                genre: None,
            }),
            milestone: jalon_du.map(|d| Jalon {
                title: "J".into(),
                due_on: Some(format!("{d}T00:00:00Z")),
            }),
            created_at: Some("2026-10-01T00:00:00Z".into()),
            pull_request: None,
        }
    }

    #[test]
    fn un_jalon_est_en_retard_le_lendemain_de_son_echeance() {
        let aujourdhui = dates::analyser("2026-11-06T15:00:00Z").unwrap();
        assert!(ticket(brut_ticket(Some("2026-11-05"), false), false, aujourdhui).en_retard);
        assert!(!ticket(brut_ticket(Some("2026-11-06"), false), false, aujourdhui).en_retard);
        assert!(!ticket(brut_ticket(Some("2026-11-20"), false), false, aujourdhui).en_retard);
        assert!(!ticket(brut_ticket(None, false), false, aujourdhui).en_retard);
        // une pull request n'est jamais « en retard »
        assert!(!ticket(brut_ticket(Some("2026-11-01"), false), true, aujourdhui).en_retard);
    }

    #[test]
    fn detecte_les_robots_par_leur_nom() {
        assert!(ticket(brut_ticket(None, true), true, 0).robot);
        assert!(!ticket(brut_ticket(None, false), true, 0).robot);
    }

    #[test]
    fn les_niveaux_d_activite_suivent_le_seuil() {
        assert_eq!(Activite::depuis(0, 60), Activite::Vif);
        assert_eq!(Activite::depuis(15, 60), Activite::Actif);
        assert_eq!(Activite::depuis(35, 60), Activite::Calme);
        assert_eq!(Activite::depuis(59, 60), Activite::Faible);
        assert_eq!(Activite::depuis(60, 60), Activite::Endormi);
        assert_eq!(Activite::depuis(500, 60), Activite::Endormi);
    }

    #[test]
    fn le_nom_n_est_pas_repete_dans_la_description() {
        assert_eq!(
            sans_le_nom("Abcom: serverless messenger", "Abcom"),
            "Serverless messenger"
        );
        assert_eq!(
            sans_le_nom("AbSante : comparateur de densité", "AbSante"),
            "Comparateur de densité"
        );
        assert_eq!(sans_le_nom("abcom: x", "Abcom"), "X");
        // pas de nom au début, ou le nom fait partie d'un mot : on ne touche à rien
        assert_eq!(
            sans_le_nom("Gestion de tickets", "Abvance"),
            "Gestion de tickets"
        );
        assert_eq!(
            sans_le_nom("Abcomplet est autre chose", "Abcom"),
            "Abcomplet est autre chose"
        );
        assert_eq!(sans_le_nom("Abnd", "Abnd"), "Abnd");
    }

    #[test]
    fn refuse_les_noms_de_depot_dangereux() {
        assert!(nom_sur("Abcom"));
        assert!(nom_sur("org-config"));
        for mauvais in ["", "a/b", "a b", "../x", "a?b", "é", "<script>"] {
            assert!(!nom_sur(mauvais), "{mauvais}");
        }
    }
}
