# Performance, poids et propreté de Rust

Le principe : **le plus léger possible, mesuré, avec un budget qui échoue aux tests s'il est dépassé.**

## Ce qu'on mesure (version de livraison, données de démonstration, 16 projets)

| Mesure | Valeur |
|---|---|
| Page d'accueil (HTML, avec toutes les données et les quêtes) | 33,8 Ko, **7,0 Ko compressés** |
| `style.css` | 16,5 Ko, 4,6 Ko compressés |
| JavaScript de la carte (9 modules) | 95,8 Ko, **30,3 Ko compressés** |
| Données de la carte (`/api/carte.json`) | 4,3 Ko, 0,9 Ko compressés, déjà incorporées dans la page |
| **Poids total d'une première visite** | environ **42 Ko compressés**, sans police, sans image (hors image de partage, que seuls les réseaux sociaux chargent), sans ressource externe |
| Visite suivante | presque rien : les modules et la feuille de style sont versionnés et mis en cache un an |
| Une fiche projet | 7,4 Ko, 2,4 Ko compressés |
| Binaire de livraison | **1,6 Mo** (TLS compris) |
| Mémoire du serveur | environ 4 Mo |
| Réponse à une requête | médiane 0,2 ms |
| Dépendances Rust (transitives comprises) | 47 crates dans `Cargo.lock`, 0 avis de sécurité |
| Dépendances JavaScript | **aucune** |

**Ce que coûte la carte.** Le JavaScript pèse 30 Ko compressés : c'est la part du jeu. Il est mis en cache longtemps et n'est chargé qu'une fois par version. Les modules ne sont pas minifiés : leurs commentaires, qui sont la documentation du code, se compressent très bien (le même code sans commentaires gagnerait environ 4 Ko compressés). **Si le poids devient un souci**, la première chose à faire est de minifier à la fabrication des pages.

### Dans le navigateur (mesuré avec un navigateur sans carte graphique, donc pessimiste)

| Mesure | Valeur |
|---|---|
| Animation d'ambiance, carte ouverte | environ **30 % d'un cœur** (dont 5 % de JavaScript, le reste est le dessin) |
| Mouvement réduit (`prefers-reduced-motion`) | **0,2 %** : la carte ne se redessine que quand on agit dessus |
| Onglet caché | 0 : l'animation s'arrête |
| Mémoire du JavaScript | environ 3 Mo, 1 100 nœuds dans la page |

L'animation d'ambiance (lumières, fumée, personnages) est plafonnée à **30 images par seconde** ; dès qu'on déplace la carte elle passe à la cadence de l'écran. Un navigateur avec carte graphique fait mieux.

La compression n'est pas faite par le service : elle revient au proxy qui est devant (Traefik, Caddy), qui sait aussi faire du brotli. Voir [deploiement.md](deploiement.md).

## Le budget (vérifié par `cargo test`)

| Élément | Limite |
|---|---|
| Page d'accueil | 14 Ko + 1,5 Ko par projet, avant compression |
| `style.css` | 20 Ko |
| JavaScript, tous modules ensemble | 110 Ko |
| `/api/carte.json` | 1 Ko + 0,8 Ko par projet |
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
| `release` | `opt-level = "z"` | un binaire plus petit d'environ 14 % qu'en `"s"` ; la vitesse n'a pas d'importance ici |
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

La page est « fluide » : des unités relatives, `clamp()`, des grilles qui se réorganisent, des requêtes de média sur la largeur, la hauteur et l'orientation. Il n'y a pas trois tailles figées. Vérifié dans un vrai navigateur, avec la politique de sécurité réelle, **sans aucune erreur ni défilement horizontal**, sur :

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

La mise en page change de forme selon l'espace :

| Espace | Disposition |
|---|---|
| Large | la carte à gauche, le panneau à droite (de 19 à 34 rem) |
| Portrait (téléphone, tablette) | la carte en haut (la moitié de la hauteur), le panneau dessous, qui défile |
| Paysage très bas (téléphone couché) | la carte et un panneau étroit côte à côte, la barre du haut réduite |
| Très étroit (moins de 20 rem) | les compteurs passent sur deux colonnes, les cartes de projet s'empilent |

Aussi vérifiés : mouvement réduit, couleurs forcées, zones sûres des téléphones à encoche, impression (la carte est retirée, le contenu s'imprime), et la page **sans JavaScript** (tout s'affiche à la suite).

Sur la carte, le cadrage initial suit la taille de la fenêtre, et les noms de projets n'apparaissent que si la carte est assez zoomée pour qu'ils restent lisibles.
