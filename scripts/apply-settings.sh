#!/bin/bash
# Apply a settings plan from services/Settings.js planImport().
#
# The plan arrives on stdin as
#   { "schema": 1, "changes": [ { "key": ..., "value": ..., "from": ... } ] }
# and every change is dispatched to the writer that already owns that
# setting. Nothing here writes a config file directly.
#
#   --dry-run   print the command for each change and run none of them
#   --plan F    read the plan from F instead of stdin
#   --snapshot F  use a snapshot from F instead of running snapshot.sh
#   --backup DIR  where to leave the undo plan and file copies
#
# A failing change is reported and the rest still run. Stopping halfway
# leaves the most confusing state possible, so the run finishes and the
# exit status tells you whether anything failed.

set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
STATE_HOME=${XDG_STATE_HOME:-$HOME/.local/state}

# snapshot.sh defends this and the writers assume it, but omarchy-shell-config
# reads it too and dies on an unbound variable when a plan is applied from
# outside a desktop session.
: "${OMARCHY_PATH:=/usr/share/omarchy}"
export OMARCHY_PATH

# The sentinel writers ask Hyprland to reload after writing, and report the
# reload's exit status as their own. With no Hyprland running that turns a
# write that succeeded into a change reported as failed. The writers already
# take this seam for their own tests.
if [[ -z ${ATMOS_SKIP_HYPR:-} ]] && ! hyprctl version >/dev/null 2>&1; then
  ATMOS_SKIP_HYPR=1
fi
export ATMOS_SKIP_HYPR=${ATMOS_SKIP_HYPR:-0}
DRY=0
PROGRESS=0
NO_BACKUP=0
PLAN_FILE=""
SNAPSHOT_FILE=""
BACKUP_DIR=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run) DRY=1; shift ;;
    --progress) PROGRESS=1; shift ;;
    --no-backup) NO_BACKUP=1; shift ;;
    --plan) PLAN_FILE=${2:-}; shift 2 ;;
    --snapshot) SNAPSHOT_FILE=${2:-}; shift 2 ;;
    --backup) BACKUP_DIR=${2:-}; shift 2 ;;
    *) echo "Usage: apply-settings.sh [--dry-run] [--progress] [--no-backup] [--plan <file>] [--snapshot <file>] [--backup <dir>]" >&2; exit 2 ;;
  esac
done

for tool in jq python3 node; do
  command -v "$tool" >/dev/null 2>&1 || { echo "apply-settings.sh: $tool is required" >&2; exit 1; }
done

if [[ -n $PLAN_FILE ]]; then
  [[ -r $PLAN_FILE ]] || { echo "apply-settings.sh: cannot read $PLAN_FILE" >&2; exit 1; }
  PLAN=$(cat -- "$PLAN_FILE")
else
  PLAN=$(cat)
fi

jq -e 'type == "object" and (.changes | type == "array")' <<<"$PLAN" >/dev/null 2>&1 ||
  { echo "apply-settings.sh: the plan needs a changes array" >&2; exit 1; }

schema=$(jq -r '.schema // 0' <<<"$PLAN")
if [[ $schema != 1 ]]; then
  echo "apply-settings.sh: this plan is schema $schema and this Atmos applies 1" >&2
  exit 1
fi

count=$(jq -r '.changes | length' <<<"$PLAN")
if [[ $count -eq 0 ]]; then
  echo "Nothing to apply."
  exit 0
fi

if [[ -n $SNAPSHOT_FILE ]]; then
  [[ -r $SNAPSHOT_FILE ]] || { echo "apply-settings.sh: cannot read $SNAPSHOT_FILE" >&2; exit 1; }
  SNAPSHOT=$(cat -- "$SNAPSHOT_FILE")
else
  SNAPSHOT=$(bash "$ROOT/snapshot.sh")
fi

