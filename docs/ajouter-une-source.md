# Ajouter une source de données

Une source sait lire trois choses : les dépôts, leurs tickets, leurs exécutions de CI. C'est le trait `Source` (`server/src/sources/mod.rs`) :

```rust
pub trait Source: Send + Sync {
    fn depots(&self) -> Result<Vec<Depot>, Erreur>;
    fn tickets(&self, depot: &Depot) -> Result<Vec<Ticket>, Erreur>;
    fn executions(&self, depot: &Depot) -> Result<Vec<Execution>, Erreur>;
}
```

Deux sources existent : `GitHub` (l'API REST, avec cache `ETag`) et `Fixtures` (des fichiers locaux). Elles sont interchangeables : le reste du code ne sait pas laquelle il utilise.

## Pour en ajouter une

1. Créer `server/src/sources/ma_source.rs` et implémenter `Source`.
2. La déclarer dans `sources/mod.rs` (`pub mod ma_source;`).
3. La choisir dans `main.rs`, là où `GitHub` et `Fixtures` sont choisis.
4. Écrire ses tests, **sans réseau** : lire des fichiers ou un faux serveur local.

Les types renvoyés (`sources/brut.rs`) sont tolérants : presque tout est optionnel, les champs inconnus sont ignorés. Une source qui n'a pas une information laisse le champ vide.

## Ce qu'une source ne fait pas

Elle ne calcule rien : pas d'activité, pas de retard, pas de drapeau. C'est le rôle de `collect.rs` et de `rules/`. Elle ne fabrique pas de HTML.

## Si la nouvelle source apporte une donnée nouvelle

Par exemple des clients ou des domaines (étape 2) : ce n'est plus une variante de GitHub mais un **second jeu de données**. Il faut alors un type `Etat` distinct ou une extension de `Etat`, et une décision sur ce qui est public. Voir [etape-2.md](etape-2.md) avant de commencer.

## Règles de sécurité communes

- **Un jeton ne sort jamais du fichier de la source** : il n'est ni journalisé, ni inclus dans une erreur, ni affiché par `Debug`. Voir `config.rs` et `github.rs`.
- **Tout nom venu de l'extérieur** qui sert dans une URL ou un nom de fichier est validé d'abord (voir `nom_sur` dans `collect.rs` et `sources/fixtures.rs`).
- **Les erreurs** disent ce qui a échoué et où, jamais ce qu'il y avait dans la requête.
