#!/bin/bash
# Mutate NetworkManager Wi-Fi radio or a saved connection.
# omarchy has no wifi-power / connection-up CLI; nmcli is the source of truth.

set -euo pipefail

action=${1:-}
target=${2:-}

usage() {
  echo "Usage: set-wifi-connection.sh radio <on|off> | up <uuid> | down <uuid> | delete <uuid> | join <ssid> | down-ssid <ssid> | delete-ssid <ssid>" >&2
  exit 1
}

if ! command -v nmcli >/dev/null 2>&1; then
  echo "set-wifi-connection.sh: nmcli is not installed" >&2
  exit 1
fi

uuid_ok() {
  [[ $1 =~ ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$ ]]
}

case "$action" in
  radio)
    [[ $target == on || $target == off ]] || usage
    nmcli radio wifi "$target"
    ;;
  up | down)
    uuid_ok "$target" || usage
    nmcli connection "$action" uuid "$target"
    ;;
  delete)
    uuid_ok "$target" || usage
    nmcli connection delete uuid "$target"
    ;;
  join)
    [[ -n $target && $target != -* ]] || usage
    pw=""
    IFS= read -r pw || true
    if [[ -n $pw ]]; then
      nmcli --wait 15 device wifi connect "$target" password "$pw"
    else
      nmcli --wait 15 device wifi connect "$target"
    fi
    ;;
  down-ssid)
    [[ -n $target && $target != -* ]] || usage
    nmcli connection down id "$target"
    ;;
  delete-ssid)
    [[ -n $target && $target != -* ]] || usage
    nmcli connection delete id "$target"
    ;;
  *)
    usage
    ;;
esac
