#!/bin/bash
# Set the system timezone. Prefs already confirmed the IANA id.
set -euo pipefail

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

pkexec timedatectl set-timezone "$tz"
