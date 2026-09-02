#!/bin/bash
# Write a user desktop launcher. Prefs already checked the name and command.
set -euo pipefail

name=${1:-}
command=${2:-}
icon=${3:-application-x-executable}

usage() {
  echo "Usage: add-desktop-launcher.sh <name> <command> [icon]" >&2
  exit 1
}

escape() {
  local value=$1
  value=${value//\\/\\\\}
  value=${value//$'\t'/\\t}
  printf '%s' "$value"
}

[[ -n $name && -n $command ]] || usage
[[ $name != -* ]] || usage
[[ $name != */* && $name != *$'\n'* && $name != *$'\r'* ]] || usage
[[ $command != *$'\n'* && $command != *$'\r'* ]] || usage
[[ ${#name} -le 80 ]] || usage
[[ -n $icon && $icon != *$'\n'* && $icon != *$'\r'* ]] || usage

dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$dir"
file="$dir/$name.desktop"

cat >"$file" <<EOF
[Desktop Entry]
Version=1.0
Name=$(escape "$name")
Comment=$(escape "$name")
Exec=$(escape "$command")
Terminal=false
Type=Application
Icon=$(escape "$icon")
StartupNotify=true
EOF

chmod +x "$file"
update-desktop-database "$dir" &>/dev/null || true
