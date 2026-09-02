#!/bin/bash
# List wallpaper images under an Omarchy theme directory.
set -euo pipefail

dir=${1:-}
if [[ -z $dir ]]; then
  dir=$(omarchy theme dir 2>/dev/null || true)
fi
[[ -n $dir && -d $dir ]] || exit 0
[[ $dir == /* && $dir != *..* ]] || exit 0

find "$dir" -maxdepth 3 -type f \( \
  -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' -o \
  -iname '*.gif' -o -iname '*.webp' -o -iname '*.bmp' \
\) 2>/dev/null | head -n 24
