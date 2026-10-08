#!/bin/bash
# Opt this user into (or out of) the Hyprland integration a package cannot
# write into $HOME: the window seed, the layout wrapper and their requires.
# Run through `atmos --setup [off|status]`.

set -euo pipefail

HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
APP=$(cd -- "$HERE/.." && pwd)
# shellcheck source=atmos-xdg.sh
. "$HERE/atmos-xdg.sh"

case ${1:-on} in
  on)
    atmos_setup_hypr "$APP"
    echo "Atmos is set up. Undo it with: atmos --setup off"
    ;;
  off)
    atmos_unsetup_hypr "$APP"
    echo "Atmos is set down. Settings you changed in Atmos stay; Reset in Atmos strips them."
    ;;
  status)
    atmos_setup_state
    ;;
  *)
    echo "Usage: atmos --setup [on|off|status]" >&2
    exit 2
    ;;
esac
