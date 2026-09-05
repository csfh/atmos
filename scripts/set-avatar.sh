#!/bin/bash
# Set or clear a login's face icon for SDDM and AccountsService.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
action=${1:-}
user=${2:-}
src=${3:-}

usage() {
  echo "Usage: set-avatar.sh set <user> <png-or-jpg> | clear <user>" >&2
  exit 1
}

[[ $action == set || $action == clear ]] || usage
[[ $user =~ ^[a-z_][a-z0-9_-]{0,31}$ ]] || usage
[[ $user != *- ]] || usage
[[ $user != *..* ]] || usage
[[ $user != root ]] || {
  echo "set-avatar.sh: will not change root's face" >&2
  exit 1
}

home=$(getent passwd "$user" | awk -F: '{ print $6; exit }')
[[ $home == /* && $home != *..* ]] || {
  echo "set-avatar.sh: no home for $user" >&2
  exit 1
}

install_face() {
  local file=$1
  mkdir -p "$home"
  cp -f -- "$file" "$home/.face"
  cp -f -- "$file" "$home/.face.icon"
  chmod 644 "$home/.face" "$home/.face.icon" 2>/dev/null || true
  if command -v setfacl >/dev/null 2>&1; then
    setfacl -m u:sddm:x "$home" 2>/dev/null || true
    setfacl -m u:sddm:r "$home/.face.icon" 2>/dev/null || true
  fi
}

install_accounts() {
  local file=$1
  "$ROOT/as-root.sh" /bin/bash -c '
    set -euo pipefail
    user=$1
    file=$2
    icon=/var/lib/AccountsService/icons/$user
    cfg=/var/lib/AccountsService/users/$user
    mkdir -p /var/lib/AccountsService/icons /var/lib/AccountsService/users
    cp -f -- "$file" "$icon"
    chmod 644 "$icon"
    if [[ -f $cfg ]] && grep -q "^Icon=" "$cfg"; then
      sed -i "s|^Icon=.*|Icon=$icon|" "$cfg"
    elif [[ -f $cfg ]]; then
      printf "Icon=%s\n" "$icon" >>"$cfg"
    else
      printf "[User]\nIcon=%s\n" "$icon" >"$cfg"
    fi
  ' bash "$user" "$file"
}

clear_face() {
  rm -f "$home/.face" "$home/.face.icon"
}

clear_accounts() {
  "$ROOT/as-root.sh" /bin/bash -c '
    set -euo pipefail
    user=$1
    rm -f "/var/lib/AccountsService/icons/$user"
    cfg=/var/lib/AccountsService/users/$user
    if [[ -f $cfg ]]; then
      grep -v "^Icon=" "$cfg" >"$cfg.tmp" || true
      if grep -qE "^[A-Za-z]" "$cfg.tmp"; then
        mv "$cfg.tmp" "$cfg"
      else
        rm -f "$cfg" "$cfg.tmp"
      fi
    fi
  ' bash "$user"
}

case $action in
  set)
    [[ -n $src ]] || usage
    [[ $src == /* ]] || usage
    [[ $src != *..* ]] || usage
    [[ -f $src ]] || {
      echo "set-avatar.sh: not a file: $src" >&2
      exit 1
    }

    # Privileged processes cannot read user-only FUSE mounts such as rclone
    # mounts without allow_other. Stage the selected file while we still have
    # the desktop user's access, then use that local copy for every write.
    staged=$(mktemp -p /tmp atmos-avatar.XXXXXX)
    trap 'rm -f -- "$staged"' EXIT
    cp -f -- "$src" "$staged"

    size=$(wc -c <"$staged")
    (( size > 0 && size <= 5000000 )) || {
      echo "set-avatar.sh: image must be 1 byte to 5 MB" >&2
      exit 1
    }
    mime=$(file -b --mime-type "$staged" 2>/dev/null || true)
    case $mime in
      image/png | image/jpeg | image/jpg) ;;
      *)
        echo "set-avatar.sh: need a PNG or JPEG" >&2
        exit 1
        ;;
    esac
    if [[ $user == "$(id -un)" ]]; then
      install_face "$staged"
    else
      "$ROOT/as-root.sh" /bin/bash -c '
        set -euo pipefail
        home=$1
        file=$2
        mkdir -p "$home"
        cp -f -- "$file" "$home/.face"
        cp -f -- "$file" "$home/.face.icon"
        chmod 644 "$home/.face" "$home/.face.icon"
        if command -v setfacl >/dev/null 2>&1; then
          setfacl -m u:sddm:x "$home" 2>/dev/null || true
          setfacl -m u:sddm:r "$home/.face.icon" 2>/dev/null || true
        fi
      ' bash "$home" "$staged"
    fi
    install_accounts "$staged"
    ;;
  clear)
    if [[ $user == "$(id -un)" ]]; then
      clear_face
    else
      "$ROOT/as-root.sh" rm -f "$home/.face" "$home/.face.icon"
    fi
    clear_accounts
    ;;
esac
