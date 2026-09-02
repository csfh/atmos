#!/bin/bash
# Write the managed window-rule block in ~/.config/hypr/atmos.lua.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

atmos_hypr_apply windows "$ATMOS_WINDOWS_FILE" "$@"
python3 "$ROOT/hypr-sentinel.py" require apply "$ATMOS_HYPRLAND_FILE"
atmos_hypr_reload set-hypr-windows.sh
