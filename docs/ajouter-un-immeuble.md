# Ajouter un bâtiment

Un dépôt absent du catalogue apparaît quand même, avec un **bâtiment généré** (une forme simple, toujours la même pour un même nom), dans « Autres projets ». Pour lui donner son bâtiment : **une entrée dans `catalogue.rs`** (nom, quartier, couleur) et **une entrée dans `batiments.js`** (la forme), sous le même `slug`.

## 1. Déclarer le projet (`server/src/catalogue.rs`)

```rust
Entree { depot: "MonDepot", slug: "mondepot", nom: "MonDepot", quartier: Web, accent: "#5fcfb4" },
```

| Champ | Contenu |
|---|---|
| `depot` | nom exact du dépôt GitHub (la casse est ignorée) |
| `slug` | identifiant des URL : minuscules, chiffres et `-` |
| `nom` | nom affiché |
| `quartier` | `Web`, `Systemes`, `Outillage` ou `Autres` (voir les domaines de la charte) |
| `accent` | la couleur propre du projet, `#rrggbb` |

`cargo test` vérifie l'unicité des slugs et le format des couleurs. La règle CSS `.v-<slug> { --ac: … }` est générée à partir de cette ligne.

## 2. Décrire la forme (`server/assets/js/batiments.js`)

Une entrée de `DESCRIPTIONS`, sous le même `slug` : des **volumes** et des **détails**.

```js
mondepot: {
  volumes: [
    { t: 'box', x: 0.4, y: 0.4, w: 2.2, d: 2.2, h: 46 },                       // le corps
    { t: 'gable', x: 0.4, y: 0.4, w: 2.2, d: 2.2, z: 46, h: 16, crete: 'x', c: 'toit' },   // un toit
  ],
  deco: ['antenne:1.5,1.5,62'],                                                 // un détail animé
},
```

Les positions sont en **tuiles, relatives à la parcelle** (qui fait 3 × 3), les hauteurs en **pixels**.

### Les volumes

| `t` | Forme | Champs |
|---|---|---|
| `box` | une boîte (avec ses fenêtres) | `x y w d h`, `z` (le pied, 0 par défaut) |
| `cyl` | un cylindre | `x y w h z` (le rayon est la moitié de `w`) |
| `gable` | un toit à deux pentes | `x y w d h z`, `crete: 'x'` ou `'y'` |
| `pyr` | une pyramide | `x y w d h z` |
| `dome` | une demi-sphère | `x y w h z` |

`c` choisit la teinte : `corps` (par défaut), `accent` (la couleur propre du projet), `toit` (corps teinté d'accent), `clair`, `sombre`, `lampe` (ambre). `fen: false` retire les fenêtres d'une boîte.

### Les détails animés (`deco`)

Une chaîne `nom:arguments`. Ceux qui existent : `fumee`, `engrenage`, `engrenage-haut`, `hexagone`, `epingle`, `antenne`, `cadenas`, `bouclier`, `globe`, `croix`, `fanion-haut`, `faisceau`, `projecteur`, `convoyeur`, `eau`, `piece`, `canal`, `baies`, `colonnes`, `balance`, `echafaudage`, `grue`, `clim`. Chacun est une petite fonction de `DETAILS` dans `batiments.js`, que l'on copie pour en inventer un autre. Elle reçoit le contexte de dessin et l'heure : **tout ce qui bouge se calcule à partir de l'heure, sans état**, donc sans fuite ni usure.

## Faire un bâtiment qui plaît

- **Un motif discret qui renvoie au projet** : une antenne pour une messagerie, un globe pour du DNS, une croix pour la santé. Il peut être très subtil. Voir la table dans [identite.md](identite.md).
- **Garder les proportions** des autres : 2 à 3 tuiles de large, 30 à 100 pixels de haut.
- **Une seule chose animée**, pas trois : la ville est plus lisible.
- **Penser au petit** : la vignette d'une carte fait environ 90 pixels de large, un détail trop fin y disparaît.
- Ne jamais rien tirer **au hasard** : utiliser `hash(slug)`, pour que la ville soit la même à chaque chargement.

## 3. Voir le résultat

`make run`, puis la page : le bâtiment sur la carte, sa vignette dans l'onglet Projets, et sa fiche. Vérifier aussi **éteint** (corps sombre, fenêtres noires) et **actif**. `make check` vérifie les types.

## Ajouter un quartier

Ajouter une variante à `Quartier` (`model.rs`) avec son titre et son identifiant, et à `Quartier::TOUS`, puis une couleur de sol dans `COULEURS_QUARTIER` (`monde.js`).
