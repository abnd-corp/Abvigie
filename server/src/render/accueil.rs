//! La page d'accueil : un écran de jeu. À gauche la carte de la ville (dessinée par `assets/js/`), à droite un panneau à
//! onglets (Fiche, Quêtes, Projets, Journal) dont **tout le contenu est déjà dans le HTML** : un robot ou une personne
//! sans JavaScript lit tout, et le JavaScript n'ajoute que la carte et son interactivité.

use super::{LOGO, Meta, Rendu, carte, classe_gravite, document, esc, identifiant, seo, tronquer};
use crate::{
    dates,
    model::{Etat, EtatCi, Progression, Projet, Quete},
};
use std::fmt::Write;

pub fn page(r: &Rendu) -> String {
    let e = r.etat;
    let description = format!(
        "Abvigie est la ville des {} projets publics de l'organisation Abnd : l'activité, les alertes et les chantiers de chacun, \
         et des quêtes pour faire monter la ville de niveau. Lu automatiquement sur GitHub.",
        e.resume.projets
    );
    let json_ld = seo::json_ld_accueil(r);
    let meta = Meta {
        titre: "Abvigie : la ville des projets Abnd",
        description: &description,
        chemin: "/",
        json_ld: &json_ld,
        indexable: true,
        entree: "carte",
        classe: "jeu",
    };

    let mut h = String::with_capacity(48_000);
    h.push_str(&hud(e));
    h.push_str("<main id=\"contenu\" class=\"jeu-zone\">");
    h.push_str(&zone_carte(e));
    h.push_str("<aside class=\"lateral\" aria-label=\"Informations sur la ville\">");
    h.push_str("<div class=\"vue-projet\" hidden><button class=\"retour\" type=\"button\">← Vue d'ensemble</button><div class=\"contenu-projet\"></div></div>");
    h.push_str("<div class=\"vue-ensemble\">");
    h.push_str(&onglets(r));
    h.push_str("</div></aside></main>");
    let _ = write!(
        h,
        "<script type=\"application/json\" id=\"donnees-carte\">{}</script>",
        carte::json_pour_page(e, r.config.endormi_jours)
    );
    document(r, &meta, &h)
}

/// La barre du haut : marque, niveau et compteurs.
fn hud(e: &Etat) -> String {
    let p = &e.progression;
    let humains = e.acteurs.iter().filter(|a| !a.robot).count();
    let robots = e.acteurs.len() - humains;
    let dans_niveau = p.xp.saturating_sub(p.xp_debut);
    let (valeur, max) = match p.xp_suivant {
        Some(s) => (dans_niveau, (s - p.xp_debut).max(1)),
        None => (1, 1),
    };
    let reste = match p.xp_suivant {
        Some(s) => format!(
            "encore {} XP pour « {} »",
            s.saturating_sub(p.xp),
            suivant(p.niveau)
        ),
        None => "niveau maximal".to_string(),
    };
    format!(
        "<header class=\"hud\"><a class=\"marque\" href=\"/\" aria-label=\"Abvigie, accueil\">{LOGO}<span>Abvigie</span></a>\
         <p class=\"slogan\">L'organisation Abnd, vue comme une ville</p>\
         <div class=\"niveau\" title=\"{reste}\"><span class=\"mini-titre\">Niveau</span><strong>{}</strong><span class=\"rang\">{}</span>\
         <progress max=\"{max}\" value=\"{valeur}\" aria-label=\"Progression vers le niveau suivant\"></progress><span class=\"xp\">{} XP</span></div>\
         <dl class=\"compteurs\">\
         <div><dt>Projets</dt><dd>{}</dd></div>\
         <div><dt>Équipe</dt><dd>{humains}</dd></div>\
         <div><dt>Robots</dt><dd>{robots}</dd></div>\
         <div class=\"c-alertes\"><dt>Alertes</dt><dd>{}</dd></div>\
         </dl></header>",
        p.niveau, p.titre, p.xp, e.resume.projets, e.resume.alertes
    )
}

fn suivant(niveau: u32) -> &'static str {
    ["Village", "Bourg", "Ville", "Cité", "Métropole", "Capitale"]
        .get(niveau as usize - 1)
        .copied()
        .unwrap_or("?")
}

