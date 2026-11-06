# Abvigie

Le tableau de bord public de l'organisation [Abnd](https://github.com/abnd-corp) : une ville dont **chaque bâtiment est un projet**.

- Les **fenêtres allumées** montrent l'activité récente ; un projet sans activité depuis 2 mois s'éteint.
- Un **fanion** signale une chose à régler selon la [charte](https://github.com/abnd-corp/gouvernance/blob/main/CHARTE.md) : licence, description, topics, jalons, intégration continue, projet dormant.
- Des **pointillés** entourent un chantier à débloquer (issue en retard, pull request qui attend).
- Un clic ouvre la **fiche du projet** : alertes, santé, quêtes (issues et PR), journal de la CI.

> **État : une idée, pas encore une version.** Le prototype fonctionne en local avec les données de démonstration. Il n'y a pas encore de CI, d'image Docker ni d'hébergement : tout cela viendra avec la V0.1 ([issue #1](https://github.com/abnd-corp/Abvigie/issues/1)).

## Essayer

```bash
make run          # http://127.0.0.1:8080, avec des données de démonstration
```

Prérequis : Rust (voir `rust-version` dans `Cargo.toml`). Rien d'autre : pas de Node, pas de base de données, pas de réseau.

Pour lire GitHub en direct au lieu des données de démonstration : `cargo run`. Sans jeton, GitHub n'accepte que 60 requêtes par heure et une seule lecture en consomme 33 : c'est bon pour essayer une fois, pas pour tenir ([le déploiement](docs/deploiement.md)).

## Ce qu'il y a dedans

Un seul binaire Rust (environ 1,5 Mo) qui lit l'API publique de GitHub, calcule l'état de chaque projet, fabrique toutes les pages en HTML et les sert. Pas de base de données, pas de framework, pas de bibliothèque JavaScript, pas de police ni de ressource externe.

| Besoin | Où |
|---|---|
| Comprendre comment c'est fait | [docs/architecture.md](docs/architecture.md) |
| Le format des données (`/api/etat.json`) | [docs/donnees.md](docs/donnees.md) |
| Les règles d'alerte et leur lien avec la charte | [docs/regles.md](docs/regles.md) |
| **Ajouter** une règle, un bâtiment, une source | [règle](docs/ajouter-une-regle.md) · [bâtiment](docs/ajouter-un-immeuble.md) · [source](docs/ajouter-une-source.md) |
| Sécurité | [docs/securite.md](docs/securite.md) |
| SEO et GEO | [docs/seo-geo.md](docs/seo-geo.md) |
| Poids, vitesse, taille de `target/` | [docs/performance.md](docs/performance.md) |
| Logo, bâtiments, crédits | [docs/identite.md](docs/identite.md) |
| Mettre en ligne | [docs/deploiement.md](docs/deploiement.md) |
| Versions et dépendances | [docs/stack.md](docs/stack.md) · [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) |
| Les décisions prises | [docs/adr/](docs/adr/) |
| La suite, avec des données sensibles | [docs/etape-2.md](docs/etape-2.md) |

## Organisation du dépôt

```
server/
  src/            le code Rust : un module par responsabilité
    sources/      d'où viennent les données (GitHub, fichiers locaux)
    rules/        une règle d'alerte par fichier
    render/       les pages HTML, le sprite de la ville, le SEO
    collect.rs    transforme les données brutes en notions de la charte
    site.rs       fabrique toutes les pages d'un coup
    http.rs       le serveur HTTP, strict et minuscule
  assets/         style.css, app.js, logo, favicon, image de partage, bâtiments (SVG)
  fixtures/       des données publiques capturées, pour le développement et les tests
docs/             toute la documentation
scripts/          la capture des données de démonstration
```

## Commandes

`make help` les liste. Les trois obligatoires de la charte (R21) : `make build`, `make test`, `make lint`. Avant une PR : `make check` (R4).

## Contribuer

Une issue d'abord, une branche `feature/…`, une PR vers `dev` : voir [CONTRIBUTING.md](CONTRIBUTING.md). Chaque nouvelle dépendance se justifie dans la PR (R19) : ce projet veut rester petit.

## Licence

[AGPL-3.0-or-later](LICENSE). Les dépendances et leurs licences sont dans [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). Aucun élément graphique ne vient de l'extérieur : le logo et les bâtiments sont dessinés pour ce projet.
