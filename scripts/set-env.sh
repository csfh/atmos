#!/bin/bash
# Write ~/.config/environment.d/10-atmos.conf from JSON on argv or stdin.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
DEST="${ATMOS_ENV_FILE:-$HOME/.config/environment.d/10-atmos.conf}"

json=${1:-}
if [[ -z $json ]]; then
  json=$(cat)
fi

python3 - "$DEST" "$json" <<'PY'
import json, os, sys
from pathlib import Path

dest = Path(sys.argv[1])
raw = sys.argv[2] if len(sys.argv) > 2 else "{}"
try:
    payload = json.loads(raw or "{}")
except json.JSONDecodeError:
    payload = {}

begin = "# atmos:env begin"
end = "# atmos:env end"
path_prepend = str(payload.get("pathPrepend") or "").strip()
vars_ = payload.get("vars") if isinstance(payload.get("vars"), list) else []
lines = [begin]
if path_prepend:
    lines.append("PATH=" + path_prepend + ":$PATH")
for item in vars_:
    if not isinstance(item, dict):
        continue
    key = str(item.get("key") or "")
    value = str(item.get("value") or "")
    if not key or key == "PATH":
        continue
    lines.append(f"{key}={value}")
lines.append(end)
body = "\n".join(lines) + "\n"
dest.parent.mkdir(parents=True, exist_ok=True)
dest.write_text(body)
PY
