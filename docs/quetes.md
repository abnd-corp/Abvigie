# Les quêtes

Les **quêtes** donnent un côté jeu à un tableau de bord, sans rien cacher : chaque chiffre vient de faits publics de GitHub.

## Les quêtes

Une **quête** est un objectif de la charte à atteindre sur **tous** les projets (non archivés). La quête affiche combien de projets la remplissent déjà. Rien ne se coche à la main : quand un fait change sur GitHub, la quête avance à la lecture suivante.

| Quête | Charte | Points par projet | Un projet la remplit quand |
|---|---|---|---|
| Une licence partout | R20 | 10 | GitHub détecte une licence sur sa branche par défaut |
| Chaque projet se présente | R20 | 5 | il a une description |
| Des étiquettes pour être trouvé | R20 | 5 | il a au moins deux topics |
| Des feux verts partout | R4 | 15 | sa dernière CI est verte |
| Chaque issue a une échéance | R46 | 10 | aucune issue humaine sans jalon |
| Personne en retard | R46 | 10 | aucune issue dont le jalon est échu |
| Les relectures n'attendent pas | R4 | 5 | aucune PR humaine n'attend depuis plus de 7 jours |
| Aucun projet oublié | R26 | 5 | il a eu de l'activité dans les 6 derniers mois |

La plupart de ces quêtes sont **l'inverse d'une règle d'alerte** ([regles.md](regles.md)) : un projet remplit la quête quand la règle ne lève pas de drapeau. C'est voulu : un seul endroit dit ce qui ne va pas, et la quête dit ce qu'il reste à faire.

Un projet **archivé** ne compte pas : il est à l'arrêt volontairement. Une quête compte des **projets**, jamais des personnes : la charte interdit d'évaluer les gens (R27).

## Où on les voit

- **L'onglet Fiche** : la prochaine quête, celle qui a le plus de projets à traiter.
- **L'onglet Quêtes** : toutes les quêtes, leur progression, et les projets qui leur restent à faire (des liens).
- **Sur la carte** : survoler une quête fait pulser en ambre le contour des bâtiments qui lui restent à faire ; cliquer une quête l'épingle, pour garder l'éclairage en déplaçant la souris.
- **Dans `llms.txt` et `/api/etat.json`** : les mêmes chiffres, pour les machines.

## Ajouter une quête

C'est **une entrée** dans `DEFINITIONS` de `server/src/progression.rs` :

```rust
Definition {
    id: "securite",                                    // stable : il apparaît dans les données
    titre: "Une politique de sécurité partout",
    description: "Chaque dépôt a un fichier SECURITY.md.",
    regle: "R9",                                       // la règle de la charte, ou ""
    ok: |p| !drapeau(p, "securite_absente"),           // le projet remplit-il l'objectif ?
},
```

1. Si la quête repose sur un fait qu'aucune règle ne regarde encore, **écrire d'abord la règle** ([ajouter-une-regle.md](ajouter-une-regle.md)), puis s'en servir ici avec `drapeau(p, "code")`.
2. Ajouter la ligne au tableau ci-dessus.
3. `cargo test`.

## Pourquoi pas de cases à cocher

La maquette d'origine avait un plan par étapes, avec des cases cochées à la main et mémorisées dans le navigateur. Ici les quêtes sont **calculées sur les faits** : elles ne mentent pas, ne se périment pas et ne dépendent de personne. En contrepartie, elles ne peuvent exprimer que ce que GitHub sait. Les étapes qui demandent un geste hors GitHub (activer la 2FA, créer un compte robot) ne peuvent pas y figurer tant qu'on ne sait pas les lire publiquement.
