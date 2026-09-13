#!/bin/bash
# Signal a process owned by this user. TERM or KILL only. No sudo.
set -euo pipefail

pid=${1:-}
sig=${2:-}

[[ $pid =~ ^[1-9][0-9]*$ ]] || {
  echo "signal-process.sh: pid must be a positive integer" >&2
  exit 1
}
if ((pid == 1)); then
  echo "signal-process.sh: refusing pid 1" >&2
  exit 1
fi

case $sig in
  TERM | SIGTERM | 15) sig=TERM ;;
  KILL | SIGKILL | 9) sig=KILL ;;
  *)
    echo "signal-process.sh: signal must be TERM or KILL" >&2
    exit 1
    ;;
esac

if ((pid == $$ || pid == PPID)); then
  echo "signal-process.sh: refusing to signal Atmos" >&2
  exit 1
fi
if [[ -n ${ATMOS_SELF_PID:-} && $pid -eq $ATMOS_SELF_PID ]]; then
  echo "signal-process.sh: refusing to signal Atmos" >&2
  exit 1
fi

SYS=${ATMOS_SYS_ROOT:-/}
status="$SYS/proc/$pid/status"
if [[ ! -r $status ]]; then
  echo "signal-process.sh: no such process $pid" >&2
  exit 1
fi

uid=$(awk '/^Uid:/{print $3; exit}' "$status")
me=${ATMOS_UID:-$(id -u)}
if [[ -z $uid || $uid != "$me" ]]; then
  echo "signal-process.sh: pid $pid is not owned by this user" >&2
  exit 1
fi

lock_dir=${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}
mkdir -p "$lock_dir"
exec {ATMOS_SIG_LOCK}>"$lock_dir/atmos-signal.lock"
flock "$ATMOS_SIG_LOCK"

if [[ ${ATMOS_SIGNAL_DRY:-0} == 1 ]]; then
  printf 'kill -s %s %s\n' "$sig" "$pid"
  exit 0
fi

kill -s "$sig" "$pid"
