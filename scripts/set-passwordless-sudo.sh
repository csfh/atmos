#!/bin/bash
# Enable or disable Omarchy passwordless sudo. Prefs already confirmed.
# Writes the same drop-in omarchy-sudo-passwordless uses, via as-root so the
# first enable can use polkit instead of a TTY sudo prompt.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
action=${1:-}
minutes=${2:-15}
user=$(id -un)

usage() {
  echo "Usage: set-passwordless-sudo.sh on <minutes> | off" >&2
  exit 1
}

[[ $user =~ ^[a-z_][a-z0-9_-]*$ ]] || usage

case $action in
  on)
    [[ $minutes =~ ^[1-9][0-9]*$ ]] || usage
    (( minutes >= 1 && minutes <= 240 )) || usage
    "$ROOT/as-root.sh" /bin/bash -c '
      set -euo pipefail
      user=$1
      minutes=$2
      file=/etc/sudoers.d/99-omarchy-nopasswd-$user
      timer=omarchy-nopasswd-expire-$user
      echo "$user ALL=(ALL) NOPASSWD: ALL" >"$file"
      chmod 440 "$file"
      systemctl stop "${timer}.timer" 2>/dev/null || true
      systemd-run --on-active="${minutes}m" --timer-property=AccuracySec=1s --unit="$timer" rm "$file"
    ' bash "$user" "$minutes"
    ;;
  off)
    "$ROOT/as-root.sh" /bin/bash -c '
      set -euo pipefail
      user=$1
      file=/etc/sudoers.d/99-omarchy-nopasswd-$user
      timer=omarchy-nopasswd-expire-$user
      rm -f "$file"
      systemctl stop "${timer}.timer" 2>/dev/null || true
    ' bash "$user"
    ;;
  *)
    usage
    ;;
esac
