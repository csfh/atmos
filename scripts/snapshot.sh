#!/bin/bash
# Dump the current Omarchy preference surface as JSON for the Quickshell UI.

set -euo pipefail

: "${OMARCHY_PATH:=/usr/share/omarchy}"
USER_SHELL_JSON="$HOME/.config/omarchy/shell.json"
DEFAULT_SHELL_JSON="$OMARCHY_PATH/config/omarchy/shell.json"
SNAP_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

present() {
  command -v "$1" >/dev/null 2>&1
}

lines_json() {
  jq -R -s 'split("\n") | map(select(length > 0))'
}

omarchy_out() {
  omarchy "$@" 2>/dev/null || true
}

GROUP=${1:-all}
case $GROUP in
  look | rest | all) ;;
  *)
    echo "snapshot.sh: group must be look, rest, or all" >&2
    exit 2
    ;;
esac

emit_look_snapshot() {
  local theme background font text_size
  local stay_awake nightlight_json nightlight nightlight_temp
  local screensaver_branded about_branded
  local themes_json extra_themes_json fonts_json
  local plymouth plymouth_themes_json
  local nightlight_day nightlight_night nightlight_night_on
  local hyprsunset_file hyprsunset_parsed parsed_day parsed_night parsed_night_on parsed_temp
  local brand_file default_brand about_file default_about
  theme=$(omarchy_out theme current)
  theme=${theme%$'\n'}
  background=$(omarchy_out theme bg current)
  background=${background%$'\n'}
  font=$(omarchy_out font current)
  font=${font%$'\n'}
  text_size=$(omarchy display text size 2>/dev/null | awk '/text size:/{print $3; exit}' || true)
  [[ $text_size =~ ^[0-9]+$ ]] || text_size=12
  stay_awake=$(omarchy_out toggle idle status | jq -r '.enabled // false' 2>/dev/null || true)
  [[ $stay_awake == true ]] || stay_awake=false
  nightlight_json=$(omarchy_out toggle nightlight --status)
  nightlight=$(jq -r '.enabled // false' <<<"$nightlight_json" 2>/dev/null || true)
  [[ $nightlight == true ]] || nightlight=false
  nightlight_temp=$(jq -r '.temperature // empty' <<<"$nightlight_json" 2>/dev/null || true)
  [[ $nightlight_temp =~ ^[0-9]+$ ]] || nightlight_temp=0
  screensaver_branded=false
  brand_file="$HOME/.config/omarchy/branding/screensaver.txt"
  default_brand="$OMARCHY_PATH/logo.txt"
  if [[ -f $brand_file ]]; then
    if [[ -f $default_brand ]] && cmp -s "$brand_file" "$default_brand"; then
      screensaver_branded=false
    else
      screensaver_branded=true
    fi
  fi
  about_branded=false
  about_file="$HOME/.config/omarchy/branding/about.txt"
  default_about="$OMARCHY_PATH/icon.txt"
  if [[ -f $about_file ]]; then
    if [[ -f $default_about ]] && cmp -s "$about_file" "$default_about"; then
      about_branded=false
    else
      about_branded=true
    fi
  fi
  themes_json=$(omarchy_out theme list | lines_json)
  extra_themes_json=$(omarchy_out theme extras | awk -F/ 'NF { print $NF }' | lines_json)
  fonts_json=$(omarchy_out font list | lines_json)
  plymouth=$(omarchy_out plymouth current)
  plymouth=${plymouth%$'\n'}
  plymouth_themes_json=$(omarchy_out plymouth list | lines_json)
  hyprsunset_file=${ATMOS_HYPRSUNSET_FILE:-"$HOME/.config/hypr/hyprsunset.conf"}
  nightlight_day=07:00
  nightlight_night=20:00
  nightlight_night_on=false
  if [[ -f $hyprsunset_file ]]; then
    hyprsunset_parsed=$(python3 - "$hyprsunset_file" <<'PY' 2>/dev/null || true
import re, sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
day = ""
night = ""
night_on = False
temp = 0
blocks = re.split(r"profile\s*\{", text)
for block in blocks[1:]:
    body = block.split("}", 1)[0]
    time_m = re.search(r'time\s*=\s*"?([0-2]?\d:[0-5]\d)"?', body)
    if not time_m:
        continue
    time = time_m.group(1)
    if re.search(r"\bidentity\s*=\s*true\b", body):
        day = time
        continue
    temp_m = re.search(r"temperature\s*=\s*([0-9]+)", body)
    if temp_m:
        night = time
        night_on = True
        temp = int(temp_m.group(1))
print(f"{day}\t{night}\t{str(night_on).lower()}\t{temp}")
PY
    )
    if [[ -n ${hyprsunset_parsed:-} ]]; then
      IFS=$'\t' read -r parsed_day parsed_night parsed_night_on parsed_temp <<<"$hyprsunset_parsed"
      [[ $parsed_day =~ ^[0-2]?\d:[0-5]\d$ ]] && nightlight_day=$parsed_day
      [[ $parsed_night =~ ^[0-2]?\d:[0-5]\d$ ]] && nightlight_night=$parsed_night
      [[ $parsed_night_on == true ]] && nightlight_night_on=true
    fi
  fi
  jq -n \
    --arg theme "$theme" \
    --arg background "$background" \
    --arg font "$font" \
    --argjson textSize "$text_size" \
    --argjson themes "$themes_json" \
    --argjson extraThemes "$extra_themes_json" \
    --argjson fonts "$fonts_json" \
    --argjson stayAwake "$stay_awake" \
    --argjson nightlight "$nightlight" \
    --argjson nightlightTemperature "$nightlight_temp" \
    --argjson screensaverBranded "$screensaver_branded" \
    --argjson aboutBranded "$about_branded" \
    --arg plymouth "$plymouth" \
    --argjson plymouthThemes "$plymouth_themes_json" \
    --arg nightlightDay "$nightlight_day" \
    --arg nightlightNight "$nightlight_night" \
    --argjson nightlightNightOn "$nightlight_night_on" \
    '{
      theme: $theme,
      background: $background,
      font: $font,
      textSize: $textSize,
      themes: $themes,
      extraThemes: $extraThemes,
      fonts: $fonts,
      stayAwake: $stayAwake,
      nightlight: $nightlight,
      nightlightTemperature: $nightlightTemperature,
      screensaverBranded: $screensaverBranded,
      aboutBranded: $aboutBranded,
      plymouth: $plymouth,
      plymouthThemes: $plymouthThemes,
      nightlightDay: $nightlightDay,
      nightlightNight: $nightlightNight,
      nightlightNightOn: $nightlightNightOn
    }'
}

if [[ $GROUP == look ]]; then
  emit_look_snapshot
  exit 0
fi

theme=$(omarchy_out theme current)
theme=${theme%$'\n'}
background=$(omarchy_out theme bg current)
background=${background%$'\n'}
font=$(omarchy_out font current)
font=${font%$'\n'}
browser=$(omarchy_out default browser)
browser=${browser%$'\n'}
terminal=$(omarchy_out default terminal)
terminal=${terminal%$'\n'}
editor=$(omarchy_out default editor)
editor=${editor%$'\n'}
agent=$(omarchy_out default agent)
agent=${agent%$'\n'}
dns=$(omarchy_out dns)
dns=${dns%$'\n'}
case $dns in
  Cloudflare | Google | DHCP | Custom) ;;
  *) dns="" ;;
esac

text_size=$(omarchy display text size 2>/dev/null | awk '/text size:/{print $3; exit}' || true)
[[ $text_size =~ ^[0-9]+$ ]] || text_size=12

stay_awake=$(omarchy_out toggle idle status | jq -r '.enabled // false' 2>/dev/null || true)
[[ $stay_awake == true ]] || stay_awake=false

nightlight_json=$(omarchy_out toggle nightlight --status)
nightlight=$(jq -r '.enabled // false' <<<"$nightlight_json" 2>/dev/null || true)
[[ $nightlight == true ]] || nightlight=false
nightlight_temp=$(jq -r '.temperature // empty' <<<"$nightlight_json" 2>/dev/null || true)
[[ $nightlight_temp =~ ^[0-9]+$ ]] || nightlight_temp=0

screensaver_enabled=true
if omarchy toggle enabled screensaver-off >/dev/null 2>&1; then
  screensaver_enabled=false
fi

bar_visible=true
if omarchy toggle enabled bar-off >/dev/null 2>&1; then
  bar_visible=false
fi

bluetooth=false
if omarchy bluetooth power is-on >/dev/null 2>&1; then
  bluetooth=true
fi

suspend_enabled=true
if omarchy toggle enabled suspend-off >/dev/null 2>&1; then
  suspend_enabled=false
fi

crash_capture=true
if omarchy toggle enabled crash-capture-off >/dev/null 2>&1; then
  crash_capture=false
fi

dnd=false
dnd_state=$(omarchy-shell notifications isDnd 2>/dev/null || true)
dnd_state=${dnd_state%$'\n'}
if [[ $dnd_state == on ]]; then
  dnd=true
elif [[ -z $dnd_state && -f $HOME/.local/state/omarchy/notifications.json ]]; then
  dnd=$(jq -r '.dnd // false' "$HOME/.local/state/omarchy/notifications.json")
  [[ $dnd == true ]] || dnd=false
fi

wifi_connected=false
wifi_band=""
wifi_selected="auto"
wifi_bands_json='["auto"]'
band_out=$(omarchy_out network band)
if [[ -n $band_out ]]; then
  wifi_connected=true
  wifi_band=$(awk -F'\t' '$1=="band"{print $2; exit}' <<<"$band_out")
  wifi_selected=$(awk -F'\t' '$1=="selected"{print $2; exit}' <<<"$band_out")
  wifi_available=$(awk -F'\t' '$1=="available"{print $2; exit}' <<<"$band_out")
  case $wifi_selected in
    auto | 2.4 | 5 | 6) ;;
    *) wifi_selected=auto ;;
  esac
  wifi_bands_json=$(jq -n --arg avail "$wifi_available" --arg selected "$wifi_selected" '
    def ok: . == "2.4" or . == "5" or . == "6";
    ["auto"] + ($avail | split(" ") | map(select(ok)))
    | if ($selected != "auto") and ($selected | ok) and (index($selected) == null) then . + [$selected] else . end
  ')
fi

wifi_iface=""
if [[ $wifi_connected == true ]]; then
  wifi_iface=$(LC_ALL=C nmcli -e no -g DEVICE,TYPE,STATE device status 2>/dev/null | awk -F: '$2 == "wifi" && $3 == "connected" { print $1; exit }' || true)
  [[ $wifi_iface =~ ^[a-zA-Z0-9._-]+$ ]] || wifi_iface=""
fi

net_kind=disconnected
net_iface=""
net_ssid=""
net_signal=""
net_ip=""
net_speed=""
status_line=$(omarchy_out network status)
net_kind=$(awk -F'\t' 'NR==1 { print $1; exit }' <<<"$status_line")
case $net_kind in
  ethernet)
    net_iface=$(awk -F'\t' 'NR==1 { print $2; exit }' <<<"$status_line")
    ;;
  wifi)
    net_ssid=$(awk -F'\t' 'NR==1 { print $2; exit }' <<<"$status_line")
    net_signal=$(awk -F'\t' 'NR==1 { print $3; exit }' <<<"$status_line")
    ;;
  *)
    net_kind=disconnected
    ;;
