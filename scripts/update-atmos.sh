#!/bin/bash
# Check or apply an Atmos update from the configured git channel.

set -euo pipefail

HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=atmos-xdg.sh
. "$HERE/atmos-xdg.sh"

ACTION=${1:-}

if [[ $ACTION != check && $ACTION != apply ]]; then
  echo "Usage: update-atmos.sh check|apply" >&2
  exit 2
fi

say() {
  printf '%s %s\n' "$1" "$2"
}

channel=$(atmos_channel)
say channel "$channel"

if ! command -v git >/dev/null 2>&1; then
  say status fetch-failed
  say summary "git is not installed."
  exit 1
fi

if ! atmos_sync_cache "$channel"; then
  say status fetch-failed
  say summary "Could not fetch the alpha branch."
  exit 1
fi

cache=$(atmos_git_cache)
remote_sha=$(git -C "$cache" rev-parse HEAD)
short_sha=$(git -C "$cache" rev-parse --short HEAD)
say remote "$remote_sha"
say short "$short_sha"

dest=$(atmos_data_home)
local_sha=""
if [[ -r $dest/REVISION ]]; then
  local_sha=$(<"$dest/REVISION")
  local_sha=${local_sha%%$'\n'*}
fi
if [[ $local_sha =~ ^[0-9a-f]{4,40}$ ]]; then
  say local "$local_sha"
else
  local_sha=""
fi

if [[ -n $local_sha && $local_sha == "$remote_sha" ]]; then
  say status current
  say summary "Atmos is up to date."
  exit 0
fi

say status behind
say summary "A newer Atmos is on alpha."

if [[ $ACTION != apply ]]; then
  exit 0
fi

atmos_stage "$cache" "$dest"
atmos_write_revision "$dest" "$remote_sha"
atmos_link_xdg "$dest"
say status current
say short "$(git -C "$cache" rev-parse --short HEAD)"
say summary "Updated. If the window looks the same, quit Atmos and open it again."
