#!/bin/bash
# Set one inline bar-widget key through omarchy-shell-config.
# `omarchy bar set --json` sends the value through qs ipc, which spreads a
# JSON array into extra arguments and fails setBarWidget (4 required).

set -euo pipefail

id=${1:-}
key=${2:-}
value=${3:-}

[[ -n $id ]] || {
  echo "set-bar-widget.sh: widget id is required" >&2
  exit 1
}
[[ -n $key ]] || {
  echo "set-bar-widget.sh: setting key is required" >&2
  exit 1
}
[[ -n $value ]] || {
  echo "set-bar-widget.sh: JSON value is required" >&2
  exit 1
}

if ! command -v omarchy-shell-config >/dev/null 2>&1; then
  echo "set-bar-widget.sh: omarchy-shell-config is not on PATH" >&2
  exit 1
fi

: "${OMARCHY_PATH:=/usr/share/omarchy}"
export OMARCHY_PATH

jq -cn --argjson value "$value" '$value' >/dev/null || {
  echo "set-bar-widget.sh: invalid JSON value: $value" >&2
  exit 1
}

# shellcheck disable=SC1091
source omarchy-shell-config

commit "$NORMALIZE
  | def entry_id:
      if type == \"object\" then (.id // \"\" | tostring) else tostring end;
    def with_key(\$key; \$value):
      (if type == \"object\" then . else {id: .} end) + {(\$key): \$value};
    def patch(\$id; \$key; \$value):
      (map(entry_id) | index(\$id)) as \$i
      | if \$i == null then .
        else .[\$i] = (.[\$i] | with_key(\$key; \$value))
        end;
    (.bar.layout.left | map(entry_id) | index(\$id)) as \$li
    | (.bar.layout.center | map(entry_id) | index(\$id)) as \$ci
    | (.bar.layout.right | map(entry_id) | index(\$id)) as \$ri
    | if \$li != null then .bar.layout.left |= patch(\$id; \$key; \$value)
      elif \$ci != null then .bar.layout.center |= patch(\$id; \$key; \$value)
      elif \$ri != null then .bar.layout.right |= patch(\$id; \$key; \$value)
      else error(\"could not find widget \" + \$id)
      end
" --arg id "$id" --arg key "$key" --argjson value "$value"
