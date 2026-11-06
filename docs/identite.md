# Identité visuelle et crédits

## Le nom

**Abvigie** : la *vigie* est le marin de guet, en haut du mât ou dans la tour, qui surveille et donne l'alerte. Le nom suit la logique des autres projets (« Ab » + un mot), n'a aucun homonyme sur GitHub, npm ou crates.io (vérifié le 6 novembre 2026), et dit ce que fait le projet.

## Le logo

Une **tour de vigie** dont le faisceau balaie la ville. Il fonctionne en petit (favicon de 16 px) comme en grand, et reprend les couleurs de la ville : une lanterne ambre sur une tour claire.

- `server/assets/favicon.svg` : sur fond bleu nuit, pour l'onglet du navigateur.
- `server/assets/logo.svg` : en ligne dans l'en-tête, il suit le thème (clair ou sombre).

Deux autres pistes ont été dessinées et mises de côté, dans `docs/identite/` :

| Piste | Idée | Pourquoi pas |
|---|---|---|
| `logo-b-oeil.svg` | un œil dont l'iris est la lanterne | trop générique : beaucoup d'outils de surveillance ont un œil |
| `logo-c-ville.svg` | cinq bâtiments aux fenêtres allumées | joli, mais illisible à 16 px et redondant avec la page |

## La palette

Reprise de la maquette de l'équipe, avec un thème clair en plus.

| Rôle | Sombre | Clair |
|---|---|---|
| Fond | `#121830` | `#eef1fb` |
| Quartier | `#1b2340` | `#dfe4f5` |
| Carte | `#1a2140` | `#ffffff` |
| Texte | `#ecebf5` | `#1b2140` |
| Texte discret | `#9ba4c8` | `#4a5580` |
| Fenêtre allumée | `#f4b860` | `#f0a63a` |
| Alerte / attention / info | `#ff7d6e` / `#f4b860` / `#5fcfb4` | `#c93a2a` / `#965600` / `#17806a` |

Les contrastes de texte sont au moins de 4,5 : 1 dans les deux thèmes (calculés sur chaque paire texte et fond utilisée ; le plus bas est 4,6 : 1, le plus haut 15,7 : 1). Les couleurs vivent en variables CSS (`:root`) dans `style.css`.

## Les bâtiments

Chaque projet a son bâtiment, dessiné pour ce projet, dans le même style (formes simples, contour fin, quatre groupes de fenêtres) et avec **une couleur propre**. Chacun porte un motif qui renvoie au projet, parfois très discrètement :

| Projet | Motif |
|---|---|
| Abatelier | toit en dents de scie d'atelier, cheminée qui fume, roue dentée sur la façade |
| Abcarte | la silhouette de l'Hexagone, quadrillée en départements |
| Abcom | une antenne et ses ondes, un cadenas pour porte (messagerie chiffrée) |
| Abdns | une façade en bouclier, surmontée d'un globe, une serrure pour porte |
| AbFlow | une maison basse, des cadres photo pour fenêtres, un ruisseau, trois baies (framboise) |
| Abloue | une épingle de carte au-dessus du toit |
| Abnd | des blocs de tailles différentes empilés (des modules), un drapeau |
| AbSante | un bâtiment de soin surmonté d'une croix, une grille de territoires |
| Abvance | une façade en ticket aux bords crantés, un escalier qui monte |
| Abvigie | le phare lui-même, son faisceau et ses bandes |
| Abwatch | une tour de guet, son faisceau, des conteneurs au pied |
| AbX2T | deux blocs côte à côte reliés par une flèche (un format vers un autre) |
| Abyss | une tour qui s'enfonce sous l'eau, une pièce pour porte |
| Gouvernance | un palais à colonnes et sa balance |
| Org-config | une tour de contrôle et son engrenage |
| Template | un bâtiment sous échafaudage, une grue |
| *(autres projets)* | un bâtiment générique, en attendant le sien |

Voir [ajouter-un-immeuble.md](ajouter-un-immeuble.md) pour en ajouter.

## L'image de partage

`server/assets/og.png` (1200 × 630, 82 Ko) montre la ville entière avec le logo. Sa source est `docs/identite/og.html`, une page autonome : l'ouvrir dans un navigateur à 1200 × 630 et capturer la fenêtre. Elle se régénère quand un bâtiment change.

## Crédits et licences

**Aucun élément graphique ne vient de l'extérieur.** Le logo, le favicon, l'image de partage et les dix-sept bâtiments sont des dessins originaux faits pour Abvigie, publiés sous la licence du projet (AGPL-3.0-or-later). Il n'y a ni police, ni icône, ni illustration, ni photo tierce : la typographie est celle du système (`system-ui`).

Si un élément venu d'ailleurs devait un jour être ajouté, sa licence serait vérifiée avant, et il serait listé ici avec son auteur, sa source et sa licence, ainsi que dans [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md).

La maquette qui a inspiré l'idée (« ABND · QG », une ville vue de dessus) utilisait les polices *Pixelify Sans*, *Atkinson Hyperlegible* et *JetBrains Mono*, publiées sous licence SIL OFL. **Elles ne sont pas reprises** : la page n'embarque aucune police.
