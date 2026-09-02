#!/bin/bash
# Write idle.screensaver and idle.lock through Omarchy's shell.json helper
# so a missing user file is copied from defaults instead of wiping the bar.

set -euo pipefail

screensaver=${1:-}
lock=${2:-}

[[ $screensaver =~ ^[0-9]+$ ]] || {
  echo "set-idle.sh: screensaver seconds must be a non-negative integer" >&2
  exit 1
}
[[ $lock =~ ^[0-9]+$ ]] || {
  echo "set-idle.sh: lock seconds must be a non-negative integer" >&2
  exit 1
}

if ! command -v omarchy-shell-config >/dev/null 2>&1; then
  echo "set-idle.sh: omarchy-shell-config is not on PATH" >&2
  exit 1
fi

# shellcheck disable=SC1091
source omarchy-shell-config

commit "$NORMALIZE
  | .idle = (.idle | object_or_empty)
  | .idle.screensaver = (\$screensaver | tonumber)
  | .idle.lock = (\$lock | tonumber)
" --arg screensaver "$screensaver" --arg lock "$lock"
