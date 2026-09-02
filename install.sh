#!/bin/bash
# Install Atmos for the current user on Omarchy.
set -euo pipefail

REPO=${ATMOS_REPO:-https://github.com/csfh/atmos.git}
DEST=${ATMOS_HOME:-$HOME/.local/share/atmos}

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "install.sh: $1 is not installed" >&2
    exit 1
  }
}

need git
need python3
need quickshell

# BASH_SOURCE is /dev/fd/N when piped from curl — no local bin/atmos, so clone.
here=""
src=${BASH_SOURCE[0]:-$0}
if [[ $src != /dev/fd/* && $src != /proc/self/fd/* ]]; then
  here=$(cd -- "$(dirname -- "$src")" 2>/dev/null && pwd) || here=""
fi

if [[ -n $here && -x $here/bin/atmos ]]; then
  ROOT=$here
else
  if [[ -d $DEST/.git ]]; then
    git -C "$DEST" pull --ff-only
  else
    mkdir -p "$(dirname "$DEST")"
    git clone --depth 1 "$REPO" "$DEST"
  fi
  ROOT=$DEST
fi

mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications" \
  "$HOME/.config/omarchy/extensions" "$HOME/.config/hypr"

ln -sfn "$ROOT/bin/atmos" "$HOME/.local/bin/atmos"
cp "$ROOT/packaging/atmos.desktop" "$HOME/.local/share/applications/atmos.desktop"

if [[ ! -f $HOME/.config/hypr/atmos.lua ]]; then
  cp "$ROOT/packaging/hypr-atmos.lua" "$HOME/.config/hypr/atmos.lua"
fi

python3 "$ROOT/scripts/hypr-sentinel.py" require apply "$HOME/.config/hypr/hyprland.lua"

python3 - "$ROOT/packaging/omarchy-menu.jsonc" "$HOME/.config/omarchy/extensions/omarchy-menu.jsonc" <<'PY'
import json, pathlib, re, sys

def load(path):
    text = pathlib.Path(path).read_text()
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//.*?$", "", text, flags=re.M)
    return json.loads(text)

src_path, dest_path = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
src = load(src_path)
dest = load(dest_path) if dest_path.exists() else {}
if not isinstance(dest, dict):
    dest = {}
dest.update(src)
dest_path.write_text(json.dumps(dest, indent=2) + "\n")
PY

if command -v hyprctl >/dev/null 2>&1; then
  hyprctl reload >/dev/null || true
fi

echo "Atmos is installed. Run: atmos"
