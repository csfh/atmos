#!/bin/bash
# Install Atmos into XDG dirs for the current user.

set -euo pipefail

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "install.sh: $1 is not installed" >&2
    exit 1
  }
}

need git
need python3
need quickshell

here=""
src=${BASH_SOURCE[0]:-$0}
if [[ $src != /dev/fd/* && $src != /proc/self/fd/* ]]; then
  here=$(cd -- "$(dirname -- "$src")" 2>/dev/null && pwd) || here=""
fi

if [[ -n $here && -x $here/bin/atmos && -f $here/scripts/atmos-xdg.sh ]]; then
  # shellcheck source=scripts/atmos-xdg.sh
  . "$here/scripts/atmos-xdg.sh"
  SOURCE=$here
else
  cache=${XDG_CACHE_HOME:-$HOME/.cache}/atmos/src
  repo=${ATMOS_REPO:-https://github.com/csfh/atmos.git}
  mkdir -p "$(dirname "$cache")"
  if [[ ! -d $cache/.git ]]; then
    git clone --depth 1 --branch alpha "$repo" "$cache"
  else
    git -C "$cache" remote set-url origin "$repo"
    git -C "$cache" fetch --depth 1 origin alpha
    git -C "$cache" checkout -B alpha FETCH_HEAD
  fi
  # shellcheck source=scripts/atmos-xdg.sh
  . "$cache/scripts/atmos-xdg.sh"
  SOURCE=$cache
fi

DEST=$(atmos_data_home)
atmos_stage "$SOURCE" "$DEST"
atmos_write_channel alpha
sha=""
if git -C "$SOURCE" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  sha=$(git -C "$SOURCE" rev-parse HEAD)
fi
atmos_write_revision "$DEST" "$sha"
atmos_link_xdg "$DEST"

echo "Atmos is installed. Run: atmos"
