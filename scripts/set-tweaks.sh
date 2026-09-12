#!/bin/bash
# Apply or reset an Atmos tweak overlay.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
ACTION=${1:-}
ID=${2:-}

# Read-modify-write branches below serialize on the destination's sidecar
# lock so two Atmos windows cannot interleave a read and a rename. Temp
# files are unique per run ($dest.tmp was shared between concurrent runs).
tweak_rmw() {
  local dest=$1
  shift
  local lock="$dest.atmos.lock"
  mkdir -p "$(dirname "$dest")" "$(dirname "$lock")"
  exec {ATMOS_TWEAK_LOCK}>"$lock"
  flock "$ATMOS_TWEAK_LOCK"
  "$@"
  exec {ATMOS_TWEAK_LOCK}>&-
}

tweak_filter() {
  local dest=$1 pattern=$2 extra=${3:-}
  local tmp
  tmp=$(mktemp "${dest}.tmp.XXXXXX")
  grep -v "$pattern" "$dest" >"$tmp" || true
  if [[ -n $extra ]]; then
    printf '%s\n' "$extra" >>"$tmp"
  fi
  mv "$tmp" "$dest"
}

tweak_seed() {
  local dest=$1
  shift
  local tmp
  tmp=$(mktemp "${dest}.tmp.XXXXXX")
  printf '%s\n' "$@" >"$tmp"
  mv "$tmp" "$dest"
}

case $ACTION in
  gtk-middle-paste)
    dest="${ATMOS_GTK4_FILE:-$HOME/.config/gtk-4.0/settings.ini}"
    mkdir -p "$(dirname "$dest")"
    if [[ ${3:-} == off ]]; then
      rm -f "$dest"
    else
      tweak_rmw "$dest" tweak_seed "$dest" "[Settings]" "gtk-enable-primary-paste=false"
    fi
    ;;
  electron-wayland)
    dest="${ATMOS_ENV_FILE:-$HOME/.config/environment.d/10-atmos.conf}"
    mkdir -p "$(dirname "$dest")"
    if [[ ${3:-} == off ]]; then
      if [[ -f $dest ]]; then
        tweak_rmw "$dest" tweak_filter "$dest" '^ELECTRON_OZONE_PLATFORM_HINT=' "ELECTRON_OZONE_PLATFORM_HINT=auto"
      else
        tweak_rmw "$dest" tweak_seed "$dest" "# atmos:env begin" "ELECTRON_OZONE_PLATFORM_HINT=auto" "# atmos:env end"
      fi
    elif [[ -f $dest ]]; then
      tweak_rmw "$dest" tweak_filter "$dest" '^ELECTRON_OZONE_PLATFORM_HINT='
    fi
    ;;
  force-zero-scaling)
    dest="${ATMOS_ENV_FILE:-$HOME/.config/environment.d/10-atmos.conf}"
    mkdir -p "$(dirname "$dest")"
    if [[ ${3:-} == off ]]; then
      if [[ -f $dest ]]; then
        tweak_rmw "$dest" tweak_filter "$dest" '^ATMOS_XWAYLAND_ZERO_SCALING=' "ATMOS_XWAYLAND_ZERO_SCALING=0"
      fi
    elif [[ -f $dest ]]; then
      tweak_rmw "$dest" tweak_filter "$dest" '^ATMOS_XWAYLAND_ZERO_SCALING='
    fi
    ;;
  swappiness)
    dest=/etc/sysctl.d/99-atmos-swappiness.conf
    if [[ ${3:-} == off ]]; then
      rm -f "$dest"
    else
      printf '%s\n' "vm.swappiness = 10" >"$dest"
    fi
    ;;
  *)
    echo "usage: set-tweaks.sh gtk-middle-paste|electron-wayland|force-zero-scaling|swappiness on|off" >&2
    exit 2
    ;;
esac
