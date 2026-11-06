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
| `collect.rs` | Transformer le brut en notions de la charte : niveau d'activité, jalon en retard, CI de référence, robot ou humain, acteurs. Produit un [`Etat`](donnees.md). | le HTML, le réseau |
| `rules/` | Une règle d'alerte par fichier : elle regarde un projet et lève ou non un `Drapeau`. | le HTML, GitHub |
| `progression.rs` | Les quêtes, calculées sur les projets et leurs drapeaux ([quetes.md](quetes.md)). | le HTML |
| `catalogue.rs` | Pour chaque dépôt connu : nom affiché, quartier, couleur propre. | |
| `render/` | Fabriquer les pages HTML (écran de jeu, fiche, fragment du panneau), les données de la carte, le SEO/GEO. | GitHub |
| `site.rs` | Fabriquer **toutes** les pages d'un `Etat` et les ranger dans une table ; servir les modules JavaScript avec leur version. | |
| `http.rs` | Servir la table : un serveur HTTP/1.1 strict (`GET` et `HEAD`, rien d'autre). | tout le reste |
| `main.rs` | Brancher le tout, et relancer la lecture toutes les `ABVIGIE_RAFRAICHIR` secondes. | |
| `assets/js/` | La carte, dessinée dans le navigateur ([carte.md](carte.md)). | GitHub, Rust |

La frontière importante est entre `collect` et le reste : **GitHub n'existe que dans `sources/`**. Pour changer de source de données, on ne touche pas aux règles, aux quêtes ni aux pages. Une autre frontière : **la carte ne connaît que `/api/carte.json`**, un résumé compact et versionné de l'état, publié par `render/carte.rs`.

## Le cycle de vie

1. **Démarrage** : lecture de la source, fabrication du site. Si la lecture échoue, le service démarre quand même avec un site « dégradé » (la page d'accueil explique la situation et répond 503) et se rétablit seul.
2. **Rafraîchissement** : un fil dédié relit la source, fabrique un nouveau site complet, puis le **remplace d'un seul geste** (`RwLock<Arc<Site>>`). Une requête en cours garde son ancien site jusqu'au bout : aucune page à moitié mise à jour.
3. **Échec de rafraîchissement** : les pages précédentes sont conservées. Mieux vaut une donnée un peu ancienne que pas de page.
4. **Dépôt illisible** : un dépôt dont les tickets ou la CI ne se lisent pas ne fait pas tomber les autres. Il apparaît, et un bandeau « données partielles » le signale.
5. **Lecture partielle** (la limite de GitHub atteinte, un dépôt illisible) : elle **ne remplace pas** un site complet, parce que les alertes des projets non lus disparaîtraient et le tableau de bord afficherait « rien à signaler » faute de données. Les pages précédentes sont gardées pour 3 lectures partielles d'affilée ; si le problème dure, l'état partiel est accepté et le dit (`site::faut_remplacer`).

## Pourquoi le HTML est fabriqué côté serveur

Un tableau de bord rendu seulement par JavaScript est invisible pour un moteur de recherche ou un assistant IA qui ne l'exécute pas. Ici chaque page contient déjà ses données ([ADR 0002](adr/0002-rendu-cote-serveur.md)) : les quêtes, la liste des projets, le journal, les compteurs. La carte est un supplément ([ADR 0004](adr/0004-carte-isometrique-sur-canvas.md)) : sans elle, tout se lit, à la suite, avec une feuille de style de secours (`sansjs.css`, chargée dans un `<noscript>`).

## Le panneau latéral

Quatre onglets (Fiche, Quêtes, Projets, Journal), faits de cases d'options cachées et de CSS : **ils marchent sans JavaScript**. Le JavaScript ajoute seulement la fiche d'un projet, qui prend la place des onglets quand on clique un bâtiment (`/projets/<nom>?partial=1`), et un retour.

## Concurrence

Huit fils de travail lisent une file bornée de connexions (64). Une connexion = une requête = une réponse, puis fermeture. Au-delà de la file, les connexions sont refusées. C'est largement suffisant derrière un proxy, et il n'y a ni `async`, ni runtime, ni dépendance pour ça.
