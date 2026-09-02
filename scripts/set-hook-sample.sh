#!/bin/bash
# Enable or disable a hook in ~/.config/omarchy/hooks by renaming .sample.
set -euo pipefail

usage() {
  echo "Usage: set-hook-sample.sh enable|disable <path>" >&2
  exit 1
}

if [[ $# -ne 2 ]]; then
  usage
fi
action=$1
src=$2
if [[ $action != enable && $action != disable ]]; then
  usage
fi
if [[ $src != /* || $src == *..* ]]; then
  echo "set-hook-sample.sh: path must be absolute and stay in the hooks folder" >&2
  exit 1
fi

root="$HOME/.config/omarchy/hooks/"
if [[ $src != "$root"* ]]; then
  echo "set-hook-sample.sh: path must be under ~/.config/omarchy/hooks" >&2
  exit 1
fi

rel=${src#"$root"}
if [[ $rel != *.d/* || $rel == */*/* ]]; then
  echo "set-hook-sample.sh: only scripts in a <type>.d folder can be renamed" >&2
  exit 1
fi

if [[ ! -f $src ]]; then
  echo "set-hook-sample.sh: file not found" >&2
  exit 1
fi

if [[ $action == enable ]]; then
  if [[ $src != *.sample ]]; then
    echo "set-hook-sample.sh: already enabled" >&2
    exit 1
  fi
  dest=${src%.sample}
else
  if [[ $src == *.sample ]]; then
    echo "set-hook-sample.sh: already a sample" >&2
    exit 1
  fi
  dest=$src.sample
fi

if [[ -e $dest ]]; then
  echo "set-hook-sample.sh: $dest already exists" >&2
  exit 1
fi

mv "$src" "$dest"
