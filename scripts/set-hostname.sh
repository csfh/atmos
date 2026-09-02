#!/bin/bash
# Set the static hostname. Prefs already confirmed the name.
set -euo pipefail

name=${1:-}

usage() {
  echo "Usage: set-hostname.sh <hostname>" >&2
  exit 1
}

[[ -n $name ]] || usage
[[ $name != -* ]] || usage
[[ ${#name} -le 253 ]] || usage
[[ $name =~ ^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)*$ ]] || usage

pkexec hostnamectl set-hostname "$name"
