# La carte

La carte est une ville en isométrique (une vue en 3D « de dessus », comme dans les jeux de gestion), dessinée sur un `<canvas>` par du JavaScript natif, sans bibliothèque. Elle est **un supplément** : tout ce qu'elle montre est aussi dans le HTML de la page ([ADR 0002](adr/0002-rendu-cote-serveur.md)).

## Les fichiers (`server/assets/js/`)

| Fichier | Rôle | Il ne connaît pas |
|---|---|---|
| `iso.js` | la projection (tuile → écran), les couleurs, les formes de base : boîte, cylindre, toits, pyramide, dôme, ombre | les projets, GitHub |
| `batiments.js` | la **description** de chaque bâtiment (des volumes assemblés), ses fenêtres, ses détails animés | la caméra, la carte |
| `monde.js` | la mise en page de la ville, le sol, les routes, les parcs, les arbres, les lampadaires, le survol | le dessin d'un bâtiment |
| `acteurs.js` | les personnages qui se promènent de carrefour en carrefour | la caméra |
| `camera.js` | le déplacement, le zoom, le pincement, l'inertie, le clavier | tout le reste |
| `carte.js` | le chef d'orchestre : la boucle de dessin, les fanions, les étiquettes, l'infobulle | |
| `interface.js` | la fiche d'un projet dans le panneau, les quêtes qui éclairent la carte, les réglages | |
| `vignettes.js` | le bâtiment en petit, dans les cartes de la liste et en tête des fiches | |
| `donnees.js` | lit les données de la carte (incorporées dans la page, sinon `/api/carte.json`) | |

La frontière importante : `iso.js` et `batiments.js` ne savent rien de la carte, donc un bâtiment se dessine à l'identique sur la carte et dans une vignette.

## Le monde

- Une **grille de tuiles** : une tuile fait 64 × 32 pixels à l'écran (un losange 2:1). Une position est `(gx, gy)` en tuiles, une hauteur `z` en pixels.
- Une **parcelle** fait 3 × 3 tuiles de terrain, plus une tuile de route : 4 tuiles avec sa route. Un bâtiment vit dans sa parcelle.
- Les projets sont rangés **par quartier** (les domaines de la charte : développement web, développement systèmes, outillage et gouvernance), chaque quartier en un bloc de parcelles. Les blocs sont posés sur des « étagères » de 8 parcelles de large.
- Les **parcelles libres** deviennent des parcs, avec des arbres et, au plus près du centre, un étang. L'**emplacement du premier client** reste libre, en pointillés : la charte demande de s'y préparer avant le premier client.
- Chaque quartier a un **liseré** à sa couleur sur les routes qui le bordent, un sol un peu plus teinté et une **enseigne** (une pastille avec son nom) au milieu de son bord avant. L'enseigne reste toujours dans l'écran. Survoler l'enseigne ou le sol du quartier éclaire son liseré et affiche son nombre de projets, d'actifs et d'alertes.

## Un bâtiment

Il est décrit dans `batiments.js` par une liste de **volumes** (boîtes, cylindres, toits à deux pentes, pyramides, dômes) et une liste de **détails**. Voir [ajouter-un-immeuble.md](ajouter-un-immeuble.md).

Ce qui vient des données :

| Fait | Effet visible |
|---|---|
| activité (`lumieres`, de 0 à 4) | la part de fenêtres allumées ; à 0, le bâtiment est éteint (corps sombre) et des « z » montent de son toit |
| gravité du drapeau le plus grave | un fanion qui flotte, rouge, ambre ou vert, avec le nombre d'alertes |
| `chantier` | une grue au sol derrière le bâtiment et des pointillés autour |
| quête survolée | un contour ambre qui pulse au sol |
| projet ouvert | un contour vert au sol, et la caméra s'approche |

Quelques fenêtres vacillent : elles s'éteignent un instant, selon un nombre tiré du nom du projet. La ville est la même à chaque chargement (rien n'est tiré au hasard), seul ce qui bouge est animé.

## Les personnages

Les auteurs des issues et pull requests ouvertes : **8 personnes et 4 robots au plus**, les plus actifs d'abord. Ce sont des **identifiants GitHub publics**, rien d'autre n'est lu. Ils marchent le long des routes. Leur pseudo s'affiche quand on les survole, ou toujours si l'option « Noms » est active et que la carte est assez zoomée.

Un personnage ne rapporte rien et ne coûte rien : les quêtes comptent des projets, jamais des personnes (charte, R27 : on n'évalue pas les gens).

## L'interaction

| Geste | Effet |
|---|---|
| glisser | déplacer la carte, avec un peu d'inertie |
| molette, ou pincement à deux doigts | zoomer autour du curseur |
| `+`, `-`, flèches (carte au focus) | zoomer, se déplacer |
| survoler | contour du bâtiment et infobulle (nom, activité, issues, PR, alertes) |
| cliquer un bâtiment ou un lien de projet | la fiche s'ouvre dans le panneau, la caméra glisse vers lui |
| **Recentrer** | recadre toute la ville |
| cases **Noms**, **Alertes**, **Personnages** | masquent ou montrent ces calques ; le choix est mémorisé |

Tant que la personne n'a ni déplacé ni zoomé, la ville se **recadre toute seule** à chaque changement de taille de la fenêtre.

## Accessibilité

- Le `<canvas>` a un rôle `img` et une description ; il prend le focus et se pilote au clavier.
- **Tout ce qu'il montre est dans le panneau**, en HTML : l'onglet Projets liste chaque projet avec ses alertes (et ses liens, utilisables au clavier et au lecteur d'écran), l'onglet Quêtes les objectifs, l'onglet Journal les événements.
- Le mouvement est réduit si la personne l'a demandé (`prefers-reduced-motion`) : plus aucune animation, et la carte ne se redessine que quand on agit dessus (consommation nulle au repos).
- Les contrastes de texte atteignent au moins 4,5 : 1 ; le focus est toujours visible.

## Le dessin, image par image

À chaque image : le fond étoilé, le sol (toutes les tuiles d'une même couleur dans un seul tracé, pour éviter les coutures), les pointillés et anneaux au sol, puis **tous les objets triés du plus loin au plus proche** (arbres, lampadaires, bâtiments, personnages), le contour du survol, et enfin les étiquettes et l'infobulle, dessinées à taille d'écran constante.

L'animation d'ambiance est plafonnée à **30 images par seconde** et passe à la cadence de l'écran dès qu'on agit sur la carte. Elle s'arrête quand l'onglet est caché.

## Tester

- `make check` vérifie les types de tous les modules (`tsc`, sans compilation, grâce aux commentaires JSDoc).
- Un test Rust (`tous_les_imports_javascript_sont_servis`) vérifie que chaque module importé est bien servi, avec sa version.
- Le rendu se juge à l'œil : `make run`, puis la page. Pour essayer un changement de JavaScript sans recompiler, servir `server/assets/` avec n'importe quel serveur de fichiers et la page d'accueil réelle comme base.

## Versions des modules

Les modules s'importent les uns les autres (`from './iso.js'`). Le serveur ajoute à **chaque import** l'empreinte de la version (`from './iso.js?v=…'`) : toutes les adresses changent ensemble à chaque mise à jour, donc un navigateur ne mélange jamais un module ancien avec un nouveau, et le cache peut durer un an. Une première version sans cela laissait des modules périmés en cache pendant une heure : c'est ce qui a motivé ce choix.
