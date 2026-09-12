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

: "${OMARCHY_PATH:=/usr/share/omarchy}"
export OMARCHY_PATH

# shellcheck disable=SC1091
source omarchy-shell-config

# shell.json is read, patched with jq, and renamed by commit(). Hold one
# shared lock across that read-modify-write so two Atmos windows changing
# different bar/idle keys cannot base their patch on the same stale file
# and lose each other's edit. Every Atmos shell.json writer takes this lock.
_SHELL_JSON_LOCK="$HOME/.config/omarchy/shell.json.atmos.lock"
mkdir -p "$(dirname "$_SHELL_JSON_LOCK")"
exec {ATMOS_SHELL_LOCK}>"$_SHELL_JSON_LOCK"
flock "$ATMOS_SHELL_LOCK"

commit "$NORMALIZE
  | .idle = (.idle | object_or_empty)
  | .idle.screensaver = (\$screensaver | tonumber)
  | .idle.lock = (\$lock | tonumber)
" --arg screensaver "$screensaver" --arg lock "$lock"
