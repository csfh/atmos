#!/bin/bash
# Set an XDG default application for a small set of file types.
set -euo pipefail

kind=${1:-}
desktop=${2:-}

usage() {
  echo "Usage: set-mime-default.sh <pdf|image|video> <desktop-id.desktop>" >&2
  exit 1
}

case $kind in
  pdf | image | video) ;;
  *) usage ;;
esac

[[ $desktop =~ ^[A-Za-z0-9._-]+\.desktop$ ]] || usage

found=""
for dir in \
  "${XDG_DATA_HOME:-$HOME/.local/share}/applications" \
  /usr/local/share/applications \
  /usr/share/applications
do
  if [[ -f $dir/$desktop ]]; then
    found=$dir/$desktop
    break
  fi
done
[[ -n $found ]] || {
  echo "set-mime-default.sh: desktop file not found: $desktop" >&2
  exit 1
}

case $kind in
  pdf)
    types=(application/pdf)
    ;;
  image)
    types=(image/png image/jpeg image/webp image/gif)
    ;;
  video)
    types=(video/mp4 video/webm video/x-matroska)
    ;;
esac

for mime in "${types[@]}"; do
  xdg-mime default "$desktop" "$mime"
done
