#!/bin/bash
# Create an 802.1X Wi-Fi profile. SSID and identity are argv; the password
# is one line on stdin so it never appears in /proc.
set -euo pipefail

ssid=${1:-}
identity=${2:-}
[[ -n $ssid && -n $identity ]] || {
  echo "Usage: enterprise-wifi-connect.sh <ssid> <identity>" >&2
  exit 1
}

IFS= read -r pw
[[ -n $pw ]] || {
  echo "enterprise-wifi-connect.sh: password cannot be empty" >&2
  exit 1
}

u=$(uuidgen)
nmcli connection add type wifi con-name "$ssid" ssid "$ssid" connection.uuid "$u" \
  wifi-sec.key-mgmt wpa-eap 802-1x.eap peap 802-1x.phase2-auth mschapv2 \
  802-1x.identity "$identity" 802-1x.auth-timeout 8 >/dev/null
if ! printf 'set 802-1x.password %s\nsave\nquit\n' "$pw" | nmcli connection edit uuid "$u" >/dev/null; then
  nmcli connection delete uuid "$u" >/dev/null 2>&1 || true
  exit 1
fi
if ! nmcli connection up uuid "$u"; then
  nmcli connection delete uuid "$u" >/dev/null 2>&1 || true
  exit 1
fi
