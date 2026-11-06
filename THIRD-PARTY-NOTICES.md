# Notices des composants tiers

Abvigie est publié sous [AGPL-3.0-or-later](LICENSE). Il s'appuie sur les composants Rust ci-dessous, tous sous des licences permissives compatibles. Leur liste est issue de `cargo tree` (dépendances de la version de livraison, plateforme macOS) ; `make audit` (`cargo deny`) vérifie que les licences restent dans la liste autorisée de `deny.toml`.

**Ce document ne couvre que du code.** Aucun élément graphique, aucune police et aucune illustration ne vient de l'extérieur : voir [docs/identite.md](docs/identite.md).

## Dépendances directes (justifiées dans [docs/stack.md](docs/stack.md))

- `serde`, `serde_json` : lecture et écriture du JSON.
- `ureq` : client HTTP(S) pour l'API de GitHub, avec `rustls` pour le TLS.

## Toutes les dépendances

| Composant | Version | Licence | Dépôt |
|---|---|---|---|
| `base64` | 0.23.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| `bytes` | 1.12.1 | MIT | https://github.com/tokio-rs/bytes |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| `http` | 1.5.0 | MIT OR Apache-2.0 | https://github.com/hyperium/http |
| `httparse` | 1.10.1 | MIT OR Apache-2.0 | https://github.com/seanmonstar/httparse |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| `libc` | 0.2.191 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| `log` | 0.4.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| `memchr` | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 (*) |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote (*) |
| `ring` | 0.17.14 | Apache-2.0 AND ISC | https://github.com/briansmith/ring |
| `ring` | 0.17.14 | Apache-2.0 AND ISC | https://github.com/briansmith/ring (*) |
| `rustls` | 0.23.46 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 | https://github.com/rustls/pki-types |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 | https://github.com/rustls/pki-types (*) |
| `rustls-webpki` | 0.103.16 | ISC | https://github.com/rustls/webpki |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_derive` | 1.0.229 (proc-macro) | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| `subtle` | 2.6.1 | BSD-3-Clause | https://github.com/dalek-cryptography/subtle |
| `syn` | 3.0.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| `untrusted` | 0.9.0 | ISC | https://github.com/briansmith/untrusted |
| `ureq` | 3.5.1 | MIT OR Apache-2.0 | https://github.com/algesten/ureq |
| `ureq-proto` | 0.7.1 | MIT OR Apache-2.0 | https://github.com/algesten/ureq-proto |
| `utf8-zero` | 0.8.1 | MIT OR Apache-2.0 | https://github.com/algesten/utf8-zero |
| `webpki-roots` | 1.0.11 | CDLA-Permissive-2.0 | https://github.com/rustls/webpki-roots |
| `zeroize` | 1.9.1 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils |
| `zmij` | 1.0.23 | MIT | https://github.com/dtolnay/zmij |

## Données incorporées

- `webpki-roots` embarque les certificats racine de Mozilla (licence **CDLA-Permissive-2.0**), pour vérifier les certificats de GitHub sans dépendre du système.
