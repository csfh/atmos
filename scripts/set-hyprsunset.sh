#!/bin/bash
# Write ~/.config/hypr/hyprsunset.conf from JSON, then restart hyprsunset.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
FILE=${OMARCHY_PREFS_HYPRSUNSET_FILE:-"$HOME/.config/hypr/hyprsunset.conf"}
SKIP_HYPR=${OMARCHY_PREFS_SKIP_HYPR:-0}

usage() {
  echo "Usage: set-hyprsunset.sh <json>" >&2
  exit 1
}

if [[ $# -ne 1 || ${1:-} == -* ]]; then
  usage
fi

json=$1
if ! printf '%s' "$json" | python3 -c '
import json, re, sys
try:
    data = json.load(sys.stdin)
except json.JSONDecodeError as exc:
    print(exc, file=sys.stderr)
    sys.exit(2)
if not isinstance(data, dict):
    sys.exit(2)
time_re = re.compile(r"^([01]?\d|2[0-3]):([0-5]\d)$")
day = str(data.get("day") or "")
night = str(data.get("night") or "")
if not time_re.fullmatch(day) or not time_re.fullmatch(night):
    sys.exit(2)
try:
    temp = int(round(float(data.get("temperature") or 4000)))
except (TypeError, ValueError):
    sys.exit(2)
if temp < 3000 or temp > 6500:
    sys.exit(2)
' ; then
  echo "set-hyprsunset.sh: JSON must include day, night (HH:MM), and temperature 3000-6500" >&2
  exit 1
fi

python3 - "$FILE" "$json" <<'PY'
import json, re, sys
from pathlib import Path

dest = Path(sys.argv[1])
data = json.loads(sys.argv[2])

def pad(n: int) -> str:
    return f"{n:02d}"

def parse_time(raw: str) -> str:
    match = re.fullmatch(r"([01]?\d|2[0-3]):([0-5]\d)", str(raw or "").strip())
    if not match:
        raise SystemExit(2)
    return f"{pad(int(match.group(1)))}:{pad(int(match.group(2)))}"

day = parse_time(data.get("day") or "07:00")
night = parse_time(data.get("night") or "20:00")
night_on = data.get("nightOn") is True
try:
    temp = int(round(float(data.get("temperature") or 4000)))
except (TypeError, ValueError):
    temp = 4000
temp = max(3000, min(6500, temp))

lines = [
    "# Written by omarchy-prefs. Day leaves the screen untinted.",
    "profile {",
    f"    time = {day}",
    "    identity = true",
    "}",
]
if night_on:
    lines.extend(
        [
            "",
            "profile {",
            f"    time = {night}",
            f"    temperature = {temp}",
            "}",
        ]
    )
dest.parent.mkdir(parents=True, exist_ok=True)
dest.write_text("\n".join(lines) + "\n")
PY

if [[ $SKIP_HYPR == 1 ]]; then
  exit 0
fi

if command -v omarchy >/dev/null 2>&1; then
  omarchy restart hyprsunset >/dev/null
fi
