#!/bin/bash
# Strip Atmos-managed Hyprland sentinels and drop the search index cache.
# Keep the Atmos window seed and require.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

if [[ $# -ne 0 ]]; then
  echo "Usage: reset-atmos.sh" >&2
  exit 2
fi

if [[ $ATMOS_SEARCH_INDEX != :memory: ]]; then
  rm -f -- "$ATMOS_SEARCH_INDEX" \
    "${ATMOS_SEARCH_INDEX}-wal" \
    "${ATMOS_SEARCH_INDEX}-shm" \
    "${ATMOS_SEARCH_INDEX}-journal"
fi

python3 "$ROOT/hypr-sentinel.py" look reset "$ATMOS_LOOK_FILE"
python3 "$ROOT/hypr-sentinel.py" input reset "$ATMOS_INPUT_FILE"
python3 "$ROOT/hypr-sentinel.py" autostart reset "$ATMOS_AUTOSTART_FILE"
python3 "$ROOT/hypr-sentinel.py" bindings reset "$ATMOS_BINDINGS_FILE"
python3 "$ROOT/hypr-sentinel.py" windows reset "$ATMOS_WINDOWS_FILE"

seed=$(cd -- "$ROOT/.." && pwd)/packaging/hypr-atmos.lua
if [[ -f $seed ]]; then
  mkdir -p "$(dirname "$ATMOS_WINDOWS_FILE")"
  if [[ ! -f $ATMOS_WINDOWS_FILE ]] || ! grep -q 'o.window("dev.csfh.atmos"' "$ATMOS_WINDOWS_FILE"; then
    cat "$seed" >>"$ATMOS_WINDOWS_FILE"
  fi
fi

python3 "$ROOT/hypr-sentinel.py" require apply "$ATMOS_HYPRLAND_FILE"
atmos_hypr_reload reset-atmos.sh errors
