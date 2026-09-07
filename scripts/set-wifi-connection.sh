#!/bin/bash
# Mutate NetworkManager Wi-Fi radio or a saved connection.
# omarchy has no wifi-power / connection-up CLI; nmcli is the source of truth.

set -euo pipefail

action=${1:-}
target=${2:-}

usage() {
  echo "Usage: set-wifi-connection.sh radio <on|off> | up <uuid> | down <uuid> | delete <uuid> | join <ssid> | down-ssid <ssid> | delete-ssid <ssid> | metered <uuid> <yes|no|unknown> | priority <uuid> <n> | mac <uuid> <default|random|stable|permanent|preserve> | ipv4 <uuid> auto | ipv4 <uuid> manual <addr> <prefix> [gateway] [dns] | wireguard-import <file> | hotspot on <ssid> <password> | hotspot off" >&2
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
  metered)
    uuid_ok "$target" || usage
    value=${3:-}
    [[ $value == yes || $value == no || $value == unknown ]] || usage
    nmcli connection modify uuid "$target" connection.metered "$value"
    ;;
  priority)
    uuid_ok "$target" || usage
    value=${3:-}
    [[ $value =~ ^-?[0-9]+$ ]] || usage
    if ((value < -999 || value > 999)); then usage; fi
    nmcli connection modify uuid "$target" connection.autoconnect-priority "$value"
    ;;
  mac)
    uuid_ok "$target" || usage
    value=${3:-}
    [[ $value == default || $value == random || $value == stable || $value == permanent || $value == preserve ]] || usage
    nmcli connection modify uuid "$target" wifi.cloned-mac-address "$value"
    ;;
  ipv4)
    uuid_ok "$target" || usage
    method=${3:-}
    if [[ $method == auto ]]; then
      nmcli connection modify uuid "$target" ipv4.method auto ipv4.addresses "" ipv4.gateway ""
      exit 0
    fi
    [[ $method == manual ]] || usage
    addr=${4:-}
    prefix=${5:-}
    gateway=${6:-}
    dns=${7:-}
    [[ $addr =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]] || usage
    [[ $prefix =~ ^[0-9]+$ && $prefix -ge 1 && $prefix -le 32 ]] || usage
    nmcli connection modify uuid "$target" ipv4.method manual ipv4.addresses "$addr/$prefix"
    if [[ -n $gateway ]]; then
      [[ $gateway =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]] || usage
      nmcli connection modify uuid "$target" ipv4.gateway "$gateway"
    fi
    if [[ -n $dns ]]; then
      nmcli connection modify uuid "$target" ipv4.dns "$dns"
    fi
    ;;
  wireguard-import)
    file=$target
    [[ $file == /* && $file != *..* && ( $file == *.conf || $file == *.nmconnection ) ]] || usage
    [[ -f $file ]] || {
      echo "set-wifi-connection.sh: missing $file" >&2
      exit 1
    }
    nmcli connection import type wireguard file "$file"
    ;;
  hotspot)
    [[ $target == on || $target == off ]] || usage
    if [[ $target == off ]]; then
      nmcli connection down id Hotspot >/dev/null 2>&1 || true
      exit 0
    fi
    ssid=${3:-}
    password=${4:-}
    [[ -n $ssid && $ssid != *$'\n'* && ${#ssid} -le 32 ]] || usage
    [[ ${#password} -ge 8 && ${#password} -le 63 && $password != *$'\n'* ]] || usage
    nmcli device wifi hotspot ssid "$ssid" password "$password"
    ;;
  *)
    usage
    ;;
esac
