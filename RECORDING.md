# Atmos recording script

Open Atmos, leave the window around 960×680, theme already on something you like. Speak the **Say** lines. Do the **Do** steps. Pause about a second after each control you touch so the viewer can read it.

Skip confirm dialogs that would change the machine unless that is the point of the take. Hover the button, then cancel.

Search lives in the top bar. Mention it once at the start, then ignore it unless a hub has nothing else to show.

---

## Open

**Say:** Atmos is the preferences window for Omarchy. Sidebar on the left is the list of hubs. Click one and the page on the right is that hub. Some hubs have extra pages behind a row with an arrow.

**Do:** Point at the sidebar. Click Appearance.

---

## Appearance

### Theme

**Say:** Current theme sits at the top. The grid is every theme on the machine. Click a thumbnail to switch. Extra actions on the current theme: refresh templates, open the theme switcher, or remove a theme you added yourself.

**Do:** Hover the current theme name. Scroll the grid slowly. Click a different theme, wait for it to apply, click back if you want the original.

### Additional themes

**Say:** Install a theme from a git URL. That clone lands in your extra themes folder.

**Do:** Hover the URL field and Install. Do not paste a URL unless you want that install on camera.

### Wallpaper and boot

**Say:** Background and boot screen are their own pages.

**Do:** Click Background.

#### Background (subpage)

**Say:** This is wallpaper. Pick a file, or leave the theme default.

**Do:** Hover Set background. Back.

**Do:** Click Boot screen.

#### Boot screen (subpage)

**Say:** Plymouth is the boot splash. You can set a logo, preview it, or reset. Direct boot is under Advanced.

**Do:** Hover the Plymouth controls. Scroll to Advanced. Back to Appearance.

### Text

**Say:** Font and size. These write through omarchy, same as the rest of appearance.

**Do:** Hover Font and Text size. Do not change size unless you will change it back in the same take.

### Display

**Say:** Night light warmth and schedule live here. That is hyprsunset.

**Do:** Hover the night light controls.

### Advanced

**Say:** Extra appearance knobs that most people leave alone.

**Do:** Expand or scroll Advanced. Next hub.

---

## Displays

### Each monitor

**Say:** Each connected monitor is a group. Resolution, scale, brightness.

**Do:** Hover those rows on the first monitor. If there are two, scroll to the second.

### Laptop

**Say:** Laptop screen on or off, mirror to the first external, touchpad, touchscreen, keyboard backlight.

**Do:** Hover Laptop. Do not toggle the internal panel off on camera.

---

## Hardware

This page is inventory. Most of it is read-only.

**Say:** Hardware is what the machine reported. Machine name, board, chipset, BIOS, Secure Boot, TPM. Processor, memory and DIMM slots, graphics. If this box is hybrid, Switch changes GPU mode. Then NPU, network adapters, audio, USB, battery, thermal sensors, virtualization.

**Do:** Scroll the whole page at a steady pace. Pause on Graphics. Hover Switch if it is there. Do not confirm a GPU mode change unless that is the take.

---

## Windows

### Shortcuts and rules

**Say:** Keybindings and window rules are separate pages.

**Do:** Click Keybindings.

#### Keybindings (subpage)

**Say:** Your overrides first. Below that, what is already bound. Add a binding if you want a chord the compositor does not ship.

**Do:** Scroll Your overrides, then What is bound. Hover Add. Back.

**Do:** Click Window rules.

#### Window rules (subpage)

**Say:** Rules pin a class to float, tile, or similar. Add one from the form at the bottom.

**Do:** Scroll Your rules. Hover Add a window rule. Back.

### Look

**Say:** Gaps, border, rounding, blur, shadow, opacity. This is the Hyprland look block Atmos writes.

**Do:** Hover a few look sliders. Change one gap by a tick and put it back if you want motion on camera.

### Behavior

**Say:** Tiling style, animations, cursor, tearing.

**Do:** Hover Behavior.

### Advanced

**Say:** The rest of the look and feel toggles.

**Do:** Scroll Advanced.

---

## Input

### Pointer

**Say:** Mouse speed and acceleration.

**Do:** Hover Pointer.

### Touchpad

**Say:** Natural scroll, tap, click-finger, disable-while-typing, the usual laptop pad options.

**Do:** Hover Touchpad.

### Keyboard

**Say:** Repeat delay and rate, numlock, follow-mouse, layout is on System if you need the keymap.

**Do:** Hover Keyboard.

### Advanced

**Say:** DPMS and extra pointer flags.

**Do:** Scroll Advanced.

---

## Accessibility

### Motion and type

**Say:** Reduce motion, text size, related type settings.

**Do:** Hover the rows. Leave reduce-motion off unless you want that look for the rest of the recording.

