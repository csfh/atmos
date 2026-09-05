#!/bin/bash
# Set hyprsunset temperature in Kelvin. 6500 is daylight.
# A live night profile also keeps this Kelvin in hyprsunset.conf, so a
# restart does not put the old warmth back. The live process is still
# updated in place; rewriting the conf does not restart hyprsunset.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

temp=${1:-}

usage() {
  echo "Usage: set-nightlight-temp.sh <3000-6500>" >&2
  exit 1
}

[[ $temp =~ ^[1-9][0-9]{3}$ ]] || usage
(( temp >= 3000 && temp <= 6500 )) || usage

# Only a live (uncommented) night profile is rewritten. Omarchy's stock
# conf comments the example out; moving the slider must not enable it.
if [[ -f $ATMOS_HYPRSUNSET_FILE ]]; then
  python3 - "$ATMOS_HYPRSUNSET_FILE" "$temp" <<'PY'
import re
import sys
from pathlib import Path

dest = Path(sys.argv[1])
temp = int(sys.argv[2])
raw = dest.read_text()
if not raw:
    raise SystemExit(0)

temp_re = re.compile(r"^(\s*temperature\s*=\s*)([0-9]+)([ \t]*(#.*)?)?(\r?\n)?$")
in_profile = False
identity = False
changed = False
out = []
for line in raw.splitlines(keepends=True):
    code = line.split("#", 1)[0]
    if re.search(r"profile\s*\{", code):
        in_profile = True
        identity = False
    if in_profile and re.search(r"\bidentity\s*=\s*true\b", code):
        identity = True
    if in_profile and not identity:
        match = temp_re.match(line)
        if match:
            line = f"{match.group(1)}{temp}{match.group(3) or ''}{match.group(5) or ''}"
            changed = True
    if in_profile and "}" in code:
        in_profile = False
    out.append(line)
if changed:
    dest.write_text("".join(out))
PY
fi

if [[ ${ATMOS_SKIP_HYPR:-0} == 1 ]]; then
  exit 0
fi

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
