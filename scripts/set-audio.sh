#!/bin/bash
# Set absolute PipeWire volumes. `omarchy audio output volume` only steps
# relatively; the Sound page slider needs a percent.

set -euo pipefail

action=${1:-}
value=${2:-}

usage() {
  echo "Usage: set-audio.sh output-volume <0-100> | input-volume <0-100>" >&2
  exit 1
}

[[ $value =~ ^[0-9]+$ ]] || usage
(( value >= 0 && value <= 100 )) || usage

case "$action" in
  output-volume)
    sink=$(omarchy audio output sink 2>/dev/null || true)
    sink=${sink%$'\n'}
    if [[ -n $sink ]]; then
      pactl set-sink-mute "$sink" 0
      pactl set-sink-volume "$sink" "${value}%"
    else
      wpctl set-volume @DEFAULT_AUDIO_SINK@ "${value}%"
      wpctl set-mute @DEFAULT_AUDIO_SINK@ 0
    fi
    ;;
  input-volume)
    wpctl set-volume @DEFAULT_AUDIO_SOURCE@ "${value}%"
    wpctl set-mute @DEFAULT_AUDIO_SOURCE@ 0
    ;;
  *)
    usage
    ;;
esac
