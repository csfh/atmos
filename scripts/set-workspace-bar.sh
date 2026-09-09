#!/bin/bash
# Swap the stock numbered workspace widget for the named clone, or back.
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

install_clone() {
  if [[ ! -d $clone_dir ]] && command -v omarchy >/dev/null 2>&1; then
    omarchy plugin clone omarchy.workspaces >/dev/null 2>&1 || true
  fi
  if [[ ! -d $clone_dir ]]; then
    [[ -f $stock_manifest && -f $widget_src ]] || {
      echo "set-workspace-bar.sh: cannot install ${clone_id}" >&2
      return 1
    }
    mkdir -p "$clone_dir"
    jq --arg id "$clone_id" --arg name "My Workspaces" '
      .id = $id
      | .name = $name
      | if (.barWidget | type) == "object" then .barWidget.displayName = $name else . end
      | .omarchy = ((if (.omarchy | type) == "object" then .omarchy else {} end) + {clonedFrom: "omarchy.workspaces"})
    ' "$stock_manifest" >"$clone_dir/manifest.json"
  fi
  dest="${clone_dir}/Workspaces.qml"
  if [[ -f $widget_src ]]; then
    if [[ ! -f $dest ]] || grep -q 'atmos:workspace-labels' "$dest" || grep -Fq 'focused ? "\uDB85\uDCFB"' "$dest"; then
      cp "$widget_src" "$dest"
    fi
  fi
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
commit "$NORMALIZE
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
" --arg from "$from_id" --arg to "$to_id"
