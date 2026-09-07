#!/bin/bash
# Write the Atmos diagnostic report, then copy, save, or hand it to an agent.
# stdin is the Atmos header from Diagnostics.reportText. omarchy-debug is appended.

set -euo pipefail

MODE=${1:-copy}
DEST=${2:-}

CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/atmos"
CACHE_FILE="$CACHE_DIR/diagnostic-report.txt"
mkdir -p "$CACHE_DIR"

HEADER=$(cat)
{
  printf '%s' "$HEADER"
  if [[ $HEADER != *$'\n' ]]; then
    printf '\n'
  fi
  if command -v omarchy-debug >/dev/null 2>&1; then
    printf '\n== omarchy debug ==\n\n'
    omarchy-debug --no-sudo --print 2>/dev/null || true
  fi
} >"$CACHE_FILE"

copy_report() {
  if command -v wl-copy >/dev/null 2>&1; then
    wl-copy -n <"$CACHE_FILE"
  fi
}

case $MODE in
  copy)
    copy_report
    printf '%s\n' "$CACHE_FILE"
    ;;
  save)
    if [[ -z $DEST || $DEST == *$'\n'* || $DEST == *$'\r'* || ${DEST:0:1} != / ]]; then
      echo "diag-report.sh: save needs an absolute path" >&2
      exit 1
    fi
    cp -f "$CACHE_FILE" "$DEST"
    printf '%s\n' "$DEST"
    ;;
  agent)
    copy_report
    if command -v omarchy >/dev/null 2>&1; then
      omarchy agent prompt "A diagnostic report from Atmos is on the clipboard and in $CACHE_FILE. Read that file and diagnose this Omarchy machine." >/dev/null 2>&1 &
    fi
    printf '%s\n' "$CACHE_FILE"
    ;;
  *)
    echo "usage: diag-report.sh copy|save|agent [path]" >&2
    exit 2
    ;;
esac
