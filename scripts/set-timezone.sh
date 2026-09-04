#!/bin/bash
# Set the system timezone. Prefs already confirmed the IANA id.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
tz=${1:-}

usage() {
  echo "Usage: set-timezone.sh <Area/City>" >&2
  exit 1
}

[[ -n $tz ]] || usage
[[ $tz != -* ]] || usage
[[ $tz != *..* ]] || usage
[[ $tz =~ ^[A-Za-z0-9/_+-]+$ ]] || usage

zone="/usr/share/zoneinfo/$tz"
if [[ ! -e $zone || -d $zone ]]; then
  echo "set-timezone.sh: unknown timezone: $tz" >&2
  exit 1
fi

"$ROOT/as-root.sh" timedatectl set-timezone "$tz"
