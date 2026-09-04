#!/bin/bash
# Run a command as root. Prefer passwordless sudo (Atmos sudo mode).
# ATMOS_SUDO_ASK=1 reads the password from stdin via sudo -S so Atmos can
# prompt in-app instead of pkexec. Otherwise fall back to pkexec.
set -euo pipefail

if command -v sudo >/dev/null 2>&1 && sudo -n true >/dev/null 2>&1; then
  exec sudo "$@"
fi
if [[ ${ATMOS_SUDO_ASK:-0} == 1 ]] && command -v sudo >/dev/null 2>&1; then
  exec sudo -S -p '' "$@"
fi
exec pkexec "$@"
