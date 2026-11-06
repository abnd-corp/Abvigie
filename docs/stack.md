# Stack et dépendances

Fiche demandée par la charte (R39) : les versions utilisées et où lire leurs changelogs. **Avant de monter une version, lire les notes de version.**

## Outils

| Outil | Version | Usage | Notes de version |
|---|---|---|---|
| Rust | 1.99.1 (édition 2024, `rust-version = "1.85"`) | le service | https://github.com/rust-lang/rust/blob/master/RELEASES.md |
| TypeScript (`tsc`) | 7.0.2 | **vérification seulement** des types des modules de `assets/js/`, jamais à l'exécution | https://github.com/microsoft/TypeScript/releases |
| `cargo-deny`, `cargo-audit` | derniers | licences, sources et avis de sécurité | https://github.com/EmbarkStudios/cargo-deny · https://github.com/rustsec/rustsec |
| `gh`, `jq` | derniers | capture des données de démonstration | https://github.com/cli/cli/releases |

## Dépendances directes

Chacune est justifiée (R19) : sans elle, il faudrait écrire et maintenir plus de code, plus risqué.

| Crate | Version | Licence | Pourquoi | Notes de version |
|---|---|---|---|---|
| `serde` (+ `derive`) | 1.0.229 | MIT OR Apache-2.0 | lire le JSON de GitHub et écrire celui de l'API | https://github.com/serde-rs/serde/releases |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | le format JSON | https://github.com/serde-rs/json/releases |
| `ureq` | 3.5.1 | MIT OR Apache-2.0 | le client HTTPS vers GitHub ; synchrone, sans runtime `async`, avec `rustls` | https://github.com/algesten/ureq/blob/main/CHANGELOG.md |

`ureq` est utilisé sans ses réglages par défaut (`default-features = false`, `features = ["rustls"]`) : pas de compression ni de JSON intégrés, ce qui évite des crates de plus.

## Ce qui a été écrit plutôt que importé

| Besoin | Raison | Où |
|---|---|---|
| Serveur HTTP | les bibliothèques simples ne sont plus maintenues ; les complètes tirent un runtime `async` énorme ([ADR 0003](adr/0003-serveur-http-minimal.md)) | `http.rs` |
| Dates (ISO 8601, français, HTTP) | `chrono` et `time` pour trois fonctions seraient excessifs | `dates.rs` |
| Gabarits HTML | du texte et `write!`, avec un échappement central | `render/` |
| Lecture des arguments | deux options | `config.rs` |

## Choix réfléchis

- **JavaScript natif en modules, typé par JSDoc** et vérifié par `tsc` (`make check`) : la sécurité de TypeScript sans étape de compilation, sans bundler et sans dépendance npm ([ADR 0001](adr/0001-pile-technique.md)). Une carte isométrique demande davantage de code que prévu (neuf modules, environ 95 Ko) : le découpage en modules et les types sont ce qui le garde lisible.
- **Un canvas 2D, pas de WebGL ni de bibliothèque 3D** : la « 3D » est une projection isométrique dessinée à la main ([ADR 0004](adr/0004-carte-isometrique-sur-canvas.md)).
- **Pas de framework web, pas de runtime `async`, pas de base de données.**

## Mettre à jour

1. Lire les notes de version des composants concernés (liens ci-dessus).
2. `cargo update -p <crate>`, puis `make check` et `make audit`.
3. Mettre ce tableau à jour et régénérer [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md) si une licence ou un composant change.
