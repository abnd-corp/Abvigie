//! Le catalogue des bâtiments : pour chaque dépôt connu, son nom affiché, son quartier, sa couleur et son dessin.
//!
//! Un dépôt absent d'ici apparaît quand même, avec le bâtiment générique, dans « Autres projets ».
//! Pour donner un bâtiment à un nouveau projet : un fichier SVG dans `assets/immeubles/` et une ligne ci-dessous.
//! Voir `docs/ajouter-un-immeuble.md`.

use crate::model::Quartier::{self, *};

pub struct Entree {
    /// Nom du dépôt GitHub (comparé sans tenir compte de la casse).
    pub depot: &'static str,
    /// Identifiant dans les URL.
    pub slug: &'static str,
    /// Nom affiché.
    pub nom: &'static str,
    pub quartier: Quartier,
    /// Couleur propre au bâtiment, dans la palette de la page.
    pub accent: &'static str,
    /// Le contenu du `<symbol>` : le dessin du bâtiment (voir `assets/immeubles/LISEZMOI.md`).
    pub dessin: &'static str,
}

macro_rules! dessin {
    ($fichier:literal) => {
        include_str!(concat!("../assets/immeubles/", $fichier, ".svg"))
    };
}

pub const GENERIQUE: Entree = Entree {
    depot: "",
    slug: "",
    nom: "",
    quartier: Autres,
    accent: "#9ba4c8",
    dessin: dessin!("generique"),
};

pub const ENTREES: &[Entree] = &[
    Entree {
        depot: "Abatelier",
        slug: "abatelier",
        nom: "Abatelier",
        quartier: Web,
        accent: "#f0a050",
        dessin: dessin!("abatelier"),
    },
    Entree {
        depot: "Abcarte",
        slug: "abcarte",
        nom: "Abcarte",
        quartier: Web,
        accent: "#6aa7ff",
        dessin: dessin!("abcarte"),
    },
    Entree {
        depot: "Abcom",
        slug: "abcom",
        nom: "Abcom",
        quartier: Systemes,
        accent: "#5fcfb4",
        dessin: dessin!("abcom"),
    },
    Entree {
        depot: "Abdns",
        slug: "abdns",
        nom: "Abdns",
        quartier: Systemes,
        accent: "#9a8cff",
        dessin: dessin!("abdns"),
    },
    Entree {
        depot: "abflow-skill-doc-test",
        slug: "abflow",
        nom: "AbFlow",
        quartier: Web,
        accent: "#55c7e0",
        dessin: dessin!("abflow"),
    },
    Entree {
        depot: "Abloue",
        slug: "abloue",
        nom: "Abloue",
        quartier: Web,
        accent: "#ff7d6e",
        dessin: dessin!("abloue"),
    },
    Entree {
        depot: "Abnd",
        slug: "abnd",
        nom: "Abnd",
        quartier: Web,
        accent: "#f08fc0",
        dessin: dessin!("abnd"),
    },
    Entree {
        depot: "AbSante",
        slug: "absante",
        nom: "AbSante",
        quartier: Web,
        accent: "#7ed39a",
        dessin: dessin!("absante"),
    },
    Entree {
        depot: "Abvance",
        slug: "abvance",
        nom: "Abvance",
        quartier: Web,
        accent: "#f4b860",
        dessin: dessin!("abvance"),
    },
    Entree {
        depot: "Abvigie",
        slug: "abvigie",
        nom: "Abvigie",
        quartier: Outillage,
        accent: "#ffd98a",
        dessin: dessin!("abvigie"),
    },
    Entree {
        depot: "Abwatch",
        slug: "abwatch",
        nom: "Abwatch",
        quartier: Systemes,
        accent: "#ff9a62",
        dessin: dessin!("abwatch"),
    },
    Entree {
        depot: "AbX2T",
        slug: "abx2t",
        nom: "AbX2T",
        quartier: Systemes,
        accent: "#b4d96b",
        dessin: dessin!("abx2t"),
    },
    Entree {
        depot: "Abyss",
        slug: "abyss",
        nom: "Abyss",
        quartier: Web,
        accent: "#6f7df0",
        dessin: dessin!("abyss"),
    },
    Entree {
        depot: "gouvernance",
        slug: "gouvernance",
        nom: "Gouvernance",
        quartier: Outillage,
        accent: "#d9b45a",
        dessin: dessin!("gouvernance"),
    },
    Entree {
        depot: "org-config",
        slug: "org-config",
        nom: "Org-config",
        quartier: Outillage,
        accent: "#8fa8d0",
        dessin: dessin!("org-config"),
    },
    Entree {
        depot: "template",
        slug: "template",
        nom: "Template",
        quartier: Outillage,
        accent: "#b8c0dc",
        dessin: dessin!("template"),
    },
];

pub fn chercher(depot: &str) -> Option<&'static Entree> {
    ENTREES.iter().find(|e| e.depot.eq_ignore_ascii_case(depot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_et_depots_sont_uniques() {
        for (i, a) in ENTREES.iter().enumerate() {
            for b in &ENTREES[i + 1..] {
                assert_ne!(a.slug, b.slug);
                assert!(!a.depot.eq_ignore_ascii_case(b.depot));
            }
        }
    }

    #[test]
    fn les_slugs_sont_sans_danger_dans_une_url_et_un_identifiant() {
        for e in ENTREES {
            assert!(
                e.slug
                    .bytes()
                    .all(|o| o.is_ascii_lowercase() || o.is_ascii_digit() || o == b'-'),
                "{}",
                e.slug
            );
        }
    }

    #[test]
    fn les_couleurs_sont_au_format_hexadecimal() {
        for e in ENTREES.iter().chain([&GENERIQUE]) {
            assert!(
                e.accent.len() == 7 && e.accent.starts_with('#'),
                "{}",
                e.accent
            );
            assert!(e.accent[1..].bytes().all(|o| o.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn chaque_dessin_est_bien_forme_et_limite() {
        for e in ENTREES.iter().chain([&GENERIQUE]) {
            assert!(!e.dessin.contains("<script"), "{}", e.slug);
            assert!(
                !e.dessin.contains("style="),
                "{} : pas de style en ligne (CSP)",
                e.slug
            );
            assert!(
                !e.dessin.contains("href"),
                "{} : pas de lien dans un dessin",
                e.slug
            );
            assert!(
                e.dessin.len() < 2_500,
                "{} : dessin trop lourd ({} octets)",
                e.slug,
                e.dessin.len()
            );
            for n in 1..=4 {
                assert!(
                    e.dessin.contains(&format!("class=\"w{n}\"")),
                    "{} : fenêtre w{n} manquante",
                    e.slug
                );
            }
        }
    }
}
