#!/bin/bash
# Enable or disable NTP. Prefs already confirmed true/false.
set -euo pipefail

on=${1:-}

usage() {
  echo "Usage: set-ntp.sh true|false" >&2
  exit 1
}

[[ $on == true || $on == false ]] || usage

pkexec timedatectl set-ntp "$on"