/// La zone de la carte : le canvas, ses réglages, sa légende. Le canvas reste vide sans JavaScript.
fn zone_carte(e: &Etat) -> String {
    format!(
        "<section class=\"carte-zone\" aria-label=\"Carte de la ville\">\
         <canvas id=\"carte\" tabindex=\"0\" role=\"img\" aria-label=\"Carte isométrique de la ville : {} bâtiments, un par projet. \
         La liste des projets est dans le panneau, onglet Projets.\">La carte demande JavaScript. La liste des projets est dans l'onglet Projets.</canvas>\
         <div class=\"outils-carte\"><label><input type=\"checkbox\" id=\"opt-noms\" checked> Noms</label>\
         <label><input type=\"checkbox\" id=\"opt-alertes\" checked> Alertes</label>\
         <label><input type=\"checkbox\" id=\"opt-acteurs\" checked> Personnages</label></div>\
         <ul class=\"legende\">\
         <li><span class=\"p-fenetre\"></span>fenêtres allumées : actif</li>\
         <li><span class=\"p-eteint\"></span>éteint : endormi</li>\
         <li><span class=\"p-fanion\"></span>fanion : à régler</li>\
         <li><span class=\"p-chantier\"></span>pointillés : chantier</li></ul>\
         <div class=\"zoom\"><button type=\"button\" data-zoom=\"1\" aria-label=\"Zoomer\">+</button>\
         <button type=\"button\" data-zoom=\"-1\" aria-label=\"Dézoomer\">−</button>\
         <button type=\"button\" data-zoom=\"0\">Recentrer</button></div>\
         <p class=\"aide\">Glisse pour te déplacer · molette ou +/− pour zoomer · clique un bâtiment</p></section>",
        e.projets.len()
    )
}

/// Les quatre onglets : cases d'options cachées, puis les étiquettes, puis les contenus. Le CSS affiche le bon contenu.
fn onglets(r: &Rendu) -> String {
    let e = r.etat;
    let mut h = String::new();
    for (i, (id, _)) in ONGLETS.iter().enumerate() {
        let _ = write!(
            h,
            "<input type=\"radio\" name=\"onglet\" id=\"o-{id}\" class=\"o-case\"{}>",
            if i == 0 { " checked" } else { "" }
        );
    }
    h.push_str("<nav class=\"onglets\" aria-label=\"Onglets\">");
    for (id, titre) in ONGLETS {
        let _ = write!(h, "<label for=\"o-{id}\">{titre}</label>");
    }
    h.push_str("</nav>");
    let _ = write!(
        h,
        "<section class=\"onglet\" id=\"p-fiche\">{}</section>",
        fiche(r)
    );
    let _ = write!(
        h,
        "<section class=\"onglet\" id=\"p-quetes\">{}</section>",
        quetes(&e.progression)
    );
    let _ = write!(
        h,
        "<section class=\"onglet\" id=\"p-projets\">{}</section>",
        projets(e)
    );
    let _ = write!(
        h,
        "<section class=\"onglet\" id=\"p-journal\">{}</section>",
        journal(e)
    );
    h.push_str(&pied(r));
    h
}

const ONGLETS: [(&str, &str); 4] = [
    ("fiche", "Fiche"),
    ("quetes", "Quêtes"),
    ("projets", "Projets"),
    ("journal", "Journal"),
];

