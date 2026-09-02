#!/bin/bash
# Set hyprsunset temperature in Kelvin. 6500 is daylight.
set -euo pipefail

temp=${1:-}

usage() {
  echo "Usage: set-nightlight-temp.sh <3000-6500>" >&2
  exit 1
}

[[ $temp =~ ^[1-9][0-9]{3}$ ]] || usage
(( temp >= 3000 && temp <= 6500 )) || usage

if ! command -v hyprctl >/dev/null 2>&1; then
  echo "set-nightlight-temp.sh: hyprctl is not on PATH" >&2
  exit 1
fi

if ! pgrep -x hyprsunset >/dev/null; then
  if command -v uwsm-app >/dev/null 2>&1; then
    setsid uwsm-app -- hyprsunset &
  elif command -v hyprsunset >/dev/null 2>&1; then
    setsid hyprsunset &
  fi
fi

for _ in 1 2 3 4 5 6 7 8 9 10; do
  hyprctl hyprsunset temperature "$temp" >/dev/null 2>&1 || true
  sleep 0.2
  current=$(hyprctl hyprsunset temperature 2>/dev/null | grep -oE '[0-9]+' | head -n1 || true)
  [[ $current == "$temp" ]] && break
done

if command -v omarchy-shell >/dev/null 2>&1; then
  omarchy-shell -q nightlight refresh >/dev/null 2>&1 || true
fi
