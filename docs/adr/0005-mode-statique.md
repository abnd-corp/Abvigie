# ADR 0005 : un mode statique pour montrer Abvigie sur GitHub Pages

- **Statut** : accepté
- **Date** : 2026-11-07

## Contexte

Abvigie est un serveur Rust qui relit GitHub à intervalle régulier. GitHub Pages ne sert que des fichiers. L'équipe veut **montrer comment ça fonctionne** sans héberger de serveur, pour l'instant, et sans rafraîchir toutes les 15 minutes (le défaut devient 6 heures, réglable dans `abvigie.conf`).

## Décision

Ajouter `abvigie --export <dossier>` : le même code qui fabrique les pages pour le serveur les écrit en fichiers, puis s'arrête. Pas de second générateur : le site statique est **exactement** celui que le serveur sert.

- La source de données est un fichier lu : `server/fixtures/` pour la démonstration (avec `capture.txt`, qui fixe la date de raisonnement).
- Les liens commencent par `/` : ils sont préfixés par le dossier de l'adresse publique (`/Abvigie`) à l'export.
- La page cherche les fiches dans `projets/<nom>/partiel.html` au lieu de `?partial=1`, que des fichiers ne savent pas servir (`"statique": true` dans les données de la carte).
- `404.html` et `.nojekyll` sont écrits pour Pages ; un export **partiel** n'écrit rien.

## Conséquences

- **Positif** : une démonstration publique sans serveur ; plus tard, une CI nocturne qui lance le binaire contre GitHub et remplace les fichiers, avec la même commande (à faire, voir `docs/deploiement.md`).
- **Négatif** : pas d'en-têtes HTTP sur Pages (CSP, etc.) ; données figées tant que rien ne les remplace ; une URL publique à connaître au moment de l'export.
- **Écarté** : un second rendu côté navigateur à partir d'un JSON (SEO et GEO perdus : la page doit être lisible sans JavaScript).
