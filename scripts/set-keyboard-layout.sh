#!/bin/bash
# Set the X11/console keyboard layout. Hyprland reads XKBLAYOUT from vconsole.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
layout=${1:-}

usage() {
  echo "Usage: set-keyboard-layout.sh <layout>" >&2
  exit 1
}

[[ -n $layout ]] || usage
[[ $layout != -* ]] || usage
[[ $layout =~ ^[a-z0-9]{1,8}$ ]] || usage

sym="/usr/share/X11/xkb/symbols/$layout"
[[ -f $sym ]] || {
  echo "set-keyboard-layout.sh: unknown layout: $layout" >&2
  exit 1
}

"$ROOT/as-root.sh" localectl set-x11-keymap "$layout"
omarchy restart hyprctl >/dev/null 2>&1 || hyprctl reload >/dev/null 2>&1 || true
