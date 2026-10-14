# Install and update

Atmos installs two ways. Pick one; they do not manage each other.

| | Install script | Omarchy package |
| --- | --- | --- |
| Command | `curl -fsSL https://raw.githubusercontent.com/csfh/atmos/stable/install.sh \| bash` | `sudo pacman -S omarchy-atmos` then `atmos --setup` |
| Lands in | `~/.local/share/atmos`, `~/.local/bin/atmos`, a desktop file | `/usr/lib/atmos` |
| Hyprland drop-in | written by the script | opt-in with `atmos --setup` |
| Updates | **System → Atmos → Update**, or re-run the command | `omarchy update` |

Both need an [Omarchy](https://omarchy.org) system (Hyprland) and `~/.local/bin` on `PATH`. The script also needs `git`, `cargo`, and `quickshell`, and checks for them. From a clone, `./install.sh` does the same thing as the curl command.

## Apple Silicon (M1, M2, and later)

Atmos has no architecture-specific code: the window is QML and `ratmos` is plain Rust. On Apple Silicon you run [Asahi Linux](https://asahilinux.org) with Omarchy and Hyprland on top, and `atmos` and `ratmos` come up the same as on x86.

- **Use the install script.** It builds `ratmos` from source for the machine it runs on, so you get a native aarch64 binary. Nothing is downloaded prebuilt, and the release tarball is source only. The Rust build is the slow step of the install.
- **Install the tools first**: `git`, `quickshell`, and Rust (`sudo pacman -S rust`). The script stops and names the first one it cannot find. Whether `quickshell` is packaged for your aarch64 setup depends on your repos.
- **The Omarchy package** (`omarchy-atmos`) works on aarch64 only if the package repo you use carries an aarch64 build. That build is made outside this repository, so check your repo before you rely on it. If it is missing, use the install script.
- **Check the build** with `file ~/.local/share/atmos/bin/ratmos`, which should say `ARM aarch64`, then run `atmos`.

What Atmos reads on Apple hardware, and what to expect:

| Page | Apple Silicon |
| --- | --- |
| Power, charge limit | Found on `macsmc-battery` (any supply with `charge_control_end_threshold`). The Asahi driver clamps the value: on one backend any limit of 95 or lower becomes 80, on the other it accepts 10 to 99. The slider may read back a different number than you picked. |
| Hardware, machine name | Apple machines have no DMI, so the name comes from `/proc/device-tree/model`. |
| Monitor, GPU | The Asahi render GPU (`asahi` driver) shows as an integrated "Apple" GPU. An extra unnamed GPU row for the display controller is possible. |
| CPU name and temperature | Likely blank. aarch64 `/proc/cpuinfo` has no `model name`, and the SMC sensor names are not mapped yet. |
| Direct boot | Off. Asahi boots through m1n1 and U-Boot, not `efibootmgr`. |
| NPU, TPM, Secure Boot, AMD P-state | Absent or hidden. |

This support follows the Asahi kernel sources, and none of it has run on Apple hardware. If a page looks wrong on your Mac, open an issue and include the output of `atmos` diagnostics (**System → Diagnostics**) and:

```bash
ls /sys/class/{power_supply,hwmon,backlight,leds}
readlink /sys/class/drm/card*/device/driver
cat /sys/class/hwmon/*/name
tr -d '\0' </proc/device-tree/model
lscpu
```

## Install script

The script copies the launcher into `~/.local/bin`, the app into `~/.local/share/atmos`, a desktop file, and the Hyprland drop-in. Then run `atmos`, or open it from the app launcher.

## Omarchy package

The package carries a `PACKAGED` marker in the app root. Atmos reads it, leaves updates to pacman, and writes nothing under the app root: **System → Atmos** shows the package version and hides Channel and Update.

Hyprland integration (the window seed and the `hypr.atmos` requires) is per user and opt-in:

```bash
atmos --setup          # add it
atmos --setup status   # prints on or off
atmos --setup off      # remove it
```

Run `atmos --setup off` before you remove the package; `packaging/atmos.hook` reminds pacman users.

If you installed with `install.sh` first, remove `~/.local/share/atmos`, `~/.local/bin/atmos` and `~/.local/share/applications/atmos.desktop` so they do not shadow the package.

## Update

**System → Atmos** Check / Update follows the **stable** git branch. Re-running the install command does the same fetch-and-stage. If the window looks the same afterward, quit Atmos and open it again. Update also repairs an install that is missing the launcher, backend, or shell.

The channel lives in `~/.config/atmos/channel`. Only `stable` exists; an old `alpha` reads as `stable`.

## Uninstall

Run `atmos --setup off` first, while Atmos is still installed, to remove the Hyprland requires. For the script install, then delete `~/.local/share/atmos`, `~/.local/bin/atmos` and `~/.local/share/applications/atmos.desktop`; for the package, `sudo pacman -R omarchy-atmos`. **System → Atmos → Reset**, before you uninstall, strips the Atmos-managed Hyprland overrides and leaves theme, wallpaper and `shell.json` alone.
