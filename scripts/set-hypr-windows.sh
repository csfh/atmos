#!/bin/bash
# Write the managed window-rule block in ~/.config/hypr/omarchy_prefs.lua.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
FILE=${OMARCHY_PREFS_WINDOWS_FILE:-"$HOME/.config/hypr/omarchy_prefs.lua"}
HYPRLAND=${OMARCHY_PREFS_HYPRLAND_FILE:-"$HOME/.config/hypr/hyprland.lua"}
SKIP_HYPR=${OMARCHY_PREFS_SKIP_HYPR:-0}

usage() {
  echo "Usage: set-hypr-windows.sh [--reset] [<json>]" >&2
  exit 1
}

reset=0
json=""
if [[ ${1:-} == --reset ]]; then
  reset=1
  shift
elif [[ ${1:-} == -* ]]; then
  usage
fi
if [[ $# -gt 1 ]]; then
  usage
fi
if [[ $# -eq 1 ]]; then
  json=$1
fi

if (( reset )); then
  python3 "$ROOT/hypr-sentinel.py" windows reset "$FILE"
else
  if [[ -n $json ]]; then
    python3 "$ROOT/hypr-sentinel.py" windows apply "$FILE" "$json"
  else
    python3 "$ROOT/hypr-sentinel.py" windows apply "$FILE"
  fi
fi

python3 "$ROOT/hypr-sentinel.py" require apply "$HYPRLAND"

if [[ $SKIP_HYPR == 1 ]]; then
  exit 0
fi

if command -v hyprctl >/dev/null 2>&1; then
  hyprctl reload >/dev/null
fi
