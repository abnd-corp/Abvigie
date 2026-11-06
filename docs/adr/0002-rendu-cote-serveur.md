# ADR 0002 · Le HTML est fabriqué par le serveur, pas par le navigateur

Date : 6 novembre 2026 · Statut : accepté

## Contexte

Le site doit être bien lu par les moteurs de recherche (SEO) et par les assistants IA (GEO), être très léger, et rester utilisable quand le JavaScript ne s'exécute pas. Un tableau de bord qui télécharge un JSON puis construit la page dans le navigateur est **vide** pour un robot qui n'exécute pas de JavaScript, et plus lent à afficher.

## Décision

- Le serveur produit **le HTML complet** de chaque page, avec les vraies données dedans.
- **Toutes les pages sont fabriquées à l'avance** à chaque rafraîchissement et rangées dans une table : répondre revient à chercher une clé.
- Le JavaScript est un **supplément** : il ouvre la fiche d'un projet dans un panneau latéral. Sans lui, les liens ouvrent la page du projet.
- Les filtres et les options de la ville sont du **CSS** (`:has()`), sans JavaScript.
- Les mêmes données sont publiées en **JSON** (`/api/etat.json`) pour qui veut les lire autrement.

## Conséquences

- **Positif** : contenu lisible par tout robot, page affichée dès la première réponse, poids minimal, réponse en moins d'une demi-milliseconde, rien à calculer à la requête.
- **Positif** : très simple à raisonner, et peu de surface d'attaque (aucune donnée de la requête ne sert à fabriquer une page).
- **Négatif** : les données ont au plus l'âge du dernier rafraîchissement (15 minutes par défaut). Acceptable pour un tableau de bord.
- **Négatif** : toute chaîne venue de GitHub doit être échappée avant d'entrer dans une page. C'est centralisé dans `render::esc` et vérifié par des tests ([sécurité](../securite.md)).
- **À surveiller** : les pages fabriquées à l'avance occupent la mémoire de façon linéaire : 409 Ko pour 16 projets (mesuré), soit environ 20 Ko par projet en comptant la page, le fragment du panneau et la part de l'accueil et du JSON. Sans importance jusqu'à plusieurs centaines de projets.
