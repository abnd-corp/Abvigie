//! La fiche d'un projet : une page complète (`/projets/<slug>`) et le même contenu en fragment pour le panneau latéral
//! (`/projets/<slug>?partial=1`).

use super::{Meta, Rendu, accueil, classe_gravite, document, esc, identifiant, seo};
use crate::{
    dates,
    model::{EtatCi, Execution, Projet, Ticket},
};
use std::fmt::Write;

/// Page complète.
pub fn page(r: &Rendu, p: &Projet) -> String {
    let description = match &p.description {
        Some(d) => format!(
            "{} : {} Activité : {}, {} alerte(s).",
            p.nom,
            d,
            p.activite.libelle(),
            p.drapeaux.len()
        ),
        None => format!(
            "{} : projet de l'organisation Abnd. Activité : {}, {} alerte(s).",
            p.nom,
            p.activite.libelle(),
            p.drapeaux.len()
        ),
    };
    let titre = format!("{} : fiche du projet · Abvigie", p.nom);
    let chemin = format!("/projets/{}", p.slug);
    let json_ld = seo::json_ld_projet(r, p);
    let meta = Meta {
        titre: &titre,
        description: &description,
        chemin: &chemin,
        json_ld: &json_ld,
        indexable: true,
        entree: "vignettes",
        classe: "fiche-page",
    };

    let mut h = String::with_capacity(8_192);
    let _ = write!(
        h,
        "<header class=\"entete entete-fiche\"><a class=\"marque\" href=\"/\" aria-label=\"Abvigie, accueil\">{}<span>Abvigie</span></a></header>",
        super::LOGO
    );
    h.push_str("<main id=\"contenu\"><nav class=\"fil\" aria-label=\"Fil d'Ariane\"><a href=\"/\">← La ville</a></nav>");
    h.push_str(&fiche(p, 1));
    h.push_str("</main>");
    h.push_str(&accueil::pied(r));
    document(r, &meta, &h)
}

/// Le contenu de la fiche. `niveau` est le niveau du titre principal : 1 sur la page, 2 dans le panneau.
pub fn fiche(p: &Projet, niveau: u8) -> String {
    let n = niveau;
    let s = niveau + 1;
    let id = identifiant(p);
    let mut h = String::with_capacity(6_144);
    let _ = write!(
        h,
        "<article class=\"fiche v-{id} n{}\"><header class=\"fiche-tete\">",
        p.activite.lumieres()
    );
    let _ = write!(
        h,
        "<canvas class=\"vignette grande\" width=\"128\" height=\"150\" data-vignette=\"{}\" data-lumieres=\"{}\" aria-hidden=\"true\"></canvas><div>",
        esc(&p.slug),
        p.activite.lumieres()
    );
    let _ = write!(h, "<h{n}>{}</h{n}>", esc(&p.nom));
    if let Some(d) = &p.description {
        let _ = write!(h, "<p class=\"desc\">{}</p>", esc(d));
    }
    let _ = write!(
        h,
        "<p><a class=\"bouton\" href=\"{}\" rel=\"noopener\">Voir le dépôt sur GitHub</a></p></div></header>",
        esc(&p.url)
    );

    // Alertes
    let _ = write!(h, "<section><h{s}>Alertes</h{s}>");
    if p.drapeaux.is_empty() {
        h.push_str("<p class=\"rien\">Rien à signaler.</p>");
    } else {
        h.push_str("<ul class=\"drapeaux\">");
        for d in &p.drapeaux {
            let regle = if d.regle.is_empty() {
                String::new()
            } else {
                format!(" <span class=\"regle\">{}</span>", esc(d.regle))
            };
            let _ = write!(
                h,
                "<li class=\"{}\">{}{regle}</li>",
                classe_gravite(d.gravite),
                esc(&d.message)
            );
        }
        h.push_str("</ul>");
    }
    h.push_str("</section>");

    // Santé
    let _ = write!(h, "<section><h{s}>Santé</h{s}><dl class=\"faits\">");
    let _ = write!(
        h,
        "<div><dt>Activité</dt><dd>{} ({})</dd></div>",
        p.activite.libelle(),
        dates::il_y_a(p.jours_inactif)
    );
    if let Some(t) = dates::analyser(&p.dernier_push) {
        let _ = write!(
            h,
            "<div><dt>Dernière modification</dt><dd>{}</dd></div>",
            dates::date_longue(t)
        );
    }
    match &p.ci {
        Some(c) => {
            let _ = write!(
                h,
                "<div><dt>Intégration continue</dt><dd>{} sur {} (<a href=\"{}\" rel=\"noopener\">{}</a>)</dd></div>",
                libelle_ci(c.etat),
                esc(&c.branche),
                esc(&c.url),
                esc(&c.workflow)
            );
        }
        None => h.push_str("<div><dt>Intégration continue</dt><dd>inconnue</dd></div>"),
    }
    let _ = write!(
        h,
        "<div><dt>Licence</dt><dd>{}</dd></div>",
        esc(p.licence.as_deref().unwrap_or("aucune détectée"))
    );
    if let Some(l) = &p.langage {
        let _ = write!(h, "<div><dt>Langage</dt><dd>{}</dd></div>", esc(l));
    }
    if let Some(t) = dates::analyser(&p.cree_le) {
        let _ = write!(
            h,
            "<div><dt>Créé le</dt><dd>{}</dd></div>",
            dates::date_longue(t)
        );
    }
    if !p.topics.is_empty() {
        h.push_str("<div><dt>Topics</dt><dd class=\"topics\">");
        for t in &p.topics {
            let _ = write!(h, "<span>{}</span>", esc(t));
        }
        h.push_str("</dd></div>");
    }
    h.push_str("</dl></section>");

    // Quêtes
    let _ = write!(h, "<section><h{s}>Quêtes</h{s}>");
    let _ = write!(
        h,
        "<h{t} class=\"sous\">Issues ouvertes ({})</h{t}>",
        p.issues.len(),
        t = s + 1
    );
    h.push_str(&tickets(&p.issues, true));
    let _ = write!(
        h,
        "<h{t} class=\"sous\">Pull requests ouvertes ({})</h{t}>",
        p.prs.len(),
        t = s + 1
    );
    h.push_str(&tickets(&p.prs, false));
    h.push_str("</section>");

    // Journal
    let _ = write!(h, "<section><h{s}>Journal</h{s}>");
    if p.executions.is_empty() {
        h.push_str("<p class=\"rien\">Aucune exécution récente de la CI.</p>");
    } else {
        h.push_str("<ol class=\"journal\">");
        for e in &p.executions {
            h.push_str(&ligne_journal(e));
        }
        h.push_str("</ol>");
    }
    h.push_str("</section></article>");
    h
}

