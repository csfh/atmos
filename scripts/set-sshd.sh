#!/bin/bash
# Stop the OpenSSH server without deleting authorized_keys.
set -euo pipefail

action=${1:-}

usage() {
  echo "Usage: set-sshd.sh disable" >&2
  exit 1
}

[[ $action == disable ]] || usage

pkexec /bin/bash -c '
  set -euo pipefail
  systemctl disable --now sshd.service 2>/dev/null || true
  if command -v ufw >/dev/null 2>&1; then
    ufw --force delete limit 22/tcp >/dev/null 2>&1 || true
    ufw reload >/dev/null 2>&1 || true
  fi
'
