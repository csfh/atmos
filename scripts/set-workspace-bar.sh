#!/bin/bash
# Swap the stock numbered workspace widget for the named clone, or back.
# Writes shell.json and plugin files; the shell watches those paths.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-env.sh
source "$ROOT/atmos-env.sh"

: "${OMARCHY_PATH:=/usr/share/omarchy}"
export OMARCHY_PATH

on_off=${1:-}
case $on_off in
  on | true) want=on ;;
  off | false) want=off ;;
  *)
    echo "Usage: set-workspace-bar.sh on|off" >&2
    exit 1
    ;;
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

from_id=omarchy.workspaces
to_id=$clone_id
if [[ $want == on ]]; then
  install_clone
else
  from_id=$clone_id
  to_id=omarchy.workspaces
fi

command -v omarchy-shell-config >/dev/null 2>&1 || {
  echo "set-workspace-bar.sh: omarchy-shell-config is not on PATH" >&2
  exit 1
}

# shellcheck disable=SC1091
source omarchy-shell-config

has_from=$(jq -r --arg from "$from_id" '
  def entry_id:
    if type == "object" then (.id // "" | tostring) else tostring end;
  [.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []]
  | add
  | map(entry_id)
  | index($from) != null
' "$(source_file)")
[[ $has_from == true ]] || exit 0

mkdir -p "$(dirname "$CONFIG_FILE")"
_SHELL_CONFIG_TMP=$(mktemp)
jq -S -e --arg from "$from_id" --arg to "$to_id" "$NORMALIZE
  | def entry_id:
      if type == \"object\" then (.id // \"\" | tostring) else tostring end;
    def retarget(\$from; \$to):
      map(
        if entry_id == \$from then
          (if type == \"object\" then . else {id: .} end) + {id: \$to}
        else . end
      );
    .bar.layout.left |= retarget(\$from; \$to)
    | .bar.layout.center |= retarget(\$from; \$to)
    | .bar.layout.right |= retarget(\$from; \$to)
" "$(source_file)" >"$_SHELL_CONFIG_TMP" || {
  rm -f "$_SHELL_CONFIG_TMP"
  echo "set-workspace-bar.sh: could not update shell config" >&2
  exit 1
}
mv "$_SHELL_CONFIG_TMP" "$CONFIG_FILE"
_SHELL_CONFIG_TMP=""
