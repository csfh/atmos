#!/bin/bash
# Set pacman ParallelDownloads in /etc/pacman.conf.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
n=${1:-}

usage() {
  echo "Usage: set-parallel-downloads.sh <1-20>" >&2
  exit 1
}

[[ $n =~ ^[1-9][0-9]?$ ]] || usage
(( n >= 1 && n <= 20 )) || usage

"$ROOT/as-root.sh" /bin/bash -c '
  set -euo pipefail
  n=$1
  conf=/etc/pacman.conf
  [[ $n =~ ^[1-9][0-9]?$ ]] || exit 1
  (( n >= 1 && n <= 20 )) || exit 1
  if grep -qE "^[[:space:]]*#?[[:space:]]*ParallelDownloads[[:space:]]*=" "$conf"; then
    sed -i -E "s/^[[:space:]]*#?[[:space:]]*ParallelDownloads[[:space:]]*=.*/ParallelDownloads = $n/" "$conf"
  else
    printf "\nParallelDownloads = %s\n" "$n" >>"$conf"
  fi
' bash "$n"
