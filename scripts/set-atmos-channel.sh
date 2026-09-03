#!/bin/bash
# Write the Atmos update channel. Only alpha is allowed.

set -euo pipefail

HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-xdg.sh
. "$HERE/atmos-xdg.sh"

name=${1:-}
if ! atmos_write_channel "$name"; then
  echo "Usage: set-atmos-channel.sh alpha" >&2
  exit 2
fi