### Pointer

**Say:** Cursor size and hide-while-typing.

**Do:** Hover Pointer.

### Touch

**Say:** Touchscreen helpers.

**Do:** Hover Touch.

### Tools

**Say:** Screen reader and related tools.

**Do:** Hover Tools.

---

## Sound

### Output

**Say:** Volume, mute, output device, skip to the next sink.

**Do:** Nudge volume a little. Hover Switch on Next output.

### Input

**Say:** Capture volume, mute mic, input device.

**Do:** Hover Input. Do not leave the mic unmuted at full gain if you are recording audio on the same box.

### Tuning

**Say:** Speaker tuning if the machine has a profile.

**Do:** Hover Tuning.

### Recovery

**Say:** Restart audio if PipeWire is stuck.

**Do:** Hover Restart. Do not click it during a take that needs sound.

### Advanced

**Say:** Dictation install and remove.

**Do:** Hover Dictation.

---

## Capture

### Screenshot

**Say:** Screenshot bindings and region capture.

**Do:** Hover Screenshot.

### Recording

**Say:** Screen recording start and stop. You can mention you are in this page while the outer recorder is running.

**Do:** Hover Recording. Do not start a nested recording unless that is the gag.

### Read

**Say:** OCR and QR from the screen.

**Do:** Hover Read.

### Save locations

**Say:** Where pictures and videos land.

**Do:** Hover the paths.

---

## Disks

### Each drive

**Say:** Each block device: name, mounts, a disk speed test.

**Do:** Scroll the first disk. Hover Run on Speed test. Skip a live test unless you want a 20-second wait.

### Encryption

**Say:** LUKS devices. Change passphrase from here.

**Do:** Hover Change. Cancel if a dialog opens.

### Snapshots

**Say:** Snapper. Create a snapshot, roll back one from the list.

**Do:** Hover Create. Do not roll back.

### Hibernation

**Say:** Set up or remove hibernation.

**Do:** Hover Setup.

### Advanced

**Say:** How many snapshots to keep, timeline snapshots, weekly TRIM.

**Do:** Hover those rows.

### Swap

**Say:** Swap devices and zram if present.

**Do:** Scroll Swap.

---

## Bar

### Layout

**Say:** Bar position, transparency, show or hide the bar.

**Do:** Hover Position. A position change is visible on the Omarchy bar; put it back in the same take.

### Spacer

**Say:** Add a spacer, set width, remove it.

**Do:** Hover Add.

### Tray

**Say:** Hidden tray icons and pinned icons.

**Do:** Hover Show all and Unpin all. Do not clear pins unless you will restore them.

### Indicators

**Say:** Which status icons always show.

**Do:** Hover the indicator chips.

### Agents

**Say:** Coding-agent usage on the bar. Refresh interval, sync folder, snapshot file, device id.

**Do:** Hover Agents. Skip Browse unless you have a folder ready.

### Clock

**Say:** Clock format, alternate format, week start, birth year and life expectancy if you use those widgets.

**Do:** Hover Clock.

### Advanced

**Say:** Bar plugins.

**Do:** Scroll Advanced.

---

## Notifications

### Quiet

**Say:** Do not disturb.

**Do:** Hover the DND control. Leave it off so later toasts still show.

### Send

**Say:** Test notification, and related send options.

**Do:** Send a test if you want a toast in frame.

### Reminders

**Say:** Timed reminders.

**Do:** Hover Reminders.

---

## Defaults

### Applications

**Say:** Default browser, terminal, editor.

**Do:** Open one dropdown, close it.

### Agent

**Say:** Default coding agent.

**Do:** Hover Agent.

### Advanced

**Say:** PDF, images, video handlers.

**Do:** Hover Advanced.

---

## Applications

### Add

**Say:** Add a desktop app, a terminal UI, or a web app. That writes a launcher.

**Do:** Hover Add. Open the add form if you want, then cancel.

### Desktop

**Say:** Regular .desktop files in your local applications folder. Remove deletes that file.

**Do:** Scroll the list.

### Terminal

**Say:** Terminal UIs you installed as launchers.

**Do:** Scroll Terminal.

### Web

**Say:** Web apps.

**Do:** Scroll Web.

### Advanced

**Say:** Autostart. Remove an autostart entry from here.

**Do:** Hover Advanced.

---

## Software

**Say:** Install and remove packages Omarchy already knows: browsers, terminals, editors, services, gaming, development.

**Do:** Click each group heading in order. Hover one Install. Do not confirm a remove.

---

## Network

### Connection

**Say:** Current connection status.

**Do:** Pause on Status.

### Connectivity

**Say:** Wi-Fi, Bluetooth, and a speed test are their own pages.

**Do:** Click Wi-Fi.

