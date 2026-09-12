#!/bin/bash
# Session Presentation Mode: stay awake, DND, screensaver off.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

STATE=${ATMOS_PRESENTATION_FILE:-$HOME/.local/state/omarchy/atmos-presentation.json}
mkdir -p "$(dirname "$STATE")"
# The state file is one small JSON document fully replaced per write.
# Publish through a temp file + rename under a sidecar lock so a second
# Atmos window (or its file watcher) never reads a half-written document.
exec {ATMOS_PRES_LOCK}>"$STATE.atmos.lock"
flock "$ATMOS_PRES_LOCK"

write_state() {
  local tmp
  tmp=$(mktemp "${STATE}.tmp.XXXXXX")
  cat >"$tmp"
  mv "$tmp" "$STATE"
}

on_off=${1:-}
minutes=${2:-120}
[[ $minutes =~ ^[0-9]+$ ]] || minutes=120
if ((minutes < 1)); then minutes=120; fi
if ((minutes > 720)); then minutes=720; fi

case $on_off in
  on | true)
    now=$(date +%s)
    until=$((now + minutes * 60))
    jq -n --argjson on true --argjson until "$until" --argjson minutes "$minutes" \
      '{on:$on, until:$until, minutes:$minutes}' | write_state
    omarchy toggle idle stay-awake >/dev/null 2>&1 || true
    if [[ $(omarchy-shell notifications isDnd 2>/dev/null || true) != on ]]; then
      omarchy toggle notification silencing >/dev/null 2>&1 || true
    fi
    if omarchy toggle enabled screensaver >/dev/null 2>&1; then
      omarchy toggle screensaver-off on >/dev/null 2>&1 || true
    fi
    ;;
  off | false)
    jq -n '{on:false, until:0, minutes:0}' | write_state
    omarchy toggle idle allow-idle >/dev/null 2>&1 || true
    if [[ $(omarchy-shell notifications isDnd 2>/dev/null || true) == on ]]; then
      omarchy toggle notification silencing >/dev/null 2>&1 || true
    fi
    if omarchy toggle enabled screensaver-off >/dev/null 2>&1; then
      omarchy toggle screensaver-off off >/dev/null 2>&1 || true
    fi
    ;;
  *)
    echo "usage: set-presentation.sh on|off [minutes]" >&2
    exit 2
    ;;
esac
