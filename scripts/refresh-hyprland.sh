#!/bin/bash
# Restore Hyprland Lua defaults, then put back the prefs drop-in require.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"
HYPRLAND=$ATMOS_HYPRLAND_FILE

omarchy refresh hyprland
python3 "$ROOT/hypr-sentinel.py" require apply "$HYPRLAND"
