#!/bin/bash
# Add/remove local users and groups. Prefs already confirmed. Password on stdin
# for add-user and set-password (one line).
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
action=${1:-}

usage() {
  echo "Usage: manage-account.sh add-user <name> <full-name> <wheel:true|false>" >&2
  echo "       manage-account.sh remove-user <name>" >&2
  echo "       manage-account.sh set-password <name>" >&2
  echo "       manage-account.sh set-full-name <name> <full-name>" >&2
  echo "       manage-account.sh add-group <name>" >&2
  echo "       manage-account.sh remove-group <name>" >&2
  echo "       manage-account.sh set-member <group> <user> <on|off>" >&2
  exit 1
}

self=$(id -un)
reserved='^(root|bin|daemon|mail|ftp|http|nobody|dbus|systemd-coredump|systemd-network|systemd-oom|systemd-journal-remote|systemd-resolve|systemd-timesync|tss|uuidd|alpm|git|avahi|cups|cups-browsed|lp|_talkd|polkitd|rtkit|qemu|brltty|gluster|rpc|libvirt-qemu|pcscd|nvidia-persistenced|sddm)$'

ok_user() {
  local n=$1
  [[ $n =~ ^[a-z_][a-z0-9_-]{0,31}$ ]] || return 1
  [[ $n != *- ]] || return 1
  [[ $n != *..* ]] || return 1
  [[ $n =~ $reserved ]] && return 1
  return 0
}

ok_group() {
  local n=$1
  [[ $n =~ ^[a-z_][a-z0-9_-]{0,31}$ ]] || return 1
  [[ $n != *- ]] || return 1
  [[ $n != *..* ]] || return 1
  [[ $n == root || $n == nobody || $n == nogroup ]] && return 1
  return 0
}

read_password() {
  local pass
  IFS= read -r pass
  [[ -n $pass ]] || {
    echo "manage-account.sh: password cannot be empty" >&2
    exit 1
  }
  [[ $pass != *$'\n'* ]] || exit 1
  printf '%s' "$pass"
}

case $action in
  add-user)
    name=${2:-}
    full=${3:-}
    wheel=${4:-false}
    ok_user "$name" || usage
    [[ $wheel == true || $wheel == false ]] || usage
    [[ ${#full} -le 256 ]] || usage
    [[ $full != -* && $full != *:* && $full != *,* ]] || usage
    pass=$(read_password)
    printf '%s:%s\n' "$name" "$pass" | "$ROOT/as-root.sh" /bin/bash -c '
      set -euo pipefail
      name=$1
      full=$2
      wheel=$3
      getent passwd "$name" >/dev/null && { echo "manage-account.sh: $name already exists" >&2; exit 1; }
      useradd -m -s /bin/bash "$name"
      if [[ -n $full ]]; then
        chfn -f "$full" "$name"
      fi
      if [[ $wheel == true ]]; then
        usermod -aG wheel "$name"
      fi
      chpasswd
    ' bash "$name" "$full" "$wheel"
    ;;
  remove-user)
    name=${2:-}
    ok_user "$name" || usage
    [[ $name != "$self" ]] || {
      echo "manage-account.sh: will not remove the logged-in user" >&2
      exit 1
    }
    "$ROOT/as-root.sh" userdel -r "$name"
    ;;
  set-password)
    name=${2:-}
    ok_user "$name" || usage
    pass=$(read_password)
    printf '%s:%s\n' "$name" "$pass" | "$ROOT/as-root.sh" chpasswd
    ;;
  set-full-name)
    name=${2:-}
    full=${3-}
    ok_user "$name" || usage
    [[ ${#full} -le 256 ]] || usage
    [[ $full != -* && $full != *$'\n'* && $full != *$'\r'* ]] || usage
    [[ $full != *:* && $full != *,* ]] || usage
    "$ROOT/as-root.sh" chfn -f "$full" "$name"
    ;;
  add-group)
    name=${2:-}
    ok_group "$name" || usage
    "$ROOT/as-root.sh" groupadd "$name"
    ;;
  remove-group)
    name=${2:-}
    ok_group "$name" || usage
    [[ $name != wheel && $name != docker ]] || {
      echo "manage-account.sh: will not remove $name" >&2
      exit 1
    }
    gid=$(getent group "$name" | awk -F: '{ print $3; exit }')
    [[ $gid =~ ^[0-9]+$ ]] || usage
    (( gid >= 1000 && gid < 65534 )) || {
      echo "manage-account.sh: $name is a system group" >&2
      exit 1
    }
    "$ROOT/as-root.sh" groupdel "$name"
    ;;
  set-member)
    group=${2:-}
    name=${3:-}
    on=${4:-}
    ok_group "$group" || usage
    ok_user "$name" || usage
    [[ $on == on || $on == off ]] || usage
    if [[ $on == off && $group == wheel && $name == "$self" ]]; then
      echo "manage-account.sh: will not drop the logged-in user from wheel" >&2
      exit 1
    fi
    if [[ $on == on ]]; then
      "$ROOT/as-root.sh" usermod -aG "$group" "$name"
    else
      "$ROOT/as-root.sh" gpasswd -d "$name" "$group"
    fi
    ;;
  *)
    usage
    ;;
esac