esac
[[ $net_iface =~ ^[a-zA-Z0-9._-]+$ ]] || net_iface=""
[[ $net_signal =~ ^[0-9]+$ ]] || net_signal=""
verbose_status=$(omarchy network status --verbose 2>/dev/null || true)
if [[ -n $verbose_status ]]; then
  [[ -n $net_iface ]] || net_iface=$(awk -F'\t' '$1=="iface"{print $2; exit}' <<<"$verbose_status")
  net_ip=$(awk -F'\t' '$1=="ip"{print $2; exit}' <<<"$verbose_status")
  net_speed=$(awk -F'\t' '$1=="speed"{print $2; exit}' <<<"$verbose_status")
fi
[[ $net_iface =~ ^[a-zA-Z0-9._-]+$ ]] || net_iface=""
[[ $net_ip =~ ^[0-9a-fA-F:.]+$ ]] || net_ip=""
[[ $net_speed =~ ^[0-9]+$ ]] || net_speed=""

wifi_hw=false
wifi_radio=false
wifi_radio_out=$(nmcli radio wifi 2>/dev/null || true)
wifi_radio_out=${wifi_radio_out%$'\n'}
if [[ $wifi_radio_out == enabled ]]; then
  wifi_radio=true
fi
wifi_hw_out=$(nmcli -t -f WIFI-HW radio 2>/dev/null || true)
wifi_hw_out=${wifi_hw_out%$'\n'}
if [[ $wifi_hw_out == enabled ]]; then
  wifi_hw=true
fi

wifi_connections_json='[]'
if present nmcli && present jq; then
  wifi_connections_json=$(
    LC_ALL=C nmcli -t -f UUID,TYPE,STATE,NAME connection show 2>/dev/null \
      | awk -F: '$2 == "802-11-wireless" {
          uuid=$1
          state=$3
          name=$4
          for (i = 5; i <= NF; i++) name = name ":" $i
          gsub(/\\:/, ":", name)
          printf "%s\t%s\t%s\n", uuid, state, name
        }' \
      | jq -R -s -c '
          split("\n")
          | map(select(length > 0) | split("\t"))
          | map(select(length >= 3 and (.[0] | test("^[0-9a-fA-F-]{36}$")))
            | {uuid: .[0], active: (.[1] == "activated"), name: .[2]})
        ' || true
  )
  [[ -n $wifi_connections_json ]] || wifi_connections_json='[]'
fi

