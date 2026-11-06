//! Le site complet, fabriqué d'un coup à chaque rafraîchissement des données.
//!
//! Toutes les pages sont calculées à l'avance et rangées dans une table : répondre à une requête revient à
//! chercher une clé, sans rendu, sans allocation ni calcul. C'est ce qui rend le service léger et prévisible.

use crate::{
    catalogue::{ENTREES, GENERIQUE},
    config::Config,
    dates,
    model::{Etat, Resume, VERSION_SCHEMA},
    render::{self, Meta, Rendu, accueil, esc, projet, seo},
};
use std::collections::HashMap;

const STYLE: &str = include_str!("../assets/style.css");
const SCRIPT: &str = include_str!("../assets/app.js");
const FAVICON: &str = include_str!("../assets/favicon.svg");
const IMAGE_PARTAGE: &[u8] = include_bytes!("../assets/og.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cache {
    /// Pages et données : fraîches une minute, puis revalidées.
    Court,
    /// Ressources versionnées dans l'URL (`?v=…`) : un an. Sans version : une heure.
    Ressource,
    /// Icône : un jour.
    Jour,
}

pub struct Page {
    pub statut: u16,
    pub type_mime: &'static str,
    pub corps: Vec<u8>,
    pub etag: String,
    pub cache: Cache,
}

pub struct Site {
    pages: HashMap<String, Page>,
    introuvable: Page,
    /// Vrai quand aucune donnée n'a pu être lue : le service répond 503 sur la page d'accueil.
    pub degrade: bool,
    /// Nombre de projets affichés.
    pub projets: usize,
}

