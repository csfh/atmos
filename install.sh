#!/bin/bash
# Install Atmos into XDG dirs for the current user. This builds ratmos; ratmos
# install stages the app, writes the desktop entry, and sets up Hyprland.

set -euo pipefail

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "install.sh: $1 is not installed${2:+ ($2)}" >&2
    exit 1
  }
}

need git
need quickshell
need cargo "install Rust to build the ratmos backend: sudo pacman -S rust"

here=""
src=${BASH_SOURCE[0]:-$0}
if [[ $src != /dev/fd/* && $src != /proc/self/fd/* ]]; then
  here=$(cd -- "$(dirname -- "$src")" 2>/dev/null && pwd) || here=""
fi

if [[ -n $here && -x $here/bin/atmos && -f $here/backend/Cargo.toml ]]; then
  SOURCE=$here
else
  SOURCE=${XDG_CACHE_HOME:-$HOME/.cache}/atmos/src
  repo=${ATMOS_REPO:-https://github.com/csfh/atmos.git}
  mkdir -p "$(dirname "$SOURCE")"
  if [[ ! -d $SOURCE/.git ]]; then
    git clone --depth 1 --branch stable "$repo" "$SOURCE"
  else
    git -C "$SOURCE" remote set-url origin "$repo"
    git -C "$SOURCE" fetch --depth 1 origin stable
    git -C "$SOURCE" checkout -B stable FETCH_HEAD
  fi
fi

# ratmos is compiled here for this machine, so aarch64 (Apple Silicon) needs no
# separate binary. Say so, because the build is the slow step.
echo "Building ratmos for $(uname -m). The Rust build is the slow step."
cargo build --release --manifest-path "$SOURCE/backend/Cargo.toml"
exec "$SOURCE/backend/target/release/ratmos" install "$SOURCE"
