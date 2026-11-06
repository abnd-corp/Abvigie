//! La page d'accueil : l'en-tête et ses compteurs, la ville, la liste des projets, la légende.

use super::{LOGO, Meta, Rendu, document, esc, seo, tronquer, ville};
use crate::{
    dates,
    model::{Activite, EtatCi, Projet},
};
use std::fmt::Write;

pub fn page(r: &Rendu) -> String {
    let e = r.etat;
    let description = format!(
        "Abvigie montre l'état des {} projets publics de l'organisation Abnd : activité, alertes, chantiers, intégration continue et licences, lus automatiquement sur GitHub.",
        e.resume.projets
    );
    let json_ld = seo::json_ld_accueil(r);
    let meta = Meta {
        titre: "Abvigie : la ville des projets Abnd",
        description: &description,
        chemin: "/",
        json_ld: &json_ld,
        indexable: true,
    };
    let sprite = ville::sprite(e.projets.iter());

    let mut h = String::with_capacity(32_768);
    let _ = write!(
        h,
        "<header class=\"entete\"><a class=\"marque\" href=\"/\" aria-label=\"Abvigie, accueil\">{LOGO}<span>Abvigie</span></a>\
         <p class=\"slogan\">L'organisation Abnd, vue comme une ville.</p>\
         <dl class=\"compteurs\">\
         <div><dt>Projets</dt><dd>{}</dd></div>\
         <div><dt>Actifs</dt><dd>{}</dd></div>\
         <div><dt>Endormis</dt><dd>{}</dd></div>\
         <div class=\"c-alertes\"><dt>Alertes</dt><dd>{}</dd></div>\
         <div><dt>Chantiers</dt><dd>{}</dd></div>\
         </dl></header>",
        e.resume.projets, e.resume.actifs, e.resume.endormis, e.resume.alertes, e.resume.chantiers
    );

    h.push_str("<main id=\"contenu\">");
    if !e.avertissements.is_empty() {
        h.push_str("<aside class=\"avis\" role=\"status\"><strong>Données partielles.</strong> ");
        let liste: Vec<String> = e
            .avertissements
            .iter()
            .take(3)
            .map(|a| esc(&tronquer(a, 140)))
            .collect();
        h.push_str(&liste.join(" · "));
        h.push_str("</aside>");
    }

    h.push_str("<section class=\"ville-bloc\" aria-labelledby=\"t-ville\"><h2 id=\"t-ville\">La ville</h2>");
    h.push_str("<div class=\"outils\"><label><input type=\"checkbox\" id=\"opt-noms\" checked> Noms</label>");
    h.push_str("<label><input type=\"checkbox\" id=\"opt-alertes\" checked> Alertes</label></div>");
    h.push_str("<div class=\"quartiers\">");
    h.push_str(&ville::ville(e));
    h.push_str("</div>");
    h.push_str(
        "<ul class=\"legende\">\
         <li><span class=\"p-fenetre\"></span> fenêtres allumées : activité récente</li>\
         <li><span class=\"p-eteint\"></span> bâtiment éteint : endormi</li>\
         <li><span class=\"p-fanion\"></span> fanion : une chose à régler</li>\
         <li><span class=\"p-chantier\"></span> pointillés : un chantier à débloquer</li></ul>",
    );
    h.push_str("</section>");

    h.push_str("<section id=\"projets\" aria-labelledby=\"t-projets\"><h2 id=\"t-projets\">Les projets</h2>");
    h.push_str(
        "<div class=\"filtres\" role=\"radiogroup\" aria-label=\"Filtrer les projets\">\
         <input type=\"radio\" name=\"filtre\" id=\"f-tous\" checked><label for=\"f-tous\">Tous</label>\
         <input type=\"radio\" name=\"filtre\" id=\"f-alertes\"><label for=\"f-alertes\">Avec alertes</label>\
         <input type=\"radio\" name=\"filtre\" id=\"f-endormis\"><label for=\"f-endormis\">Endormis</label></div>",
    );
    h.push_str("<ul class=\"cartes\">");
    for p in &e.projets {
        h.push_str(&carte(p));
    }
    h.push_str("</ul></section>");

    let _ = write!(
        h,
        "<section class=\"apropos\" aria-labelledby=\"t-apropos\"><h2 id=\"t-apropos\">À propos</h2>\
         <p>Abvigie est le tableau de bord public de l'organisation Abnd (<a href=\"https://github.com/{org}\">{org}</a> sur GitHub). \
         Chaque bâtiment est un dépôt public. Les fenêtres allumées montrent l'activité récente ; un projet sans activité depuis {endormi} jours s'éteint. \
         Un fanion signale une chose à régler selon la <a href=\"https://github.com/{org}/gouvernance/blob/main/CHARTE.md\">charte Abnd</a> \
         (licence, description, topics, jalons, intégration continue, projets dormants), et des pointillés un chantier à débloquer.</p>\
         <p>Les données viennent de l'API publique de GitHub, sans aucune donnée privée. Elles sont relues toutes les {minutes} minutes. \
         Le code d'Abvigie est libre (<a href=\"https://github.com/{org}/Abvigie\">AGPL-3.0-or-later</a>) et les données sont disponibles en \
         <a href=\"/api/etat.json\">JSON</a>.</p></section>",
        org = esc(&r.config.organisation),
        endormi = r.config.endormi_jours,
        minutes = r.config.rafraichir_secondes / 60,
    );
    h.push_str("</main>");
    h.push_str(&pied(r));
    document(r, &meta, &sprite, &h)
}

