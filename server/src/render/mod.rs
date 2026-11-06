//! Le rendu : du HTML complet, écrit par le serveur, avec les vraies données dans la page.
//!
//! C'est volontaire (SEO et GEO) : un moteur de recherche ou un assistant IA lit la page sans exécuter de JavaScript.
//! Le JavaScript (`assets/app.js`) n'ajoute que le panneau latéral.
//!
//! **Règle de sécurité** : toute chaîne qui vient de GitHub passe par [`esc`] avant d'entrer dans une page.

pub mod accueil;
pub mod projet;
pub mod seo;
pub mod ville;

use crate::{config::Config, model::Etat};
use std::fmt::Write;

/// Ce dont toutes les pages ont besoin.
pub struct Rendu<'a> {
    pub config: &'a Config,
    pub etat: &'a Etat,
    /// Empreintes des ressources, pour le cache longue durée (`?v=…`).
    pub version_css: &'a str,
    pub version_js: &'a str,
}

/// Les informations propres à une page, pour l'en-tête HTML.
pub struct Meta<'a> {
    pub titre: &'a str,
    pub description: &'a str,
    /// Chemin de la page (« / », « /projets/abcom »), pour l'URL canonique.
    pub chemin: &'a str,
    pub json_ld: &'a str,
    pub indexable: bool,
}

/// Échappe un texte pour l'insérer dans du HTML, y compris dans un attribut entre guillemets.
pub fn esc(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len() + 8);
    for c in texte.chars() {
        match c {
            '&' => sortie.push_str("&amp;"),
            '<' => sortie.push_str("&lt;"),
            '>' => sortie.push_str("&gt;"),
            '"' => sortie.push_str("&quot;"),
            '\'' => sortie.push_str("&#39;"),
            // les caractères de contrôle n'ont rien à faire dans une page
            c if c.is_control() && c != '\n' && c != '\t' => {}
            c => sortie.push(c),
        }
    }
    sortie
}

/// Raccourcit un texte à `max` caractères, en ajoutant « … ».
pub fn tronquer(texte: &str, max: usize) -> String {
    if texte.chars().count() <= max {
        texte.to_string()
    } else {
        let mut court: String = texte.chars().take(max.saturating_sub(1)).collect();
        court.push('…');
        court
    }
}

/// Le logo : une tour de vigie dont le faisceau balaie la ville. Même dessin que `favicon.svg`.
pub const LOGO: &str = include_str!("../../assets/logo.svg");

/// Enveloppe une page : en-tête, métadonnées SEO/GEO, corps, pied de page.
pub fn document(r: &Rendu, meta: &Meta, sprite: &str, corps: &str) -> String {
    let base = &r.config.url_publique;
    let canonique = format!("{base}{}", meta.chemin);
    let mut h = String::with_capacity(corps.len() + sprite.len() + 4_096);
    h.push_str("<!doctype html>\n<html lang=\"fr\">\n<head>\n<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\">\n");
    let _ = writeln!(h, "<title>{}</title>", esc(meta.titre));
    let _ = writeln!(
        h,
        "<meta name=\"description\" content=\"{}\">",
        esc(meta.description)
    );
    if meta.indexable {
        let _ = writeln!(h, "<link rel=\"canonical\" href=\"{}\">", esc(&canonique));
        h.push_str("<meta name=\"robots\" content=\"index, follow, max-image-preview:large\">\n");
    } else {
        h.push_str("<meta name=\"robots\" content=\"noindex\">\n");
    }
    h.push_str("<meta name=\"color-scheme\" content=\"dark light\">\n");
    h.push_str(
        "<meta name=\"theme-color\" content=\"#121830\" media=\"(prefers-color-scheme: dark)\">\n",
    );
    h.push_str(
        "<meta name=\"theme-color\" content=\"#eef1fb\" media=\"(prefers-color-scheme: light)\">\n",
    );
    let _ = writeln!(h, "<meta property=\"og:type\" content=\"website\">");
    let _ = writeln!(h, "<meta property=\"og:site_name\" content=\"Abvigie\">");
    let _ = writeln!(h, "<meta property=\"og:locale\" content=\"fr_FR\">");
    let _ = writeln!(
        h,
        "<meta property=\"og:title\" content=\"{}\">",
        esc(meta.titre)
    );
    let _ = writeln!(
        h,
        "<meta property=\"og:description\" content=\"{}\">",
        esc(meta.description)
    );
    let _ = writeln!(
        h,
        "<meta property=\"og:url\" content=\"{}\">",
        esc(&canonique)
    );
    let _ = writeln!(
        h,
        "<meta property=\"og:image\" content=\"{}\">",
        esc(&format!("{base}/assets/og.png"))
    );
    h.push_str("<meta property=\"og:image:width\" content=\"1200\">\n<meta property=\"og:image:height\" content=\"630\">\n");
    h.push_str("<meta name=\"twitter:card\" content=\"summary_large_image\">\n");
    h.push_str("<link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\">\n");
    let _ = writeln!(
        h,
        "<link rel=\"stylesheet\" href=\"/assets/style.css?v={}\">",
        r.version_css
    );
    let _ = writeln!(
        h,
        "<script src=\"/assets/app.js?v={}\" defer></script>",
        r.version_js
    );
    if !meta.json_ld.is_empty() {
        let _ = writeln!(
            h,
            "<script type=\"application/ld+json\">{}</script>",
            meta.json_ld
        );
    }
    h.push_str("</head>\n<body>\n<a class=\"saut\" href=\"#contenu\">Aller au contenu</a>\n");
    h.push_str(sprite);
    h.push_str(corps);
    h.push_str("\n<dialog id=\"panneau\" aria-label=\"Fiche d'un projet\"><button class=\"fermer\" type=\"button\" aria-label=\"Fermer la fiche\">×</button><div class=\"panneau-contenu\"></div></dialog>\n");
    h.push_str("</body>\n</html>\n");
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echappe_tout_ce_qui_peut_casser_une_page() {
        assert_eq!(
            esc("<script>alert('x')</script>"),
            "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"
        );
        assert_eq!(esc("a\"b&c"), "a&quot;b&amp;c");
        assert_eq!(esc("\u{0}\u{7}ok"), "ok");
        assert_eq!(esc("é — français"), "é — français");
    }

    #[test]
    fn tronque_sans_couper_un_caractere() {
        assert_eq!(tronquer("court", 10), "court");
        assert_eq!(tronquer("éééééé", 4), "ééé…");
    }
}
