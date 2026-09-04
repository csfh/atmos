#!/bin/bash
# Enable or disable NTP. Prefs already confirmed true/false.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
on=${1:-}

usage() {
  echo "Usage: set-ntp.sh true|false" >&2
  exit 1
}

[[ $on == true || $on == false ]] || usage

"$ROOT/as-root.sh" timedatectl set-ntp "$on"
