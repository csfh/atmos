#!/bin/bash
# Write the managed input block in ~/.config/hypr/input.lua, then reload Hyprland.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
FILE=${OMARCHY_PREFS_INPUT_FILE:-"$HOME/.config/hypr/input.lua"}
SKIP_HYPR=${OMARCHY_PREFS_SKIP_HYPR:-0}

usage() {
  echo "Usage: set-hypr-input.sh [--reset] [<json>]" >&2
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
  python3 "$ROOT/hypr-sentinel.py" input reset "$FILE"
else
  if [[ -n $json ]]; then
    python3 "$ROOT/hypr-sentinel.py" input apply "$FILE" "$json"
  else
    python3 "$ROOT/hypr-sentinel.py" input apply "$FILE"
  fi
fi

if [[ $SKIP_HYPR == 1 ]]; then
  exit 0
fi

if command -v hyprctl >/dev/null 2>&1; then
  hyprctl reload >/dev/null
  errors=$(hyprctl configerrors 2>/dev/null || true)
  if [[ -n $errors && $errors != "nothing" ]]; then
    echo "set-hypr-input.sh: hyprctl configerrors:" >&2
    echo "$errors" >&2
    exit 1
  fi
fi
