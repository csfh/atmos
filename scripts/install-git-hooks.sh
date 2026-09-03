#!/bin/bash
# Point this clone at the versioned hooks in .githooks/.
set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if ! git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
  echo "install-git-hooks.sh: not a git checkout" >&2
  exit 1
fi
git -C "$root" config core.hooksPath .githooks