fn fiche(r: &Rendu) -> String {
    let e = r.etat;
    let org = esc(&r.config.organisation);
    let sans = |code: &str| {
        e.projets
            .iter()
            .filter(|p| !p.archive && p.drapeaux.iter().any(|d| d.code == code))
            .count()
    };
    let mut h = format!(
        "<p class=\"sur\">Abnd · vue d'ensemble</p><h2>Abnd en un coup d'œil</h2>\
         <p>Chaque bâtiment est un dépôt public, chaque personnage quelqu'un qui travaille dedans, chaque robot une automatisation. \
         Les fenêtres allumées montrent l'activité ; un projet sans activité depuis {} jours s'éteint. Un fanion signale une chose à régler selon la \
         <a href=\"https://github.com/{org}/gouvernance/blob/main/CHARTE.md\">charte</a>, des pointillés un chantier à débloquer.</p>\
         <div class=\"tuiles\">\
         <div><strong class=\"v-ok\">{}</strong><span>projets actifs</span></div>\
         <div><strong class=\"v-dim\">{}</strong><span>projets endormis</span></div>\
         <div><strong class=\"v-alerte\">{}</strong><span>sans licence</span></div>\
         <div><strong class=\"v-attention\">{}</strong><span>sans description</span></div></div>",
        r.config.endormi_jours,
        e.resume.actifs,
        e.resume.endormis,
        sans("licence_absente"),
        sans("description_absente"),
    );
    if let Some(q) = prochaine(&e.progression) {
        let _ = write!(
            h,
            "<div class=\"prochaine\"><p class=\"sur\">Prochaine quête</p><p class=\"titre-q\">{}</p>\
             <p class=\"menu\">{}/{} faits · +{} XP par projet</p><progress max=\"{}\" value=\"{}\" aria-label=\"Progression de la quête\"></progress>\
             <p><label class=\"lien\" for=\"o-quetes\">Voir toutes les quêtes →</label></p></div>",
            esc(q.titre),
            q.fait,
            q.total,
            q.points,
            q.total.max(1),
            q.fait
        );
    }
    let _ = write!(
        h,
        "<h3>Ressources</h3><ul class=\"liens\">\
         <li><a href=\"https://github.com/{org}\">L'organisation sur GitHub</a></li>\
         <li><a href=\"https://github.com/{org}/gouvernance/blob/main/CHARTE.md\">La charte Abnd</a></li>\
         <li><a href=\"https://github.com/{org}/Abvigie\">Le code d'Abvigie (AGPL-3.0-or-later)</a></li>\
         <li><a href=\"/api/etat.json\">Les données en JSON</a></li>\
         <li><a href=\"/llms.txt\">Résumé pour les assistants IA (llms.txt)</a></li></ul>\
         <p class=\"menu\">Les données viennent de l'API publique de GitHub, sans aucune donnée privée, et sont relues toutes les {} minutes. \
         Les personnages sont des identifiants GitHub publics, vus dans les issues et pull requests ouvertes.</p>",
        r.config.rafraichir_secondes / 60
    );
    h
}

/// La quête à mettre en avant : celle qui rapporterait le plus d'XP si on la finissait.
fn prochaine(p: &Progression) -> Option<&Quete> {
    p.quetes
        .iter()
        .filter(|q| !q.accomplie())
        .max_by_key(|q| q.points as usize * (q.total - q.fait))
}

fn quetes(p: &Progression) -> String {
    let mut h =
        String::from("<p class=\"sur\">Journal de quêtes</p><h2>Les objectifs de la charte</h2>");
    let _ = write!(
        h,
        "<p>Une quête est un objectif à atteindre sur <strong>tous</strong> les projets. Chaque projet qui le remplit rapporte de l'XP à la ville. \
         Rien ne se coche à la main : quand un fait change sur GitHub, la quête avance. {} XP sur {} pour l'instant.</p>",
        p.xp, p.xp_max
    );
    for q in &p.quetes {
        let classe = if q.accomplie() { " faite" } else { "" };
        let _ = write!(
            h,
            "<article class=\"quete{classe}\" data-quete=\"{}\" data-projets=\"{}\"><h3>{}{} <span class=\"regle\">{}</span></h3><p class=\"menu\">{}</p>\
             <progress max=\"{}\" value=\"{}\" aria-label=\"Progression\"></progress><p class=\"avance\">{}/{} · +{} XP par projet</p>",
            q.id,
            esc(&q.a_traiter.join(" ")),
            if q.accomplie() { "✓ " } else { "" },
            esc(q.titre),
            esc(q.regle),
            esc(q.description),
            q.total.max(1),
            q.fait,
            q.fait,
            q.total,
            q.points,
        );
        if !q.a_traiter.is_empty() {
            h.push_str("<p class=\"a-faire\">À faire : ");
            let liens: Vec<String> = q
                .a_traiter
                .iter()
                .take(8)
                .map(|s| format!("<a href=\"/projets/{0}\" data-projet>{0}</a>", esc(s)))
                .collect();
            h.push_str(&liens.join(", "));
            if q.a_traiter.len() > 8 {
                let _ = write!(h, " et {} autres", q.a_traiter.len() - 8);
            }
            h.push_str("</p>");
        }
        h.push_str("</article>");
    }
    h
}

