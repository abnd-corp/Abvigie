# ADR 0001 · La pile technique d'Abvigie

Date : 6 novembre 2026 · Statut : accepté

## Contexte

Abvigie est un tableau de bord public, léger, qui doit être **rapide, sûr, peu gourmand, évolutif, et bien documenté**. La stack de référence de l'orga (R37) fait de Vue + TypeScript le choix par défaut pour le web ; une techno hors de ce cercle demande un ADR.

L'équipe a écarté Vue pour ce projet : peu de composants, presque aucune réutilisation, et un framework apporterait un poids et une chaîne d'outils sans rien résoudre. Elle a retenu un **service en Rust**, pour la sécurité mémoire, la petite empreinte et un binaire unique, et sait déjà l'écrire (Abcom, Abdns).

## Décision

- **Backend : Rust**, un seul binaire, **trois dépendances directes** (`serde`, `serde_json`, `ureq`) et aucun runtime `async`.
- **Pages : HTML fabriqué côté serveur** ([ADR 0002](0002-rendu-cote-serveur.md)), CSS écrit à la main, **aucun framework**.
- **Interactivité : JavaScript natif** (`app.js`, environ 2,5 Ko), typé par des commentaires JSDoc et **vérifié par `tsc`** : on garde la sécurité des types de TypeScript sans étape de compilation, sans bundler et sans dépendance npm.
- **Aucune ressource externe** : ni police, ni CDN, ni script tiers.
- **Une structure qui évolue sans effort** : un fichier par règle, un fichier par bâtiment, un trait pour les sources ([architecture](../architecture.md)), et un guide « comment ajouter un X » pour chacun.

## Pourquoi pas TypeScript compilé

Le script fait environ 60 lignes. Un compilateur, un bundler et un `package.json` apporteraient plus de fichiers, de dépendances à auditer et de mises à jour que de code à protéger. Le typage JSDoc donne la même vérification, et `tsc` peut rester un outil de développement optionnel.

## Conséquences

- Écart à R37 (« Vue + TypeScript ») : assumé, justifié ici. Si l'interface grossit (beaucoup de composants réutilisés, état complexe), **on rouvrira cette décision** ; la séparation en modules rend le changement possible.
- Le jour où l'étape 2 demandera de l'interactivité riche, on pourra ajouter un framework pour cette seule partie sans toucher au service public.
- Une mise à jour de Rust demande de relire ses notes de version ([stack.md](../stack.md), R39).
