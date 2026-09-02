#!/bin/bash
# Change a LUKS passphrase. Reads three lines from stdin: device, current
# passphrase, new passphrase. Secrets never appear on argv.
set -euo pipefail

IFS= read -r device
IFS= read -r old_pass
IFS= read -r new_pass

if [[ -z $device || $device != /dev/* || $device == *..* ]]; then
  echo "luks-change-key.sh: device must be a /dev path" >&2
  exit 1
fi
if [[ ! $device =~ ^/dev/[A-Za-z0-9/_-]+$ ]]; then
  echo "luks-change-key.sh: device path has unexpected characters" >&2
  exit 1
fi
if [[ -z $old_pass || -z $new_pass ]]; then
  echo "luks-change-key.sh: passphrases cannot be empty" >&2
  exit 1
fi

found=false
while IFS= read -r path; do
  [[ $path == "$device" ]] && found=true
done < <(blkid -t TYPE=crypto_LUKS -o device 2>/dev/null || true)
if [[ $found != true ]]; then
  echo "luks-change-key.sh: $device is not a LUKS device" >&2
  exit 1
fi

oldf=$(mktemp /dev/shm/atmos-luks-XXXXXX)
newf=$(mktemp /dev/shm/atmos-luks-XXXXXX)
cleanup() {
  rm -f "$oldf" "$newf"
}
trap cleanup EXIT
chmod 600 "$oldf" "$newf"
printf '%s' "$old_pass" >"$oldf"
printf '%s' "$new_pass" >"$newf"
old_pass=""
new_pass=""

pkexec cryptsetup luksChangeKey --pbkdf argon2id --iter-time 2000 --key-file "$oldf" "$device" "$newf"
