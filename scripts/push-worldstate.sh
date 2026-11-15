#!/bin/bash
# Fetches the raw game state of every platform from Warframe's server and pushes it to the WFHub API, which keeps it in D1.
# The game's server answers 403 to Cloudflare Workers and to GitHub Actions, so it has to run somewhere it is let in: a Mac
# (scripts/install-worldstate-agent.sh runs it every 5 minutes) or any server with a normal address.
#
#   WORLDSTATE_PUSH_TOKEN=... bash scripts/push-worldstate.sh        # the same value as the Worker's secret
#   (without it, the token is read from ~/.config/wfhub/worldstate-token, or the file in WFHUB_TOKEN_FILE)
#   WFHUB_URL=http://localhost:8000 ...                              # to push to a local `npm run dev:web`
set -uo pipefail

base="${WFHUB_URL:-https://wfhub.cagatayldzz.com}"
token_file="${WFHUB_TOKEN_FILE:-$HOME/.config/wfhub/worldstate-token}"
if [ -z "${WORLDSTATE_PUSH_TOKEN:-}" ] && [ -r "$token_file" ]; then
  WORLDSTATE_PUSH_TOKEN="$(tr -d '[:space:]' < "$token_file")"
fi
: "${WORLDSTATE_PUSH_TOKEN:?set WORLDSTATE_PUSH_TOKEN or put the token in $token_file}"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

failed=0
for entry in pc=api xb1=api-xb1 ps4=api-ps4 swi=api-swi; do
  platform="${entry%%=*}"
  host="${entry#*=}"
  if ! status=$(curl -sS --retry 3 --retry-delay 2 --max-time 60 -o "$tmp" -w '%{http_code}' "https://$host.warframe.com/cdn/worldState.php") || [ "$status" != 200 ]; then
    echo "$platform: the game answered ${status:-no answer}" >&2
    failed=1
    continue
  fi
  if ! answer=$(curl -sS --retry 3 --retry-delay 2 --max-time 60 -X PUT \
    -H "Authorization: Bearer $WORLDSTATE_PUSH_TOKEN" -H "Content-Type: application/json" \
    --data-binary "@$tmp" -w ' [%{http_code}]' "$base/api/ingest/worldstate/$platform"); then
    echo "$platform: push failed" >&2
    failed=1
    continue
  fi
  echo "$platform: $answer"
  case "$answer" in *"[200]") ;; *) failed=1 ;; esac
done
exit "$failed"
