#!/bin/bash
# Write the managed monitor block in ~/.config/hypr/monitors.lua.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

atmos_hypr_apply monitors "$ATMOS_MONITORS_FILE" "$@"
atmos_hypr_reload set-hypr-monitors.sh errors
