#!/bin/bash
# Set LANG. Generates the locale from locale.gen when it is not installed yet.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
locale=${1:-}

usage() {
  echo "Usage: set-locale.sh <xx_YY.UTF-8>" >&2
  exit 1
}

[[ -n $locale ]] || usage
[[ $locale != -* ]] || usage
[[ $locale == C.UTF-8 || $locale =~ ^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$ ]] || usage

supported=/usr/share/i18n/SUPPORTED
if [[ -r $supported ]]; then
  awk -v loc="$locale" '$2 == "UTF-8" && $1 == loc { found = 1 } END { exit !found }' "$supported" || {
    echo "set-locale.sh: unknown locale: $locale" >&2
    exit 1
  }
fi

if localectl list-locales 2>/dev/null | grep -Fxq -- "$locale"; then
  "$ROOT/as-root.sh" localectl set-locale "LANG=$locale"
  exit 0
fi

"$ROOT/as-root.sh" /bin/bash -c '
  set -euo pipefail
  loc=$1
  gen=/etc/locale.gen
  if [[ $loc != C.UTF-8 && -f $gen ]]; then
    if ! awk -v loc="$loc" '\''$1 == loc { found = 1 } END { exit !found }'\'' "$gen"; then
      printf "%s UTF-8\n" "$loc" >>"$gen"
    fi
    locale-gen
  fi
  localectl set-locale "LANG=$loc"
' bash "$locale"
