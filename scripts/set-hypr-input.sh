#!/bin/bash
# Write the managed input block in ~/.config/hypr/input.lua, then reload Hyprland.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

atmos_hypr_apply input "$ATMOS_INPUT_FILE" "$@"
atmos_hypr_reload set-hypr-input.sh errors
