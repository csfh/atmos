# Shared XDG paths and staging for Atmos install/update. Source from install.sh or scripts/.

atmos_repo() {
  printf '%s\n' "${ATMOS_REPO:-https://github.com/csfh/atmos.git}"
}

atmos_data_home() {
  printf '%s\n' "${XDG_DATA_HOME:-$HOME/.local/share}/atmos"
}

atmos_cache_home() {
  printf '%s\n' "${XDG_CACHE_HOME:-$HOME/.cache}/atmos"
}

atmos_config_home() {
  printf '%s\n' "${XDG_CONFIG_HOME:-$HOME/.config}/atmos"
}

atmos_bin_home() {
  printf '%s\n' "${XDG_BIN_HOME:-$HOME/.local/bin}"
}

atmos_valid_channel() {
  [[ $1 == alpha ]]
}

atmos_channel() {
  local file
  file="$(atmos_config_home)/channel"
  local c=""
  if [[ -r $file ]]; then
    c=$(<"$file")
    c=${c%%$'\n'*}
    c=${c%% *}
  fi
  if atmos_valid_channel "$c"; then
    printf '%s\n' "$c"
    return
  fi
  printf '%s\n' alpha
}

atmos_write_channel() {
  local name=$1
  atmos_valid_channel "$name" || return 1
  local dir
  dir=$(atmos_config_home)
  mkdir -p "$dir"
  printf '%s\n' "$name" >"$dir/channel"
}

atmos_stage() {
  local src=$1
  local dest=$2
  [[ -d $src && -x $src/bin/atmos && -f $src/shell.qml ]] || return 1
  mkdir -p "$dest"
  local item
  for item in bin components pages services scripts packaging shell.qml; do
    rm -rf "$dest/$item"
    cp -a "$src/$item" "$dest/$item"
  done
  chmod +x "$dest/bin/atmos"
  find "$dest/scripts" -maxdepth 1 -type f -name '*.sh' -exec chmod +x {} +
}

atmos_write_revision() {
  local dest=$1
  local sha=$2
  [[ $sha =~ ^[0-9a-f]{4,40}$ ]] || return 0
  printf '%s\n' "$sha" >"$dest/REVISION"
}

atmos_git_cache() {
  printf '%s\n' "$(atmos_cache_home)/src"
}

atmos_sync_cache() {
  local channel=$1
  local cache repo
  cache=$(atmos_git_cache)
  repo=$(atmos_repo)
  atmos_valid_channel "$channel" || return 1
  mkdir -p "$(dirname "$cache")"
  if [[ ! -d $cache/.git ]]; then
    git clone --depth 1 --branch "$channel" "$repo" "$cache"
    return
  fi
  git -C "$cache" remote set-url origin "$repo"
  git -C "$cache" fetch --depth 1 origin "$channel"
  git -C "$cache" checkout -B "$channel" "FETCH_HEAD"
}

atmos_strip_omarchy_menu() {
  local dest=${1:-$HOME/.config/omarchy/extensions/omarchy-menu.jsonc}
  [[ -f $dest ]] || return 0
  python3 - "$dest" <<'PY'
import json, pathlib, re, sys

def load(path):
    text = pathlib.Path(path).read_text()
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//.*?$", "", text, flags=re.M)
    return json.loads(text)

path = pathlib.Path(sys.argv[1])
try:
    dest = load(path)
except Exception:
    sys.exit(0)
if not isinstance(dest, dict):
    sys.exit(0)

def is_atmos(entry):
    if not isinstance(entry, dict):
        return False
    action = str(entry.get("action") or "").strip()
    return action == "atmos" or action.startswith("atmos ")

kept = {key: value for key, value in dest.items() if not is_atmos(value)}
if kept == dest:
    sys.exit(0)
path.write_text("{}\n" if not kept else json.dumps(kept, indent=2) + "\n")
PY
}

atmos_link_xdg() {
  local dest=$1
  local bin
  bin=$(atmos_bin_home)
  mkdir -p "$bin" "$HOME/.local/share/applications" "$HOME/.config/hypr"
  cat >"$bin/atmos" <<EOF
#!/bin/bash
exec "$dest/bin/atmos" "\$@"
EOF
  chmod +x "$bin/atmos"
  cp "$dest/packaging/atmos.desktop" "$HOME/.local/share/applications/atmos.desktop"
  if [[ ! -f $HOME/.config/hypr/atmos.lua ]]; then
    cp "$dest/packaging/hypr-atmos.lua" "$HOME/.config/hypr/atmos.lua"
  fi
  python3 "$dest/scripts/hypr-sentinel.py" require apply "$HOME/.config/hypr/hyprland.lua"
  atmos_strip_omarchy_menu
  if command -v hyprctl >/dev/null 2>&1; then
    hyprctl reload >/dev/null || true
  fi
}
