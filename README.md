# Omarchy Preferences

A standalone [Quickshell](https://quickshell.org) app for Omarchy settings. It is its own process, not a plugin inside `omarchy-shell`.

The window follows the active Omarchy theme from `~/.local/state/omarchy/current/theme`. Changes go through existing `omarchy` commands. `~/.config` stays the source of truth.

## Run

```bash
./bin/omarchy-prefs
```

Open a page directly:

```bash
./bin/omarchy-prefs appearance
./bin/omarchy-prefs appearance/background
./bin/omarchy-prefs appearance/boot
./bin/omarchy-prefs display
./bin/omarchy-prefs windows
./bin/omarchy-prefs windows/bindings
./bin/omarchy-prefs windows/rules
./bin/omarchy-prefs input
./bin/omarchy-prefs sound
./bin/omarchy-prefs disks
./bin/omarchy-prefs bar
./bin/omarchy-prefs defaults
./bin/omarchy-prefs network
./bin/omarchy-prefs power
./bin/omarchy-prefs idle
./bin/omarchy-prefs security
./bin/omarchy-prefs system
```

A second launch focuses the running window instead of starting another engine.

Install the desktop entry and menu rows (optional):

```bash
mkdir -p ~/.local/share/applications ~/.local/bin ~/.config/omarchy/extensions ~/.config/hypr
ln -sf "$PWD/bin/omarchy-prefs" ~/.local/bin/omarchy-prefs
cp packaging/omarchy-prefs.desktop ~/.local/share/applications/
cp packaging/omarchy-menu.jsonc ~/.config/omarchy/extensions/omarchy-menu.jsonc
cp packaging/hypr-omarchy_prefs.lua ~/.config/hypr/omarchy_prefs.lua
```

Then add `require("hypr.omarchy_prefs")` to `~/.config/hypr/hyprland.lua` next to `require("hypr.omafetch")`, before `require("default.hypr.toggles")`. Lua module names cannot use a hyphen.

## First slice

- Appearance: theme and extra themes on the main page, background, font, text size, night light, boot screen, extra themes from git (Add opens a modal), wallpaper and boot logo through a file picker, Plymouth reset after a confirm dialog
- Displays: per-monitor resolution, focused scale via `omarchy hyprland monitor scaling`, brightness via `omarchy brightness display`, laptop panel via `omarchy hyprland monitor internal`, mirror via `omarchy hyprland monitor internal mirror`, touchpad via `omarchy toggle touchpad`, touchscreen via `omarchy toggle touchscreen`, keyboard backlight via `omarchy brightness keyboard`, hybrid GPU via `omarchy toggle hybrid gpu`
- Windows: gaps, borders, rounding, blur, shadow, and tiling in a managed block of `~/.config/hypr/looknfeel.lua`; keybinding overrides in a managed block of `~/.config/hypr/bindings.lua`; window rules in a managed block of `~/.config/hypr/omarchy_prefs.lua`; tight windows and square aspect via `omarchy hyprland toggle`; workspace layout and focused-window toggles via `omarchy hyprland`
- Input: pointer, touchpad, and repeat feel in a managed block of `~/.config/hypr/input.lua`; optional Hyprland layout list and a three-finger workspace swipe
- Sound: output volume and mute via `omarchy audio output volume`, default sink via `omarchy audio output set default`, input mute via `omarchy audio input mute`, default source via `omarchy audio input set default`, speaker tuning via `omarchy audio tuning`, restart via `omarchy restart audio`, Voxtype via `omarchy voxtype`
- Disks: inventory via `lsblk` and `omarchy drive info`, usage bars via `df`, LUKS passphrase form, Snapper list and rollback, hibernation setup/remove after confirm, in-window disk speedtest, Snapper keep-count and timeline, weekly TRIM
- Bar: position, transparency, visibility via `omarchy toggle bar`, clock format via `omarchy bar set`, alternate clock format via `omarchy bar set`, calendar week start via `omarchy bar set`, clock birth year and life expectancy via `omarchy bar set`, always-show indicators and which icons to show via `omarchy bar set`, agent usage refresh, sync, sync folder, snapshot file, and device id via `omarchy bar set`, add a spacer via `omarchy bar put`, spacer width via `omarchy bar set`, hidden and pinned tray icons via `omarchy bar set`, shell plugins via `omarchy plugin enable|disable`
- Defaults: browser, terminal, editor, agent, plus PDF, image, and video MIME defaults
- Applications: add desktop launchers, TUIs via `omarchy tui install`, and web apps via `omarchy webapp install`; remove via the matching `omarchy` remove commands
- Network: Connectivity links for Wi-Fi, Bluetooth, and speed test; custom DNS field; adapter power on the Wi-Fi and Bluetooth pages; Tailscale install/remove
- Power: profile via `omarchy powerprofiles set`, AC profile via `omarchy powerprofiles set ac`, battery profile via `omarchy powerprofiles set battery`, battery status via `omarchy notification battery`, battery percentage via `omarchy bar set`
- Idle: screensaver and lock timings in `shell.json`, stay awake via `omarchy toggle idle`, screensaver availability via `omarchy toggle screensaver`, suspend in the system menu via `omarchy toggle suspend`, screensaver logo via `omarchy branding screensaver`
- Security: fingerprint, FIDO2, SSH, timed passwordless sudo, and sudoless Docker through `omarchy setup|remove security` and `omarchy sudo passwordless`
- System: hostname via `hostnamectl set-hostname`, full name via `chfn --full-name`, keyboard layout via `localectl set-x11-keymap`, locale via `localectl set-locale`, timezone via `timedatectl set-timezone`, NTP via `timedatectl set-ntp`, pacman ParallelDownloads, printers via system-config-printer and the CUPS admin page, crash capture, weather, About logo, Omarchy channel and update, restore via `omarchy refresh hyprland` and `omarchy refresh shell`

## Layout

```
shell.qml           window, sidebar, search, IPC
services/           theme follow + omarchy command runner
components/         page, group, row, toggle, select, slider, chips, spin box, confirm, progress
pages/              one file per sidebar page
scripts/snapshot.sh live values as JSON
scripts/set-idle.sh writes idle timings via omarchy-shell-config
scripts/set-wifi-connection.sh  nmcli radio and saved Wi-Fi connections
scripts/set-audio.sh            absolute PipeWire volume percents
scripts/set-dns-custom.sh       custom resolvers through omarchy dns Custom
scripts/luks-change-key.sh      LUKS passphrase change, secrets on stdin
scripts/disk-inventory.py       lsblk + df + omarchy drive info + snapper list
scripts/set-timezone.sh         system timezone through timedatectl (polkit)
scripts/set-ntp.sh              NTP on/off through timedatectl (polkit)
scripts/set-locale.sh           system LANG through localectl (polkit)
scripts/set-parallel-downloads.sh  pacman ParallelDownloads through pkexec
scripts/set-hostname.sh         static hostname through hostnamectl (polkit)
scripts/set-full-name.sh        account full name through chfn (polkit)
scripts/set-keyboard-layout.sh  XKB layout through localectl (polkit)
scripts/set-hypr-look.sh        managed looknfeel.lua sentinel, then hyprctl reload
scripts/set-hypr-input.sh       managed input.lua sentinel, then hyprctl reload
scripts/set-hypr-autostart.sh   managed autostart.lua sentinel, then hyprctl reload
scripts/set-hypr-bindings.sh    managed bindings.lua sentinel, then hyprctl reload
scripts/set-hypr-windows.sh     managed omarchy_prefs.lua window rules, then hyprctl reload
scripts/refresh-hyprland.sh     omarchy refresh hyprland, then restore the prefs require
scripts/set-nightlight-temp.sh  hyprsunset Kelvin
scripts/set-snapper-policy.sh   Snapper NUMBER_LIMIT and timeline timers
scripts/set-fstrim.sh           weekly fstrim.timer
scripts/set-mime-default.sh     xdg-mime defaults for pdf/image/video
scripts/set-sshd.sh             stop sshd without deleting authorized_keys
```

This tree does not import `$OMARCHY_PATH/shell` QML.

## Tests

```bash
./tests/run
```

## License

This project is licensed under the [MIT License](LICENSE), Copyright (c) 2026 Christoffer Hallas.

Contributions are subject to the [Contributor License Agreement](CLA.md). See [CONTRIBUTING.md](CONTRIBUTING.md).