bluetooth_devices_json='[]'
if present bluetoothctl && present jq; then
  paired_list=$(timeout 2s bluetoothctl devices Paired 2>/dev/null || true)
  connected_list=$(timeout 2s bluetoothctl devices Connected 2>/dev/null || true)
  bluetooth_devices_json=$(
    jq -n -c --arg paired "$paired_list" --arg connected "$connected_list" '
      def parse:
        split("\n")
        | map(select(test("^Device ")))
        | map(capture("^Device (?<address>[0-9A-Fa-f:]{17})(?: (?<name>.*))?$")
          | {address: .address, name: ((.name // "") | if . == "" then .address else . end)});
      ([$connected | parse[] | .address] | unique) as $on
      | [$paired | parse[] as $d | $d + {connected: ($on | index($d.address) != null)}]
    ' || true
  )
  [[ -n $bluetooth_devices_json ]] || bluetooth_devices_json='[]'
fi

audio_sinks_json='[]'
audio_sources_json='[]'
audio_output_volume=0
audio_output_muted=false
audio_input_volume=0
audio_input_muted=false
if present pactl && present jq; then
  default_sink=$(pactl get-default-sink 2>/dev/null || true)
  default_sink=${default_sink%$'\n'}
  default_source=$(pactl get-default-source 2>/dev/null || true)
  default_source=${default_source%$'\n'}
  sinks_raw=$(pactl -f json list sinks 2>/dev/null || true)
  sources_raw=$(pactl -f json list sources 2>/dev/null || true)
  if [[ -n $sinks_raw ]]; then
    audio_sinks_json=$(jq -c --arg def "$default_sink" '
      def pct:
        ((.volume // {}) | to_entries | map((.value.value_percent // "0%") | rtrimstr("%") | tonumber) ) as $p
        | if ($p | length) == 0 then 0 else (($p | add / ($p | length)) | floor) end;
      map({
        id: ((.properties["object.id"] // "0") | tonumber),
        name: .name,
        description: (.description // .name),
        default: (.name == $def),
        volume: pct,
        muted: (.mute == true)
      })
    ' <<<"$sinks_raw" || true)
  fi
  [[ -n $audio_sinks_json ]] || audio_sinks_json='[]'
  if [[ -n $sources_raw ]]; then
    audio_sources_json=$(jq -c --arg def "$default_source" '
      def pct:
        ((.volume // {}) | to_entries | map((.value.value_percent // "0%") | rtrimstr("%") | tonumber) ) as $p
        | if ($p | length) == 0 then 0 else (($p | add / ($p | length)) | floor) end;
      map(select((.name // "") | endswith(".monitor") | not) | {
        id: ((.properties["object.id"] // "0") | tonumber),
        name: .name,
        description: (.description // .name),
        default: (.name == $def),
        volume: pct,
        muted: (.mute == true)
      })
    ' <<<"$sources_raw" || true)
  fi
  [[ -n $audio_sources_json ]] || audio_sources_json='[]'
  audio_output_volume=$(jq -r '[.[] | select(.default == true)][0].volume // .[0].volume // 0' <<<"$audio_sinks_json")
  audio_output_muted=$(jq -r '[.[] | select(.default == true)][0].muted // false' <<<"$audio_sinks_json")
  audio_input_volume=$(jq -r '[.[] | select(.default == true)][0].volume // .[0].volume // 0' <<<"$audio_sources_json")
  audio_input_muted=$(jq -r '[.[] | select(.default == true)][0].muted // false' <<<"$audio_sources_json")
fi
[[ $audio_output_volume =~ ^[0-9]+$ ]] || audio_output_volume=0
[[ $audio_input_volume =~ ^[0-9]+$ ]] || audio_input_volume=0
[[ $audio_output_muted == true ]] || audio_output_muted=false
[[ $audio_input_muted == true ]] || audio_input_muted=false

audio_tuning_match=false
if omarchy audio tuning match >/dev/null 2>&1; then
  audio_tuning_match=true
fi
audio_tuning_on=false
tuning_status=$(omarchy audio tuning status 2>/dev/null || true)
if [[ $tuning_status == *"Tuning sink:"*"present"* || $tuning_status == *"Installed:"*"yes"* ]]; then
  audio_tuning_on=true
fi

disks_json='[]'
luks_devices_json='[]'
swap_devices_json='[]'
snapshots_json='[]'
if present python3 && [[ -x $SNAP_DIR/disk-inventory.py ]]; then
  disk_inv=$(python3 "$SNAP_DIR/disk-inventory.py" 2>/dev/null || true)
  if [[ -n $disk_inv ]]; then
    disks_json=$(jq -c '.disks // []' <<<"$disk_inv" 2>/dev/null || echo '[]')
    luks_devices_json=$(jq -c '.luksDevices // []' <<<"$disk_inv" 2>/dev/null || echo '[]')
    swap_devices_json=$(jq -c '.swapDevices // []' <<<"$disk_inv" 2>/dev/null || echo '[]')
    snapshots_json=$(jq -c '.snapshots // []' <<<"$disk_inv" 2>/dev/null || echo '[]')
  fi
fi
[[ -n $disks_json ]] || disks_json='[]'
[[ -n $luks_devices_json ]] || luks_devices_json='[]'
[[ -n $swap_devices_json ]] || swap_devices_json='[]'
[[ -n $snapshots_json ]] || snapshots_json='[]'

hardware_json='{}'
if present python3 && [[ -x $SNAP_DIR/hw-inventory.py ]]; then
  hw_inv=$(python3 "$SNAP_DIR/hw-inventory.py" 2>/dev/null || true)
  if [[ -n $hw_inv ]]; then
    hardware_json=$(jq -c '.' <<<"$hw_inv" 2>/dev/null || echo '{}')
  fi
fi
[[ -n $hardware_json ]] || hardware_json='{}'

desktop_apps_json='[]'
tui_apps_json='[]'
web_apps_json='[]'
if present python3 && [[ -x $SNAP_DIR/list-apps.py ]]; then
  apps_inv=$(python3 "$SNAP_DIR/list-apps.py" 2>/dev/null || true)
  if [[ -n $apps_inv ]]; then
    desktop_apps_json=$(jq -c '.desktop // []' <<<"$apps_inv" 2>/dev/null || echo '[]')
    tui_apps_json=$(jq -c '.tui // []' <<<"$apps_inv" 2>/dev/null || echo '[]')
    web_apps_json=$(jq -c '.web // []' <<<"$apps_inv" 2>/dev/null || echo '[]')
  fi
fi
[[ -n $desktop_apps_json ]] || desktop_apps_json='[]'
[[ -n $tui_apps_json ]] || tui_apps_json='[]'
[[ -n $web_apps_json ]] || web_apps_json='[]'

snapper_present=false
snapper_configs_json='[]'
if present snapper; then
  snapper_present=true
  snapper_configs_json=$(
    snapper --csvout list-configs 2>/dev/null \
      | awk -F, 'NR > 1 && $1 != "" { printf "%s\t%s\n", $1, $2 }' \
      | jq -R -s -c '
          split("\n")
          | map(select(length > 0) | split("\t") | {name: .[0], subvolume: (.[1] // "")})
        ' || true
  )
fi
[[ -n $snapper_configs_json ]] || snapper_configs_json='[]'

hibernation_available=false
if omarchy hibernation available >/dev/null 2>&1; then
  hibernation_available=true
fi

hibernation_supported=false
if [[ -f /sys/power/image_size ]] && command -v limine-mkinitcpio >/dev/null 2>&1; then
  hibernation_supported=true
fi

hibernation_configured=false
resume_conf=/etc/mkinitcpio.conf.d/omarchy_resume.conf
if [[ -f $resume_conf ]] && grep -q '^HOOKS+=(resume)$' "$resume_conf"; then
  hibernation_configured=true
fi

screensaver_branded=false
brand_file="$HOME/.config/omarchy/branding/screensaver.txt"
default_brand="$OMARCHY_PATH/logo.txt"
if [[ -f $brand_file ]]; then
  if [[ -f $default_brand ]] && cmp -s "$brand_file" "$default_brand"; then
    screensaver_branded=false
  else
    screensaver_branded=true
  fi
fi

about_branded=false
about_file="$HOME/.config/omarchy/branding/about.txt"
default_about="$OMARCHY_PATH/icon.txt"
if [[ -f $about_file ]]; then
  if [[ -f $default_about ]] && cmp -s "$about_file" "$default_about"; then
    about_branded=false
  else
    about_branded=true
  fi
fi

# `omarchy weather location` curls wttr.in when no location is stored.
weather=""
weather_auto=true
weather_coords=""
weather_file="$HOME/.local/state/omarchy/settings/weather.json"
if [[ -f $weather_file ]]; then
  weather=$(jq -r '.name // empty' "$weather_file" 2>/dev/null || true)
  weather=${weather%$'\n'}
  if [[ -n $weather ]]; then
    weather_auto=false
  fi
  weather_lat=$(jq -r '.latitude // empty' "$weather_file" 2>/dev/null || true)
  weather_lon=$(jq -r '.longitude // empty' "$weather_file" 2>/dev/null || true)
  weather_lat=${weather_lat%$'\n'}
  weather_lon=${weather_lon%$'\n'}
  if [[ $weather_lat =~ ^-?[0-9]+(\.[0-9]+)?$ && $weather_lon =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
    weather_coords="$weather_lat,$weather_lon"
  fi
fi

themes_json=$(omarchy_out theme list | lines_json)
extra_themes_json=$(omarchy_out theme extras | awk -F/ 'NF { print $NF }' | lines_json)
fonts_json=$(omarchy_out font list | lines_json)
power_profiles_json=$(omarchy_out powerprofiles list | lines_json)
power_profile=$(omarchy_out powerprofiles list --active-state | awk -F'\t' '$2 == 1 { print $1; exit }')
power_profile=${power_profile%$'\n'}
power_profile_ac=""
powerprofiles_state="${OMARCHY_POWERPROFILES_STATE_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/omarchy/powerprofiles}"
if [[ -r $powerprofiles_state/ac ]]; then
  power_profile_ac=$(<"$powerprofiles_state/ac")
  power_profile_ac=${power_profile_ac%$'\n'}
fi
if [[ -z $power_profile_ac ]]; then
  if jq -e 'index("performance") != null' <<<"$power_profiles_json" >/dev/null 2>&1; then
    power_profile_ac=performance
  else
    power_profile_ac=balanced
  fi
fi
power_profile_battery=""
if [[ -r $powerprofiles_state/battery ]]; then
  power_profile_battery=$(<"$powerprofiles_state/battery")
  power_profile_battery=${power_profile_battery%$'\n'}
fi
[[ -n $power_profile_battery ]] || power_profile_battery=balanced
plymouth=$(omarchy_out plymouth current)
plymouth=${plymouth%$'\n'}
plymouth_themes_json=$(omarchy_out plymouth list | lines_json)

shell_file=$DEFAULT_SHELL_JSON
if [[ -s $USER_SHELL_JSON ]]; then
  shell_file=$USER_SHELL_JSON
fi

bar_position=top
bar_transparent=false
screensaver=150
lock=300
if [[ -f $shell_file ]]; then
  bar_position=$(jq -r '.bar.position // "top"' "$shell_file")
  bar_transparent=$(jq -r '.bar.transparent // false' "$shell_file")
  screensaver=$(jq -r '.idle.screensaver // 150' "$shell_file")
  lock=$(jq -r '.idle.lock // 300' "$shell_file")
fi

case $bar_position in
  top | bottom | left | right) ;;
  *) bar_position=top ;;
esac

clock_present=false
clock_format=""
clock_format_alt=""
clock_week_start=""
clock_birth_year=0
clock_life_expectancy=0
if [[ -f $shell_file ]]; then
  clock_json=$(jq -c '[.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []] | add | map(select(.id == "omarchy.clock")) | .[0] // empty' "$shell_file")
  if [[ -n $clock_json ]]; then
    clock_present=true
    if [[ $bar_position == left || $bar_position == right ]]; then
      clock_format=$(jq -r '.verticalFormat // "HH\n—\nmm"' <<<"$clock_json")
      clock_format_alt=$(jq -r --arg def $'dd\nMMM\n\'W\'ww\n\'\'yy' '.verticalFormatAlt // $def' <<<"$clock_json")
    else
      clock_format=$(jq -r '.format // "dddd HH:mm"' <<<"$clock_json")
      clock_format_alt=$(jq -r --arg def "d MMMM 'W'ww yyyy" '.formatAlt // $def' <<<"$clock_json")
    fi
    clock_week_start=$(jq -r '.weekStartDay // empty' <<<"$clock_json")
    clock_week_start=${clock_week_start%$'\n'}
    clock_week_start=${clock_week_start,,}
    case $clock_week_start in
      sunday | monday | tuesday | wednesday | thursday | friday | saturday) ;;
      *) clock_week_start="" ;;
    esac
    clock_birth_year=$(jq -r '.birthYear // 0' <<<"$clock_json")
    [[ $clock_birth_year =~ ^[0-9]+$ ]] || clock_birth_year=0
    clock_life_expectancy=$(jq -r '.lifeExpectancy // 0' <<<"$clock_json")
    [[ $clock_life_expectancy =~ ^[0-9]+$ ]] || clock_life_expectancy=0
    if (( clock_life_expectancy < 1 || clock_life_expectancy > 150 )); then
      clock_life_expectancy=0
    fi
  fi
fi

indicators_present=false
indicators_always_show=false
indicators_items_json='[]'
if [[ -f $shell_file ]]; then
  indicators_json=$(jq -c '[.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []] | add | map(select(.id == "omarchy.indicators")) | .[0] // empty' "$shell_file")
  if [[ -n $indicators_json ]]; then
    indicators_present=true
    indicators_always_show=$(jq -r '.alwaysShow // false' <<<"$indicators_json")
    [[ $indicators_always_show == true ]] || indicators_always_show=false
    indicators_items_json=$(jq -c '
      def known: ["Dictation","ScreenRecording","Reminder","NightLight","Dnd","StayAwake"];
      ((.items // .indicators // []) | if type == "array" then . else [] end)
      | map(
          if type == "string" then .
          elif type == "object" and (.id | type == "string") then .id
          else empty end
        )
      | map(select(. as $id | known | index($id) != null))
    ' <<<"$indicators_json")
    [[ -n $indicators_items_json ]] || indicators_items_json='[]'
  fi
fi

agents_present=false
agents_refresh=900
agents_sync=false
agents_sync_dir=""
agents_sync_file=""
agents_sync_device=""
if [[ -f $shell_file ]]; then
  agents_json=$(jq -c '[.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []] | add | map(select(.id == "omarchy.agents")) | .[0] // empty' "$shell_file")
  if [[ -n $agents_json ]]; then
    agents_present=true
    agents_refresh=$(jq -r '.refreshIntervalSec // 900' <<<"$agents_json")
    [[ $agents_refresh =~ ^[0-9]+$ ]] || agents_refresh=900
    if (( agents_refresh < 30 )); then
      agents_refresh=30
    fi
    agents_sync=$(jq -r '.syncMode // "Off"' <<<"$agents_json")
    [[ $agents_sync == On ]] && agents_sync=true || agents_sync=false
    agents_sync_dir=$(jq -r '.syncDir // empty' <<<"$agents_json")
    agents_sync_dir=${agents_sync_dir%$'\n'}
    agents_sync_file=$(jq -r '.syncFileName // empty' <<<"$agents_json")
    agents_sync_file=${agents_sync_file%$'\n'}
    agents_sync_device=$(jq -r '.syncDeviceId // empty' <<<"$agents_json")
    agents_sync_device=${agents_sync_device%$'\n'}
  fi
fi

spacer_present=false
spacer_size=12
if [[ -f $shell_file ]]; then
  spacer_json=$(jq -c '[.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []] | add | map(select(.id == "omarchy.spacer")) | .[0] // empty' "$shell_file")
  if [[ -n $spacer_json ]]; then
    spacer_present=true
    spacer_size=$(jq -r '.size // 12' <<<"$spacer_json")
    [[ $spacer_size =~ ^[0-9]+$ ]] || spacer_size=12
    if (( spacer_size > 64 )); then
      spacer_size=64
    fi
  fi
fi

tray_present=false
tray_hidden_json='[]'
tray_pinned_json='[]'
if [[ -f $shell_file ]]; then
  tray_json=$(jq -c '[.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []] | add | map(select(.id == "omarchy.tray")) | .[0] // empty' "$shell_file")
  if [[ -n $tray_json ]]; then
    tray_present=true
    tray_hidden_json=$(jq -c '
      ((.hidden // []) | if type == "array" then . else [] end)
      | map(select(type == "string" and length > 0))
    ' <<<"$tray_json")
    [[ -n $tray_hidden_json ]] || tray_hidden_json='[]'
    tray_pinned_json=$(jq -c '
      ((.pinned // []) | if type == "array" then . else [] end)
      | map(select(type == "string" and length > 0))
    ' <<<"$tray_json")
    [[ -n $tray_pinned_json ]] || tray_pinned_json='[]'
  fi
fi

power_present=false
power_show_percentage=false
if [[ -f $shell_file ]]; then
  power_json=$(jq -c '[.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []] | add | map(select(.id == "omarchy.power")) | .[0] // empty' "$shell_file")
  if [[ -n $power_json ]]; then
    power_present=true
    power_show_percentage=$(jq -r '.showPercentage // false' <<<"$power_json")
    [[ $power_show_percentage == true ]] || power_show_percentage=false
  fi
fi

is_laptop=false
if omarchy hw laptop >/dev/null 2>&1; then
  is_laptop=true
fi

external_present=false
if omarchy hw external monitors >/dev/null 2>&1; then
  external_present=true
fi

monitors_json='[]'
if present hyprctl && present jq; then
  monitors_json=$(hyprctl monitors all -j 2>/dev/null | jq -c '
    def internal: test("^(eDP|LVDS|DSI)-");
    [.[] | {
      name: (.name // ""),
      description: (.description // ""),
      focused: (.focused == true),
      enabled: (.disabled != true),
      internal: ((.name // "") | internal),
      width: ((.width // 0) | floor),
      height: ((.height // 0) | floor),
      refresh: ((.refreshRate // 0) | floor),
      scale: (.scale // 1),
      x: ((.x // 0) | floor),
      y: ((.y // 0) | floor),
      mirrorOf: (if (.mirrorOf // "none") == "none" then "" else (.mirrorOf | tostring) end),
      availableModes: (
        (.availableModes // [])
        | if type == "array" then .[:80] | map(tostring) else [] end
      ),
      brightness: 0,
      brightnessAvailable: false
    }]
  ' || true)
  [[ -n $monitors_json ]] || monitors_json='[]'
  while IFS= read -r name; do
    [[ $name =~ ^[A-Za-z0-9._-]+$ ]] || continue
    bright=$(omarchy brightness display --no-osd --monitor "$name" 2>/dev/null | awk 'NR==1 && $1 ~ /^[0-9]+$/ { print $1; exit }' || true)
    if [[ $bright =~ ^[0-9]+$ ]]; then
      (( bright > 100 )) && bright=100
      monitors_json=$(jq -c --arg n "$name" --argjson b "$bright" \
        'map(if .name == $n then . + {brightness:$b, brightnessAvailable:true} else . end)' <<<"$monitors_json")
    fi
  done < <(jq -r '.[].name' <<<"$monitors_json" 2>/dev/null || true)
fi

internal_present=false
internal_enabled=false
mirroring=false
if [[ $monitors_json != '[]' ]]; then
  if jq -e '[.[] | select(.internal == true)] | length > 0' <<<"$monitors_json" >/dev/null 2>&1; then
    internal_present=true
  fi
  if jq -e '[.[] | select(.internal == true and .enabled == true)] | length > 0' <<<"$monitors_json" >/dev/null 2>&1; then
    internal_enabled=true
  fi
  if jq -e '[.[] | select((.mirrorOf // "") != "")] | length > 0' <<<"$monitors_json" >/dev/null 2>&1; then
    mirroring=true
  fi
fi

touchpad_present=false
touchpad_enabled=true
touchpad_disabled_file="$HOME/.local/state/omarchy/toggles/hypr/touchpad-disabled-name"
if omarchy hw touchpad >/dev/null 2>&1; then
  touchpad_present=true
fi
if [[ -f $touchpad_disabled_file ]]; then
  touchpad_present=true
  touchpad_enabled=false
fi

touchscreen_present=false
touchscreen_enabled=true
touchscreen_disabled_file="$HOME/.local/state/omarchy/toggles/hypr/touchscreen-disabled-name"
if omarchy hw touchscreen >/dev/null 2>&1; then
  touchscreen_present=true
fi
if [[ -f $touchscreen_disabled_file ]]; then
  touchscreen_present=true
  touchscreen_enabled=false
fi

keyboard_backlight_present=false
keyboard_brightness=0
for candidate in /sys/class/leds/*kbd_backlight*; do
  [[ -e $candidate/brightness && -e $candidate/max_brightness ]] || continue
  keyboard_backlight_present=true
  kb_cur=$(<"$candidate/brightness")
  kb_max=$(<"$candidate/max_brightness")
  if [[ $kb_cur =~ ^[0-9]+$ && $kb_max =~ ^[1-9][0-9]*$ ]]; then
    keyboard_brightness=$(( kb_cur * 100 / kb_max ))
    (( keyboard_brightness > 100 )) && keyboard_brightness=100
  fi
  break
done

battery_present=false
if omarchy battery present >/dev/null 2>&1; then
  battery_present=true
fi

weather_present=false
weather_unit=auto
weather_refresh=15
reminder_count=0
reminder_active=false
reminders_json='[]'
reminder_json=$(omarchy_out reminder show --json)
if [[ -n $reminder_json ]]; then
  reminder_count=$(jq -r '.count // 0' <<<"$reminder_json" 2>/dev/null || true)
  reminder_active=$(jq -r '.active // false' <<<"$reminder_json" 2>/dev/null || true)
  reminders_json=$(jq -c '.reminders // []' <<<"$reminder_json" 2>/dev/null || echo '[]')
fi
[[ $reminder_count =~ ^[0-9]+$ ]] || reminder_count=0
[[ $reminder_active == true ]] || reminder_active=false
[[ -n $reminders_json ]] || reminders_json='[]'
if [[ -f $shell_file ]]; then
  weather_json=$(jq -c '[.bar.layout.left // [], .bar.layout.center // [], .bar.layout.right // []] | add | map(select(.id == "omarchy.weather")) | .[0] // empty' "$shell_file")
  if [[ -n $weather_json ]]; then
    weather_present=true
    weather_unit=$(jq -r '.unit // empty' <<<"$weather_json")
    case $weather_unit in
      metric | imperial | auto) ;;
      *) weather_unit=auto ;;
    esac
    weather_refresh=$(jq -r '.refreshMinutes // 15' <<<"$weather_json")
    [[ $weather_refresh =~ ^[0-9]+$ ]] || weather_refresh=15
    if (( weather_refresh < 1 )); then
      weather_refresh=1
    fi
  fi
fi

hostname=""
if [[ -r /etc/hostname ]]; then
  hostname=$(< /etc/hostname)
  hostname=${hostname%%$'\n'*}
  hostname=${hostname%% *}
fi
if [[ -z $hostname ]] && present hostnamectl; then
  hostname=$(hostnamectl --static 2>/dev/null || true)
  hostname=${hostname%$'\n'}
fi
if [[ $hostname == -* || ${#hostname} -gt 253 || ! $hostname =~ ^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)*$ ]]; then
  hostname=""
fi

full_name=""
user_name=$(id -un 2>/dev/null || true)
if [[ $user_name =~ ^[a-z_][a-z0-9_-]*$ ]]; then
  full_name=$(getent passwd "$user_name" 2>/dev/null | awk -F: '{print $5}' | awk -F, '{print $1}')
fi
full_name=${full_name%%$'\n'*}
if [[ $full_name == -* || ${#full_name} -gt 256 || $full_name == *:* ]]; then
  full_name=""
fi

timezone=""
timezones_json='[]'
if present timedatectl; then
  timezone=$(timedatectl show -p Timezone --value 2>/dev/null || true)
  timezone=${timezone%$'\n'}
  if [[ $timezone == *..* || ! $timezone =~ ^[A-Za-z0-9/_+-]+$ ]]; then
    timezone=""
  fi
  timezones_json=$(timedatectl list-timezones 2>/dev/null | awk '
    /^[A-Za-z0-9/_+-]+$/ && !/\.\./ { print }
  ' | lines_json || true)
fi
[[ -n $timezones_json ]] || timezones_json='[]'

ntp=false
ntp_available=false
ntp_synchronized=false
if present timedatectl; then
  ntp_can=$(timedatectl show -p CanNTP --value 2>/dev/null || true)
  ntp_can=${ntp_can%$'\n'}
  ntp_val=$(timedatectl show -p NTP --value 2>/dev/null || true)
  ntp_val=${ntp_val%$'\n'}
  ntp_sync=$(timedatectl show -p NTPSynchronized --value 2>/dev/null || true)
  ntp_sync=${ntp_sync%$'\n'}
  [[ $ntp_can == yes ]] && ntp_available=true
  [[ $ntp_val == yes ]] && ntp=true
  [[ $ntp_sync == yes ]] && ntp_synchronized=true
fi

locale=""
if [[ -r /etc/locale.conf ]]; then
  locale=$(unset LANG; . /etc/locale.conf; printf '%s' "${LANG:-}")
fi
[[ $locale == C.UTF-8 || $locale =~ ^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$ ]] || locale=""

locales_json='[]'
supported=/usr/share/i18n/SUPPORTED
if [[ -r $supported ]]; then
  locales_json=$(awk '
    $2 == "UTF-8" && ($1 == "C.UTF-8" || $1 ~ /^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/) { print $1 }
  ' "$supported" | lines_json)
fi
[[ -n $locales_json ]] || locales_json='[]'

parallel_downloads=5
if [[ -r /etc/pacman.conf ]]; then
  parallel_downloads=$(awk '
    /^[[:space:]]*ParallelDownloads[[:space:]]*=/ {
      if (match($0, /[0-9]+/)) { print substr($0, RSTART, RLENGTH); exit }
    }
  ' /etc/pacman.conf)
fi
[[ $parallel_downloads =~ ^[0-9]+$ ]] || parallel_downloads=5
if (( parallel_downloads < 1 )); then
  parallel_downloads=5
fi
if (( parallel_downloads > 20 )); then
  parallel_downloads=20
fi

keyboard_layout=""
if [[ -r /etc/vconsole.conf ]]; then
  keyboard_layout=$(awk -F= '
    $1 == "XKBLAYOUT" {
      gsub(/[[:space:]"'\'']/, "", $2)
      print $2
      exit
    }
  ' /etc/vconsole.conf)
fi
keyboard_layout=${keyboard_layout%%,*}
[[ $keyboard_layout =~ ^[a-z0-9]{1,8}$ ]] || keyboard_layout=""

keyboard_layouts_json='[]'
xkb_lst="/usr/share/X11/xkb/rules/evdev.lst"
if [[ -r $xkb_lst ]] && present jq; then
  keyboard_layouts_json=$(
    awk '
      $0 ~ /^! layout[[:space:]]*$/ { p=1; next }
      p && /^! / { exit }
      p && $1 ~ /^[a-z0-9]{1,8}$/ && NF >= 2 {
        id=$1
        $1=""
        sub(/^[[:space:]]+/, "")
        print id "\t" $0
      }
    ' "$xkb_lst" \
      | jq -R -s -c '
          split("\n")
          | map(select(length > 0) | split("\t"))
          | map(select(length >= 2) | {value: .[0], label: .[1]})
        ' || true
  )
fi
[[ -n $keyboard_layouts_json ]] || keyboard_layouts_json='[]'

hypr_opt() {
  local raw
  raw=$(hyprctl -j getoption "$1" 2>/dev/null || true)
  if printf '%s' "$raw" | jq -e . >/dev/null 2>&1; then
    printf '%s' "$raw"
  else
    echo '{}'
  fi
}

cursor_size=${HYPRCURSOR_SIZE:-${XCURSOR_SIZE:-24}}
[[ $cursor_size =~ ^[0-9]+$ ]] || cursor_size=24
if (( cursor_size < 8 )); then cursor_size=8; fi
if (( cursor_size > 64 )); then cursor_size=64; fi

hypr_look_json=$(jq -n \
  --argjson gapsIn "$(hypr_opt general:gaps_in)" \
  --argjson gapsOut "$(hypr_opt general:gaps_out)" \
  --argjson borderSize "$(hypr_opt general:border_size)" \
  --argjson rounding "$(hypr_opt decoration:rounding)" \
  --argjson blur "$(hypr_opt decoration:blur:enabled)" \
  --argjson shadow "$(hypr_opt decoration:shadow:enabled)" \
  --argjson layout "$(hypr_opt general:layout)" \
  --argjson columnWidth "$(hypr_opt scrolling:column_width)" \
  --argjson dimInactive "$(hypr_opt decoration:dim_inactive)" \
  --argjson dimStrength "$(hypr_opt decoration:dim_strength)" \
  --argjson animations "$(hypr_opt animations:enabled)" \
  --argjson cursorHideOnKey "$(hypr_opt cursor:hide_on_key_press)" \
  --argjson cursorWarp "$(hypr_opt cursor:warp_on_change_workspace)" \
  --argjson cursorSize "$cursor_size" \
  --argjson allowTearing "$(hypr_opt general:allow_tearing)" \
  --argjson resizeOnBorder "$(hypr_opt general:resize_on_border)" \
  --argjson activeOpacity "$(hypr_opt decoration:active_opacity)" \
  --argjson preserveSplit "$(hypr_opt dwindle:preserve_split)" \
  --argjson focusOnActivate "$(hypr_opt misc:focus_on_activate)" \
  '
  def num(o; fb):
    if o.int != null then o.int
    elif o.float != null then o.float
    elif (o.css | type) == "string" then (o.css | split(" ")[0] | tonumber? // fb)
    else fb end;
  def flag(o; fb): if o.bool != null then o.bool else fb end;
  def txt(o; fb):
    if (o.str | type) == "string" and o.str != "[[EMPTY]]" then o.str else fb end;
  {
    gapsIn: num($gapsIn; 5),
    gapsOut: num($gapsOut; 10),
    borderSize: num($borderSize; 2),
    rounding: num($rounding; 0),
    blur: flag($blur; false),
    shadow: flag($shadow; false),
    layout: txt($layout; "dwindle"),
    columnWidth: num($columnWidth; 0.49),
    dimInactive: flag($dimInactive; false),
    dimStrength: num($dimStrength; 0.15),
    animations: flag($animations; true),
    cursorHideOnKey: flag($cursorHideOnKey; true),
    cursorWarp: ((num($cursorWarp; 1) | tonumber) != 0),
    cursorSize: $cursorSize,
    allowTearing: flag($allowTearing; false),
    resizeOnBorder: flag($resizeOnBorder; false),
    activeOpacity: num($activeOpacity; 1),
    preserveSplit: flag($preserveSplit; false),
    focusOnActivate: flag($focusOnActivate; false)
  }
' 2>/dev/null || echo '{}')
[[ -n $hypr_look_json ]] || hypr_look_json='{}'

hypr_input_json=$(jq -n \
  --argjson sensitivity "$(hypr_opt input:sensitivity)" \
  --argjson accelProfile "$(hypr_opt input:accel_profile)" \
  --argjson emulateDiscreteScroll "$(hypr_opt input:emulate_discrete_scroll)" \
  --argjson naturalScroll "$(hypr_opt input:touchpad:natural_scroll)" \
  --argjson scrollFactor "$(hypr_opt input:touchpad:scroll_factor)" \
  --argjson clickfinger "$(hypr_opt input:touchpad:clickfinger_behavior)" \
  --argjson disableWhileTyping "$(hypr_opt input:touchpad:disable_while_typing)" \
  --argjson drag3fg "$(hypr_opt input:touchpad:drag_3fg)" \
  --argjson repeatRate "$(hypr_opt input:repeat_rate)" \
  --argjson repeatDelay "$(hypr_opt input:repeat_delay)" \
  --argjson numlock "$(hypr_opt input:numlock_by_default)" \
  --argjson followMouse "$(hypr_opt input:follow_mouse)" \
  --argjson keyPressDpms "$(hypr_opt misc:key_press_enables_dpms)" \
  --argjson mouseMoveDpms "$(hypr_opt misc:mouse_move_enables_dpms)" \
  --argjson kbLayout "$(hypr_opt input:kb_layout)" \
  --argjson kbVariant "$(hypr_opt input:kb_variant)" \
  --argjson kbOptions "$(hypr_opt input:kb_options)" \
  '
  def num(o; fb):
    if o.int != null then o.int
    elif o.float != null then o.float
    else fb end;
  def flag(o; fb): if o.bool != null then o.bool else fb end;
  def txt(o; fb):
    if (o.str | type) == "string" and o.str != "[[EMPTY]]" then o.str else fb end;
  {
    sensitivity: num($sensitivity; 0),
    accelProfile: txt($accelProfile; ""),
    emulateDiscreteScroll: num($emulateDiscreteScroll; 1),
    naturalScroll: flag($naturalScroll; false),
    scrollFactor: num($scrollFactor; 0.4),
    clickfinger: flag($clickfinger; true),
    disableWhileTyping: flag($disableWhileTyping; true),
    drag3fg: num($drag3fg; 0),
    repeatRate: num($repeatRate; 40),
    repeatDelay: num($repeatDelay; 250),
    numlock: flag($numlock; true),
    followMouse: num($followMouse; 1),
    keyPressDpms: flag($keyPressDpms; true),
    mouseMoveDpms: flag($mouseMoveDpms; true),
    kbLayout: txt($kbLayout; ""),
    kbVariant: txt($kbVariant; ""),
    kbOptions: txt($kbOptions; "")
  }
' 2>/dev/null || echo '{}')
[[ -n $hypr_input_json ]] || hypr_input_json='{}'

hypr_look_managed=false
if [[ -f $HOME/.config/hypr/looknfeel.lua ]] && grep -q -- '-- atmos:look begin' "$HOME/.config/hypr/looknfeel.lua"; then
  hypr_look_managed=true
fi
hypr_input_managed=false
if [[ -f $HOME/.config/hypr/input.lua ]] && grep -q -- '-- atmos:input begin' "$HOME/.config/hypr/input.lua"; then
  hypr_input_managed=true
fi
hypr_workspace_gesture=false
if [[ -f $HOME/.config/hypr/input.lua ]] && grep -q -- '-- atmos:input begin' "$HOME/.config/hypr/input.lua" && grep -q 'action = "workspace"' "$HOME/.config/hypr/input.lua"; then
  hypr_workspace_gesture=true
fi

hypr_no_gaps=false
[[ -f $HOME/.local/state/omarchy/toggles/hypr/window-no-gaps.lua ]] && hypr_no_gaps=true
hypr_square_aspect=false
[[ -f $HOME/.local/state/omarchy/toggles/hypr/single-window-aspect-ratio.lua ]] && hypr_square_aspect=true

hypr_workspace_layout=$(hyprctl activeworkspace -j 2>/dev/null | jq -r '.tiledLayout // "dwindle"' || true)
case $hypr_workspace_layout in
  scrolling) ;;
  *) hypr_workspace_layout=dwindle ;;
esac

fingerprint_available=false
if present omarchy-hw-fingerprint && omarchy-hw-fingerprint >/dev/null 2>&1; then
  fingerprint_available=true
fi
fingerprint_configured=false
if [[ -r /etc/pam.d/sudo ]] && grep -q 'pam_fprintd.so' /etc/pam.d/sudo; then
  fingerprint_configured=true
fi
fido2_configured=false
if [[ -r /etc/pam.d/sudo ]] && grep -q 'pam_u2f.so' /etc/pam.d/sudo; then
  fido2_configured=true
fi
sshd_enabled=false
sshd_active=false
if present systemctl; then
  [[ $(systemctl is-enabled sshd.service 2>/dev/null || true) == enabled ]] && sshd_enabled=true
  [[ $(systemctl is-active sshd.service 2>/dev/null || true) == active ]] && sshd_active=true
fi
passwordless_sudo=false
if [[ -f /etc/sudoers.d/99-omarchy-nopasswd-$USER ]]; then
  passwordless_sudo=true
fi
sudoless_docker=false
if id -nG 2>/dev/null | grep -qw docker; then
  sudoless_docker=true
fi

omarchy_version=$(omarchy version 2>/dev/null | head -n1 || true)
omarchy_version=${omarchy_version%$'\n'}
omarchy_channel=$(omarchy channel current 2>/dev/null | head -n1 || true)
omarchy_channel=${omarchy_channel%$'\n'}
case $omarchy_channel in
  stable | rc | edge | dev) ;;
  *) omarchy_channel="" ;;
esac
update_available=false
update_summary="Omarchy is up to date"
if update_out=$(omarchy update available 2>/dev/null); then
  update_available=true
  update_summary=$(head -n1 <<<"$update_out")
elif [[ -n ${update_out:-} ]]; then
  update_summary=$(head -n1 <<<"$update_out")
fi
[[ -n $update_summary ]] || update_summary="Omarchy is up to date"

# shellcheck source=atmos-xdg.sh
. "$SNAP_DIR/atmos-xdg.sh"
atmos_revision=""
atmos_channel=$(atmos_channel)
atmos_installed=false
atmos_data=$(atmos_data_home)
if [[ -x $atmos_data/bin/atmos ]]; then
  atmos_installed=true
fi
if [[ -r $atmos_data/REVISION ]]; then
  atmos_revision=$(<"$atmos_data/REVISION")
  atmos_revision=${atmos_revision%%$'\n'*}
  [[ $atmos_revision =~ ^[0-9a-f]{4,40}$ ]] || atmos_revision=""
fi

voxtype_installed=false
present voxtype && voxtype_installed=true

hybrid_gpu_available=false
hybrid_gpu_mode=""
if present omarchy-hw-hybrid-gpu && omarchy-hw-hybrid-gpu >/dev/null 2>&1; then
  hybrid_gpu_available=true
fi
if present supergfxctl; then
  hybrid_gpu_mode=$(timeout --kill-after=1s 2s supergfxctl -g 2>/dev/null || true)
  hybrid_gpu_mode=${hybrid_gpu_mode%$'\n'}
  case $hybrid_gpu_mode in
    Integrated | Hybrid) ;;
    *) hybrid_gpu_mode="" ;;
  esac
fi

hw_flag() {
  if omarchy hw "$@" >/dev/null 2>&1; then
    echo true
  else
    echo false
  fi
}

hw_nvidia=$(hw_flag nvidia)
hw_nvidia_gsp=$(hw_flag nvidia gsp)
hw_nvidia_without_gsp=$(hw_flag nvidia without gsp)
hw_vulkan=$(hw_flag vulkan)
hw_intel=$(hw_flag intel)
hw_intel_ptl=$(hw_flag intel ptl)
hw_webcam=$(hw_flag webcam)
hw_framework16=$(hw_flag framework16)
hw_asus_rog=$(hw_flag asus rog)
hw_surface=$(hw_flag surface)

dmi_field() {
  local path=$1
  local v=""
  if [[ -r $path ]]; then
    v=$(< "$path")
    v=${v%%$'\n'*}
    v=${v//$'\r'/}
  fi
  printf '%s' "$v"
}

dmi_vendor=$(dmi_field /sys/class/dmi/id/sys_vendor)
dmi_product=$(dmi_field /sys/class/dmi/id/product_name)
dmi_family=$(dmi_field /sys/class/dmi/id/product_family)

cpu_stat=""
memory_stat=""
if stats_out=$(omarchy system stats 2>/dev/null); then
  while IFS= read -r line || [[ -n $line ]]; do
    case $line in
      cpu*) cpu_stat=${line#cpu} ;;
      memory*) memory_stat=${line#memory} ;;
    esac
  done <<<"$stats_out"
fi

cpu_identity=""
if [[ -r /proc/cpuinfo ]]; then
  cpu_identity=$(awk -F: '
    BEGIN { IGNORECASE=1 }
    $1 ~ /^model name[ \t]*$/ {
      sub(/^[ \t]+/, "", $2)
      sub(/[ \t]+$/, "", $2)
      print $2
      exit
    }
  ' /proc/cpuinfo)
  if [[ -z $cpu_identity ]]; then
    cpu_identity=$(awk -F: '
      BEGIN { IGNORECASE=1 }
      $1 ~ /^(Hardware|cpu model)[ \t]*$/ {
        sub(/^[ \t]+/, "", $2)
        sub(/[ \t]+$/, "", $2)
        print $2
        exit
      }
    ' /proc/cpuinfo)
  fi
fi

pci_identity() {
  local kind=$1
  present lspci || return 0
  lspci -nn 2>/dev/null | awk -v kind="$kind" '
    BEGIN { IGNORECASE=1 }
    {
      gpu = (index($0, "VGA compatible controller") || index($0, "3D controller") || index($0, "Display controller"))
      npu = (index($0, "Neural Processing") || index($0, "NPU") || index($0, "XDNA") || index($0, "TPU") || index($0, "Habana") || index($0, "Hailo") || index($0, "Coral") || index($0, "AI accelerator"))
      if (kind == "gpu" && !gpu) next
      if (kind == "npu" && !npu) next
      idx = index($0, ": ")
      if (idx == 0) next
      $0 = substr($0, idx + 2)
      sub(/ \[[0-9a-fA-F]{4}:[0-9a-fA-F]{4}\].*/, "")
      sub(/ \(rev [^)]+\)$/, "")
      gsub(/^[ \t]+|[ \t]+$/, "")
      if (length($0) > 0) { print; exit }
    }
  '
}

gpu_identity=$(pci_identity gpu || true)
npu_identity=$(pci_identity npu || true)

tailscale_installed=false
tailscale_running=false
if present tailscale; then
  tailscale_installed=true
fi
if present systemctl && [[ $(systemctl is-active tailscaled.service 2>/dev/null || true) == active ]]; then
  tailscale_running=true
  tailscale_installed=true
fi

plugins_json='[]'
if plugins_out=$(omarchy plugin list --json 2>/dev/null); then
  if jq -e 'type == "array"' <<<"$plugins_out" >/dev/null 2>&1; then
    plugins_json=$plugins_out
  fi
fi

snapper_number_limit=5
snapper_timeline=false
if [[ -r /etc/snapper/configs/root ]]; then
  snapper_limit_raw=$(awk -F= '/^[[:space:]]*NUMBER_LIMIT=/{gsub(/"/, "", $2); print $2; exit}' /etc/snapper/configs/root)
  [[ $snapper_limit_raw =~ ^[0-9]+$ ]] && snapper_number_limit=$snapper_limit_raw
  snapper_timeline_raw=$(awk -F= '/^[[:space:]]*TIMELINE_CREATE=/{gsub(/"/, "", $2); print $2; exit}' /etc/snapper/configs/root)
  [[ $snapper_timeline_raw == yes ]] && snapper_timeline=true
fi
(( snapper_number_limit >= 1 && snapper_number_limit <= 50 )) || snapper_number_limit=5

fstrim_enabled=false
if present systemctl && [[ $(systemctl is-enabled fstrim.timer 2>/dev/null || true) == enabled ]]; then
  fstrim_enabled=true
fi

direct_boot_available=false
direct_boot=false
if [[ -d /sys/firmware/efi ]] && present efibootmgr; then
  direct_boot_available=true
  if efibootmgr 2>/dev/null | grep -qE '^Boot[0-9A-Fa-f]+\*? Omarchy([[:space:]]|$)'; then
    direct_boot=true
  fi
fi

mime_pdf=$(xdg-mime query default application/pdf 2>/dev/null || true)
mime_image=$(xdg-mime query default image/png 2>/dev/null || true)
mime_video=$(xdg-mime query default video/mp4 2>/dev/null || true)
[[ $mime_pdf =~ ^[A-Za-z0-9._-]+\.desktop$ ]] || mime_pdf=""
[[ $mime_image =~ ^[A-Za-z0-9._-]+\.desktop$ ]] || mime_image=""
[[ $mime_video =~ ^[A-Za-z0-9._-]+\.desktop$ ]] || mime_video=""

desktop_option() {
  local id=$1
  local label=$2
  local dir
  for dir in "${XDG_DATA_HOME:-$HOME/.local/share}/applications" /usr/local/share/applications /usr/share/applications; do
    if [[ -f $dir/$id ]]; then
      jq -n --arg value "$id" --arg label "$label" '{value:$value,label:$label}'
      return
    fi
  done
}

mime_pdf_options=$(jq -s '.' \
  < <(
    desktop_option org.gnome.Evince.desktop Evince
    desktop_option org.gnome.Papers.desktop Papers
    desktop_option org.pwmt.zathura.desktop Zathura
    desktop_option org.kde.okular.desktop Okular
    desktop_option firefox.desktop Firefox
    desktop_option chromium.desktop Chromium
  ) 2>/dev/null || echo '[]')
mime_image_options=$(jq -s '.' \
  < <(
    desktop_option imv.desktop imv
    desktop_option org.gnome.Loupe.desktop Loupe
    desktop_option org.gnome.eog.desktop "Image Viewer"
    desktop_option firefox.desktop Firefox
    desktop_option chromium.desktop Chromium
  ) 2>/dev/null || echo '[]')
mime_video_options=$(jq -s '.' \
  < <(
    desktop_option mpv.desktop mpv
    desktop_option vlc.desktop VLC
    desktop_option org.gnome.Totem.desktop Videos
    desktop_option firefox.desktop Firefox
    desktop_option chromium.desktop Chromium
  ) 2>/dev/null || echo '[]')
[[ -n $mime_pdf_options ]] || mime_pdf_options='[]'
[[ -n $mime_image_options ]] || mime_image_options='[]'
[[ -n $mime_video_options ]] || mime_video_options='[]'

if [[ -f ${XDG_CONFIG_HOME:-$HOME/.config}/user-dirs.dirs ]]; then
  # shellcheck disable=SC1090
  source "${XDG_CONFIG_HOME:-$HOME/.config}/user-dirs.dirs"
fi
pictures_dir="${XDG_PICTURES_DIR:-$HOME/Pictures}"
videos_dir="${XDG_VIDEOS_DIR:-$HOME/Videos}"
if present xdg-user-dir; then
  pictures_dir=$(xdg-user-dir PICTURES 2>/dev/null || echo "$pictures_dir")
  videos_dir=$(xdg-user-dir VIDEOS 2>/dev/null || echo "$videos_dir")
fi

recording_active=false
pgrep -f "^gpu-screen-recorder" >/dev/null 2>&1 && recording_active=true
webcam_overlay=false
if present hyprctl && present jq; then
  if hyprctl clients -j 2>/dev/null | jq -e '.[] | select(.title == "WebcamOverlay")' >/dev/null 2>&1; then
    webcam_overlay=true
  fi
fi

localsend_installed=false
present localsend && localsend_installed=true
herdr_installed=false
present herdr && herdr_installed=true
chatgpt_installed=false
present chatgpt && chatgpt_installed=true
docker_installed=false
present docker && docker_installed=true

services_json=$(jq -n \
  --argjson onepassword "$(present 1password && echo true || echo false)" \
  --argjson dropbox "$( { present dropbox || present dropbox-cli; } && echo true || echo false)" \
  --argjson nordvpn "$(present nordvpn && echo true || echo false)" \
  --argjson signal "$(present signal-desktop && echo true || echo false)" \
  --argjson spotify "$(present spotify && echo true || echo false)" \
  --argjson sunshine "$(present sunshine && echo true || echo false)" \
  --argjson tailscale "$tailscale_installed" \
  '{onepassword:$onepassword,dropbox:$dropbox,nordvpn:$nordvpn,signal:$signal,spotify:$spotify,sunshine:$sunshine,tailscale:$tailscale}')
[[ -n $services_json ]] || services_json='{}'

gaming_battlenet=false
[[ -f "$HOME/Games/battlenet/drive_c/Program Files (x86)/Battle.net/Battle.net Launcher.exe" ]] && gaming_battlenet=true
gaming_geforce=false
{ present nvidia-geforcenow || present geforcenow || present GeForceNOW; } && gaming_geforce=true
gaming_xbox_cloud=false
if compgen -G "$HOME/.local/share/applications/*[Xx]box*[Cc]loud*.desktop" >/dev/null; then
  gaming_xbox_cloud=true
fi
gaming_xbox_controllers=false
[[ -f /etc/modules-load.d/xpadneo.conf ]] && gaming_xbox_controllers=true

gaming_json=$(jq -n \
  --argjson steam "$(present steam && echo true || echo false)" \
  --argjson heroic "$(present heroic && echo true || echo false)" \
  --argjson lutris "$(present lutris && echo true || echo false)" \
  --argjson retroarch "$(present retroarch && echo true || echo false)" \
  --argjson battlenet "$gaming_battlenet" \
  --argjson geforceNow "$gaming_geforce" \
  --argjson xboxCloud "$gaming_xbox_cloud" \
  --argjson xboxControllers "$gaming_xbox_controllers" \
  '{steam:$steam,heroic:$heroic,lutris:$lutris,retroarch:$retroarch,battlenet:$battlenet,geforceNow:$geforceNow,xboxCloud:$xboxCloud,xboxControllers:$xboxControllers}')
[[ -n $gaming_json ]] || gaming_json='{}'

extras_json=$(jq -n \
  --argjson chatgpt "$chatgpt_installed" \
  --argjson docker "$docker_installed" \
  --argjson herdr "$herdr_installed" \
  --argjson localsend "$localsend_installed" \
  '{chatgpt:$chatgpt,docker:$docker,herdr:$herdr,localsend:$localsend}')
[[ -n $extras_json ]] || extras_json='{}'

hooks_json='[]'
if present python3; then
  hooks_json=$(python3 "$SNAP_DIR/list-hooks.py" 2>/dev/null || echo '[]')
fi
[[ -n $hooks_json ]] || hooks_json='[]'

autostart_file=${ATMOS_AUTOSTART_FILE:-"$HOME/.config/hypr/autostart.lua"}
autostart_json='[]'
autostart_managed=false
if present python3; then
  autostart_json=$(python3 "$SNAP_DIR/hypr-sentinel.py" autostart list "$autostart_file" 2>/dev/null || echo '[]')
fi
[[ -n $autostart_json ]] || autostart_json='[]'
if [[ -f $autostart_file ]] && grep -q -- '-- atmos:autostart begin' "$autostart_file"; then
  autostart_managed=true
fi

bindings_file=${ATMOS_BINDINGS_FILE:-"$HOME/.config/hypr/bindings.lua"}
bindings_json='[]'
bindings_managed=false
if present python3; then
  bindings_json=$(python3 "$SNAP_DIR/hypr-sentinel.py" bindings list "$bindings_file" 2>/dev/null || echo '[]')
fi
[[ -n $bindings_json ]] || bindings_json='[]'
if [[ -f $bindings_file ]] && grep -q -- '-- atmos:bindings begin' "$bindings_file"; then
  bindings_managed=true
fi

windows_file=${ATMOS_WINDOWS_FILE:-"$HOME/.config/hypr/atmos.lua"}
window_rules_json='[]'
window_rules_managed=false
if present python3; then
  window_rules_json=$(python3 "$SNAP_DIR/hypr-sentinel.py" windows list "$windows_file" 2>/dev/null || echo '[]')
fi
[[ -n $window_rules_json ]] || window_rules_json='[]'
if [[ -f $windows_file ]] && grep -q -- '-- atmos:windows begin' "$windows_file"; then
  window_rules_managed=true
fi

keybindings_json='[]'
if present omarchy && present python3; then
  print_cmd=(omarchy menu keybindings --print)
  if present timeout; then
    print_cmd=(timeout 10 omarchy menu keybindings --print)
  fi
  keybindings_json=$("${print_cmd[@]}" 2>/dev/null | python3 -c '
import json, sys
out = []
for line in sys.stdin:
    line = line.rstrip("\n")
    sep = line.find(" \u2192 ")
    arrow = 3
    if sep < 0:
        sep = line.find(" -> ")
        arrow = 4
    if sep < 0:
        continue
    keys = line[:sep].rstrip()
    action = line[sep + arrow:].strip()
    if keys and action:
        out.append({"keys": keys, "action": action})
print(json.dumps(out))
' || echo '[]')
fi
[[ -n $keybindings_json ]] || keybindings_json='[]'

focused_class=""
if present hyprctl && present jq; then
  focused_class=$(hyprctl activewindow -j 2>/dev/null | jq -r '.class // empty' || true)
fi

cups_active=false
if systemctl is-active --quiet cups.service 2>/dev/null; then
  cups_active=true
fi
printer_setup=false
if present system-config-printer; then
  printer_setup=true
fi

hyprsunset_file=${ATMOS_HYPRSUNSET_FILE:-"$HOME/.config/hypr/hyprsunset.conf"}
nightlight_day=07:00
nightlight_night=20:00
nightlight_night_on=false
if [[ -f $hyprsunset_file ]]; then
  hyprsunset_parsed=$(python3 - "$hyprsunset_file" <<'PY' 2>/dev/null || true
import re, sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
day = ""
night = ""
night_on = False
temp = 0
blocks = re.split(r"profile\s*\{", text)
for block in blocks[1:]:
    body = block.split("}", 1)[0]
    time_m = re.search(r'time\s*=\s*"?([0-2]?\d:[0-5]\d)"?', body)
    if not time_m:
        continue
    time = time_m.group(1)
    if re.search(r"\bidentity\s*=\s*true\b", body):
        day = time
        continue
    temp_m = re.search(r"temperature\s*=\s*([0-9]+)", body)
    if temp_m:
        night = time
        night_on = True
        temp = int(temp_m.group(1))
print(f"{day}\t{night}\t{str(night_on).lower()}\t{temp}")
PY
  )
  if [[ -n ${hyprsunset_parsed:-} ]]; then
    IFS=$'\t' read -r parsed_day parsed_night parsed_night_on parsed_temp <<<"$hyprsunset_parsed"
    [[ $parsed_day =~ ^[0-2]?\d:[0-5]\d$ ]] && nightlight_day=$parsed_day
    [[ $parsed_night =~ ^[0-2]?\d:[0-5]\d$ ]] && nightlight_night=$parsed_night
    [[ $parsed_night_on == true ]] && nightlight_night_on=true
  fi
fi

tailscale_peers_json='[]'
if [[ $tailscale_running == true ]] && present tailscale && present jq; then
  tailscale_peers_json=$(tailscale status --json 2>/dev/null | jq -c '[.Peer[]? | {name: (.HostName // (.DNSName // "" | split(".")[0])), online: (.Online == true)}] | map(select(.name != ""))' || echo '[]')
fi
[[ -n $tailscale_peers_json ]] || tailscale_peers_json='[]'

jq -n \
  --arg theme "$theme" \
  --arg background "$background" \
  --arg font "$font" \
  --arg browser "$browser" \
  --arg terminal "$terminal" \
  --arg editor "$editor" \
  --arg agent "$agent" \
  --arg dns "$dns" \
  --arg powerProfile "$power_profile" \
  --arg powerProfileAc "$power_profile_ac" \
  --arg powerProfileBattery "$power_profile_battery" \
  --arg plymouth "$plymouth" \
  --arg barPosition "$bar_position" \
  --argjson barTransparent "$bar_transparent" \
  --arg clockFormat "$clock_format" \
  --arg clockFormatAlt "$clock_format_alt" \
  --arg clockWeekStart "$clock_week_start" \
  --argjson clockPresent "$clock_present" \
  --argjson clockBirthYear "$clock_birth_year" \
  --argjson clockLifeExpectancy "$clock_life_expectancy" \
  --argjson textSize "$text_size" \
  --argjson screensaver "$screensaver" \
  --argjson lock "$lock" \
  --argjson stayAwake "$stay_awake" \
  --argjson nightlight "$nightlight" \
  --argjson nightlightTemperature "$nightlight_temp" \
  --argjson screensaverEnabled "$screensaver_enabled" \
  --argjson screensaverBranded "$screensaver_branded" \
  --argjson aboutBranded "$about_branded" \
  --argjson barVisible "$bar_visible" \
  --argjson bluetooth "$bluetooth" \
  --argjson wifiConnected "$wifi_connected" \
  --arg wifiBand "$wifi_band" \
  --arg wifiBandSelected "$wifi_selected" \
  --argjson wifiBands "$wifi_bands_json" \
  --arg wifiIface "$wifi_iface" \
  --arg netKind "$net_kind" \
  --arg netIface "$net_iface" \
  --arg netSsid "$net_ssid" \
  --arg netSignal "$net_signal" \
  --arg netIp "$net_ip" \
  --arg netSpeed "$net_speed" \
  --argjson wifiHw "$wifi_hw" \
  --argjson wifiRadio "$wifi_radio" \
  --argjson wifiConnections "$wifi_connections_json" \
  --argjson bluetoothDevices "$bluetooth_devices_json" \
  --argjson audioSinks "$audio_sinks_json" \
  --argjson audioSources "$audio_sources_json" \
  --argjson audioOutputVolume "$audio_output_volume" \
  --argjson audioOutputMuted "$audio_output_muted" \
  --argjson audioInputVolume "$audio_input_volume" \
  --argjson audioInputMuted "$audio_input_muted" \
  --argjson audioTuningMatch "$audio_tuning_match" \
  --argjson audioTuningOn "$audio_tuning_on" \
  --argjson disks "$disks_json" \
  --argjson hardware "$hardware_json" \
  --argjson luksDevices "$luks_devices_json" \
  --argjson swapDevices "$swap_devices_json" \
  --argjson snapperPresent "$snapper_present" \
  --argjson snapperConfigs "$snapper_configs_json" \
  --argjson snapshots "$snapshots_json" \
  --argjson hibernationAvailable "$hibernation_available" \
  --argjson hibernationSupported "$hibernation_supported" \
  --argjson hibernationConfigured "$hibernation_configured" \
  --argjson suspendEnabled "$suspend_enabled" \
  --argjson crashCapture "$crash_capture" \
  --argjson doNotDisturb "$dnd" \
  --arg weatherLocation "$weather" \
  --arg weatherCoords "$weather_coords" \
  --argjson weatherAuto "$weather_auto" \
  --argjson weatherPresent "$weather_present" \
  --arg weatherUnit "$weather_unit" \
  --argjson weatherRefreshMinutes "$weather_refresh" \
  --argjson reminderCount "$reminder_count" \
  --argjson reminderActive "$reminder_active" \
  --argjson reminders "$reminders_json" \
  --argjson indicatorsPresent "$indicators_present" \
  --argjson indicatorsAlwaysShow "$indicators_always_show" \
  --argjson indicatorsItems "$indicators_items_json" \
  --argjson agentsPresent "$agents_present" \
  --argjson agentsRefreshIntervalSec "$agents_refresh" \
  --argjson agentsSync "$agents_sync" \
  --arg agentsSyncDir "$agents_sync_dir" \
  --arg agentsSyncFileName "$agents_sync_file" \
  --arg agentsSyncDeviceId "$agents_sync_device" \
  --argjson spacerPresent "$spacer_present" \
  --argjson spacerSize "$spacer_size" \
  --argjson trayPresent "$tray_present" \
  --argjson trayHidden "$tray_hidden_json" \
  --argjson trayPinned "$tray_pinned_json" \
  --argjson powerPresent "$power_present" \
  --argjson powerShowPercentage "$power_show_percentage" \
  --argjson isLaptop "$is_laptop" \
  --argjson batteryPresent "$battery_present" \
  --argjson monitors "$monitors_json" \
  --argjson internalPresent "$internal_present" \
  --argjson internalEnabled "$internal_enabled" \
  --argjson externalPresent "$external_present" \
  --argjson mirroring "$mirroring" \
  --argjson touchpadPresent "$touchpad_present" \
  --argjson touchpadEnabled "$touchpad_enabled" \
  --argjson touchscreenPresent "$touchscreen_present" \
  --argjson touchscreenEnabled "$touchscreen_enabled" \
  --argjson keyboardBacklightPresent "$keyboard_backlight_present" \
  --argjson keyboardBrightness "$keyboard_brightness" \
  --argjson themes "$themes_json" \
  --argjson extraThemes "$extra_themes_json" \
  --argjson desktopApps "$desktop_apps_json" \
  --argjson tuiApps "$tui_apps_json" \
  --argjson webApps "$web_apps_json" \
  --argjson fonts "$fonts_json" \
  --argjson powerProfiles "$power_profiles_json" \
  --argjson plymouthThemes "$plymouth_themes_json" \
  --arg hostname "$hostname" \
  --arg fullName "$full_name" \
  --arg timezone "$timezone" \
  --argjson timezones "$timezones_json" \
  --argjson ntp "$ntp" \
  --argjson ntpAvailable "$ntp_available" \
  --argjson ntpSynchronized "$ntp_synchronized" \
  --arg locale "$locale" \
  --argjson locales "$locales_json" \
  --argjson parallelDownloads "$parallel_downloads" \
  --arg keyboardLayout "$keyboard_layout" \
  --argjson keyboardLayouts "$keyboard_layouts_json" \
  --argjson hyprLook "$hypr_look_json" \
  --argjson hyprInput "$hypr_input_json" \
  --argjson hyprLookManaged "$hypr_look_managed" \
  --argjson hyprInputManaged "$hypr_input_managed" \
  --argjson hyprWorkspaceGesture "$hypr_workspace_gesture" \
  --argjson hyprNoGaps "$hypr_no_gaps" \
  --argjson hyprSquareAspect "$hypr_square_aspect" \
  --arg hyprWorkspaceLayout "$hypr_workspace_layout" \
  --argjson fingerprintAvailable "$fingerprint_available" \
  --argjson fingerprintConfigured "$fingerprint_configured" \
  --argjson fido2Configured "$fido2_configured" \
  --argjson sshdEnabled "$sshd_enabled" \
  --argjson sshdActive "$sshd_active" \
  --argjson passwordlessSudo "$passwordless_sudo" \
  --argjson sudolessDocker "$sudoless_docker" \
  --arg omarchyVersion "$omarchy_version" \
  --arg omarchyChannel "$omarchy_channel" \
  --argjson updateAvailable "$update_available" \
  --arg updateSummary "$update_summary" \
  --arg atmosRevision "$atmos_revision" \
  --arg atmosChannel "$atmos_channel" \
  --argjson atmosInstalled "$atmos_installed" \
  --argjson voxtypeInstalled "$voxtype_installed" \
  --argjson hybridGpuAvailable "$hybrid_gpu_available" \
  --arg hybridGpuMode "$hybrid_gpu_mode" \
  --argjson hwNvidia "$hw_nvidia" \
  --argjson hwNvidiaGsp "$hw_nvidia_gsp" \
  --argjson hwNvidiaWithoutGsp "$hw_nvidia_without_gsp" \
  --argjson hwVulkan "$hw_vulkan" \
  --argjson hwIntel "$hw_intel" \
  --argjson hwIntelPtl "$hw_intel_ptl" \
  --argjson hwWebcam "$hw_webcam" \
  --argjson hwFramework16 "$hw_framework16" \
  --argjson hwAsusRog "$hw_asus_rog" \
  --argjson hwSurface "$hw_surface" \
  --arg dmiVendor "$dmi_vendor" \
  --arg dmiProduct "$dmi_product" \
  --arg dmiFamily "$dmi_family" \
  --arg cpuStat "$cpu_stat" \
  --arg memoryStat "$memory_stat" \
  --arg cpuIdentity "$cpu_identity" \
  --arg gpuIdentity "$gpu_identity" \
  --arg npuIdentity "$npu_identity" \
  --argjson tailscaleInstalled "$tailscale_installed" \
  --argjson tailscaleRunning "$tailscale_running" \
  --argjson plugins "$plugins_json" \
  --argjson snapperNumberLimit "$snapper_number_limit" \
  --argjson snapperTimeline "$snapper_timeline" \
  --argjson fstrimEnabled "$fstrim_enabled" \
  --argjson directBootAvailable "$direct_boot_available" \
  --argjson directBoot "$direct_boot" \
  --arg mimePdf "$mime_pdf" \
  --arg mimeImage "$mime_image" \
  --arg mimeVideo "$mime_video" \
  --argjson mimePdfOptions "$mime_pdf_options" \
  --argjson mimeImageOptions "$mime_image_options" \
  --argjson mimeVideoOptions "$mime_video_options" \
  --arg picturesDir "$pictures_dir" \
  --arg videosDir "$videos_dir" \
  --argjson recordingActive "$recording_active" \
  --argjson webcamOverlay "$webcam_overlay" \
  --argjson services "$services_json" \
  --argjson gaming "$gaming_json" \
  --argjson extras "$extras_json" \
  --argjson hooks "$hooks_json" \
  --argjson autostart "$autostart_json" \
  --argjson autostartManaged "$autostart_managed" \
  --argjson bindings "$bindings_json" \
  --argjson bindingsManaged "$bindings_managed" \
  --argjson windowRules "$window_rules_json" \
  --argjson windowRulesManaged "$window_rules_managed" \
  --argjson keybindings "$keybindings_json" \
  --arg focusedClass "$focused_class" \
  --argjson cupsActive "$cups_active" \
  --argjson printerSetup "$printer_setup" \
  --arg nightlightDay "$nightlight_day" \
  --arg nightlightNight "$nightlight_night" \
  --argjson nightlightNightOn "$nightlight_night_on" \
  --argjson tailscalePeers "$tailscale_peers_json" \
  --argjson hasAether "$(present aether && echo true || echo false)" \
  --argjson browsers "$(jq -n \
    --argjson chromium "$(present chromium && echo true || echo false)" \
    --argjson chrome "$(present google-chrome-stable && echo true || echo false)" \
    --argjson brave "$(present brave && echo true || echo false)" \
    --argjson braveOrigin "$(present brave-origin && echo true || echo false)" \
    --argjson edge "$(present microsoft-edge-stable && echo true || echo false)" \
    --argjson firefox "$(present firefox && echo true || echo false)" \
    --argjson zen "$(present zen-browser && echo true || echo false)" \
    '{chromium:$chromium,chrome:$chrome,brave:$brave,"brave-origin":$braveOrigin,edge:$edge,firefox:$firefox,zen:$zen}')" \
  --argjson terminals "$(jq -n \
    --argjson alacritty "$(present alacritty && echo true || echo false)" \
    --argjson foot "$(present foot && echo true || echo false)" \
    --argjson ghostty "$(present ghostty && echo true || echo false)" \
    --argjson kitty "$(present kitty && echo true || echo false)" \
    '{alacritty:$alacritty,foot:$foot,ghostty:$ghostty,kitty:$kitty}')" \
  --argjson editors "$(jq -n \
    --argjson nvim "$(present nvim && echo true || echo false)" \
    --argjson code "$(present code && echo true || echo false)" \
    --argjson cursor "$(present cursor && echo true || echo false)" \
    --argjson zeditor "$(present zeditor && echo true || echo false)" \
    --argjson sublime_text "$(present sublime_text && echo true || echo false)" \
    --argjson helix "$(present helix && echo true || echo false)" \
    --argjson vim "$(present vim && echo true || echo false)" \
    --argjson emacs "$(present emacs && echo true || echo false)" \
    '{nvim:$nvim,code:$code,cursor:$cursor,zeditor:$zeditor,sublime_text:$sublime_text,helix:$helix,vim:$vim,emacs:$emacs}')" \
  '{
    theme: $theme,
    background: $background,
    font: $font,
    textSize: $textSize,
    themes: $themes,
    extraThemes: $extraThemes,
    desktopApps: $desktopApps,
    tuiApps: $tuiApps,
    webApps: $webApps,
    fonts: $fonts,
    barPosition: $barPosition,
    barTransparent: $barTransparent,
    clockFormat: $clockFormat,
    clockFormatAlt: $clockFormatAlt,
    clockWeekStart: $clockWeekStart,
    clockPresent: $clockPresent,
    clockBirthYear: $clockBirthYear,
    clockLifeExpectancy: $clockLifeExpectancy,
    browser: $browser,
    terminal: $terminal,
    editor: $editor,
    agent: $agent,
    dns: $dns,
    idleScreensaver: $screensaver,
    idleLock: $lock,
    stayAwake: $stayAwake,
    nightlight: $nightlight,
    nightlightTemperature: $nightlightTemperature,
    screensaverEnabled: $screensaverEnabled,
    screensaverBranded: $screensaverBranded,
    aboutBranded: $aboutBranded,
    barVisible: $barVisible,
    bluetooth: $bluetooth,
    wifiConnected: $wifiConnected,
    wifiBand: $wifiBand,
    wifiBandSelected: $wifiBandSelected,
    wifiBands: $wifiBands,
    wifiIface: $wifiIface,
    netKind: $netKind,
    netIface: $netIface,
    netSsid: $netSsid,
    netSignal: $netSignal,
    netIp: $netIp,
    netSpeed: $netSpeed,
    wifiHw: $wifiHw,
    wifiRadio: $wifiRadio,
    wifiConnections: $wifiConnections,
    bluetoothDevices: $bluetoothDevices,
    audioSinks: $audioSinks,
    audioSources: $audioSources,
    audioOutputVolume: $audioOutputVolume,
    audioOutputMuted: $audioOutputMuted,
    audioInputVolume: $audioInputVolume,
    audioInputMuted: $audioInputMuted,
    audioTuningMatch: $audioTuningMatch,
    audioTuningOn: $audioTuningOn,
    disks: $disks,
    hardware: $hardware,
    luksDevices: $luksDevices,
    swapDevices: $swapDevices,
    snapperPresent: $snapperPresent,
    snapperConfigs: $snapperConfigs,
    snapshots: $snapshots,
    hibernationAvailable: $hibernationAvailable,
    hibernationSupported: $hibernationSupported,
    hibernationConfigured: $hibernationConfigured,
    suspendEnabled: $suspendEnabled,
    crashCapture: $crashCapture,
    doNotDisturb: $doNotDisturb,
    weatherLocation: $weatherLocation,
    weatherCoords: $weatherCoords,
    weatherAuto: $weatherAuto,
    weatherPresent: $weatherPresent,
    weatherUnit: $weatherUnit,
    weatherRefreshMinutes: $weatherRefreshMinutes,
    reminderCount: $reminderCount,
    reminderActive: $reminderActive,
    reminders: $reminders,
    indicatorsPresent: $indicatorsPresent,
    indicatorsAlwaysShow: $indicatorsAlwaysShow,
    indicatorsItems: $indicatorsItems,
    agentsPresent: $agentsPresent,
    agentsRefreshIntervalSec: $agentsRefreshIntervalSec,
    agentsSync: $agentsSync,
    agentsSyncDir: $agentsSyncDir,
    agentsSyncFileName: $agentsSyncFileName,
    agentsSyncDeviceId: $agentsSyncDeviceId,
    spacerPresent: $spacerPresent,
    spacerSize: $spacerSize,
    trayPresent: $trayPresent,
    trayHidden: $trayHidden,
    trayPinned: $trayPinned,
    powerPresent: $powerPresent,
    powerShowPercentage: $powerShowPercentage,
    isLaptop: $isLaptop,
    batteryPresent: $batteryPresent,
    monitors: $monitors,
    internalPresent: $internalPresent,
    internalEnabled: $internalEnabled,
    externalPresent: $externalPresent,
    mirroring: $mirroring,
    touchpadPresent: $touchpadPresent,
    touchpadEnabled: $touchpadEnabled,
    touchscreenPresent: $touchscreenPresent,
    touchscreenEnabled: $touchscreenEnabled,
    keyboardBacklightPresent: $keyboardBacklightPresent,
    keyboardBrightness: $keyboardBrightness,
    powerProfile: $powerProfile,
    powerProfileAc: $powerProfileAc,
    powerProfileBattery: $powerProfileBattery,
    powerProfiles: $powerProfiles,
    plymouth: $plymouth,
    plymouthThemes: $plymouthThemes,
    hasAether: $hasAether,
    browsers: $browsers,
    terminals: $terminals,
    editors: $editors,
    hostname: $hostname,
    fullName: $fullName,
    timezone: $timezone,
    timezones: $timezones,
    ntp: $ntp,
    ntpAvailable: $ntpAvailable,
    ntpSynchronized: $ntpSynchronized,
    locale: $locale,
    locales: $locales,
    parallelDownloads: $parallelDownloads,
    keyboardLayout: $keyboardLayout,
    keyboardLayouts: $keyboardLayouts,
    hyprLook: $hyprLook,
    hyprInput: $hyprInput,
    hyprLookManaged: $hyprLookManaged,
    hyprInputManaged: $hyprInputManaged,
    hyprWorkspaceGesture: $hyprWorkspaceGesture,
    hyprNoGaps: $hyprNoGaps,
    hyprSquareAspect: $hyprSquareAspect,
    hyprWorkspaceLayout: $hyprWorkspaceLayout,
    fingerprintAvailable: $fingerprintAvailable,
    fingerprintConfigured: $fingerprintConfigured,
    fido2Configured: $fido2Configured,
    sshdEnabled: $sshdEnabled,
    sshdActive: $sshdActive,
    passwordlessSudo: $passwordlessSudo,
    sudolessDocker: $sudolessDocker,
    omarchyVersion: $omarchyVersion,
    omarchyChannel: $omarchyChannel,
    updateAvailable: $updateAvailable,
    updateSummary: $updateSummary,
    atmosRevision: $atmosRevision,
    atmosChannel: $atmosChannel,
    atmosInstalled: $atmosInstalled,
    voxtypeInstalled: $voxtypeInstalled,
    hybridGpuAvailable: $hybridGpuAvailable,
    hybridGpuMode: $hybridGpuMode,
    hwNvidia: $hwNvidia,
    hwNvidiaGsp: $hwNvidiaGsp,
    hwNvidiaWithoutGsp: $hwNvidiaWithoutGsp,
    hwVulkan: $hwVulkan,
    hwIntel: $hwIntel,
    hwIntelPtl: $hwIntelPtl,
    hwWebcam: $hwWebcam,
    hwFramework16: $hwFramework16,
    hwAsusRog: $hwAsusRog,
    hwSurface: $hwSurface,
    dmiVendor: $dmiVendor,
    dmiProduct: $dmiProduct,
    dmiFamily: $dmiFamily,
    cpuStat: $cpuStat,
    memoryStat: $memoryStat,
    cpuIdentity: $cpuIdentity,
    gpuIdentity: $gpuIdentity,
    npuIdentity: $npuIdentity,
    tailscaleInstalled: $tailscaleInstalled,
    tailscaleRunning: $tailscaleRunning,
    plugins: $plugins,
    snapperNumberLimit: $snapperNumberLimit,
    snapperTimeline: $snapperTimeline,
    fstrimEnabled: $fstrimEnabled,
    directBootAvailable: $directBootAvailable,
    directBoot: $directBoot,
    mimePdf: $mimePdf,
    mimeImage: $mimeImage,
    mimeVideo: $mimeVideo,
    mimePdfOptions: $mimePdfOptions,
    mimeImageOptions: $mimeImageOptions,
    mimeVideoOptions: $mimeVideoOptions,
    picturesDir: $picturesDir,
    videosDir: $videosDir,
    recordingActive: $recordingActive,
    webcamOverlay: $webcamOverlay,
    services: $services,
    gaming: $gaming,
    extras: $extras,
    hooks: $hooks,
    autostart: $autostart,
    autostartManaged: $autostartManaged,
    bindings: $bindings,
    bindingsManaged: $bindingsManaged,
    windowRules: $windowRules,
    windowRulesManaged: $windowRulesManaged,
    keybindings: $keybindings,
    focusedClass: $focusedClass,
    cupsActive: $cupsActive,
    printerSetup: $printerSetup,
    nightlightDay: $nightlightDay,
    nightlightNight: $nightlightNight,
    nightlightNightOn: $nightlightNightOn,
    tailscalePeers: $tailscalePeers
  }'
