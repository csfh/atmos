#!/bin/bash
# Snapper NUMBER_LIMIT and TIMELINE_CREATE for the root config.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
action=${1:-}
value=${2:-}

usage() {
  echo "Usage: set-snapper-policy.sh number-limit <1-50> | timeline <on|off>" >&2
  exit 1
}

conf=/etc/snapper/configs/root
[[ -r $conf ]] || {
  echo "set-snapper-policy.sh: $conf is not readable" >&2
  exit 1
}

case $action in
  number-limit)
    [[ $value =~ ^[1-9][0-9]?$ ]] || usage
    (( value >= 1 && value <= 50 )) || usage
    "$ROOT/as-root.sh" /bin/bash -c '
      set -euo pipefail
      n=$1
      conf=/etc/snapper/configs/root
      [[ $n =~ ^[1-9][0-9]?$ ]] || exit 1
      (( n >= 1 && n <= 50 )) || exit 1
      [[ -f $conf ]] || exit 1
      if grep -qE "^[[:space:]]*NUMBER_LIMIT=" "$conf"; then
        sed -i -E "s/^[[:space:]]*NUMBER_LIMIT=.*/NUMBER_LIMIT=\"$n\"/" "$conf"
      else
        printf "\nNUMBER_LIMIT=\"%s\"\n" "$n" >>"$conf"
      fi
    ' bash "$value"
    ;;
  timeline)
    case $value in
      on | off) ;;
      *) usage ;;
    esac
    want=$([[ $value == on ]] && echo yes || echo no)
    "$ROOT/as-root.sh" /bin/bash -c '
      set -euo pipefail
      want=$1
      conf=/etc/snapper/configs/root
      [[ $want == yes || $want == no ]] || exit 1
      [[ -f $conf ]] || exit 1
      if grep -qE "^[[:space:]]*TIMELINE_CREATE=" "$conf"; then
        sed -i -E "s/^[[:space:]]*TIMELINE_CREATE=.*/TIMELINE_CREATE=\"$want\"/" "$conf"
      else
        printf "\nTIMELINE_CREATE=\"%s\"\n" "$want" >>"$conf"
      fi
      if [[ $want == yes ]]; then
        systemctl enable --now snapper-timeline.timer
        systemctl enable --now snapper-cleanup.timer
      else
        systemctl disable --now snapper-timeline.timer || true
      fi
    ' bash "$want"
    ;;
  *)
    usage
    ;;
esac
