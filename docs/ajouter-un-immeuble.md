# Ajouter un bâtiment

Un dépôt absent du catalogue apparaît quand même, avec le **bâtiment générique**, dans « Autres projets ». Pour lui donner son bâtiment : **un fichier SVG + une ligne** dans `server/src/catalogue.rs`.

## 1. Dessiner

Créer `server/assets/immeubles/<slug>.svg`. Ce fichier est le **contenu** d'un `<symbol viewBox="0 0 100 170">` : pas de balise `<svg>`, seulement les formes.

- **Le sol est à `y = 150`.** Les 20 dernières unités servent aux fondations ou à l'eau (comme Abyss).
- **Largeur : 100**, le bâtiment tient en général entre `x = 10` et `x = 90`.
- **Un motif discret qui renvoie au projet** : une antenne pour une messagerie, un globe pour du DNS, une croix pour la santé… Il peut être très subtil. Voir les exemples dans [identite.md](identite.md).
- **Réutiliser l'esprit des autres bâtiments** : mêmes proportions, mêmes classes de couleur, pas plus de détail que nécessaire.

### Les classes autorisées

Les couleurs vivent dans `style.css`, pas dans le dessin : un bâtiment change de thème (clair, sombre) sans qu'on le retouche.

| Classe | Rôle |
|---|---|
| `c` | corps du bâtiment |
| `o` | partie dans l'ombre (toit, bandeau, fondations) |
| `t` | contour fin, à combiner avec `c` ou `o` : `class="c t"` |
| `a` | remplissage à la **couleur propre** du projet |
| `as` | trait à la couleur propre du projet |
| `l`, `l2` | traits fins et épais, sans remplissage (échafaudage, quadrillage) |
| `g` | roue dentée : un cercle à pointillés épais |
| `p` | porte |
| `w1` à `w4` | **quatre groupes de fenêtres**, obligatoires : plus le projet est actif, plus de groupes sont allumés |

### Les règles que les tests vérifient

Pas de `style=` (la CSP l'interdit), pas de `<script>`, pas de lien, moins de 2 500 octets, et les quatre classes `w1` à `w4` présentes. Les attributs de présentation (`opacity`, `stroke-dasharray`, `rx`…) sont autorisés.

## 2. Déclarer

Dans `server/src/catalogue.rs`, ajouter une entrée à `ENTREES` :

```rust
Entree { depot: "MonDepot", slug: "mondepot", nom: "MonDepot", quartier: Web, accent: "#5fcfb4", dessin: dessin!("mondepot") },
```

| Champ | Contenu |
|---|---|
| `depot` | nom exact du dépôt GitHub (la casse est ignorée) |
| `slug` | identifiant des URL : minuscules, chiffres et `-` |
| `nom` | nom affiché |
| `quartier` | `Web`, `Systemes`, `Outillage` ou `Autres` (voir les domaines de la charte) |
| `accent` | la couleur propre du projet, `#rrggbb`, dans la palette de la page |

La règle CSS `.v-<slug> { --ac: … }` est **générée** à partir de cette ligne. `cargo test` vérifie l'unicité des slugs, le format des couleurs et la forme du dessin.

## 3. Voir le résultat

`make run`, puis ouvrir la page. Vérifier en clair et en sombre, et en petit (la miniature d'une carte fait environ 55 px de large : un détail trop fin disparaît).

## Ajouter un quartier

Ajouter une variante à `Quartier` (`model.rs`) avec son titre et son identifiant, et l'ajouter à `Quartier::TOUS`.
