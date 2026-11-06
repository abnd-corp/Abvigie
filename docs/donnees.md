# Les données

Tout ce que la page affiche vient d'un seul objet, `Etat` (`server/src/model.rs`), publié tel quel sur **`/api/etat.json`**. Les données sont **publiques** : rien de privé n'y entre ([sécurité](securite.md), R13).

## Le format

```jsonc
{
  "schema": 1,                          // version du format, voir plus bas
  "genere_le": "2026-11-06T12:00:00Z",
  "organisation": "abnd-corp",
  "resume": { "projets": 16, "actifs": 16, "endormis": 0, "alertes": 24, "chantiers": 6 },
  "avertissements": [],                 // ce qui n'a pas pu être lu ; vide quand tout va bien
  "progression": {                      // les quêtes de la ville : voir quetes.md
    "quetes": [ { "id": "licences", "titre": "…", "regle": "R20", "fait": 6, "total": 16, "a_traiter": ["abcarte", "…"] } ]
  },
  "acteurs": [ { "login": "hugolagouardat", "robot": false, "activite": 12 } ],   // les personnages de la carte
  "projets": [
    {
      "slug": "abcom",                  // identifiant dans les URL : /projets/abcom
      "depot": "Abcom",                 // nom du dépôt GitHub
      "nom": "Abcom",                   // nom affiché
      "url": "https://github.com/abnd-corp/Abcom",
      "description": "…", "langage": "Rust", "licence": "AGPL-3.0",
      "topics": ["rust", "p2p"],
      "quartier": "systemes",           // web | systemes | outillage | autres
      "cree_le": "…", "dernier_push": "…", "jours_inactif": 12,
      "activite": "actif",              // vif | actif | calme | faible | endormi
      "archive": false,
      "issues": [ { "numero": 52, "titre": "…", "url": "…", "auteur": "…", "robot": false,
                    "jalon": "Semaine du 26 octobre", "echeance": "2026-11-01",
                    "en_retard": true, "cree_le": "…", "age_jours": 12 } ],
      "prs": [ /* même forme */ ],
      "ci": { "etat": "rouge", "workflow": "build", "branche": "main", "date": "…", "url": "…" },
      "executions": [ { "workflow": "build", "branche": "main", "etat": "vert", "date": "…", "age_jours": 2, "url": "…" } ],
      "drapeaux": [ { "code": "licence_absente", "regle": "R20", "gravite": "alerte", "message": "…" } ],
      "chantier": true
    }
  ]
}
```

La forme exacte est dans `model.rs` : c'est la référence, ce document en est le commentaire.

## Les données de la carte : `/api/carte.json`

La carte ne lit pas `/api/etat.json` (60 Ko) mais un **résumé compact** (environ 4 Ko), aussi **incorporé dans la page d'accueil** (`<script type="application/json" id="donnees-carte">`) pour que la carte se dessine sans aller-retour réseau. Il est fabriqué par `render/carte.rs` et versionné à part (`version`, actuellement 1).

Il contient, par projet : `slug`, `nom`, `quartier`, `accent` (la couleur), `lumieres` (0 à 4), `jours_inactif`, `chantier`, `archive`, `gravite` (la plus grave), `alertes`, `issues`, `prs` ; puis les `quartiers` (id, titre, nombre), les `acteurs` et, par quête, `accomplie` et `a_traiter`. Rien de plus : aucune description, aucun titre d'issue.

## Version du format

`schema` vaut `VERSION_SCHEMA` (`model.rs`). **Ajouter** un champ ne change pas la version. **Retirer, renommer ou changer le sens** d'un champ l'incrémente : quelqu'un lit peut-être ce fichier. Si on la change, on le dit dans le message de la PR.

## D'où viennent les valeurs

| Valeur | Calcul | Où |
|---|---|---|
| `jours_inactif` | jours depuis le dernier `push` du dépôt | `collect.rs` |
| `activite` | `vif` ≤ 7 j, `actif` ≤ 21 j, `calme` ≤ 40 j, `faible` jusqu'au seuil d'endormissement (60 j), puis `endormi` | `model.rs` (`Activite::depuis`) |
| `en_retard` | le jalon de l'issue a une échéance strictement avant aujourd'hui (une PR n'est jamais en retard) | `collect.rs` |
| `robot` | auteur de type `Bot` ou nom finissant par `[bot]` (Dependabot, GitHub Actions) | `collect.rs` |
| `ci` | la plus récente exécution terminée ou en cours sur `dev`, `staging`, `main` ou la branche par défaut, hors « Dependabot Updates » et exécutions annulées | `collect.rs` |
| `drapeaux` | les règles de `rules/` ([catalogue](regles.md)) | `rules/` |
| `progression` | les quêtes sur tous les projets non archivés ([quetes.md](quetes.md)) | `progression.rs` |
| `acteurs` | les auteurs d'issues et de PR ouvertes, 8 personnes et 4 robots au plus, les plus actifs d'abord | `collect.rs` |
| `chantier` | vrai s'il y a une issue en retard ou une pull request humaine qui attend | `collect.rs` |

## Limites connues

- **L'activité est le dernier `push` du dépôt**, y compris celui d'un robot : un projet qui ne reçoit que des mises à jour automatiques paraît actif. Pour mieux faire, il faudrait lire le dernier commit humain (un appel de plus par dépôt).
- **La licence est celle que GitHub détecte sur la branche par défaut.** Une licence ajoutée sur `dev` n'apparaît qu'après la promotion jusqu'à `main` : le drapeau reste levé jusque-là, ce qui est voulu.
- **Seules les 100 premières issues et PR ouvertes** de chaque dépôt sont lues.
- **Les dates sont en UTC** et le fuseau horaire n'est pas géré.

## Données de démonstration

`server/fixtures/` contient une capture des données publiques de l'organisation, au format brut de l'API réduit aux champs utilisés. `make fixtures` la renouvelle (avec `gh`, déjà connecté : aucun jeton n'est lu). Elle sert à développer hors ligne, aux tests et à la démonstration.
