# SEO et GEO

**SEO** : être bien lu par les moteurs de recherche. **GEO** (*generative engine optimization*) : être bien lu et bien cité par les assistants IA. Les deux demandent la même chose : **une page dont le contenu est dans le HTML**, rapide, structurée, et décrite par des données lisibles par une machine.

## Ce qui est en place

| Élément | Où | Pourquoi |
|---|---|---|
| **Contenu dans le HTML** | `render/` | tout est rendu côté serveur : un robot qui n'exécute pas de JavaScript voit les projets, leurs alertes et leurs chiffres |
| Une **page par projet** | `/projets/<slug>` | une URL stable à indexer et à citer par projet |
| HTML sémantique | `header`, `main`, `section`, `article`, `nav`, `footer`, titres `h1` à `h3` sans saut | le plan du document est lisible par une machine |
| `<title>` et `description` uniques | chaque page | un titre et un résumé propres à chaque projet |
| URL canonique | `<link rel="canonical">` | évite les doublons (`?partial=1`, `?v=…`) |
| **JSON-LD** schema.org | `render/seo.rs` | `WebSite`, `Organization`, `ItemList` sur l'accueil ; `SoftwareSourceCode` par projet (langage, licence SPDX, dépôt, date de modification, mots-clés) |
| Open Graph et Twitter Card | `<head>` | un aperçu soigné quand on colle le lien (`/assets/og.png`, 1200 × 630) |
| **`/sitemap.xml`** | généré | tous les projets, avec leur date de dernière modification |
| **`/robots.txt`** | généré | tout est ouvert ; l'API est exclue ; le plan du site est annoncé |
| **`/llms.txt`** | généré | un résumé Markdown du site à destination des assistants IA : ce que c'est, les chiffres du jour, la liste des projets avec une phrase chacun, comment lire les signaux |
| **`/api/etat.json`** | généré | les mêmes données, structurées et versionnées ([donnees.md](donnees.md)) |
| `lang="fr"`, `og:locale` | `<head>` | langue déclarée |
| Vitesse | [performance.md](performance.md) | 13,5 Ko compressés pour une première visite : très bon pour les signaux de performance |
| Accessibilité | contrastes, clavier, `aria`, lien d'évitement | un site accessible est mieux lu, par les humains comme par les machines |
| `/.well-known/security.txt` | généré | signal de sérieux, et utile aux chercheurs en sécurité |

## Ce qu'il faut régler au déploiement

**`ABVIGIE_URL`** : l'adresse publique, sans `/` final (par exemple `https://abvigie.exemple.org`). Elle sert aux URL canoniques, au plan du site, au JSON-LD, à `robots.txt` et à l'image de partage. Par défaut, c'est `http://localhost:8080` : **à ne pas oublier en production**.

## Comment vérifier

```bash
curl -s http://127.0.0.1:8080/ | grep -E '<title>|canonical|og:title|ld\+json'
curl -s http://127.0.0.1:8080/robots.txt
curl -s http://127.0.0.1:8080/llms.txt
curl -s http://127.0.0.1:8080/sitemap.xml | head
```

Une fois en ligne : le [test des résultats enrichis](https://search.google.com/test/rich-results), le [validateur schema.org](https://validator.schema.org), et l'inspection d'URL de Google Search Console pour demander l'indexation.

## Bonnes pratiques à garder

- **Le contenu d'abord.** Ne jamais déplacer un chiffre ou un nom dans du JavaScript : un robot ne le verrait plus.
- **Des phrases complètes** dans les messages de drapeaux et les descriptions : c'est ce qu'un assistant citera.
- **Des faits datés.** Chaque page dit quand ses données ont été lues ; `llms.txt` donne la date du jour.
- **Pas de contenu caché** ni de texte écrit pour les robots seuls : tout ce qui est dans le HTML est lisible par une personne.

## Ce qui n'est pas fait

- Pas d'inscription à un outil de webmaster (Search Console, Bing) : c'est une action sur le domaine, à faire quand il sera choisi.
- Pas de version anglaise : le public est francophone. Si on en veut une, la règle de langue de l'orga s'applique (anglais d'abord, français en second) et `hreflang` devra être ajouté.
