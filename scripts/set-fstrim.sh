#!/bin/bash
# Enable or disable weekly SSD TRIM.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
value=${1:-}

usage() {
  echo "Usage: set-fstrim.sh <on|off>" >&2
  exit 1
}

case $value in
  on | off) ;;
  *) usage ;;
esac

"$ROOT/as-root.sh" /bin/bash -c '
  set -euo pipefail
  value=$1
  if [[ $value == on ]]; then
    systemctl enable --now fstrim.timer
  else
    systemctl disable --now fstrim.timer
  fi
' bash "$value"
