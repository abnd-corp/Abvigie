# Mettre en ligne

Le service sert les pages, la carte (des modules JavaScript, versionnés) et les données : rien d'autre à héberger.

> **État : rien n'est déployé, et rien n'est automatisé.** Il n'y a pas encore de CI, d'image Docker ni d'hébergement. Ce document décrit ce qu'il faudra faire pour la V0.1 ; il n'a pas été essayé en production. L'hébergement reste à décider (domaine « Infra et hébergement » de la charte), dans un ADR (R25).

## Lancer le binaire

```bash
make build                      # target/release/abvigie, environ 1,6 Mo
ABVIGIE_ADRESSE=127.0.0.1:8080 \
ABVIGIE_URL=https://abvigie.exemple.org \
GITHUB_TOKEN=…                  \
target/release/abvigie
```

Le binaire contient tout (pages, styles, script, images) : **un seul fichier à copier**. Il ne lit ni n'écrit rien sur le disque.

## Les réglages

Tous par variables d'environnement. Aucune n'est obligatoire.

| Variable | Défaut | Rôle |
|---|---|---|
| `ABVIGIE_ORG` | `abnd-corp` | organisation GitHub à observer (lettres, chiffres, tirets) |
| `ABVIGIE_ADRESSE` | `127.0.0.1:8080` | adresse d'écoute ; en conteneur, `0.0.0.0:8080` |
| `ABVIGIE_URL` | `http://localhost:8080` | **adresse publique**, sans `/` final : canonique, plan du site, JSON-LD, image de partage ([seo-geo.md](seo-geo.md)) |
| `ABVIGIE_RAFRAICHIR` | `900` | secondes entre deux lectures de GitHub (60 au minimum) |
| `ABVIGIE_ENDORMI_JOURS` | `60` | jours sans activité avant l'extinction des lumières |
| `ABVIGIE_REVUE_JOURS` | `180` | jours sans activité avant le drapeau de revue (R26) |
| `GITHUB_TOKEN` | aucun | jeton **en lecture seule** ; sans lui, GitHub limite à 60 requêtes par heure |

Options : `--fixtures <dossier>` (données locales), `--adresse <hôte:port>`, `--version`, `--aide`.

## Le jeton GitHub

- **En lecture seule, limité aux dépôts publics**, jamais un jeton personnel (R14). La cible est la GitHub App d'`abnd-bot` ([gouvernance#25](https://github.com/abnd-corp/gouvernance/issues/25)).
- **Jamais dans le dépôt, jamais dans une image** : seulement dans l'environnement du processus, depuis le coffre d'équipe (R9, R42).
- **Combien de requêtes ?** Une lecture complète fait 1 (la liste) + 2 par dépôt (tickets et CI), soit **33 pour 16 dépôts**. Toutes les 15 minutes, cela fait environ 130 par heure : bien sous les 5 000 d'un jeton.
- **Sans jeton, ça ne tient pas.** Mesuré le 6 novembre 2026 : le démarrage consomme 33 des 60 requêtes horaires, et la lecture suivante (60 secondes plus tard) a épuisé le reste et laissé **6 dépôts illisibles**. Le cache `ETag` du service n'a **pas** protégé le quota sans jeton : les réponses `304` y ont été décomptées. Avec un jeton, la documentation de GitHub dit que les `304` ne comptent pas ; cela **n'a pas été vérifié ici**, et ne change rien à la marge confortable de 5 000.
- **Un rafraîchissement partiel ne remplace pas un site complet** : il serait trompeur, les alertes des projets illisibles disparaîtraient. Le service garde les pages précédentes pendant 3 lectures partielles d'affilée, puis accepte l'état partiel, qui affiche un bandeau « données partielles » (`site::faut_remplacer`, testé).
- **Il faut donc un jeton en production**, et la lecture en direct sans jeton ne sert qu'à essayer une fois.

## Derrière un proxy (obligatoire)

Le serveur est volontairement minimal ([sécurité](securite.md)). Il faut un proxy devant : **Traefik** ou **Caddy**, qui apporte :

| À configurer au proxy | Pourquoi |
|---|---|
| TLS (Let's Encrypt) | le service parle HTTP simple |
| Compression **brotli et gzip** | 40 Ko deviennent 7,5 Ko ; le service ne compresse pas |
| Limitation de débit par adresse | protection contre les abus |
| Redirection HTTP vers HTTPS | |

Exemple avec Caddy :

```
abvigie.exemple.org {
    encode zstd gzip
    reverse_proxy 127.0.0.1:8080
}
```

## Garder le service en vie

`systemd` (sur un serveur ou un Raspberry Pi) :

```ini
[Unit]
Description=Abvigie
After=network-online.target

[Service]
ExecStart=/opt/abvigie/abvigie
# ABVIGIE_URL, GITHUB_TOKEN… ; fichier en droits 0600, propriétaire root
EnvironmentFile=/etc/abvigie.env
DynamicUser=yes
Restart=on-failure
# durcissement : le service n'a besoin de rien d'autre que du réseau
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
PrivateDevices=yes
CapabilityBoundingSet=
RestrictAddressFamilies=AF_INET AF_INET6
MemoryMax=64M

[Install]
WantedBy=multi-user.target
```

`MemoryMax=64M` est très large : le processus utilise environ 4 Mo.

Santé : `GET /sante` répond `200` et `{"ok":true,…}` quand des données sont chargées, `503` sinon.

## Ce qui est prévu pour la V0.1 et n'existe pas encore

- [ ] CI : format, clippy, tests, types JavaScript, `cargo audit`, `cargo deny` (actions figées sur leur empreinte, R24).
- [ ] Image Docker (une étape `prod` minimale, sans shell), publiée sur GHCR par la CI partagée (R44), et un contrôle de santé sans `curl`.
- [ ] Règles de branche (`scripts/appliquer-flux.sh` de [gouvernance](https://github.com/abnd-corp/gouvernance)) et jalon daté (R46).
- [ ] Choix de l'hébergement et du domaine, dans un ADR (R25).
- [ ] Jeton : la GitHub App d'`abnd-bot` (R14).
- [ ] Un retour arrière prévu avant la mise en production (R41, R43).
