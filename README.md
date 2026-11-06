# Abvigie

Le tableau de bord public de l'organisation [Abnd](https://github.com/abnd-corp), sous la forme d'**une petite ville en 3D que l'on explore**.

- **Chaque bâtiment est un projet**, dessiné pour lui, avec un détail animé qui renvoie à ce qu'il fait : une antenne dont les ondes rayonnent pour Abcom, un phare dont le faisceau balaie la ville pour Abvigie, une croix lumineuse pour AbSante, un globe qui tourne pour Abdns.
- **Les fenêtres s'allument selon l'activité**, et un projet sans activité depuis 2 mois s'éteint (et se met à dormir : des « z » montent de son toit).
- **Les fanions et les pointillés** montrent ce qui est à régler selon la [charte](https://github.com/abnd-corp/gouvernance/blob/main/CHARTE.md) : licence, description, topics, jalons, intégration continue, projets dormants. Une grue entoure un chantier à débloquer.
- **Des personnages se promènent** : les personnes et les robots qui travaillent dans l'organisation, vus depuis leurs issues et pull requests.
- **La ville a un niveau et des quêtes.** Les quêtes sont les objectifs de la charte à atteindre sur tous les projets ; elles se complètent d'elles-mêmes quand les faits changent. Survoler une quête éclaire sur la carte les projets qui lui restent à faire.

> **État : une idée, pas encore une version.** Le prototype fonctionne en local avec des données de démonstration. Il n'y a pas encore de CI, d'image Docker ni d'hébergement : tout cela viendra avec la V0.1 ([issue #1](https://github.com/abnd-corp/Abvigie/issues/1)).

## Essayer

```bash
make run          # http://127.0.0.1:8080, avec des données de démonstration
```

Prérequis : Rust (voir `rust-version` dans `Cargo.toml`). Rien d'autre : pas de Node, pas de base de données, pas de réseau.

Glisser pour se déplacer, molette ou pincement pour zoomer, cliquer un bâtiment pour sa fiche. Au clavier : flèches, `+` et `-`.

Pour lire GitHub en direct au lieu des données de démonstration : `cargo run`. Sans jeton, GitHub n'accepte que 60 requêtes par heure et une seule lecture en consomme 33 : c'est bon pour essayer une fois, pas pour tenir ([le déploiement](docs/deploiement.md)).

## Ce qu'il y a dedans

Un seul binaire Rust (environ 1,6 Mo) qui lit l'API publique de GitHub, calcule l'état de chaque projet, fabrique toutes les pages en HTML et les sert. La carte est dessinée dans le navigateur par du JavaScript natif, sans bibliothèque. Pas de base de données, pas de framework, pas de police ni de ressource externe.

**Tout le contenu est aussi dans le HTML** (quêtes, projets, journal) : la page se lit sans JavaScript, et les moteurs de recherche et les assistants IA la lisent telle quelle. La carte est un supplément.

| Besoin | Où |
|---|---|
| Comprendre comment c'est fait | [docs/architecture.md](docs/architecture.md) |
| La carte : le monde, les bâtiments, la caméra | [docs/carte.md](docs/carte.md) |
| Les quêtes, l'XP et le niveau | [docs/quetes.md](docs/quetes.md) |
| Le format des données (`/api/etat.json`, `/api/carte.json`) | [docs/donnees.md](docs/donnees.md) |
| Les règles d'alerte et leur lien avec la charte | [docs/regles.md](docs/regles.md) |
| **Ajouter** une règle, un bâtiment, une quête, une source | [règle](docs/ajouter-une-regle.md) · [bâtiment](docs/ajouter-un-immeuble.md) · [quête](docs/quetes.md#ajouter-une-quête) · [source](docs/ajouter-une-source.md) |
| Sécurité | [docs/securite.md](docs/securite.md) |
| SEO et GEO | [docs/seo-geo.md](docs/seo-geo.md) |
| Poids, vitesse, taille de `target/` | [docs/performance.md](docs/performance.md) |
| Logo, style, crédits | [docs/identite.md](docs/identite.md) |
| Mettre en ligne | [docs/deploiement.md](docs/deploiement.md) |
| Versions et dépendances | [docs/stack.md](docs/stack.md) · [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) |
| Les décisions prises | [docs/adr/](docs/adr/) |
| La suite, avec des données sensibles | [docs/etape-2.md](docs/etape-2.md) |

## Organisation du dépôt

```
server/
  src/               le code Rust : un module par responsabilité
    sources/         d'où viennent les données (GitHub, fichiers locaux)
    rules/           une règle d'alerte par fichier
    render/          les pages HTML, les données de la carte, le SEO
    progression.rs   les quêtes, l'XP et le niveau
    collect.rs       transforme les données brutes en notions de la charte
    site.rs          fabrique toutes les pages d'un coup
    http.rs          le serveur HTTP, strict et minuscule
  assets/
    js/              la carte, en modules (un fichier par responsabilité)
    style.css        la mise en page, de 320 px à la 4K
    …                logo, favicon, image de partage
  fixtures/          des données publiques capturées, pour le développement et les tests
docs/                toute la documentation
scripts/             la capture des données de démonstration
```

## Commandes

`make help` les liste. Les trois obligatoires de la charte (R21) : `make build`, `make test`, `make lint`. Avant une PR : `make check` (R4).

## Contribuer

Une issue d'abord, une branche `feature/…`, une PR vers `dev` : voir [CONTRIBUTING.md](CONTRIBUTING.md). Chaque nouvelle dépendance se justifie dans la PR (R19) : ce projet veut rester petit.

## Licence

[AGPL-3.0-or-later](LICENSE). Les dépendances et leurs licences sont dans [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). Aucun élément graphique ne vient de l'extérieur : le logo et les bâtiments sont dessinés pour ce projet, par du code.
