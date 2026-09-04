#!/bin/bash
# Snapper rollback of one config + snapshot id. Prefs already confirmed.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
config=${1:-}
id=${2:-}

[[ $config =~ ^[A-Za-z0-9_-]+$ ]] || {
  echo "Usage: rollback-snapshot.sh <config> <id>" >&2
  exit 1
}
[[ $id =~ ^[0-9]+$ ]] || {
  echo "Usage: rollback-snapshot.sh <config> <id>" >&2
  exit 1
}

"$ROOT/as-root.sh" snapper -c "$config" rollback "$id"