/// Empreinte FNV-1a 64 bits : sert d'`ETag` et de version de ressource. Pas de cryptographie ici.
fn empreinte(octets: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for o in octets {
        h ^= u64::from(*o);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn page(statut: u16, type_mime: &'static str, corps: impl Into<Vec<u8>>, cache: Cache) -> Page {
    let corps = corps.into();
    let etag = format!("\"{}\"", empreinte(&corps));
    Page {
        statut,
        type_mime,
        corps,
        etag,
        cache,
    }
}

const HTML: &str = "text/html; charset=utf-8";
const TEXTE: &str = "text/plain; charset=utf-8";

/// Feuille de style : le fichier écrit à la main, plus une règle de couleur par bâtiment (voir `catalogue.rs`).
fn feuille_de_style() -> String {
    let mut css = String::from(STYLE);
    // la largeur d'un quartier suit son nombre de bâtiments (--nb), sans style en ligne : la CSP les interdit
    for n in 1..=32 {
        css.push_str(&format!(".qc{n}{{--nb:{n}}}\n"));
    }
    for e in ENTREES.iter().chain([&GENERIQUE]) {
        let slug = if e.slug.is_empty() {
            "generique"
        } else {
            e.slug
        };
        css.push_str(&format!(".v-{slug}{{--ac:{}}}\n", e.accent));
    }
    css
}

impl Site {
    pub fn construire(etat: &Etat, config: &Config, maintenant: i64) -> Site {
        let css = feuille_de_style();
        let (version_css, version_js) = (
            empreinte(css.as_bytes())[..8].to_string(),
            empreinte(SCRIPT.as_bytes())[..8].to_string(),
        );
        let r = Rendu {
            config,
            etat,
            version_css: &version_css,
            version_js: &version_js,
        };

        let mut pages = HashMap::new();
        let mut ajouter = |chemin: &str, p: Page| {
            pages.insert(chemin.to_string(), p);
        };

        ajouter("/", page(200, HTML, accueil::page(&r), Cache::Court));
        for p in &etat.projets {
            ajouter(
                &format!("/projets/{}", p.slug),
                page(200, HTML, projet::page(&r, p), Cache::Court),
            );
            let fragment = format!(
                "{}{}",
                render::ville::sprite(std::iter::once(p)),
                projet::fiche(p, 2)
            );
            ajouter(
                &format!("/projets/{}\u{0}partiel", p.slug),
                page(200, HTML, fragment, Cache::Court),
            );
        }
        ajouter(
            "/api/etat.json",
            page(
                200,
                "application/json; charset=utf-8",
                serde_json::to_vec(etat).unwrap_or_default(),
                Cache::Court,
            ),
        );
        ajouter(
            "/sitemap.xml",
            page(
                200,
                "application/xml; charset=utf-8",
                seo::plan_du_site(&r),
                Cache::Court,
            ),
        );
        ajouter(
            "/robots.txt",
            page(200, TEXTE, seo::robots(&r), Cache::Jour),
        );
        ajouter(
            "/llms.txt",
            page(
                200,
                "text/markdown; charset=utf-8",
                seo::llms(&r),
                Cache::Court,
            ),
        );
        ajouter(
            "/.well-known/security.txt",
            page(200, TEXTE, seo::security_txt(&r, maintenant), Cache::Jour),
        );
        let sante = format!(
            "{{\"ok\":true,\"genere_le\":\"{}\",\"projets\":{}}}",
            esc(&etat.genere_le),
            etat.resume.projets
        );
        ajouter(
            "/sante",
            page(200, "application/json; charset=utf-8", sante, Cache::Court),
        );

        ajouter(
            "/assets/style.css",
            page(200, "text/css; charset=utf-8", css, Cache::Ressource),
        );
        ajouter(
            "/assets/app.js",
            page(
                200,
                "text/javascript; charset=utf-8",
                SCRIPT,
                Cache::Ressource,
            ),
        );
        ajouter(
            "/assets/og.png",
            page(200, "image/png", IMAGE_PARTAGE, Cache::Jour),
        );
        ajouter(
            "/favicon.svg",
            page(200, "image/svg+xml", FAVICON, Cache::Jour),
        );

        let corps_404 = "<main id=\"contenu\" class=\"vide\"><h1>Page introuvable</h1><p>Cette page n'existe pas. <a href=\"/\">Retour à la ville</a>.</p></main>";
        let meta = Meta {
            titre: "Page introuvable · Abvigie",
            description: "Page introuvable.",
            chemin: "/",
            json_ld: "",
            indexable: false,
        };
        let introuvable = page(
            404,
            HTML,
            render::document(&r, &meta, "", corps_404),
            Cache::Court,
        );

        Site {
            pages,
            introuvable,
            degrade: false,
            projets: etat.projets.len(),
        }
    }

    /// Un site sans données : la page d'accueil explique la situation et répond 503. Le service reste en vie
    /// et se rétablit tout seul au prochain rafraîchissement réussi.
    pub fn degrade(config: &Config, raison: &str, maintenant: i64) -> Site {
        let etat = Etat {
            schema: VERSION_SCHEMA,
            genere_le: dates::iso(maintenant),
            organisation: config.organisation.clone(),
            resume: Resume::default(),
            avertissements: vec![format!(
                "Les données de GitHub ne sont pas encore disponibles ({raison})"
            )],
            projets: Vec::new(),
        };
        let mut site = Site::construire(&etat, config, maintenant);
        if let Some(accueil) = site.pages.get_mut("/") {
            accueil.statut = 503;
        }
        if let Some(sante) = site.pages.get_mut("/sante") {
            sante.statut = 503;
        }
        site.degrade = true;
        site
    }

    /// La page d'un chemin, ou la page 404. `partiel` demande le fragment du panneau latéral.
    pub fn chercher(&self, chemin: &str, partiel: bool) -> &Page {
        let cle = if partiel {
            format!("{chemin}\u{0}partiel")
        } else {
            chemin.to_string()
        };
        self.pages.get(&cle).unwrap_or(&self.introuvable)
    }

    #[cfg(test)]
    pub fn chemins(&self) -> Vec<&str> {
        self.pages.keys().map(String::as_str).collect()
    }
}
