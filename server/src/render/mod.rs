//! Le rendu : du HTML complet, écrit par le serveur, avec les vraies données dans la page.
//!
//! C'est volontaire (SEO et GEO) : un moteur de recherche ou un assistant IA lit la page sans exécuter de JavaScript.
//! Le JavaScript (`assets/app.js`) n'ajoute que le panneau latéral.
//!
//! **Règle de sécurité** : toute chaîne qui vient de GitHub passe par [`esc`] avant d'entrer dans une page.

pub mod accueil;
pub mod carte;
pub mod projet;
pub mod seo;

use crate::{
    catalogue,
    config::Config,
    model::{Etat, Gravite, Projet},
};
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
    /// Module JavaScript chargé par la page (`carte`, `vignettes`).
    pub entree: &'a str,
    /// Classe du `<body>` (`jeu` pour l'écran de jeu, `fiche-page` pour une page de projet).
    pub classe: &'a str,
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

/// Identifiant stable d'un projet pour les classes CSS (`v-<slug>` donne sa couleur propre).
pub fn identifiant(projet: &Projet) -> &'static str {
    catalogue::chercher(&projet.depot)
        .map(|e| e.slug)
        .unwrap_or("generique")
}

pub fn classe_gravite(g: Gravite) -> &'static str {
    match g {
        Gravite::Alerte => "g-alerte",
        Gravite::Attention => "g-attention",
        Gravite::Info => "g-info",
    }
}

/// Enveloppe une page : en-tête, métadonnées SEO/GEO, corps, pied de page.
pub fn document(r: &Rendu, meta: &Meta, corps: &str) -> String {
    let base = &r.config.url_publique;
    let canonique = format!("{base}{}", meta.chemin);
    let mut h = String::with_capacity(corps.len() + 4_096);
    h.push_str(
        "<!doctype html>
<html lang=\"fr\">
<head>
<meta charset=\"utf-8\">
",
    );
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\">
");
    let _ = writeln!(h, "<title>{}</title>", esc(meta.titre));
    let _ = writeln!(
        h,
        "<meta name=\"description\" content=\"{}\">",
        esc(meta.description)
    );
    if meta.indexable {
        let _ = writeln!(h, "<link rel=\"canonical\" href=\"{}\">", esc(&canonique));
        h.push_str(
            "<meta name=\"robots\" content=\"index, follow, max-image-preview:large\">
",
        );
    } else {
        h.push_str(
            "<meta name=\"robots\" content=\"noindex\">
",
        );
    }
    h.push_str(
        "<meta name=\"color-scheme\" content=\"dark\">
",
    );
    h.push_str(
        "<meta name=\"theme-color\" content=\"#121830\">
",
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
    h.push_str(
        "<meta property=\"og:image:width\" content=\"1200\">
<meta property=\"og:image:height\" content=\"630\">
",
    );
    h.push_str(
        "<meta name=\"twitter:card\" content=\"summary_large_image\">
",
    );
    h.push_str(
        "<link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\">
",
    );
    let _ = writeln!(
        h,
        "<link rel=\"stylesheet\" href=\"/assets/style.css?v={}\">",
        r.version_css
    );
    // sans JavaScript, tout s'affiche à la suite : la carte laisse la place au contenu
    h.push_str(
        "<noscript><link rel=\"stylesheet\" href=\"/assets/sansjs.css\"></noscript>
",
    );
    let _ = writeln!(
        h,
        "<script type=\"module\" src=\"/assets/js/{}.js?v={}\"></script>",
        esc(meta.entree),
        r.version_js
    );
    if !meta.json_ld.is_empty() {
        let _ = writeln!(
            h,
            "<script type=\"application/ld+json\">{}</script>",
            meta.json_ld
        );
    }
    let _ = write!(
        h,
        "</head>
<body class=\"{}\">
<a class=\"saut\" href=\"#contenu\">Aller au contenu</a>
",
        esc(meta.classe)
    );
    h.push_str(corps);
    h.push_str(
        "
</body>
</html>
",
    );
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
