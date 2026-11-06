# ADR 0003 · Un serveur HTTP minimal écrit pour ce projet

Date : 6 novembre 2026 · Statut : accepté

## Contexte

Le service doit répondre à des requêtes `GET` sur des pages déjà fabriquées. Il a besoin de très peu : lire une ligne de requête, quelques en-têtes, écrire une réponse. Les bibliothèques disponibles se partagent en deux familles :

| Option | Constat (6 novembre 2026) |
|---|---|
| `tiny_http` | la plus simple, mais la dernière version date d'**octobre 2022** (0.12.0), avec 77 issues ouvertes, et elle a eu un avis de *request smuggling* (RUSTSEC-2020-0031, corrigé depuis 0.8) |
| `hyper`, `axum` et leur écosystème | maintenus, mais ils tirent `tokio` et des dizaines de crates : plusieurs Mo de binaire et une surface d'audit bien plus grande, pour un besoin de trois fonctions |

## Décision

Écrire un serveur HTTP/1.1 **minimal et strict** (`http.rs`, environ 350 lignes de code), avec la seule bibliothèque standard :

- `GET` et `HEAD` uniquement ; **aucun corps de requête n'est jamais lu** (toute annonce de corps est refusée) ;
- une requête par connexion, puis fermeture ;
- limites de taille et de temps, nombre de connexions borné ;
- aucun accès au disque : les chemins sont des clés d'une table.

Il tourne **derrière un proxy** (TLS, compression, limitation de débit). C'est écrit dans [deploiement.md](../deploiement.md) et [securite.md](../securite.md).

## Pourquoi c'est raisonnable ici

Écrire un serveur HTTP est d'ordinaire déconseillé, parce que la plupart des failles viennent de la souplesse du protocole (corps, `Transfer-Encoding`, réutilisation de connexion). Ici, **on supprime cette souplesse** : sans corps, sans keep-alive et sans pipelining, il n'y a presque rien à mal interpréter. Le code est court et chaque refus est couvert par un test (smuggling, en-têtes repliés, caractères de contrôle, cibles trop longues, requêtes sans fin).

## Conséquences

- **Positif** : une dépendance de moins ; binaire de 1,5 Mo ; surface d'audit minuscule ; comportement entièrement lisible.
- **Négatif** : c'est du code à nous, à relire avec attention (R35 : une PR sur la sécurité demande une validation à trois).
- **Négatif** : pas de HTTP/2, pas de keep-alive : sans importance derrière un proxy, qui les apporte côté client.
- **À rouvrir** si le service devait un jour être exposé sans proxy, ou servir autre chose que des pages statiques (formulaires, authentification) : à ce moment-là, une bibliothèque maintenue devient préférable.
