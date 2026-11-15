#!/bin/bash
# Downloads the raw game state of every platform from Warframe's server and pushes it to the WFHub API, which keeps it in D1.
# The game's server answers 403 to Cloudflare Workers and to GitHub Actions, so the download goes through a proxy whose
# addresses it accepts (WORLDSTATE_PROXY; only the download uses it, the push to WFHub does not).
# .github/workflows/worldstate.yml runs this every 10 minutes.
#
#   WORLDSTATE_PUSH_TOKEN=... WORLDSTATE_PROXY=http://user:password@host:port bash scripts/push-worldstate.sh
#   (the token is the Worker's secret of that name; without it in the environment it is read from
#   ~/.config/wfhub/worldstate-token, or the file in WFHUB_TOKEN_FILE; without a proxy the download is direct)
#   WFHUB_URL=http://localhost:8000 ...        # where to push: a local `npm run dev:web` (token in .dev.vars); the workflow uses the
#                                              # Worker's workers.dev address (the Bot Fight Mode of wfhub.cagatayldzz.com challenges GitHub)
set -uo pipefail

base="${WFHUB_URL:-https://wfhub.cagatayldzz.com}"
token_file="${WFHUB_TOKEN_FILE:-$HOME/.config/wfhub/worldstate-token}"
if [ -z "${WORLDSTATE_PUSH_TOKEN:-}" ] && [ -r "$token_file" ]; then
  WORLDSTATE_PUSH_TOKEN="$(tr -d '[:space:]' < "$token_file")"
fi
: "${WORLDSTATE_PUSH_TOKEN:?set WORLDSTATE_PUSH_TOKEN or put the token in $token_file}"
proxy=()
[ -n "${WORLDSTATE_PROXY:-}" ] && proxy=(--proxy "$WORLDSTATE_PROXY")
tmp="$(mktemp)"
out="$(mktemp)"
trap 'rm -f "$tmp" "$out"' EXIT

failed=0
for entry in pc=api xb1=api-xb1 ps4=api-ps4 swi=api-swi; do
  platform="${entry%%=*}"
  host="${entry#*=}"
  if ! status=$(curl -sS --compressed ${proxy[@]+"${proxy[@]}"} --retry 3 --retry-delay 2 --max-time 60 -o "$tmp" -w '%{http_code}' "https://$host.warframe.com/cdn/worldState.php") || [ "$status" != 200 ]; then
    echo "$platform: the game answered ${status:-no answer}" >&2
    failed=1
    continue
  fi
  code=$(curl -sS --retry 3 --retry-delay 2 --max-time 60 -X PUT -o "$out" -w '%{http_code}' \
    -H "Authorization: Bearer $WORLDSTATE_PUSH_TOKEN" -H "Content-Type: application/json" \
    --data-binary "@$tmp" "$base/api/ingest/worldstate/$platform") || code="no answer"
  if [ "$code" = 200 ]; then
    echo "$platform: $(head -c 200 "$out") [200]"
  else
    echo "$platform: the push to $base answered $code: $(head -c 160 "$out" | tr '\n' ' ')" >&2
    if [ "$code" = 403 ] && grep -q "Just a moment" "$out" 2>/dev/null; then
      echo "  (Cloudflare challenges this address: push to the Worker's workers.dev address, set the repository variable WFHUB_INGEST_URL)" >&2
    fi
    failed=1
  fi
done
exit "$failed"
