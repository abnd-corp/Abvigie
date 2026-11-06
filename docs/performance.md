# Performance, poids et propreté de Rust

Le principe : **le plus léger possible, mesuré, avec un budget qui échoue aux tests s'il est dépassé.**

## Ce qu'on mesure (version de livraison, données de démonstration, 16 projets)

| Mesure | Valeur |
|---|---|
| Page d'accueil | 39,7 Ko, **7,5 Ko compressés** (gzip) |
| `style.css` | 14,5 Ko, 4,5 Ko compressés |
| `app.js` | 2,6 Ko, 1,2 Ko compressés |
| Une fiche projet | 8,3 Ko, 2,6 Ko compressés |
| **Poids total d'une première visite** | environ **13,5 Ko compressés** (page + style + script), sans police, sans image, sans ressource externe |
| Binaire de livraison | **1,5 Mo** (TLS compris) |
| Mémoire du processus | **2,3 Mo** au repos, 2,9 Mo après 4 000 requêtes |
| Réponse à une requête | médiane **0,17 ms**, 99 % sous 0,3 ms |
| Débit | environ 7 800 requêtes par seconde, limité par le client de test et non par le serveur |
| Dépendances (transitives comprises) | 47 crates dans `Cargo.lock`, 0 avis de sécurité |

Ces chiffres sont faciles à tenir parce que **répondre ne calcule rien** : toutes les pages sont fabriquées à l'avance ([architecture](architecture.md)).

La compression n'est pas faite par le service : elle revient au proxy qui est devant (Traefik, Caddy), qui sait aussi faire du brotli. Voir [deploiement.md](deploiement.md).

## Le budget (vérifié par `cargo test`)

| Élément | Limite |
|---|---|
| Page d'accueil | 20 Ko + 2,5 Ko par projet (donc environ 60 Ko pour 16 projets, avant compression) |
| `style.css` | 16 Ko |
| `app.js` | 4 Ko |
| `favicon.svg` | 1 Ko |
| `og.png` (image de partage) | 120 Ko, chargée par les réseaux sociaux et pas par la page |
| Ressources externes | **aucune** : pas de police, de CDN, d'image distante, de script tiers |

Dépasser une limite fait échouer `la_page_reste_dans_son_budget_de_poids`. Le relever est une décision, à écrire dans la PR.

**Pourquoi pas de police web :** elle coûterait des dizaines de Ko, un aller-retour réseau, un flash de texte, et une requête vers un tiers. La pile `system-ui` donne une interface native et lisible sur chaque système.

## Rust : ne pas devenir obèse

### Les profils (`Cargo.toml`)

| Profil | Réglage | Pourquoi |
|---|---|---|
| `dev` | `debug = "line-tables-only"` | les numéros de ligne pour les erreurs, sans les symboles complets |
| `dev.package."*"` | `debug = false` | aucune information de débogage pour les dépendances |
| `dev` | `incremental = false` | le cache incrémental pesait 56 Mo pour un crate qui compile en quelques secondes |
| `dev.build-override` | `debug = false` | pas de symboles pour les scripts de build |
| `release` | `opt-level = "z"` | 1,45 Mo contre 1,68 Mo en `"s"` ; la vitesse n'a pas d'importance ici |
| `release` | `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"` | binaire minimal |
| `release` | `panic = "unwind"` (le défaut) | gardé exprès, voir ci-dessous |

**Pourquoi pas `panic = "abort"`** : un bogue de rendu ne doit pas tuer le service. Chaque requête et chaque rafraîchissement sont protégés par `catch_unwind`. Cela coûte une trentaine de Ko : un prix acceptable.

### Le dossier `target/`

Mesuré en repartant de zéro :

| | Avant réglage | Après |
|---|---|---|
| `target/` après `build` + `test` + `clippy` + `release` | 322 Mo | **229 Mo** |
| dont débogage et tests seulement | 205 Mo | **140 Mo** |

Ce qui reste est surtout fait des bibliothèques de `rustls` et `ring` (le TLS), compilées pour chaque profil. C'est le prix d'un client HTTPS ; il n'y a pas de manière honnête de le réduire davantage sans retirer le TLS.

`make size` affiche le poids du binaire et de `target/`. `make clean` le supprime. Pour garder `target/` hors du dépôt : `export CARGO_TARGET_DIR=~/.cache/abvigie-target`.

### Les dépendances

Trois dépendances directes, chacune justifiée ([stack.md](stack.md)) : `serde` et `serde_json` pour le JSON, `ureq` pour appeler GitHub. Pas de runtime `async`, pas de serveur web, pas de moteur de gabarits, pas de bibliothèque de dates : ce qu'il faut a été écrit, en peu de lignes et avec ses tests. **Ajouter une dépendance se justifie dans la PR** (R19), avec son poids en binaire.

## Tester toutes les tailles d'écran

La page est « fluide » : des unités relatives, `clamp()`, des grilles qui s'adaptent, des requêtes de média sur la largeur, la hauteur et l'orientation. Il n'y a pas trois tailles figées. Vérifié dans un vrai navigateur, **sans aucun défilement horizontal**, sur :

| Format | Taille |
|---|---|
| Petit téléphone, portrait | 320 × 568 |
| Téléphone, portrait et paysage | 390 × 844, 844 × 390 |
| Tablette, portrait et paysage | 768 × 1024, 1024 × 768 |
| Carré | 600 × 600 |
| Ordinateur | 1440 × 900 |
| Ultra-large | 2560 × 1080 |
| 4K | 3840 × 2160 |
| Très haut | 400 × 2400 |
| Très court | 1600 × 260 |
| Extrême, vérifié à la main | 200 px de large |

Aussi vérifiés : thèmes clair et sombre (`prefers-color-scheme`), mouvement réduit (`prefers-reduced-motion`), zones sûres des téléphones à encoche, couleurs forcées, impression, et navigation au clavier.

Sous 34 rem de large, la ville est masquée et la liste des projets prend le relais (chaque carte porte la miniature de son bâtiment) : une ville illisible ne sert à personne.
