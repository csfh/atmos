#!/bin/bash
# Restore Hyprland Lua defaults, then put back the prefs drop-in require.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
HYPRLAND=${OMARCHY_PREFS_HYPRLAND_FILE:-"$HOME/.config/hypr/hyprland.lua"}

omarchy refresh hyprland
python3 "$ROOT/hypr-sentinel.py" require apply "$HYPRLAND"