/// Le pied de page, commun à toutes les pages.
pub fn pied(r: &Rendu) -> String {
    let genere = dates::analyser(&r.etat.genere_le).unwrap_or(0);
    format!(
        "<footer class=\"pied\"><p>Abvigie {} · <a href=\"https://github.com/{org}/Abvigie\">code source</a> · AGPL-3.0-or-later · \
         <a href=\"/api/etat.json\">données</a> · <a href=\"/llms.txt\">llms.txt</a></p>\
         <p>Données de GitHub du {} à {}.</p></footer>",
        env!("CARGO_PKG_VERSION"),
        dates::date_longue(genere),
        esc(r.etat.genere_le.get(11..16).unwrap_or("")),
        org = esc(&r.config.organisation),
    )
}

fn carte(p: &Projet) -> String {
    let id = ville::identifiant(p);
    let mut h = String::new();
    let _ = write!(
        h,
        "<li class=\"carte v-{id} n{}\" data-alertes=\"{}\"{}><article>",
        p.activite.lumieres(),
        p.drapeaux.len(),
        if p.activite == Activite::Endormi {
            " data-endormi"
        } else {
            ""
        },
    );
    let _ = write!(
        h,
        "<svg class=\"mini\" viewBox=\"0 0 100 170\" aria-hidden=\"true\"><use href=\"#b-{id}\" width=\"100\" height=\"170\"/></svg><div class=\"corps\">"
    );
    let _ = write!(
        h,
        "<h3><a href=\"/projets/{}\" data-projet>{}</a></h3>",
        esc(&p.slug),
        esc(&p.nom)
    );
    if let Some(d) = &p.description {
        let _ = write!(h, "<p class=\"desc\">{}</p>", esc(&tronquer(d, 180)));
    }
    h.push_str("<ul class=\"meta\">");
    if let Some(l) = &p.langage {
        let _ = write!(h, "<li>{}</li>", esc(l));
    }
    let _ = write!(
        h,
        "<li>{}</li>",
        esc(p.licence.as_deref().unwrap_or("sans licence"))
    );
    let _ = write!(
        h,
        "<li class=\"act\">{} · {}</li>",
        p.activite.libelle(),
        dates::il_y_a(p.jours_inactif)
    );
    h.push_str("</ul>");
    h.push_str(&puces(p));
    h.push_str("<ul class=\"drapeaux\">");
    for d in &p.drapeaux {
        let _ = write!(
            h,
            "<li class=\"{}\">{}</li>",
            ville::classe_gravite(d.gravite),
            esc(&d.message)
        );
    }
    h.push_str("</ul></div></article></li>");
    h
}

/// Compteurs d'une ligne : issues, pull requests, CI.
pub fn puces(p: &Projet) -> String {
    let ci = match p.ci.as_ref().map(|c| c.etat) {
        Some(EtatCi::Vert) => "<li class=\"ci-vert\">CI verte</li>",
        Some(EtatCi::Rouge) => "<li class=\"ci-rouge\">CI rouge</li>",
        Some(EtatCi::EnCours) => "<li class=\"ci-cours\">CI en cours</li>",
        None => "<li>CI inconnue</li>",
    };
    let humaines = p.prs.iter().filter(|t| !t.robot).count();
    let robots = p.prs.len() - humaines;
    let mut prs = format!("{humaines} PR");
    if robots > 0 {
        let _ = write!(
            prs,
            " + {robots} robot{}",
            if robots > 1 { "s" } else { "" }
        );
    }
    format!(
        "<ul class=\"puces\"><li>{} issue{}</li><li>{prs}</li>{ci}</ul>",
        p.issues.len(),
        if p.issues.len() > 1 { "s" } else { "" }
    )
}