fn libelle_ci(e: EtatCi) -> &'static str {
    match e {
        EtatCi::Vert => "verte",
        EtatCi::Rouge => "rouge",
        EtatCi::EnCours => "en cours",
    }
}

fn ligne_journal(e: &Execution) -> String {
    let classe = match e.etat {
        EtatCi::Vert => "ci-vert",
        EtatCi::Rouge => "ci-rouge",
        EtatCi::EnCours => "ci-cours",
    };
    format!(
        "<li class=\"{classe}\"><span class=\"quand\">{}</span> <a href=\"{}\" rel=\"noopener\">{}</a> sur {} : {}</li>",
        dates::il_y_a(e.age_jours),
        esc(&e.url),
        esc(&e.workflow),
        esc(&e.branche),
        libelle_ci(e.etat),
    )
}

/// Les issues sont groupées par jalon, pour montrer les échéances ; les pull requests restent en liste simple.
fn tickets(liste: &[Ticket], par_jalon: bool) -> String {
    if liste.is_empty() {
        return "<p class=\"rien\">Aucune.</p>".into();
    }
    let mut h = String::new();
    if par_jalon {
        let mut jalons: Vec<(Option<&str>, Option<&str>)> = Vec::new();
        for t in liste {
            let cle = (t.jalon.as_deref(), t.echeance.as_deref());
            if !jalons.contains(&cle) {
                jalons.push(cle);
            }
        }
        // les jalons datés d'abord, du plus proche au plus lointain ; « sans jalon » en dernier
        jalons.sort_by(|a, b| match (a.1, b.1) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        for (jalon, echeance) in jalons {
            let titre = match (jalon, echeance) {
                (Some(j), Some(e)) => format!("{} (échéance {})", esc(j), esc(e)),
                (Some(j), None) => esc(j),
                _ => "Sans jalon".to_string(),
            };
            let _ = write!(h, "<p class=\"jalon\">{titre}</p><ul class=\"tickets\">");
            for t in liste
                .iter()
                .filter(|t| (t.jalon.as_deref(), t.echeance.as_deref()) == (jalon, echeance))
            {
                h.push_str(&ligne_ticket(t));
            }
            h.push_str("</ul>");
        }
    } else {
        h.push_str("<ul class=\"tickets\">");
        for t in liste {
            h.push_str(&ligne_ticket(t));
        }
        h.push_str("</ul>");
    }
    h
}

fn ligne_ticket(t: &Ticket) -> String {
    format!(
        "<li{}><a href=\"{}\" rel=\"noopener\"><span class=\"num\">#{}</span> {}</a> <span class=\"age\">{}{}</span></li>",
        if t.en_retard { " class=\"retard\"" } else { "" },
        esc(&t.url),
        t.numero,
        esc(&t.titre),
        if t.robot { "robot · " } else { "" },
        dates::il_y_a(t.age_jours),
    )
}