#### Wi-Fi (subpage)

**Say:** Adapter on or off, known networks, nearby networks, connection details. Forget is on a saved network.

**Do:** Scroll Networks. Hover Forget on one row. Cancel. Back.

**Do:** Click Bluetooth.

#### Bluetooth (subpage)

**Say:** Adapter, paired devices, nearby scan. Forget a device from Paired.

**Do:** Scroll Paired and Nearby. Back.

**Do:** Click Speed test.

#### Speed test (subpage)

**Say:** Run a network speed test. Results show below.

**Do:** Hover Run. Skip the live run unless you want to wait. Back.

### DNS

**Say:** DNS provider, or custom nameservers.

**Do:** Hover DNS provider.

### Advanced

**Say:** Tailscale install and remove. LocalSend for clipboard, file, folder. Taildrop send and receive.

**Do:** Hover those rows. Do not start Tailscale install.

---

## Power

### Profile

**Say:** Power profile: performance, balanced, saver. Separate defaults for AC and battery.

**Do:** Hover the profile control. A live switch is fine if you switch back.

### Battery

**Say:** Battery status and whether the percentage shows in the bar.

**Do:** Hover Battery. Click Show if you want the status line in frame.

---

## Idle and lock

### Timings

**Say:** Screensaver delay and lock delay.

**Do:** Hover the timing rows.

### Behavior

**Say:** Stay awake, allow screensaver, allow suspend.

**Do:** Hover Stay awake. Leave it off unless you need the machine awake for a long take.

### Branding

**Say:** Screensaver logo: pick an image, edit, or reset.

**Do:** Hover Image, Edit, Reset.

### Advanced

**Say:** Lid close on a laptop.

**Do:** Hover Lid close.

---

## Security

### Login

**Say:** Fingerprint and security key setup. Those open confirmations.

**Do:** Hover Set up fingerprint and Set up a security key. Do not walk a full enroll unless that is the take.

### Remote

**Say:** SSH on or off.

**Do:** Hover Remote.

### Advanced

**Say:** Passwordless sudo and sudoless Docker. Both are explicit, both confirm.

**Do:** Hover those rows. Cancel any dialog.

---

## Hooks

### Install

**Say:** Install a hook script for events like theme-set or post-update.

**Do:** Hover Install.

### Each installed hook

**Say:** Each hook is a group. You can remove it.

**Do:** Scroll one installed hook if any exist.

---

## System

### Machine

**Say:** Hostname and your account full name.

**Do:** Hover Machine. Do not rename the host on camera.

### Keyboard

**Say:** Keyboard layout.

**Do:** Hover Keyboard.

### Language

**Say:** Locale.

**Do:** Hover Language.

### Date and time

**Say:** Timezone and NTP.

**Do:** Hover Date and time.

### Updates

**Say:** Omarchy update, firmware, orphans, package cache prune.

**Do:** Hover the update actions. Skip running Update Omarchy in a short take.

### Atmos

**Say:** Channel, check, and update for Atmos itself. Alpha is the branch install.sh follows.

**Do:** Hover Atmos. Do not switch channel mid-recording unless you will explain it.

### Advanced

**Say:** Restore Hyprland defaults, restore shell defaults, other recovery.

**Do:** Hover Restore. Never confirm restore in a demo take.

### Printers

**Say:** CUPS printers.

**Do:** Hover Printers.

### Packages

**Say:** Parallel downloads and related pacman knobs.

**Do:** Hover Packages.

### Branding

**Say:** fastfetch logo and related branding.

**Do:** Hover Branding.

### Weather

**Say:** Location, units, refresh. Used by the weather indicator.

**Do:** Hover Weather.

### Diagnostics

**Say:** Crash capture and related diagnostics.

**Do:** Hover Diagnostics.

---

## Search (optional closer)

**Say:** Type in the search field and Atmos lists matching hubs. Click a hit to jump.

**Do:** Type `dns`, click Network, clear search.

---

## Suggested order and length

| Take | Hubs | Rough length |
|------|------|----------------|
| 1 | Open, Appearance | 2–3 min |
| 2 | Displays, Hardware | 1–2 min |
| 3 | Windows (plus bindings and rules) | 2 min |
| 4 | Input, Accessibility | 1–2 min |
| 5 | Sound, Capture | 1–2 min |
| 6 | Disks | 1–2 min |
| 7 | Bar, Notifications | 2 min |
| 8 | Defaults, Applications, Software | 2 min |
| 9 | Network (plus wifi, bluetooth, speedtest) | 2 min |
| 10 | Power, Idle, Security, Hooks | 2 min |
| 11 | System, Search | 2–3 min |

Total spoken walkthrough is about 20 minutes if you do every section once and skip live installs, speed tests, and restores.
