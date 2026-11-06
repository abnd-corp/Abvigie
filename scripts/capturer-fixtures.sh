#!/usr/bin/env bash
# Capture les données PUBLIQUES d'une organisation dans server/fixtures/, au format qu'attend la source « fixtures ».
# Sert au développement hors ligne, aux tests et à la démonstration. Utilise `gh` (déjà connecté) : aucun jeton n'est lu ici.
# Usage : scripts/capturer-fixtures.sh [organisation]
set -euo pipefail

ORG="${1:-abnd-corp}"
DOSSIER="$(cd "$(dirname "$0")/.." && pwd)/server/fixtures"
command -v gh >/dev/null || { echo "gh est requis (https://cli.github.com)" >&2; exit 1; }
command -v jq >/dev/null || { echo "jq est requis" >&2; exit 1; }

mkdir -p "$DOSSIER"
rm -f "$DOSSIER"/*.json

# Seuls les champs lus par le serveur sont gardés (voir server/src/sources/brut.rs) : fichiers petits, rien de superflu.
gh api --paginate "/orgs/$ORG/repos?type=public&per_page=100" \
  | jq -s 'add | [.[] | {name, html_url, description, language,
      license: (if .license then {spdx_id: .license.spdx_id} else null end),
      topics, created_at, pushed_at, archived, has_issues, default_branch}]' \
  > "$DOSSIER/depots.json"

jq -r '.[] | "\(.name) \(.has_issues)"' "$DOSSIER/depots.json" | while read -r nom issues; do
  # le nom finit dans un nom de fichier : on refuse tout ce que GitHub n'autorise pas
  [[ "$nom" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "nom ignoré : $nom" >&2; continue; }
  sur="${nom//[^A-Za-z0-9_-]/_}"
  ressource=$([ "$issues" = "true" ] && echo issues || echo pulls)
  gh api "/repos/$ORG/$nom/$ressource?state=open&per_page=100" \
    | jq '[.[] | {number, title: .title[:200], html_url,
        user: {login: .user.login, type: .user.type},
        milestone: (if .milestone then {title: .milestone.title, due_on: .milestone.due_on} else null end),
        created_at, pull_request: (if .pull_request then {} else null end)}]' \
    > "$DOSSIER/$sur.tickets.json"
  gh api "/repos/$ORG/$nom/actions/runs?per_page=15" \
    | jq '{workflow_runs: [.workflow_runs[] | {name, event, head_branch, status, conclusion, created_at, html_url}]}' \
    > "$DOSSIER/$sur.executions.json"
  echo "capturé : $nom"
done
echo "Fait : $(ls "$DOSSIER" | wc -l | tr -d ' ') fichiers dans $DOSSIER"
