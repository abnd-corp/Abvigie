# Les règles d'alerte

Une règle regarde un projet et décide de lever, ou non, un **drapeau**. Chacune est un fichier de `server/src/rules/`, avec ses tests, et se rattache à une règle de la [charte Abnd](https://github.com/abnd-corp/gouvernance/blob/main/CHARTE.md). Un projet **archivé** ne lève aucun drapeau : il est à l'arrêt volontairement.

| Code | Charte | Gravité | Se déclenche quand | Message |
|---|---|---|---|---|
| `ci_rouge` | R4 | alerte | la dernière CI d'une branche principale est en échec | « CI en échec sur main : build » |
| `licence_absente` | R20 | alerte | aucune licence détectée sur la branche par défaut | « Aucune licence détectée sur la branche par défaut » |
| `description_absente` | R20 | attention | pas de description, ou seulement des espaces | « Pas de description » |
| `topics_insuffisants` | R20 | info | moins de 2 topics | « Moins de 2 topics (1 actuellement) » |
| `jalon_echu` | R46 | attention | au moins une issue dont le jalon a dépassé son échéance | « 2 issues en retard (échéance du jalon dépassée) » |
| `jalon_manquant` | R46 | attention | au moins une issue humaine sans jalon, donc sans échéance | « 1 issue sans jalon, donc sans échéance » |
| `pr_en_attente` | R4 | info | une pull request **humaine** attend depuis plus de 7 jours (les robots sont ignorés) | « 1 pull request attend une relecture depuis plus de 7 jours » (« 2 pull requests attendent… » au pluriel) |
| `dormant` | R26 | attention | aucune activité depuis 180 jours (réglable) | « Sans activité depuis 200 jours : à relancer ou à archiver » |

Les drapeaux sont triés par gravité (alerte, attention, info). Le fanion d'un bâtiment prend la couleur du plus grave.

## Ce qui n'est pas un drapeau

- **L'extinction des lumières** (60 jours sans activité, réglable) est une information visuelle : le bâtiment n'est pas en faute. Le drapeau `dormant` n'apparaît qu'au seuil de la revue de la charte (6 mois).
- **Le chantier** (pointillés) n'est pas une règle : c'est un projet qui a au moins un drapeau `jalon_echu` ou `pr_en_attente`.

## Réglages

| Variable | Défaut | Effet |
|---|---|---|
| `ABVIGIE_ENDORMI_JOURS` | 60 | extinction des lumières |
| `ABVIGIE_REVUE_JOURS` | 180 | drapeau `dormant` |

Les autres seuils (7 jours pour une PR, 2 topics) sont des constantes en tête de leur fichier : un seuil qui change se discute, il ne se règle pas sans trace.

## Ajouter une règle

Voir [ajouter-une-regle.md](ajouter-une-regle.md).