# Files a writer may rewrite, so a copy exists before it does.
backup_for() {
  case $1 in
    hyprLook) printf '%s\n' "$HOME/.config/hypr/looknfeel.lua" ;;
    hyprInput) printf '%s\n' "$HOME/.config/hypr/input.lua" ;;
    bindings) printf '%s\n' "$HOME/.config/hypr/bindings.lua" ;;
    autostart) printf '%s\n' "$HOME/.config/hypr/autostart.lua" ;;
    windowRules) printf '%s\n' "$HOME/.config/hypr/atmos.lua" ;;
    nightlightSchedule | nightlightTemp) printf '%s\n' "${ATMOS_HYPRSUNSET_FILE:-$HOME/.config/hypr/hyprsunset.conf}" ;;
    clock | barLayout) printf '%s\n' "$HOME/.config/omarchy/shell.json" ;;
    *) : ;;
  esac
}

declare -a RUN_KEYS=()
declare -a RUN_CMDS=()
declare -a RUN_GROUPS=()
declare -a RUN_STDIN=()

queue() {
  local key=$1 group=$2
  shift 2
  RUN_KEYS+=("$key")
  RUN_GROUPS+=("$group")
  RUN_CMDS+=("$(printf '%s\0' "$@" | base64 -w0)")
  RUN_STDIN+=("")
}

