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
for bat in /sys/class/power_supply/BAT*/charge_control_end_threshold; do
  [[ -e $bat ]] || continue
  found=$bat
  break
done
if [[ -z $found ]]; then
  echo "set-charge-limit.sh: no charge_control_end_threshold on this machine" >&2
  exit 1
fi

if [[ -w $found ]]; then
  printf '%s\n' "$limit" >"$found"
  exit 0
fi
bash "$ROOT/as-root.sh" tee "$found" <<<"$limit" >/dev/null
