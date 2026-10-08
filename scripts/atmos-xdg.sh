# Shared XDG paths and staging for Atmos install/update. Source from install.sh or scripts/.

# A package manager owns the app when the packager left a PACKAGED marker in
# the app root. Its first line is the package version.
atmos_app_root() {
  if [[ -n ${ATMOS_ROOT:-} ]]; then
    printf '%s\n' "$ATMOS_ROOT"
    return
  fi
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd
}

atmos_packaged() {
  [[ -f $(atmos_app_root)/PACKAGED ]]
}

atmos_package_version() {
  local file v=""
  file=$(atmos_app_root)/PACKAGED
  [[ -r $file ]] && v=$(head -n 1 -- "$file")
  [[ $v =~ ^[0-9A-Za-z][0-9A-Za-z.+_~-]{0,39}$ ]] || v=""
  printf '%s\n' "$v"
}

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
  [[ $1 == stable ]]
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
  # alpha was the only channel before stable. Installs that wrote it follow stable now.
  [[ $c == alpha ]] && c=stable
  if atmos_valid_channel "$c"; then
    printf '%s\n' "$c"
    return
  fi
  printf '%s\n' stable
}

atmos_write_channel() {
  local name=$1
  atmos_valid_channel "$name" || return 1
  local dir
  dir=$(atmos_config_home)
  mkdir -p "$dir"
  printf '%s\n' "$name" >"$dir/channel"
}

atmos_build_backend() {
  local src=$1
  local dest=$2
  [[ -f $src/backend/Cargo.toml ]] || {
    echo "atmos: missing $src/backend/Cargo.toml" >&2
    return 1
  }
  command -v cargo >/dev/null 2>&1 || {
    echo "atmos: cargo is not installed" >&2
    return 1
  }
  cargo build --release --manifest-path "$src/backend/Cargo.toml" || return 1
  mkdir -p "$dest/bin" || return 1
  cp -a "$src/backend/target/release/ratmos" "$dest/bin/ratmos" || return 1
  chmod +x "$dest/bin/ratmos"
}

atmos_stage() (
  local src=$1
  local dest=$2
  [[ -d $src && -x $src/bin/atmos && -f $src/shell.qml ]] || return 1
  # Prepare the whole app before touching the installed copy. A failed Rust
  # build must leave the existing launcher, backend and QML together.
  local stage
  stage=$(mktemp -d) || return 1
  trap 'rm -rf -- "$stage"' EXIT
  local item
  for item in bin components pages services scripts packaging icons shell.qml; do
    cp -a "$src/$item" "$stage/$item" || return 1
  done
  chmod +x "$stage/bin/atmos" || return 1
  find "$stage/scripts" -maxdepth 1 -type f -name '*.sh' -exec chmod +x {} + || return 1
  atmos_build_backend "$src" "$stage" || return 1
  mkdir -p "$dest" || return 1
  for item in bin components pages services scripts packaging icons shell.qml; do
    rm -rf "$dest/$item" || return 1
    cp -a "$stage/$item" "$dest/$item" || return 1
  done
)

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

atmos_write_desktop() {
  local dest=$1
  local out=${2:-${XDG_DATA_HOME:-$HOME/.local/share}/applications/atmos.desktop}
  local src=$dest/packaging/atmos.desktop
  local exe=$dest/bin/atmos
  [[ -f $src && -x $exe ]] || return 1
  mkdir -p "$(dirname -- "$out")"
  python3 - "$src" "$out" "$exe" <<'PY'
from pathlib import Path
import sys

src, dest, exe = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
text = src.read_text()
if "Exec=atmos\n" not in text:
    raise SystemExit("atmos.desktop missing Exec=atmos")
text = text.replace("Exec=atmos\n", f"Exec={exe}\n", 1)
text = text.replace("TryExec=atmos\n", f"TryExec={exe}\n", 1)
dest.write_text(text)
PY
}

atmos_write_bin_wrapper() {
  local dest=$1
  local bin
  bin=$(atmos_bin_home)
  mkdir -p "$bin"
  # A leftover symlink here would make `cat >` overwrite dest/bin/atmos.
  rm -f "$bin/atmos"
  cat >"$bin/atmos" <<EOF
#!/bin/bash
exec "$dest/bin/atmos" "\$@"
EOF
  chmod +x "$bin/atmos"
}

atmos_link_xdg() {
  local dest=$1
  mkdir -p "$HOME/.local/share/applications" "$HOME/.config/hypr"
  atmos_write_bin_wrapper "$dest"
  atmos_write_desktop "$dest"
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${XDG_DATA_HOME:-$HOME/.local/share}/applications" >/dev/null 2>&1 || true
  fi
  atmos_setup_hypr "$dest"
}

# Hyprland integration for the current user: the window seed, the layout
# wrapper, and their requires in hyprland.lua. Idempotent.
atmos_setup_hypr() {
  local dest=$1
  local hypr="${ATMOS_HYPR_DIR:-$HOME/.config/hypr}"
  mkdir -p "$hypr"
  if [[ ! -f $hypr/atmos.lua ]]; then
    cp "$dest/packaging/hypr-atmos.lua" "$hypr/atmos.lua"
  fi
  cp "$dest/packaging/hypr-atmos-layout.lua" "$hypr/atmos_layout.lua"
  python3 "$dest/scripts/hypr-sentinel.py" require enable "$hypr/hyprland.lua"
  atmos_strip_omarchy_menu
  if command -v hyprctl >/dev/null 2>&1; then
    hyprctl reload >/dev/null || true
  fi
}

# Undo atmos_setup_hypr. atmos.lua is the user's file once edited, so it goes
# only while it is still the seed. Sentinel blocks (the user's settings) stay;
# Reset in Atmos strips those.
atmos_unsetup_hypr() {
  local dest=$1
  local hypr="${ATMOS_HYPR_DIR:-$HOME/.config/hypr}"
  python3 "$dest/scripts/hypr-sentinel.py" require reset "$hypr/hyprland.lua"
  rm -f -- "$hypr/atmos_layout.lua"
  if [[ -f $hypr/atmos.lua ]] && cmp -s -- "$hypr/atmos.lua" "$dest/packaging/hypr-atmos.lua"; then
    rm -f -- "$hypr/atmos.lua"
  fi
  if command -v hyprctl >/dev/null 2>&1; then
    hyprctl reload >/dev/null || true
  fi
}

atmos_setup_state() {
  local hypr="${ATMOS_HYPR_DIR:-$HOME/.config/hypr}"
  if [[ -f $hypr/hyprland.lua ]] && grep -q 'hypr\.atmos' "$hypr/hyprland.lua"; then
    printf 'on\n'
  else
    printf 'off\n'
  fi
}
