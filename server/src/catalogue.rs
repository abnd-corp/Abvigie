//! Le catalogue des bâtiments : pour chaque dépôt connu, son nom affiché, son quartier et sa couleur.
//!
//! Un dépôt absent d'ici apparaît quand même, avec un bâtiment généré, dans « Autres projets ».
//! La **forme** du bâtiment est décrite côté navigateur, dans `assets/js/batiments.js`, sous le même `slug`.
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
}

pub const GENERIQUE: Entree = Entree {
    depot: "",
    slug: "",
    nom: "",
    quartier: Autres,
    accent: "#9ba4c8",
};

pub const ENTREES: &[Entree] = &[
    Entree {
        depot: "Abatelier",
        slug: "abatelier",
        nom: "Abatelier",
        quartier: Web,
        accent: "#f0a050",
    },
    Entree {
        depot: "Abcarte",
        slug: "abcarte",
        nom: "Abcarte",
        quartier: Web,
        accent: "#6aa7ff",
    },
    Entree {
        depot: "Abcom",
        slug: "abcom",
        nom: "Abcom",
        quartier: Systemes,
        accent: "#5fcfb4",
    },
    Entree {
        depot: "Abdns",
        slug: "abdns",
        nom: "Abdns",
        quartier: Systemes,
        accent: "#9a8cff",
    },
    Entree {
        depot: "abflow-skill-doc-test",
        slug: "abflow",
        nom: "AbFlow",
        quartier: Web,
        accent: "#55c7e0",
    },
    Entree {
        depot: "Abloue",
        slug: "abloue",
        nom: "Abloue",
        quartier: Web,
        accent: "#ff7d6e",
    },
    Entree {
        depot: "Abnd",
        slug: "abnd",
        nom: "Abnd",
        quartier: Web,
        accent: "#f08fc0",
    },
    Entree {
        depot: "AbSante",
        slug: "absante",
        nom: "AbSante",
        quartier: Web,
        accent: "#7ed39a",
    },
    Entree {
        depot: "Abvance",
        slug: "abvance",
        nom: "Abvance",
        quartier: Web,
        accent: "#f4b860",
    },
    Entree {
        depot: "Abvigie",
        slug: "abvigie",
        nom: "Abvigie",
        quartier: Outillage,
        accent: "#ffd98a",
    },
    Entree {
        depot: "Abwatch",
        slug: "abwatch",
        nom: "Abwatch",
        quartier: Systemes,
        accent: "#ff9a62",
    },
    Entree {
        depot: "AbX2T",
        slug: "abx2t",
        nom: "AbX2T",
        quartier: Systemes,
        accent: "#b4d96b",
    },
    Entree {
        depot: "Abyss",
        slug: "abyss",
        nom: "Abyss",
        quartier: Web,
        accent: "#6f7df0",
    },
    Entree {
        depot: "gouvernance",
        slug: "gouvernance",
        nom: "Gouvernance",
        quartier: Outillage,
        accent: "#d9b45a",
    },
    Entree {
        depot: "org-config",
        slug: "org-config",
        nom: "Org-config",
        quartier: Outillage,
        accent: "#8fa8d0",
    },
    Entree {
        depot: "template",
        slug: "template",
        nom: "Template",
        quartier: Outillage,
        accent: "#b8c0dc",
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
}
