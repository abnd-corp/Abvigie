# Les bâtiments

Chaque fichier `.svg` est le **contenu** d'un `<symbol viewBox="0 0 100 170">` : le dessin d'un bâtiment dont le sol est à `y = 150`
(les 20 dernières unités servent aux fondations ou à l'eau, comme pour Abyss). Le serveur les rassemble dans un sprite
inclus une seule fois dans la page.

## Règles de dessin

Pas de `style=`, pas de `<script>`, pas de lien, moins de 2 500 octets, uniquement ces classes (leurs couleurs sont dans `style.css`) :

| Classe | Rôle |
|---|---|
| `c` | corps du bâtiment |
| `o` | partie dans l'ombre (toit, bandeau, fondations) |
| `t` | contour fin, à combiner avec `c` ou `o` |
| `a` | remplissage à la couleur propre du projet |
| `as` | trait à la couleur propre du projet |
| `l`, `l2` | traits fins et épais, sans remplissage |
| `g` | roue dentée (cercle à pointillés épais) |
| `p` | porte |
| `w1` … `w4` | quatre groupes de fenêtres, allumés selon l'activité du projet |

Les quatre classes `w1` à `w4` sont **obligatoires** (un test le vérifie) : plus le projet est actif, plus de groupes sont allumés.
Chaque dessin porte un motif discret qui renvoie à son projet. Voir `docs/identite.md`.
