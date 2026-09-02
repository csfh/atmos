#!/bin/bash
# Write a one-line hook script and install it with omarchy hook install.
set -euo pipefail

TYPES='theme-set font-set post-boot post-update pre-refresh-pacman battery-low'

usage() {
  echo "Usage: create-hook.sh <type> <name> <command>" >&2
  exit 1
}

if [[ $# -ne 3 ]]; then
  usage
fi
if [[ $1 == -* || $2 == -* ]]; then
  usage
fi

type=$1
name=$2
command=$3

ok_type=0
for known in $TYPES; do
  if [[ $type == "$known" ]]; then
    ok_type=1
    break
  fi
done
if (( !ok_type )) && [[ $type =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
  ok_type=1
fi
if (( !ok_type )); then
  echo "create-hook.sh: unknown hook type" >&2
  exit 1
fi

if [[ $name == *.sample || $name == */* || $name == *..* ]]; then
  echo "create-hook.sh: bad hook name" >&2
  exit 1
fi
if [[ ! $name =~ ^[A-Za-z0-9][A-Za-z0-9._+-]*$ ]]; then
  echo "create-hook.sh: bad hook name" >&2
  exit 1
fi
if [[ ! $name =~ \.[A-Za-z0-9]+$ ]]; then
  name=$name.sh
fi

if [[ -z $command || ${#command} -gt 512 || $command == *$'\n'* || $command == *$'\r'* ]]; then
  echo "create-hook.sh: bad command" >&2
  exit 1
fi

case $type in
  theme-set) comment='# $1 is the snake-cased theme name that was just set.' ;;
  font-set) comment='# $1 is the snake-cased font name that was just set.' ;;
  battery-low) comment='# $1 is the current battery percentage.' ;;
  *) comment='# This event has no extra argument.' ;;
esac

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
printf '#!/bin/bash\n%s\n%s\n' "$comment" "$command" >"$tmp/$name"

omarchy hook install "$type" "$tmp/$name"
