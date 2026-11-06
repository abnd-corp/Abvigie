# Identité visuelle et crédits

## Le nom

**Abvigie** : la *vigie* est le marin de guet, en haut du mât ou dans la tour, qui surveille et donne l'alerte. Le nom suit la logique des autres projets (« Ab » + un mot), n'a aucun homonyme sur GitHub, npm ou crates.io (vérifié le 6 novembre 2026), et dit ce que fait le projet.

## Le style

Une **ville de nuit vue en isométrique**, dans l'esprit de la maquette de l'équipe : un fond bleu nuit étoilé, des bâtiments en volume aux fenêtres allumées en ambre, des fanions, des grues, des parcs. Les couleurs viennent de la maquette ; le dessin des bâtiments est plus travaillé (dégradés, ombres portées, détails animés, portes, équipements de toit).

Un seul thème, **sombre** : la ville est une ville de nuit. Une version claire n'aurait de sens ni pour les lumières ni pour l'ambiance.

| Rôle | Couleur |
|---|---|
| Fond | `#121830` |
| Panneau | `#181f40` |
| Carte, tuile | `#1d2550` |
| Trait | `#323e6b` |
| Texte | `#ecebf5` |
| Texte discret | `#a3acd0` |
| Fenêtre allumée | `#ffc75f` |
| Alerte, attention, info | `#ff7d6e`, `#f4b860`, `#5fcfb4` |

Les contrastes de texte sont au moins de 4,5 : 1 (vérifiés sur chaque paire texte et fond utilisée).

**Chaque projet a sa couleur propre** (`accent`, dans `catalogue.rs`), utilisée pour son bâtiment, sa vignette, le liseré de sa carte et son étiquette.

## Le logo

Une **tour de vigie** dont le faisceau balaie la ville. Il fonctionne en petit (favicon de 16 px) comme en grand : une lanterne ambre sur une tour claire.

- `server/assets/favicon.svg` : sur fond bleu nuit, pour l'onglet du navigateur.
- `server/assets/logo.svg` : en ligne dans la barre du haut.

Deux autres pistes ont été dessinées et mises de côté, dans `docs/identite/` : un œil dont l'iris est la lanterne (trop générique : beaucoup d'outils de surveillance ont un œil), et cinq bâtiments aux fenêtres allumées (illisible à 16 px, et redondant avec la carte).

## Les bâtiments

Chaque projet a son bâtiment, **dessiné pour lui par du code** (`assets/js/batiments.js`), dans le même style (volumes simples, fenêtres, contour fin) avec **une couleur propre** et **un détail animé qui renvoie au projet**, parfois très discrètement :

| Projet | Motif |
|---|---|
| Abatelier | un atelier : toit en dents de scie, cheminée qui fume, engrenage qui tourne sur la façade |
| Abcarte | la silhouette de l'Hexagone en toit, quadrillée en départements, une épingle |
| Abcom | une tour d'antenne dont les ondes rayonnent, un cadenas sur la façade (messagerie chiffrée) |
| Abdns | un dôme surmonté d'un globe qui tourne, un bouclier sur la façade |
| AbFlow | une maison basse, une rivière de petits fichiers qui descend le courant, trois voyants (framboise) |
| Abloue | une épingle de carte qui flotte au-dessus du toit |
| Abnd | des blocs de tailles différentes empilés (des modules) |
| AbSante | un bâtiment de soin surmonté d'une croix lumineuse qui bat |
| Abvance | un bâtiment en escalier qui monte, un fanion au sommet |
| Abvigie | le phare lui-même, dont le faisceau balaie la ville |
| Abwatch | une tour de guet et son projecteur qui tourne, des conteneurs au pied |
| AbX2T | deux blocs reliés par des flèches qui défilent (un format qui devient un autre) |
| Abyss | une tour qui s'enfonce dans l'eau, une pièce qui tourne au sommet |
| Gouvernance | un palais à colonnes, son fronton et sa balance |
| Org-config | une tour de contrôle et son engrenage |
| Template | un bâtiment sous échafaudage, avec sa grue |
| *(autres projets)* | un bâtiment généré, toujours le même pour un même nom |

Voir [ajouter-un-immeuble.md](ajouter-un-immeuble.md) pour en ajouter.

## L'image de partage

`server/assets/og.png` (1200 × 630, 67 Ko) montre la vraie carte avec le logo et le nom. Elle a été produite en ouvrant la page à 1200 × 630, en masquant l'interface, en déplaçant la ville vers le bas et la droite pour laisser la place au titre, puis en capturant la fenêtre ; l'image a ensuite été réduite (`pngquant`, `oxipng`). Elle se refait de la même façon quand la ville change beaucoup.

## Crédits et licences

**Aucun élément graphique ne vient de l'extérieur.** Le logo, le favicon, l'image de partage et tous les bâtiments, personnages, arbres et détails sont des dessins originaux faits pour Abvigie, par du code, et publiés sous la licence du projet (AGPL-3.0-or-later). Il n'y a ni police, ni icône, ni illustration, ni photo, ni sprite tiers : la typographie est celle du système (`system-ui` et sa version à chasse fixe), et les noms sur la carte sont écrits par le canvas.

Si un élément venu d'ailleurs devait un jour être ajouté, sa licence serait vérifiée avant, et il serait listé ici avec son auteur, sa source et sa licence, ainsi que dans [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md).

La maquette qui a inspiré le projet (« ABND · QG ») utilisait les polices *Pixelify Sans*, *Atkinson Hyperlegible* et *JetBrains Mono*, publiées sous licence SIL OFL. **Elles ne sont pas reprises** : la page n'embarque aucune police. Le dessin des bâtiments de la maquette n'a pas non plus été copié : il s'en inspire pour le style, pas pour le tracé.
