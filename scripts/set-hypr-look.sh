#!/bin/bash
# Write the managed look block in ~/.config/hypr/looknfeel.lua, then reload Hyprland.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
FILE=${OMARCHY_PREFS_LOOK_FILE:-"$HOME/.config/hypr/looknfeel.lua"}
SKIP_HYPR=${OMARCHY_PREFS_SKIP_HYPR:-0}

usage() {
  echo "Usage: set-hypr-look.sh [--reset] [<json>]" >&2
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
  python3 "$ROOT/hypr-sentinel.py" look reset "$FILE"
else
  if [[ -n $json ]]; then
    python3 "$ROOT/hypr-sentinel.py" look apply "$FILE" "$json"
  else
    python3 "$ROOT/hypr-sentinel.py" look apply "$FILE"
  fi
fi

if [[ $SKIP_HYPR == 1 ]]; then
  exit 0
fi

if command -v hyprctl >/dev/null 2>&1; then
  hyprctl reload >/dev/null
  errors=$(hyprctl configerrors 2>/dev/null || true)
  if [[ -n $errors && $errors != "nothing" ]]; then
    echo "set-hypr-look.sh: hyprctl configerrors:" >&2
    echo "$errors" >&2
    exit 1
  fi
  if [[ -n $json ]] && command -v jq >/dev/null 2>&1; then
    size=$(jq -r '.cursorSize // empty' <<<"$json" 2>/dev/null || true)
    theme=${HYPRCURSOR_THEME:-${XCURSOR_THEME:-}}
    if [[ $size =~ ^[0-9]+$ && -n $theme ]]; then
      hyprctl setcursor "$theme" "$size" >/dev/null 2>&1 || true
    fi
  fi
fi
