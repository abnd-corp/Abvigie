//! SEO et GEO : les données structurées (JSON-LD), le plan du site, `robots.txt`, `llms.txt` et `security.txt`.
//!
//! `llms.txt` est un résumé Markdown du site à destination des assistants IA : il décrit le service
//! et liste les projets avec un lien et une phrase chacun.

use super::Rendu;
use crate::{dates, model::Projet};
use serde_json::{Value, json};

const LICENCE_URL: &str = "https://spdx.org/licenses/AGPL-3.0-or-later.html";

/// Sérialise un JSON-LD pour l'insérer dans un `<script>` : `</script>` ne peut pas s'y glisser.
fn serialiser(valeur: &Value) -> String {
    valeur
        .to_string()
        .replace("</", "<\\/")
        .replace("<!--", "<\\!--")
}

pub fn json_ld_accueil(r: &Rendu) -> String {
    let base = &r.config.url_publique;
    let org = &r.config.organisation;
    let elements: Vec<Value> = r
        .etat
        .projets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            json!({ "@type": "ListItem", "position": i + 1, "name": p.nom, "url": format!("{base}/projets/{}", p.slug) })
        })
        .collect();
    serialiser(&json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "WebSite",
                "@id": format!("{base}/#site"),
                "name": "Abvigie",
                "url": format!("{base}/"),
                "inLanguage": "fr",
                "description": "Tableau de bord public de l'organisation Abnd : l'état de ses projets, vu comme une ville.",
                "publisher": { "@id": format!("{base}/#org") },
                "license": LICENCE_URL
            },
            {
                "@type": "Organization",
                "@id": format!("{base}/#org"),
                "name": "Abnd",
                "url": format!("https://github.com/{org}"),
                "sameAs": [format!("https://github.com/{org}")]
            },
            { "@type": "ItemList", "name": "Projets publics d'Abnd", "numberOfItems": elements.len(), "itemListElement": elements }
        ]
    }))
}

pub fn json_ld_projet(r: &Rendu, p: &Projet) -> String {
    let base = &r.config.url_publique;
    let mut v = json!({
        "@context": "https://schema.org",
        "@type": "SoftwareSourceCode",
        "name": p.nom,
        "url": format!("{base}/projets/{}", p.slug),
        "codeRepository": p.url,
        "inLanguage": "fr",
        "isPartOf": { "@type": "WebSite", "name": "Abvigie", "url": format!("{base}/") },
    });
    let objet = v.as_object_mut().expect("objet JSON");
    if let Some(d) = &p.description {
        objet.insert("description".into(), json!(d));
    }
    if let Some(l) = &p.langage {
        objet.insert("programmingLanguage".into(), json!(l));
    }
    if let Some(l) = &p.licence {
        objet.insert(
            "license".into(),
            json!(format!("https://spdx.org/licenses/{l}.html")),
        );
    }
    if let Some(t) = dates::analyser(&p.dernier_push) {
        objet.insert("dateModified".into(), json!(dates::iso(t)));
    }
    if !p.topics.is_empty() {
        objet.insert("keywords".into(), json!(p.topics.join(", ")));
    }
    serialiser(&v)
}

pub fn plan_du_site(r: &Rendu) -> String {
    let base = &r.config.url_publique;
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    let modif = r.etat.genere_le.get(..10).unwrap_or("");
    s.push_str(&format!("<url><loc>{}/</loc><lastmod>{modif}</lastmod><changefreq>daily</changefreq><priority>1.0</priority></url>\n", super::esc(base)));
    for p in &r.etat.projets {
        let date = p.dernier_push.get(..10).unwrap_or(modif);
        s.push_str(&format!(
            "<url><loc>{}/projets/{}</loc><lastmod>{date}</lastmod><changefreq>weekly</changefreq><priority>0.6</priority></url>\n",
            super::esc(base),
            super::esc(&p.slug)
        ));
    }
    s.push_str("</urlset>\n");
    s
}

pub fn robots(r: &Rendu) -> String {
    format!(
        "User-agent: *\nAllow: /\nDisallow: /api/\n\nSitemap: {}/sitemap.xml\n",
        r.config.url_publique
    )
}

pub fn llms(r: &Rendu) -> String {
    let base = &r.config.url_publique;
    let e = r.etat;
    let mut s = String::new();
    s.push_str("# Abvigie\n\n");
    s.push_str(&format!(
        "> Tableau de bord public de l'organisation Abnd ({org} sur GitHub). Chaque bâtiment de la ville est un dépôt public ; \
         ses fenêtres montrent l'activité, un fanion signale une chose à régler selon la charte Abnd.\n\n",
        org = r.config.organisation
    ));
    s.push_str(&format!(
        "Au {} : {} projets, dont {} actifs et {} endormis ; {} alertes ouvertes. Données lues sur l'API publique de GitHub, sans donnée privée. \
         Langue : français. Code libre sous AGPL-3.0-or-later.\n\n",
        e.genere_le.get(..10).unwrap_or(""),
        e.resume.projets,
        e.resume.actifs,
        e.resume.endormis,
        e.resume.alertes
    ));
    s.push_str("## Pages\n\n");
    s.push_str(&format!(
        "- [Accueil : la ville et la liste des projets]({base}/)\n"
    ));
    s.push_str(&format!(
        "- [Données au format JSON]({base}/api/etat.json) : l'état complet, schéma versionné\n"
    ));
    s.push_str(&format!(
        "- [Code source](https://github.com/{}/Abvigie)\n\n",
        r.config.organisation
    ));
    s.push_str("## Projets\n\n");
    for p in &e.projets {
        let d = p
            .description
            .as_deref()
            .unwrap_or("Pas de description.")
            .replace('\n', " ");
        s.push_str(&format!(
            "- [{}]({base}/projets/{}) : {} ({})\n",
            p.nom,
            p.slug,
            d,
            p.activite.libelle()
        ));
    }
    s.push_str("\n## Comment lire les signaux\n\n");
    s.push_str("- Fenêtres allumées : activité récente. Bâtiment éteint : sans activité depuis longtemps.\n");
    s.push_str("- Fanion : une règle de la charte n'est pas respectée (licence, description, topics, jalons, CI, projet dormant).\n");
    s.push_str(
        "- Pointillés : un chantier à débloquer (issue en retard ou pull request en attente).\n",
    );
    s
}

/// `security.txt` : où signaler une faille (la charte demande le signalement privé de GitHub).
pub fn security_txt(r: &Rendu, maintenant: i64) -> String {
    format!(
        "Contact: https://github.com/{}/Abvigie/security/advisories/new\nExpires: {}\nPreferred-Languages: fr, en\nCanonical: {}/.well-known/security.txt\n",
        r.config.organisation,
        dates::iso(maintenant + 365 * 86_400),
        r.config.url_publique
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_json_ld_ne_peut_pas_fermer_la_balise_script() {
        let v = json!({ "description": "x</script><script>alert(1)</script>" });
        let s = serialiser(&v);
        assert!(!s.contains("</script>"));
        assert!(s.contains("<\\/script>"));
        // et reste un JSON valide
        let relu: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(relu["description"], "x</script><script>alert(1)</script>");
    }
}
