# Étape 2 : la version avancée, avec des informations sensibles

> **À faire plus tard, sans échéance.** Rien de sensible n'entre dans la version actuelle, qui est **entièrement publique** (R13). Ce document garde la trace de ce qui a été décidé et de ce qu'il faut réfléchir avant de commencer.

## L'idée

Une seconde vue, **protégée par authentification**, qui montre en plus ce que seule l'équipe doit voir : clients, comptes, domaines et leurs expirations, outils et licences, déploiements (quel projet, quelle version, chez quel client). Ces informations vivent dans le dépôt privé `inventaire` (R13, R27 à R32).

**Sans exigence d'actualité** : une mise à jour manuelle ou ponctuelle suffit. Ce n'est pas une surveillance temps réel.

## Ce qui est décidé

- La partie publique **reste publique et ne change pas** : elle ne reçoit jamais de donnée privée, même agrégée par erreur.
- Les données sensibles ne sont **jamais** dans ce dépôt, ni dans une image, ni dans `server/fixtures/`.
- L'accès est **nominatif et protégé** (R8, R11) : pas de lien secret, pas de mot de passe partagé.

## Ce qu'il faut décider avant de commencer

| Question | Pistes |
|---|---|
| Où vit la vue privée ? | un **second service** (binaire ou dépôt séparé) plutôt qu'une route cachée du service public : un bogue public ne doit pas pouvoir exposer du privé |
| Comment l'authentifier ? | au niveau du proxy (Traefik ou Caddy avec un fournisseur d'identité), ce qui évite d'écrire de l'authentification dans Abvigie |
| Comment lire `inventaire` ? | une nouvelle `Source` ([ajouter-une-source.md](ajouter-une-source.md)) avec un jeton qui n'existe que pour elle, en lecture seule |
| Quel format côté `inventaire` ? | un fichier de données structuré, versionné, que la source lit ; c'est `inventaire` qui définit sa forme |
| Quelle fraîcheur ? | un rafraîchissement manuel (un bouton, un signal) ou quotidien suffit |
| Où est servi le résultat ? | jamais sur GitHub Pages ni sur un hébergement public ; derrière l'authentification, sur l'infrastructure de l'orga |
| Que montre-t-on exactement ? | arbitrer avec Rudy : le minimum utile, pas un export de l'inventaire |

## Ce qui est déjà prêt dans l'architecture

- Le trait `Source` isole la provenance des données : une source de plus ne touche ni aux règles ni aux pages.
- Les règles sont des fichiers isolés : « domaine qui expire dans 30 jours » (R30) ou « client sans déploiement tracé » (R32) seraient de nouvelles règles.
- Le catalogue et le sprite supportent de nouveaux types de bâtiments (un « quartier » Clients, par exemple).

## Ce qui reste à construire

Un modèle de données pour clients, comptes, domaines et déploiements ; une source `inventaire` ; l'authentification ; un second jeu de pages ; et leurs règles. Tout cela est **hors périmètre de la V0.1**.