# The kernel caps a single argv entry at 128 KiB, so a long list handed to a
# writer as an argument fails the exec with "Argument list too long". The
# sentinel writers take their payload on stdin when given no argument, which
# has no such limit.
queue_stdin() {
  local key=$1 group=$2 payload=$3
  shift 3
  queue "$key" "$group" "$@"
  RUN_STDIN[${#RUN_STDIN[@]}-1]=$payload
}

PLAN_PATH=$PLAN_FILE
SNAPSHOT_PATH=$SNAPSHOT_FILE
PLAN_TEMP=""
SNAP_TEMP=""
cleanup_dispatch() {
  [[ -n $PLAN_TEMP ]] && rm -f -- "$PLAN_TEMP"
  [[ -n $SNAP_TEMP ]] && rm -f -- "$SNAP_TEMP"
  return 0
}
trap cleanup_dispatch EXIT
if [[ -z $PLAN_PATH ]]; then
  PLAN_TEMP=$(mktemp)
  printf '%s' "$PLAN" >"$PLAN_TEMP"
  PLAN_PATH=$PLAN_TEMP
fi
if [[ -z $SNAPSHOT_PATH ]]; then
  SNAP_TEMP=$(mktemp)
  printf '%s' "$SNAPSHOT" >"$SNAP_TEMP"
  SNAPSHOT_PATH=$SNAP_TEMP
fi

# JSON on stdout only. cmd.sudo is ignored here: import already wraps the
# whole job, and writers like set-hostname.sh call as-root.sh themselves.
COMMANDS=$(node "$ROOT/../services/Settings.js" commands \
  --plan "$PLAN_PATH" \
  --snapshot "$SNAPSHOT_PATH" \
  --scripts-root "$ROOT") || exit 1
jq -e 'type == "array"' <<<"$COMMANDS" >/dev/null 2>&1 ||
  { echo "apply-settings.sh: command plan was not a JSON array" >&2; exit 1; }

cmd_count=$(jq -r 'length' <<<"$COMMANDS")
for ((i = 0; i < cmd_count; i++)); do
  key=$(jq -r --argjson i "$i" '.[$i].key' <<<"$COMMANDS")
  backup=$(jq -r --argjson i "$i" '.[$i].backup // empty' <<<"$COMMANDS")
  stdin_payload=$(jq -r --argjson i "$i" '.[$i].stdin // empty' <<<"$COMMANDS")
  argv=()
  if jq -e --argjson i "$i" '.[$i].argv | type == "array" and length > 0' <<<"$COMMANDS" >/dev/null; then
    mapfile -d '' -t argv < <(jq -j --argjson i "$i" '.[$i].argv[] | tostring + "\u0000"' <<<"$COMMANDS")
    if [[ ${#argv[@]} -gt 0 && -z ${argv[-1]} ]]; then
      unset 'argv[-1]'
    fi
  fi
  # Prefer stdin when non-empty so a long list does not hit the 128 KiB argv cap.
  if [[ -n $stdin_payload && ${#argv[@]} -gt 0 && ${argv[-1]} == "$stdin_payload" ]]; then
    unset 'argv[-1]'
  fi
  if [[ -n $stdin_payload ]]; then
    queue_stdin "$key" "$backup" "$stdin_payload" "${argv[@]}"
  else
    queue "$key" "$backup" "${argv[@]}"
  fi
done

# The undo plan is the same plan with from and value swapped, so reversing an
# import is the ordinary path rather than a special one.
EXPLICIT_BACKUP=0
if [[ -n $BACKUP_DIR ]]; then
  EXPLICIT_BACKUP=1
else
  BACKUP_DIR="$STATE_HOME/atmos/imports/$(date -u +%Y%m%dT%H%M%SZ)"
fi

# Undoing must not leave a way back of its own. If it did, the newest way
# back would be the one that reverses the undo, and pressing undo twice
# would put the import straight back rather than doing nothing.
if [[ $NO_BACKUP -eq 1 ]]; then
  EXPLICIT_BACKUP=0
fi

# A dry run writes the undo plan when you name a directory, so you can read
# the way back before you commit to the way forward. It never copies files,
# because nothing is about to change them.
if [[ $NO_BACKUP -eq 0 && ( $DRY -eq 0 || $EXPLICIT_BACKUP -eq 1 ) ]]; then
  mkdir -p -- "$BACKUP_DIR"
  jq '{schema: .schema, changes: [.changes[] | select(.from != null) | {key: .key, value: .from, from: .value}]}' \
    <<<"$PLAN" >"$BACKUP_DIR/undo.json"
fi

if [[ $DRY -eq 0 && $NO_BACKUP -eq 0 ]]; then
  for group in "${RUN_GROUPS[@]:-}"; do
    [[ -n $group ]] || continue
    file=$(backup_for "$group")
    [[ -n ${file:-} && -f $file ]] || continue
    cp -- "$file" "$BACKUP_DIR/$(basename -- "$file")"
  done
fi

status=0
results=()
for i in "${!RUN_KEYS[@]}"; do
  key=${RUN_KEYS[$i]}
  mapfile -d '' -t argv < <(base64 -d <<<"${RUN_CMDS[$i]}")
  stdin_payload=${RUN_STDIN[$i]:-}
  # Named before it runs, not after: a writer that takes seconds should not
  # leave the window looking stuck with nothing to read.
  if [[ $PROGRESS -eq 1 && $DRY -eq 0 ]]; then
    printf 'progress\t%s\t%s\t%s\n' "$((i + 1))" "${#RUN_KEYS[@]}" "$key"
  fi
  if [[ $DRY -eq 1 ]]; then
    # A dry run is for reading, so show what would be sent rather than only
    # how big it is. Long payloads are cut so one change stays one line.
    shown=""
    if [[ -n $stdin_payload ]]; then
      if (( ${#stdin_payload} > 400 )); then
        shown="  <stdin ${#stdin_payload} bytes> ${stdin_payload:0:400}…"
      else
        shown="  <stdin> $stdin_payload"
      fi
    fi
    printf '%s\t%s%s\n' "$key" "$(printf '%q ' "${argv[@]}")" "$shown"
    results+=("$(jq -nc --arg k "$key" '{key:$k, status:"dry-run"}')")
    continue
  fi
  # Combined, not stderr alone: omarchy reports its failures on stdout, so
  # capturing stderr by itself left every failure with an empty reason.
  if err=$(if [[ -n $stdin_payload ]]; then printf '%s' "$stdin_payload" | ATMOS_HYPR_JSON=$stdin_payload "${argv[@]}" 2>&1; else "${argv[@]}" 2>&1; fi); then
    results+=("$(jq -nc --arg k "$key" '{key:$k, status:"applied"}')")
  else
    status=1
    echo "apply-settings.sh: $key failed: $err" >&2
    results+=("$(jq -nc --arg k "$key" --arg e "$err" '{key:$k, status:"failed", error:$e}')")
  fi
done

if [[ $DRY -eq 0 ]]; then
  printf '%s\n' "${results[@]}" | jq -sc \
    --arg dir "$([[ $NO_BACKUP -eq 1 ]] && echo "" || echo "$BACKUP_DIR")" \
    '{backup:$dir, results:.}'
fi

exit $status
