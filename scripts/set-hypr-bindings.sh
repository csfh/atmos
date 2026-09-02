#!/bin/bash
# Write the managed bindings block in ~/.config/hypr/bindings.lua.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

atmos_hypr_apply bindings "$ATMOS_BINDINGS_FILE" "$@"
atmos_hypr_reload set-hypr-bindings.sh
