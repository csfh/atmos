#!/bin/bash
# Hold an exclusive flock for the life of the parent Atmos process.
#
# Quickshell only refuses a second instance of the same -p path, so an
# install and a checkout can both write ~/.config. This lock is on the
# app, not the path. flock is kernel-released; a PID file can go stale.
#
# Contract:
#   - Exit 1 immediately if another holder already has the lock.
#   - Hold the lock on this process, then exec tail. Do not use flock -c
#     or flock FILE COMMAND: both fork, and the lock fd is inherited by
#     a grandchild that can outlive Atmos (reparented to init).
#   - Die with the parent: tail --pid=$parent exits when Atmos disappears,
#     including SIGKILL. QProcess killing this pid also releases the lock.
#   - Exit 0 (do not hold) if flock is missing. Refusing to start over a
#     missing utility is worse than the race this prevents.
#
# Lock path: $XDG_RUNTIME_DIR/atmos-$UID.lock, or /tmp/atmos-$UID.lock
# when XDG_RUNTIME_DIR is unset. Override with ATMOS_INSTANCE_LOCK.

set -euo pipefail

if ! command -v flock >/dev/null 2>&1; then
  exit 0
fi

parent=$PPID
uid=$(id -u)
dir=${XDG_RUNTIME_DIR:-/tmp}
lock=${ATMOS_INSTANCE_LOCK:-$dir/atmos-$uid.lock}

# Open the lock file on this process, then exec tail so the same pid
# keeps the fd. GNU tail --pid exits when the captured parent dies.
exec 9>"$lock"
flock -n 9 || exit 1
exec tail --pid="$parent" --sleep-interval=0.1 -f /dev/null
