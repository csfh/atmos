#!/bin/bash
# Write the managed autostart block in ~/.config/hypr/autostart.lua.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

atmos_hypr_apply autostart "$ATMOS_AUTOSTART_FILE" "$@"
atmos_hypr_reload set-hypr-autostart.sh
