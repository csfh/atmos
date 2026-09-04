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

for tool in jq python3; do
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

plan_value() { jq -r --arg k "$1" '.changes[] | select(.key == $k) | .value' <<<"$PLAN"; }
# A list value has to stay JSON on the way to its writer.
plan_list() { jq -c --arg k "$1" '[.changes[] | select(.key == $k) | .value] | .[0] // []' <<<"$PLAN"; }
plan_has() { jq -e --arg k "$1" 'any(.changes[]; .key == $k)' <<<"$PLAN" >/dev/null 2>&1; }
snap_value() { jq -r --arg p "$1" 'getpath($p | split(".")) // empty' <<<"$SNAPSHOT"; }

# The value the plan sets, or the machine's current value when the plan does
# not mention it. A paired writer takes every value at once, so the halves the
# plan left alone have to be read back rather than dropped.
plan_or_snapshot() {
  if plan_has "$1"; then plan_value "$1"; else snap_value "$1"; fi
}

# hypr-sentinel replaces the managed block wholesale, so a grouped writer has
# to be handed the current object with the changes merged in. Sending only the
# changed fields would drop every setting the plan did not mention.
merged_group() {
  local group=$1
  jq -c --arg g "$group" '
    (.snapshot[$g] // {}) as $base
    | reduce (.plan.changes[] | select(.key | startswith($g + "."))) as $c
        ($base; .[$c.key | split(".")[1]] = $c.value)
  ' <<<"$(jq -n --argjson snapshot "$SNAPSHOT" --argjson plan "$PLAN" '{snapshot:$snapshot, plan:$plan}')"
}

# Files a writer may rewrite, so a copy exists before it does.
backup_for() {
  case $1 in
    hyprLook) printf '%s\n' "$HOME/.config/hypr/looknfeel.lua" ;;
    hyprInput) printf '%s\n' "$HOME/.config/hypr/input.lua" ;;
    bindings) printf '%s\n' "$HOME/.config/hypr/bindings.lua" ;;
    autostart) printf '%s\n' "$HOME/.config/hypr/autostart.lua" ;;
    windowRules) printf '%s\n' "$HOME/.config/hypr/atmos.lua" ;;
    nightlightSchedule) printf '%s\n' "$HOME/.config/hypr/hyprsunset.conf" ;;
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

bool_arg() { [[ $1 == true ]] && printf 'true\n' || printf 'false\n'; }

# A toggle has no "set" verb, so it only runs when the plan actually flips it.
toggle_needed() {
  local key=$1
  local want current
  want=$(plan_value "$key")
  current=$(snap_value "$key")
  [[ $want != "$current" ]]
}

seen_group() {
  local g=$1 i
  for i in "${RUN_GROUPS[@]:-}"; do
    [[ $i == "$g" ]] && return 0
  done
  return 1
}

