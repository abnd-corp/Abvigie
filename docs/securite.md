# Sécurité

Abvigie est un site **public**, sans compte, sans formulaire, sans base de données, qui n'affiche que des **données publiques**. La surface d'attaque est volontairement minuscule. Ce document dit ce qui est fait, et ce qui ne l'est pas.

## Ce que le service ne fait jamais

- Il n'a **aucun accès en écriture** : ni vers GitHub (jeton en lecture seule), ni sur le disque.
- Il ne **lit aucun fichier** pour répondre : les pages sont en mémoire, les chemins sont des clés d'une table. Il n'y a donc rien à traverser (`/../../etc/passwd` répond 404, c'est testé).
- Il n'accepte **aucun corps de requête**, aucun cookie, aucun formulaire.
- Il n'affiche **aucune donnée privée** (R13). Les dépôts privés ne sont pas lus : la liste demandée est `type=public`.

## Le serveur HTTP (`http.rs`)

Écrit à la main pour être strict et lisible : environ 350 lignes de code, et presque autant de tests. La bibliothèque la plus courante pour cela (`tiny_http`) n'est plus publiée depuis octobre 2022 et a eu un avis de *request smuggling* (RUSTSEC-2020-0031, corrigé depuis la version 0.8) ; un service exposé ne s'appuie pas sur du code qui n'évolue plus ([ADR 0003](adr/0003-serveur-http-minimal.md)).

| Protection | Détail |
|---|---|
| Méthodes | `GET` et `HEAD` seulement, les autres répondent 405 |
| Pas de corps | `Content-Length` non nul, `Transfer-Encoding`, `Expect` et `Upgrade` sont refusés (400) : pas de *request smuggling* |
| Pas de réutilisation | une requête par connexion (`Connection: close`) : aucune confusion possible entre deux requêtes |
| Taille | requête limitée à 8 Kio (431), cible à 2 Kio, 64 en-têtes au plus |
| Caractères | ASCII imprimable seulement : pas de NUL, pas d'octet de contrôle, pas de ligne repliée |
| Cibles | doit commencer par `/`, pas de `//`, pas de `#`, pas d'espace |
| Délais | 5 s par lecture et 10 s au total (contre les connexions *slowloris*), 10 s en écriture |
| Capacité | 8 travailleurs, file de 64 connexions : au-delà, on refuse plutôt que d'enfler |
| Robustesse | un échec interne devient une réponse 500 ; le service ne s'arrête pas (`catch_unwind`) |

Ce n'est **pas** un serveur Internet complet : il est prévu pour tourner **derrière un proxy** (Traefik, Caddy) qui apporte le TLS, la compression, la limitation de débit par adresse et la protection contre les abus. Ne pas l'exposer directement.

## Les en-têtes envoyés

| En-tête | Valeur |
|---|---|
| `Content-Security-Policy` | `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | caméra, micro, localisation, paiement, USB : tous refusés |
| `Cross-Origin-Opener-Policy` | `same-origin` |
| `Strict-Transport-Security` | un an, sous-domaines compris (ignoré en HTTP simple) |

La CSP n'autorise **ni script en ligne, ni style en ligne, ni origine externe**. C'est pour cela que le code n'a aucun attribut `style=` dans le HTML : les couleurs passent par des classes CSS générées côté serveur. Le JavaScript, lui, ne fait que des modifications du modèle d'objets de style (`element.style.width = …`, pour dimensionner une vignette), que la CSP autorise : ce ne sont pas des styles en ligne. Les modules JavaScript viennent tous du même serveur (`script-src 'self'`) et le JavaScript n'utilise ni `eval`, ni `new Function`, ni `innerHTML` sur autre chose que du HTML du même serveur. Un test (`aucune_ressource_externe_dans_les_pages`) vérifie qu'aucune page ne contient `style="`, `onclick=`, `javascript:`, `<iframe`, `<form`, ni script ou feuille de style externes.

## Les données qui viennent de GitHub sont non fiables

Un nom de dépôt, une description ou un titre d'issue peut contenir n'importe quoi, y compris `<script>`. Donc :

- **Toute chaîne passe par `esc()`** (`render/mod.rs`) avant d'entrer dans une page ; les caractères de contrôle sont retirés. Des tests vérifient qu'un nom de projet malveillant ne produit aucun HTML.
- **Le JSON-LD** (dans un `<script type="application/ld+json">`) est sérialisé avec `</` remplacé par `<\/` : impossible de fermer la balise (testé).
- **Les noms de dépôt** qui servent dans une URL ou un nom de fichier sont validés (`nom_sur`) : seulement lettres, chiffres, `-`, `_`, `.`.
- **Le panneau latéral** reçoit du HTML qui vient du même serveur, déjà échappé. Le script ne construit jamais de HTML à partir de données. Les noms dessinés **sur la carte** (projets, pseudos) le sont avec `fillText` du canvas, qui n'interprète rien : pas de HTML, donc pas d'injection possible.
- **Les données de la carte** incorporées dans la page (`<script type="application/json">`) sont sérialisées avec `</` remplacé par `<\/`, comme le JSON-LD, et ne contiennent que des champs prévus (testé : ni jeton, ni titre d'issue, ni description).
- **Les personnages** sont des identifiants GitHub publics, vus dans les issues et pull requests ouvertes. Aucune autre information sur eux n'est lue, et rien de personnel n'est affiché.

## Le jeton GitHub (R9, R18)

- Il vient de la variable d'environnement `GITHUB_TOKEN` et n'est **jamais écrit** : ni dans le dépôt, ni dans les journaux, ni dans les erreurs.
- Son `Debug` affiche `Jeton(***)` : même une erreur de programmation ne peut pas le laisser filer (testé).
- **Il doit être en lecture seule** et limité aux dépôts publics. Cible : la GitHub App d'`abnd-bot` (R14, [gouvernance#25](https://github.com/abnd-corp/gouvernance/issues/25)).
- **Sans jeton**, le service marche mais GitHub limite à 60 requêtes par heure : suffisant pour essayer, pas pour tenir en continu.

## Les dépendances

47 crates dans `Cargo.lock` (dépendances transitives comprises, certaines propres à d'autres systèmes), toutes sous licences permissives. `cargo audit` ne trouve aucun avis de sécurité ; `cargo deny` vérifie licences, sources et doublons (`make audit`). Le TLS est fait par `rustls` (écrit en Rust, sans OpenSSL).

## Ce qui n'est pas couvert

- **Le TLS, la compression, la limitation de débit** : au proxy.
- **Le déni de service volumétrique** : au proxy et à l'hébergeur.
- **La disponibilité de GitHub** : si GitHub est indisponible, les pages précédentes sont conservées.
- **L'étape 2** (données sensibles, authentification) : [etape-2.md](etape-2.md). Rien de sensible n'entre dans cette version.

## Signaler une faille

Voir [SECURITY.md](../SECURITY.md). Le service publie aussi `/.well-known/security.txt`.
