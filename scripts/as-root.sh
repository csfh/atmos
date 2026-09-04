#!/bin/bash
# Run a command as root. Prefer passwordless sudo (Atmos sudo mode);
# otherwise fall back to pkexec so polkit can prompt.
set -euo pipefail

if command -v sudo >/dev/null 2>&1 && sudo -n true >/dev/null 2>&1; then
  exec sudo "$@"
fi
exec pkexec "$@"
