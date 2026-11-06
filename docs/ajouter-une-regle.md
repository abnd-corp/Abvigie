# Ajouter une règle d'alerte

Une règle = **un fichier** dans `server/src/rules/` + **une ligne** dans `rules/mod.rs` + **ses tests**. Rien d'autre à toucher.

## 1. Écrire le fichier

Exemple : « un projet public doit avoir un fichier `SECURITY.md` » (si on lisait cette donnée).

```rust
//! R-xx : une phrase qui dit ce que la règle exige, et pourquoi.

use super::Contexte;
use crate::model::{Drapeau, Gravite};

pub fn evaluer(c: &Contexte) -> Option<Drapeau> {
    // `c` donne : description, licence, topics, jours_inactif, issues, prs, ci, archive, config
    (condition).then(|| Drapeau {
        code: "mon_code",        // stable, en minuscules avec des _ : il apparaît dans l'API
        regle: "R20",            // la règle de la charte, ou "" s'il n'y en a pas
        gravite: Gravite::Attention,
        message: "Une phrase claire, qui dit quoi faire".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::essais::sain;   // un contexte où aucune règle ne se déclenche
    use super::*;
    use crate::config::Config;

    #[test]
    fn se_declenche_quand_il_faut() {
        let config = Config::default();
        let mut c = sain(&config, &[]);
        // modifier le contexte pour déclencher la règle, puis vérifier le drapeau…
        assert!(evaluer(&c).is_some());
        // …et vérifier qu'elle se tait dans le cas normal
    }
}
```

## 2. La brancher

Dans `server/src/rules/mod.rs` : déclarer `mod mon_fichier;` et ajouter `mon_fichier::evaluer` à la liste `REGLES`. L'ordre n'a pas d'importance : les drapeaux sont triés par gravité.

## 3. Documenter

Ajouter une ligne au tableau de [regles.md](regles.md).

## Si la règle a besoin d'une donnée qu'on ne lit pas encore

C'est le seul cas où il faut toucher plus d'un fichier :

1. ajouter le champ à `Contexte` (`rules/mod.rs`) et au contexte de test `essais::sain` ;
2. le remplir dans `collect.rs` ;
3. si la donnée vient de GitHub, la lire dans `sources/` (un appel d'API de plus coûte une requête par dépôt à chaque rafraîchissement : à justifier) ;
4. si elle doit s'afficher, l'ajouter à `Projet` (`model.rs`) : c'est un ajout compatible, la version du format ne change pas.

## Bonnes pratiques

- **Un message qui dit quoi faire**, pas seulement ce qui ne va pas.
- **Une règle ne modifie rien** et ne dépend pas d'une autre règle.
- **Pas d'appel réseau** dans une règle : elle reçoit tout ce qu'il lui faut.
- **Un test pour le cas qui se déclenche et un pour celui qui se tait**, avec les cas limites (le seuil exact, le projet archivé, les robots).
