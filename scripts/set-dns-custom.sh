#!/bin/bash
# Apply custom DNS servers through omarchy dns Custom (stdin).
set -euo pipefail

servers=${1:-}

usage() {
  echo "Usage: set-dns-custom.sh <ipv4-or-ipv6> [more...]" >&2
  exit 1
}

[[ -n $servers ]] || usage

# Remaining args are extra servers.
if (( $# > 1 )); then
  shift
  servers="$servers $*"
fi

if [[ $servers == -* ]]; then
  echo "set-dns-custom.sh: DNS servers cannot start with -" >&2
  exit 1
fi

normalized=$(printf '%s' "$servers" | tr ',\t' ' ' | xargs)
[[ -n $normalized ]] || usage

for token in $normalized; do
  if [[ $token == *.* ]]; then
    [[ $token =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]] || {
      echo "set-dns-custom.sh: not an IPv4 address: $token" >&2
      exit 1
    }
  elif [[ $token == *:* ]]; then
    [[ $token =~ ^[0-9a-fA-F:]+$ ]] || {
      echo "set-dns-custom.sh: not an IPv6 address: $token" >&2
      exit 1
    }
  else
    echo "set-dns-custom.sh: not a DNS address: $token" >&2
    exit 1
  fi
done

printf '%s\n' "$normalized" | omarchy dns Custom
