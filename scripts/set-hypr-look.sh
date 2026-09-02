#!/bin/bash
# Write the managed look block in ~/.config/hypr/looknfeel.lua, then reload Hyprland.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

atmos_hypr_apply look "$ATMOS_LOOK_FILE" "$@"
atmos_hypr_reload set-hypr-look.sh errors

json=$ATMOS_HYPR_JSON
if [[ ${ATMOS_SKIP_HYPR:-0} != 1 && -n $json ]] && command -v hyprctl >/dev/null 2>&1 && command -v jq >/dev/null 2>&1; then
  size=$(jq -r '.cursorSize // empty' <<<"$json" 2>/dev/null || true)
  theme=${HYPRCURSOR_THEME:-${XCURSOR_THEME:-}}
  if [[ $size =~ ^[0-9]+$ && -n $theme ]]; then
    hyprctl setcursor "$theme" "$size" >/dev/null 2>&1 || true
  fi
fi
