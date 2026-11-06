# ADR 0004 · La carte est une ville isométrique dessinée sur un canvas

Date : 7 novembre 2026 · Statut : accepté

## Contexte

La maquette d'origine de l'équipe (« ABND · QG ») était une **petite ville isométrique que l'on explore** : des bâtiments en volume dont les fenêtres s'allument, des personnages, des chantiers avec leurs grues, un niveau et des quêtes. Le premier jet d'Abvigie était, par erreur, une page de liste avec des silhouettes de bâtiments plates : il ne correspondait pas à ce que l'équipe voulait. Ce n'était pas une question de style mais de nature du produit : un tableau de bord pour être lu, ou un petit jeu pour être exploré.

Les contraintes restent celles de l'ADR 0001 : léger, sûr, sans framework, évolutif, bien documenté, lisible par les moteurs de recherche et les assistants IA.

## Décision

- **La carte est un canvas 2D**, en projection isométrique dessinée à la main : la « 3D » est une illusion obtenue avec des losanges, des dégradés et des ombres. **Pas de WebGL**, pas de Three.js ni d'autre bibliothèque.
- **Le code est découpé en modules natifs** (`assets/js/`), un par responsabilité : projection, bâtiments, monde, personnages, caméra, chef d'orchestre, interface, vignettes. [carte.md](../carte.md) les décrit.
- **Un bâtiment est décrit par des données** (des volumes assemblés et des détails nommés), pas par du code spécifique à chaque projet. Ajouter un bâtiment = ajouter une entrée.
- **Rien d'essentiel n'est réservé à la carte.** Tout ce qu'elle montre existe aussi en HTML (quêtes, projets, journal, compteurs), rendu par le serveur ([ADR 0002](0002-rendu-cote-serveur.md)). La carte est un supplément : sans JavaScript, la page se lit à la suite ; un robot la lit comme avant.
- **Un niveau et des quêtes**, calculés côté serveur sur des faits publics ([quetes.md](../quetes.md)), pour que le jeu ne mente pas et soit lisible par tous. L'XP est celui de la ville, jamais d'une personne (R27).
- **La carte ne lit que `/api/carte.json`**, un résumé compact et versionné, incorporé dans la page.
- **L'animation est économe** : 30 images par seconde en ambiance, 60 quand on agit, arrêt quand l'onglet est caché, aucune animation en mouvement réduit.

## Pourquoi pas WebGL ou une bibliothèque 3D

Une vraie 3D demanderait une bibliothèque de plusieurs centaines de Ko, un contexte WebGL (indisponible ou lent sur certains appareils), et une gestion de scène, pour une ville de 16 bâtiments qu'on regarde de dessus. L'isométrique en 2D donne le même rendu, pèse presque rien et marche partout.

## Conséquences

- **Positif** : une carte vivante (lumières, fumée, personnages, grues, faisceaux), une exploration au doigt ou à la souris, un rendu qui ressemble à la maquette, et pas une seule dépendance.
- **Positif** : l'ajout d'un projet, d'un détail animé ou d'une quête est local et documenté.
- **Négatif** : environ 95 Ko de JavaScript (30 Ko compressés), contre 2,5 Ko avant. C'est le prix du jeu, mis en cache longtemps ([performance.md](../performance.md)).
- **Négatif** : un canvas est opaque pour les technologies d'assistance et pour les robots : d'où la règle « rien d'essentiel n'y est réservé », et un panneau complet en HTML.
- **Négatif** : l'animation consomme du processeur (environ 30 % d'un cœur dans un navigateur sans carte graphique). Elle est plafonnée et arrêtée dès que possible.
- **À surveiller** : au-delà d'une cinquantaine de projets, la ville devra gagner en place (plus d'étagères) et le tri des objets à chaque image deviendra à optimiser.
