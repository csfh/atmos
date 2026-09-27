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

write_attribute() {
  local file=$1 value=$2
  if [[ -w $file ]]; then
    printf '%s\n' "$value" >"$file"
  else
    bash "$ROOT/as-root.sh" tee "$file" <<<"$value" >/dev/null
  fi
}

# Dell exposes thresholds even in Adaptive mode, where they do not apply.
# Select Custom when the battery advertises it; other hardware keeps its mode.
charge_types=${found%/*}/charge_types
if [[ -r $charge_types ]]; then
  modes=" $(<"$charge_types") "
  if [[ $modes == *" Custom "* ]]; then
    write_attribute "$charge_types" Custom
  fi
fi

write_attribute "$found" "$limit"
