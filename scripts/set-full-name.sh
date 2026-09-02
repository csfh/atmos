#!/bin/bash
# Set the current user's GECOS full name via chfn.
set -euo pipefail

name=${1-}
user=$(id -un)

usage() {
  echo "Usage: set-full-name.sh <full-name>" >&2
  exit 1
}

[[ $name != -* ]] || usage
[[ ${#name} -le 256 ]] || usage
[[ $name != *$'\n'* && $name != *$'\r'* ]] || usage
[[ $name != *:* && $name != *,* ]] || usage
[[ $user =~ ^[a-z_][a-z0-9_-]*$ ]] || usage

pkexec chfn -f "$name" "$user"
