#!/bin/bash
# Apply or reset an Atmos tweak overlay.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
ACTION=${1:-}
ID=${2:-}

case $ACTION in
  gtk-middle-paste)
    dest="${ATMOS_GTK4_FILE:-$HOME/.config/gtk-4.0/settings.ini}"
    mkdir -p "$(dirname "$dest")"
    if [[ ${3:-} == off ]]; then
      rm -f "$dest"
    else
      printf '%s\n' "[Settings]" "gtk-enable-primary-paste=false" >"$dest"
    fi
    ;;
  electron-wayland)
    dest="${ATMOS_ENV_FILE:-$HOME/.config/environment.d/10-atmos.conf}"
    mkdir -p "$(dirname "$dest")"
    if [[ ${3:-} == off ]]; then
      if [[ -f $dest ]]; then
        grep -v '^ELECTRON_OZONE_PLATFORM_HINT=' "$dest" >"$dest.tmp" || true
        printf '%s\n' "ELECTRON_OZONE_PLATFORM_HINT=auto" >>"$dest.tmp"
        mv "$dest.tmp" "$dest"
      else
        printf '%s\n' "# atmos:env begin" "ELECTRON_OZONE_PLATFORM_HINT=auto" "# atmos:env end" >"$dest"
      fi
    elif [[ -f $dest ]]; then
      grep -v '^ELECTRON_OZONE_PLATFORM_HINT=' "$dest" >"$dest.tmp" || true
      mv "$dest.tmp" "$dest"
    fi
    ;;
  force-zero-scaling)
    dest="${ATMOS_ENV_FILE:-$HOME/.config/environment.d/10-atmos.conf}"
    mkdir -p "$(dirname "$dest")"
    if [[ ${3:-} == off ]]; then
      if [[ -f $dest ]]; then
        grep -v '^ATMOS_XWAYLAND_ZERO_SCALING=' "$dest" >"$dest.tmp" || true
        printf '%s\n' "ATMOS_XWAYLAND_ZERO_SCALING=0" >>"$dest.tmp"
        mv "$dest.tmp" "$dest"
      fi
    elif [[ -f $dest ]]; then
      grep -v '^ATMOS_XWAYLAND_ZERO_SCALING=' "$dest" >"$dest.tmp" || true
      mv "$dest.tmp" "$dest"
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