while IFS= read -r key; do
  value=$(plan_value "$key")
  case $key in
    theme) queue "$key" "" omarchy theme set "$value" ;;
    background) queue "$key" "" omarchy theme bg set "$value" ;;
    font) queue "$key" "" omarchy font set "$value" ;;
    textSize) queue "$key" "" omarchy display text size "$value" ;;

    hyprLook.*)
      seen_group hyprLook || queue_stdin "$key" hyprLook "$(merged_group hyprLook)" bash "$ROOT/set-hypr-look.sh"
      ;;
    hyprInput.*)
      seen_group hyprInput || queue_stdin "$key" hyprInput "$(merged_group hyprInput)" bash "$ROOT/set-hypr-input.sh"
      ;;
    hyprNoGaps) queue "$key" "" omarchy hyprland toggle window-no-gaps ;;
    hyprSquareAspect) queue "$key" "" omarchy hyprland toggle single-window-aspect-ratio ;;

    barPosition) queue "$key" "" omarchy bar position "$value" ;;
    barTransparent) queue "$key" "" omarchy bar transparent "$(bool_arg "$value")" ;;
    barVisible)
      toggle_needed "$key" && queue "$key" "" omarchy toggle bar "$([[ $value == true ]] && echo on || echo off)"
      ;;
    clockFormat) queue "$key" clock omarchy bar set omarchy.clock format "$value" ;;
    clockFormatAlt) queue "$key" clock omarchy bar set omarchy.clock formatAlt "$value" ;;
    clockWeekStart) queue "$key" clock omarchy bar set omarchy.clock weekStartDay "$value" ;;

    browser | terminal | editor | agent) queue "$key" "" omarchy default "$key" "$value" ;;
    mimePdf) queue "$key" "" bash "$ROOT/set-mime-default.sh" pdf "$value" ;;
    mimeImage) queue "$key" "" bash "$ROOT/set-mime-default.sh" image "$value" ;;
    mimeVideo) queue "$key" "" bash "$ROOT/set-mime-default.sh" video "$value" ;;

    idleScreensaver | idleLock)
      if ! seen_group idle; then
        screensaver=$(plan_or_snapshot idleScreensaver)
        lock=$(plan_or_snapshot idleLock)
        queue "$key" idle bash "$ROOT/set-idle.sh" "$screensaver" "$lock"
      fi
      ;;
    stayAwake)
      toggle_needed "$key" && queue "$key" "" omarchy toggle idle "$([[ $value == true ]] && echo stay-awake || echo allow-idle)"
      ;;
    screensaverEnabled)
      toggle_needed "$key" && queue "$key" "" omarchy toggle screensaver-off "$([[ $value == true ]] && echo off || echo on)"
      ;;
    doNotDisturb)
      toggle_needed "$key" && queue "$key" "" omarchy toggle notification silencing
      ;;
    nightlight)
      toggle_needed "$key" && queue "$key" "" omarchy toggle nightlight
      ;;
    nightlightTemperature) queue "$key" "" bash "$ROOT/set-nightlight-temp.sh" "$value" ;;
    nightlightDay | nightlightNight | nightlightNightOn)
      if ! seen_group nightlightSchedule; then
        day=$(plan_or_snapshot nightlightDay)
        night=$(plan_or_snapshot nightlightNight)
        on=$(plan_or_snapshot nightlightNightOn)
        temp=$(snap_value nightlightTemperature)
        queue "$key" nightlightSchedule bash "$ROOT/set-hyprsunset.sh" \
          "$(jq -nc --arg d "$day" --arg n "$night" --argjson o "${on:-false}" --argjson t "${temp:-0}" \
            '{day:$d, night:$n, nightOn:$o, temperature:$t}')"
      fi
      ;;

    # The sentinel writers take the whole list, so a list setting is one call.
    # `managed` is a fact about where a row currently lives, not part of the
    # row, so it never travels to the writer.
    bindings)
      queue_stdin "$key" bindings \
        "$(jq -c '{items: [.[] | {keys, label, command, unbind}]}' <<<"$(plan_list "$key")")" \
        bash "$ROOT/set-hypr-bindings.sh"
      ;;
    windowRules)
      queue_stdin "$key" windowRules \
        "$(jq -c '{items: [.[] | del(.managed)]}' <<<"$(plan_list "$key")")" \
        bash "$ROOT/set-hypr-windows.sh"
      ;;
    autostart)
      queue_stdin "$key" autostart \
        "$(jq -c '{commands: [.[] | .command]}' <<<"$(plan_list "$key")")" \
        bash "$ROOT/set-hypr-autostart.sh"
      ;;

    audioOutputVolume) queue "$key" "" bash "$ROOT/set-audio.sh" output-volume "$value" ;;
    audioInputVolume) queue "$key" "" bash "$ROOT/set-audio.sh" input-volume "$value" ;;
    audioOutputMuted)
      toggle_needed "$key" && queue "$key" "" omarchy audio output volume mute-toggle
      ;;
    audioInputMuted)
      toggle_needed "$key" && queue "$key" "" omarchy audio input mute
      ;;
    audioTuningOn) queue "$key" "" omarchy audio tuning "$([[ $value == true ]] && echo on || echo off)" ;;

    powerProfileAc) queue "$key" "" omarchy powerprofiles set ac "$value" ;;
    powerProfileBattery) queue "$key" "" omarchy powerprofiles set battery "$value" ;;
    # The verb names the state being turned off, so the argument is inverted.
    suspendEnabled)
      toggle_needed "$key" && queue "$key" "" omarchy toggle suspend-off "$([[ $value == true ]] && echo off || echo on)"
      ;;
    crashCapture)
      toggle_needed "$key" && queue "$key" "" omarchy toggle crash capture
      ;;

    clockBirthYear) queue "$key" clock omarchy bar set omarchy.clock birthYear "$value" --json ;;
    clockLifeExpectancy) queue "$key" clock omarchy bar set omarchy.clock lifeExpectancy "$value" --json ;;
    indicatorsAlwaysShow) queue "$key" clock omarchy bar set omarchy.indicators alwaysShow "$(bool_arg "$value")" --json ;;
    powerShowPercentage) queue "$key" clock omarchy bar set omarchy.power showPercentage "$(bool_arg "$value")" --json ;;
    spacerSize) queue "$key" clock omarchy bar set omarchy.spacer size "$value" --json ;;
    weatherUnit) queue "$key" clock omarchy bar set omarchy.weather unit "$value" ;;
    weatherRefreshMinutes) queue "$key" clock omarchy bar set omarchy.weather refreshMinutes "$value" --json ;;
    weatherLocation)
      if [[ -z $value ]]; then
        queue "$key" "" omarchy weather location --clear
      else
        queue "$key" "" omarchy weather location --set "$value"
      fi
      ;;
    agentsRefreshIntervalSec) queue "$key" clock omarchy bar set omarchy.agents refreshIntervalSec "$value" --json ;;
    agentsSync) queue "$key" clock omarchy bar set omarchy.agents syncMode "$([[ $value == true ]] && echo On || echo Off)" ;;
    agentsSyncDir) queue "$key" clock omarchy bar set omarchy.agents syncDir "$value" ;;
    agentsSyncFileName) queue "$key" clock omarchy bar set omarchy.agents syncFileName "$value" ;;

    indicatorsItems)
      queue "$key" clock bash "$ROOT/set-bar-widget.sh" omarchy.indicators items "$(plan_list "$key")"
      ;;
    trayHidden)
      queue "$key" clock bash "$ROOT/set-bar-widget.sh" omarchy.tray hidden "$(plan_list "$key")"
      ;;
    trayPinned)
      queue "$key" clock bash "$ROOT/set-bar-widget.sh" omarchy.tray pinned "$(plan_list "$key")"
      ;;

    plymouth) queue "$key" "" omarchy plymouth set by theme "$value" ;;

    touchpadEnabled)
      toggle_needed "$key" && queue "$key" "" omarchy toggle touchpad "$([[ $value == true ]] && echo on || echo off)"
      ;;
    touchscreenEnabled)
      toggle_needed "$key" && queue "$key" "" omarchy toggle touchscreen "$([[ $value == true ]] && echo on || echo off)"
      ;;
    bluetooth)
      toggle_needed "$key" && queue "$key" "" omarchy bluetooth power "$([[ $value == true ]] && echo on || echo off)"
      ;;

    hostname) queue "$key" "" bash "$ROOT/set-hostname.sh" "$value" ;;
    timezone) queue "$key" "" bash "$ROOT/set-timezone.sh" "$value" ;;
    locale) queue "$key" "" bash "$ROOT/set-locale.sh" "$value" ;;
    keyboardLayout) queue "$key" "" bash "$ROOT/set-keyboard-layout.sh" "$value" ;;
    ntp) queue "$key" "" bash "$ROOT/set-ntp.sh" "$(bool_arg "$value")" ;;
    fullName) queue "$key" "" bash "$ROOT/set-full-name.sh" "$value" ;;
    parallelDownloads) queue "$key" "" bash "$ROOT/set-parallel-downloads.sh" "$value" ;;
    dns) queue "$key" "" omarchy dns "$value" ;;

    *)
      echo "apply-settings.sh: no writer for $key" >&2
      exit 1
      ;;
  esac
done < <(jq -r '.changes[].key' <<<"$PLAN")

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
  if err=$(if [[ -n $stdin_payload ]]; then printf '%s' "$stdin_payload" | "${argv[@]}" 2>&1; else "${argv[@]}" 2>&1; fi); then
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
