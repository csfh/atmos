# Source from Atmos scripts.

: "${ATMOS_SKIP_HYPR:=0}"
: "${ATMOS_PAGE:=}"
: "${ATMOS_LOOK_FILE:=$HOME/.config/hypr/looknfeel.lua}"
: "${ATMOS_INPUT_FILE:=$HOME/.config/hypr/input.lua}"
: "${ATMOS_AUTOSTART_FILE:=$HOME/.config/hypr/autostart.lua}"
: "${ATMOS_BINDINGS_FILE:=$HOME/.config/hypr/bindings.lua}"
: "${ATMOS_HYPRLAND_FILE:=$HOME/.config/hypr/hyprland.lua}"
: "${ATMOS_HYPRSUNSET_FILE:=$HOME/.config/hypr/hyprsunset.conf}"
: "${ATMOS_WINDOWS_FILE:=$HOME/.config/hypr/atmos.lua}"
ATMOS_HYPR_JSON=""

atmos_hypr_apply() {
  local kind=$1 file=$2
  shift 2
  local reset=0
  ATMOS_HYPR_JSON=""
  if [[ ${1:-} == --reset ]]; then
    reset=1
    shift
  elif [[ ${1:-} == -* ]]; then
    echo "Usage: set-hypr-${kind}.sh [--reset] [<json>]" >&2
    return 1
  fi
  if [[ $# -gt 1 ]]; then
    echo "Usage: set-hypr-${kind}.sh [--reset] [<json>]" >&2
    return 1
  fi
  if [[ $# -eq 1 ]]; then
    ATMOS_HYPR_JSON=$1
  fi
  if (( reset )); then
    python3 "$ROOT/hypr-sentinel.py" "$kind" reset "$file"
  elif [[ -n $ATMOS_HYPR_JSON ]]; then
    python3 "$ROOT/hypr-sentinel.py" "$kind" apply "$file" "$ATMOS_HYPR_JSON"
  else
    python3 "$ROOT/hypr-sentinel.py" "$kind" apply "$file"
  fi
}

atmos_hypr_reload() {
  local label=${1:-hypr}
  local check=${2:-}
  if [[ ${ATMOS_SKIP_HYPR:-0} == 1 ]]; then
    return 0
  fi
  command -v hyprctl >/dev/null 2>&1 || return 0
  hyprctl reload >/dev/null
  if [[ $check == errors ]]; then
    local errors
    errors=$(hyprctl configerrors 2>/dev/null || true)
    if [[ -n $errors && $errors != nothing ]]; then
      echo "${label}: hyprctl configerrors:" >&2
      echo "$errors" >&2
      return 1
    fi
  fi
}
