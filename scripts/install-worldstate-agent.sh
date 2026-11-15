#!/bin/bash
# Installs a macOS launch agent that runs scripts/push-worldstate.sh every 5 minutes (while the Mac is awake), so the API
# has a fresh game state. Asks for the token once (the same value as the Worker's WORLDSTATE_PUSH_TOKEN secret) and keeps it
# in ~/.config/wfhub/worldstate-token (only you can read it). Log: ~/Library/Logs/wfhub-worldstate.log
#
#   bash scripts/install-worldstate-agent.sh              # install (or update)
#   bash scripts/install-worldstate-agent.sh --uninstall  # remove the agent (the token file stays)
set -euo pipefail

label="com.wfhub.worldstate"
plist="$HOME/Library/LaunchAgents/$label.plist"
domain="gui/$(id -u)"

if [ "${1:-}" = "--uninstall" ]; then
  launchctl bootout "$domain/$label" 2>/dev/null || true
  rm -f "$plist"
  echo "Removed $label"
  exit 0
fi

script="$(cd "$(dirname "$0")" && pwd)/push-worldstate.sh"
token_file="$HOME/.config/wfhub/worldstate-token"
if [ ! -s "$token_file" ]; then
  mkdir -p "$(dirname "$token_file")"
  read -r -s -p "WORLDSTATE_PUSH_TOKEN: " token
  echo
  [ -n "$token" ] || { echo "No token given." >&2; exit 1; }
  (umask 077 && printf '%s\n' "$token" > "$token_file")
fi
chmod 600 "$token_file"

# One push now, so a wrong token shows up here and not only in the log.
bash "$script"

mkdir -p "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"
cat > "$plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>$label</string>
  <key>ProgramArguments</key>
  <array><string>/bin/bash</string><string>$script</string></array>
  <key>StartInterval</key><integer>300</integer>
  <key>RunAtLoad</key><true/>
  <key>StandardOutPath</key><string>$HOME/Library/Logs/wfhub-worldstate.log</string>
  <key>StandardErrorPath</key><string>$HOME/Library/Logs/wfhub-worldstate.log</string>
</dict>
</plist>
PLIST
plutil -lint "$plist" >/dev/null
launchctl bootout "$domain/$label" 2>/dev/null || true
launchctl bootstrap "$domain" "$plist"
echo "Installed $label: runs every 5 minutes. Log: ~/Library/Logs/wfhub-worldstate.log"
