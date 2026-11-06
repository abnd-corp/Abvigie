# Architecture

## En une image

```
                     toutes les 15 minutes
 API publique   ┌──────────────────────────────────────────────┐
 de GitHub ───► │ sources ──► collect ──► rules ──► Etat       │
 (ou fichiers)  │                                    │         │
                └────────────────────────────────────┼─────────┘
                                                     ▼
                                    site : TOUTES les pages, déjà fabriquées
                                                     │   (remplacées d'un coup)
              navigateur ◄── http ◄── une table « chemin → page »
```

Le service fait deux choses seulement : **rafraîchir** (lire GitHub, recalculer, fabriquer les pages) et **répondre** (chercher une page dans une table). Répondre ne calcule rien : pas de rendu, pas d'allocation, pas de disque. C'est ce qui le rend léger et prévisible.

## Les modules

| Module | Rôle | Il ne connaît pas |
|---|---|---|
| `sources/` | Lire des données brutes : `GitHub` (l'API REST, avec cache `ETag`) ou `Fixtures` (des fichiers). Un trait `Source` les unifie. | les règles, le HTML |
| `collect.rs` | Transformer le brut en notions de la charte : niveau d'activité, jalon en retard, CI de référence, robot ou humain. Produit un [`Etat`](donnees.md). | le HTML, le réseau |
| `rules/` | Une règle d'alerte par fichier : elle regarde un projet et lève ou non un `Drapeau`. | le HTML, GitHub |
| `catalogue.rs` | Pour chaque dépôt connu : nom affiché, quartier, couleur, dessin. | |
| `render/` | Fabriquer les pages HTML (accueil, fiche, fragment du panneau), la ville en SVG, le SEO/GEO. | GitHub |
| `site.rs` | Fabriquer **toutes** les pages d'un `Etat` et les ranger dans une table. | |
| `http.rs` | Servir la table : un serveur HTTP/1.1 strict (`GET` et `HEAD`, rien d'autre). | tout le reste |
| `main.rs` | Brancher le tout, et relancer la lecture toutes les `ABVIGIE_RAFRAICHIR` secondes. | |

La frontière importante est entre `collect` et le reste : **GitHub n'existe que dans `sources/`**. Pour changer de source de données, on ne touche pas aux règles ni aux pages.

## Le cycle de vie

1. **Démarrage** : lecture de la source, fabrication du site. Si la lecture échoue, le service démarre quand même avec un site « dégradé » (la page d'accueil explique la situation et répond 503) et se rétablit seul.
2. **Rafraîchissement** : un fil dédié relit la source, fabrique un nouveau site complet, puis le **remplace d'un seul geste** (`RwLock<Arc<Site>>`). Une requête en cours garde son ancien site jusqu'au bout : aucune page à moitié mise à jour.
3. **Échec de rafraîchissement** : les pages précédentes sont conservées. Mieux vaut une donnée un peu ancienne que pas de page.
4. **Dépôt illisible** : un dépôt dont les tickets ou la CI ne se lisent pas ne fait pas tomber les autres. Il apparaît, et un bandeau « données partielles » le signale.

## Pourquoi le HTML est fabriqué côté serveur

Un tableau de bord rendu seulement par JavaScript est invisible pour un moteur de recherche ou un assistant IA qui ne l'exécute pas. Ici chaque page contient déjà ses données ([ADR 0002](adr/0002-rendu-cote-serveur.md)). Le JavaScript (`server/assets/app.js`, environ 2,5 Ko) ne fait qu'ouvrir la fiche dans un panneau, et tout marche sans lui.

## La ville

- Chaque projet est un `<symbol>` SVG dans un sprite inclus une fois dans la page ; chaque bâtiment est un `<use>` de ce symbole.
- Les **fenêtres** ont quatre classes (`w1` à `w4`). La classe `n0` à `n4` posée sur le lien du bâtiment allume autant de groupes : c'est du CSS pur (variables CSS qui traversent le `<use>`).
- La **couleur propre** d'un bâtiment est une variable CSS `--ac`, générée à partir du catalogue et ajoutée à la fin de la feuille de style (la CSP interdit les `style=` en ligne).
- Les **filtres** (Noms, Alertes, Tous / Avec alertes / Endormis) sont du CSS avec `:has()` : aucun JavaScript.

## Concurrence

Huit fils de travail lisent une file bornée de connexions (64). Une connexion = une requête = une réponse, puis fermeture. Au-delà de la file, les connexions sont refusées. C'est largement suffisant derrière un proxy, et il n'y a ni `async`, ni runtime, ni dépendance pour ça.
