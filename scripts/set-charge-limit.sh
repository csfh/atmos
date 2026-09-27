#!/bin/bash
# Write a battery charge end threshold when the kernel exposes one.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
limit=${1:-}
if [[ ! $limit =~ ^[0-9]+$ ]]; then
  echo "usage: set-charge-limit.sh <50-100>" >&2
  exit 2
fi
if ((limit < 50 || limit > 100)); then
  echo "set-charge-limit.sh: limit must be 50-100" >&2
  exit 2
fi

found=""
power_supply_dir=${ATMOS_POWER_SUPPLY_DIR:-/sys/class/power_supply}
for bat in "$power_supply_dir"/BAT*/charge_control_end_threshold; do
  [[ -e $bat ]] || continue
  found=$bat
  break
done
if [[ -z $found ]]; then
  echo "set-charge-limit.sh: no charge_control_end_threshold on this machine" >&2
  exit 1
fi

# Dell exposes thresholds even in Adaptive mode, where they do not apply.
# Kernel marks the active type in [brackets], so " Custom " means offered
# but not selected. [Custom] and hardware without Custom stay untouched.
charge_types=${found%/*}/charge_types
mode_file=""
if [[ -r $charge_types ]]; then
  modes=" $(<"$charge_types") "
  if [[ $modes == *" Custom "* ]]; then
    mode_file=$charge_types
  fi
fi

if [[ -w $found && ( -z $mode_file || -w $mode_file ) ]]; then
  [[ -z $mode_file ]] || printf 'Custom\n' >"$mode_file"
  printf '%s\n' "$limit" >"$found"
  exit 0
fi
# One elevation for both writes, so polkit asks once. Custom goes first and a
# rejected mode write leaves the threshold alone. /bin/bash is an absolute
# path so the pkexec fallback can run it.
bash "$ROOT/as-root.sh" /bin/bash -c '
  set -euo pipefail
  if [[ -n $1 ]]; then
    printf "Custom\n" >"$1"
  fi
  printf "%s\n" "$2" >"$3"
' bash "$mode_file" "$limit" "$found"
