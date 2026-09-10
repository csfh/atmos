#!/bin/bash
# Install the named workspace bar clone and set showNames / shown count.
# Writes shell.json and plugin files; the shell watches those paths.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

: "${OMARCHY_PATH:=/usr/share/omarchy}"
export OMARCHY_PATH

usage() {
  echo "Usage: set-workspace-bar.sh on|off|count <1-10>" >&2
  exit 1
}

mode=${1:-}
count_value=""
case $mode in
  on | true) mode=on ;;
  off | false) mode=off ;;
  count)
    count_value=${2:-}
    [[ $count_value =~ ^[1-9]$|^10$ ]] || usage
    ;;
  *) usage ;;
esac

user=${USER:-$(id -un)}
clone_id="${user}.workspaces"
plugins_dir="$HOME/.config/omarchy/plugins"
clone_dir="${plugins_dir}/${clone_id}"
widget_src="$ROOT/workspace-bar/Workspaces.qml"
stock_manifest="${OMARCHY_PATH}/shell/plugins/bar/widgets/Workspaces.manifest.json"

install_file() {
  local src=$1 dest=$2
  [[ -f $src ]] || return 1
  mkdir -p "$(dirname "$dest")"
  if [[ -f $dest ]] && cmp -s "$src" "$dest"; then
    return 0
  fi
  cp "$src" "$dest"
}

install_clone() {
  [[ -f $widget_src ]] || {
    echo "set-workspace-bar.sh: missing $widget_src" >&2
    return 1
  }
  mkdir -p "$clone_dir"
  if [[ -f $stock_manifest ]]; then
    local staged
    staged=$(mktemp)
    jq --arg id "$clone_id" --arg name "My Workspaces" '
      .id = $id
      | .name = $name
      | if (.barWidget | type) == "object" then .barWidget.displayName = $name else . end
      | .omarchy = ((if (.omarchy | type) == "object" then .omarchy else {} end) + {clonedFrom: "omarchy.workspaces"})
    ' "$stock_manifest" >"$staged"
    install_file "$staged" "$clone_dir/manifest.json"
    rm -f "$staged"
  elif [[ ! -f $clone_dir/manifest.json ]]; then
    echo "set-workspace-bar.sh: cannot install ${clone_id}" >&2
    return 1
  fi
  install_file "$widget_src" "$clone_dir/Workspaces.qml"
}

command -v omarchy-shell-config >/dev/null 2>&1 || {
  echo "set-workspace-bar.sh: omarchy-shell-config is not on PATH" >&2
  exit 1
}

# shellcheck disable=SC1091
source omarchy-shell-config

write_shell() {
  mkdir -p "$(dirname "$CONFIG_FILE")"
  _SHELL_CONFIG_TMP=$(mktemp)
  if ! jq -S -e "$@" "$NORMALIZE $program" "$(source_file)" >"$_SHELL_CONFIG_TMP"; then
    rm -f "$_SHELL_CONFIG_TMP"
    echo "set-workspace-bar.sh: could not update shell config" >&2
    exit 1
  fi
  mv "$_SHELL_CONFIG_TMP" "$CONFIG_FILE"
  _SHELL_CONFIG_TMP=""
}

if [[ ! -f $clone_dir/Workspaces.qml ]]; then
  install_clone
fi

program='
  | def entry_id:
      if type == "object" then (.id // "" | tostring) else tostring end;
    def retarget($from; $to):
      map(
        if entry_id == $from then
          (if type == "object" then . else {id: .} end) + {id: $to}
        else . end
      );
    def patch($id; $key; $value):
      (map(entry_id) | index($id)) as $i
      | if $i == null then .
        else .[$i] = (.[$i] | if type == "object" then . else {id: .} end | .[$key] = $value)
        end;
    .bar.layout.left |= retarget("omarchy.workspaces"; $id)
    | .bar.layout.center |= retarget("omarchy.workspaces"; $id)
    | .bar.layout.right |= retarget("omarchy.workspaces"; $id)
    | .bar.layout.left |= patch($id; "count"; $count)
    | .bar.layout.center |= patch($id; "count"; $count)
    | .bar.layout.right |= patch($id; "count"; $count)
    | .bar.layout.left |= patch($id; "showNames"; $names)
    | .bar.layout.center |= patch($id; "showNames"; $names)
    | .bar.layout.right |= patch($id; "showNames"; $names)
'

cur=$(jq -c --arg id "$clone_id" '
  def entry_id:
    if type == "object" then (.id // "" | tostring) else tostring end;
  [.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []]
  | add
  | map(select(entry_id == $id))
  | .[0] // {}
' "$(source_file)")
cur_count=$(jq -r '.count // 5' <<<"$cur")
cur_names=$(jq -r 'if .showNames == false then "false" else "true" end' <<<"$cur")
[[ $cur_count =~ ^[1-9]$|^10$ ]] || cur_count=5

names_json=$cur_names
count_json=$cur_count
if [[ $mode == on ]]; then
  names_json=true
elif [[ $mode == off ]]; then
  names_json=false
else
  count_json=$count_value
fi

write_shell \
  --arg id "$clone_id" \
  --argjson names "$names_json" \
  --argjson count "$count_json"

# omarchy-shell talks to $OMARCHY_PATH/shell. A live desktop often runs
# /usr/share/omarchy/shell instead, so ping every candidate. Skip when HOME
# is a test sandbox so we do not poke the session bar.
session_home=$(getent passwd "$(id -un)" | cut -d: -f6)
live_shell=""
if [[ $HOME == "$session_home" ]] && command -v qs >/dev/null 2>&1; then
  for shell_root in /usr/share/omarchy/shell "${OMARCHY_PATH:-}/shell"; do
    [[ -f $shell_root/shell.qml ]] || continue
    if qs ipc -n -p "$shell_root" call -- shell ping >/dev/null 2>&1; then
      live_shell=$shell_root
      break
    fi
  done
fi
if [[ -n $live_shell ]]; then
  qs ipc -n -p "$live_shell" call -- shell setBarWidget "$clone_id" count "$(jq -cn --argjson v "$count_json" '$v')" "{}" >/dev/null 2>&1 || true
  qs ipc -n -p "$live_shell" call -- shell setBarWidget "$clone_id" showNames "$(jq -cn --argjson v "$names_json" '$v')" "{}" >/dev/null 2>&1 || true
fi
