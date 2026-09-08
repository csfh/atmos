#!/bin/bash
# Write the Favorites list. UI chrome, not Omarchy settings.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

FILE=${ATMOS_FAVORITES_FILE:-$HOME/.local/state/omarchy/atmos-favorites.json}

if [[ $# -ne 2 || $1 != write ]]; then
  echo "usage: set-favorites.sh write <json>" >&2
  exit 2
fi

python3 - "$FILE" "$2" <<'PY'
import json, os, sys, tempfile

path, raw = sys.argv[1], sys.argv[2]
try:
    data = json.loads(raw)
except json.JSONDecodeError:
    sys.exit(2)
items = data.get("items") if isinstance(data, dict) else data
if not isinstance(items, list):
    sys.exit(2)
payload = json.dumps({"items": items}, indent=2) + "\n"
os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
fd, tmp = tempfile.mkstemp(prefix=".favorites.", dir=os.path.dirname(path) or ".")
try:
    with os.fdopen(fd, "w") as fh:
        fh.write(payload)
    os.replace(tmp, path)
except Exception:
    try:
        os.unlink(tmp)
    except OSError:
        pass
    raise
PY
