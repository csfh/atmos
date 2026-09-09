#!/bin/bash
# Write the managed workspace rules in ~/.config/hypr/atmos.lua.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

atmos_hypr_apply workspaces "$ATMOS_WINDOWS_FILE" "$@"
python3 "$ROOT/hypr-sentinel.py" require apply "$ATMOS_HYPRLAND_FILE"
atmos_hypr_reload set-hypr-workspaces.sh errors

# default_name only applies when Hyprland creates a workspace. Persistent 1–N
# already exist, so rename them or the bar keeps showing the number.
if [[ ${ATMOS_SKIP_HYPR:-0} != 1 ]] && command -v hyprctl >/dev/null 2>&1 && [[ -n ${ATMOS_HYPR_JSON:-} ]]; then
  python3 - "$ATMOS_HYPR_JSON" <<'PY' || true
import json, subprocess, sys

def lua_string(value):
    return '"' + str(value).replace("\\", "\\\\").replace('"', '\\"') + '"'

raw = json.loads(sys.argv[1])
items = raw.get("items") if isinstance(raw, dict) else []
if not isinstance(items, list):
    raise SystemExit(0)
for item in items:
    if not isinstance(item, dict):
        continue
    wid = str(item.get("id") or "").strip()
    if not wid or wid.startswith("special:"):
        continue
    name = str(item.get("name") or "").strip() or wid
    code = (
        "hl.dispatch(hl.dsp.workspace.rename({ workspace = "
        + lua_string(wid)
        + ", name = "
        + lua_string(name)
        + " }))"
    )
    subprocess.run(["hyprctl", "eval", code], check=False, capture_output=True)
PY
fi