fn projets(e: &Etat) -> String {
    let mut h = String::from("<p class=\"sur\">Les bâtiments</p><h2>Les projets</h2>");
    h.push_str(
        "<div class=\"filtres\" role=\"radiogroup\" aria-label=\"Filtrer les projets\">\
         <input type=\"radio\" name=\"filtre\" id=\"f-tous\" checked><label for=\"f-tous\">Tous</label>\
         <input type=\"radio\" name=\"filtre\" id=\"f-alertes\"><label for=\"f-alertes\">Avec alertes</label>\
         <input type=\"radio\" name=\"filtre\" id=\"f-endormis\"><label for=\"f-endormis\">Endormis</label></div><ul class=\"cartes\">",
    );
    for p in &e.projets {
        h.push_str(&carte_projet(p));
    }
    h.push_str("</ul>");
    h
}

fn carte_projet(p: &Projet) -> String {
    let id = identifiant(p);
    let mut h = String::new();
    let _ = write!(
        h,
        "<li class=\"carte v-{id}\" data-alertes=\"{}\"{}><article><canvas class=\"vignette\" width=\"96\" height=\"112\" data-vignette=\"{}\" data-lumieres=\"{}\" aria-hidden=\"true\"></canvas><div class=\"corps\">",
        p.drapeaux.len(),
        if p.activite == crate::model::Activite::Endormi {
            " data-endormi"
        } else {
            ""
        },
        esc(&p.slug),
        p.activite.lumieres(),
    );
    let _ = write!(
        h,
        "<h3><a href=\"/projets/{}\" data-projet>{}</a></h3>",
        esc(&p.slug),
        esc(&p.nom)
    );
    if let Some(d) = &p.description {
        let _ = write!(h, "<p class=\"desc\">{}</p>", esc(&tronquer(d, 140)));
    }
    let _ = write!(
        h,
        "<ul class=\"meta\">{}<li>{}</li><li>{} · {}</li></ul>",
        p.langage
            .as_ref()
            .map(|l| format!("<li>{}</li>", esc(l)))
            .unwrap_or_default(),
        esc(p.licence.as_deref().unwrap_or("sans licence")),
        p.activite.libelle(),
        dates::il_y_a(p.jours_inactif)
    );
    h.push_str(&puces(p));
    h.push_str("<ul class=\"drapeaux\">");
    for d in &p.drapeaux {
        let _ = write!(
            h,
            "<li class=\"{}\">{}</li>",
            classe_gravite(d.gravite),
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

/// Ce qui s'est passé récemment : les dernières exécutions de la CI, tous projets confondus.
fn journal(e: &Etat) -> String {
    let mut evenements: Vec<(&str, i64, &Projet, &crate::model::Execution)> = Vec::new();
    for p in &e.projets {
        for x in &p.executions {
            evenements.push((&x.date, x.age_jours, p, x));
        }
    }
    evenements.sort_by(|a, b| b.0.cmp(a.0));
    let mut h = String::from(
        "<p class=\"sur\">Journal</p><h2>Ce qui s'est passé</h2><ol class=\"journal\">",
    );
    for (_, age, p, x) in evenements.into_iter().take(20) {
        let (classe, mot) = match x.etat {
            EtatCi::Vert => ("ci-vert", "verte"),
            EtatCi::Rouge => ("ci-rouge", "rouge"),
            EtatCi::EnCours => ("ci-cours", "en cours"),
        };
        let _ = write!(
            h,
            "<li class=\"{classe}\"><span class=\"quand\">{}</span><span><a href=\"/projets/{}\" data-projet>{}</a> : {} sur {} <strong>{mot}</strong></span></li>",
            dates::il_y_a(age),
            esc(&p.slug),
            esc(&p.nom),
            esc(&x.workflow),
            esc(&x.branche),
        );
    }
    h.push_str("</ol><p class=\"menu\">Les dernières exécutions de la CI de tous les projets, de la plus récente à la plus ancienne.</p>");
    h
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
