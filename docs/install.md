# Install and update

Atmos installs two ways. Pick one; they do not manage each other.

| | Install script | Omarchy package |
| --- | --- | --- |
| Command | `curl -fsSL https://raw.githubusercontent.com/csfh/atmos/stable/install.sh \| bash` | `sudo pacman -S omarchy-atmos` then `atmos --setup` |
| Lands in | `~/.local/share/atmos`, `~/.local/bin/atmos`, a desktop file | `/usr/lib/atmos` |
| Hyprland drop-in | written by the script | opt-in with `atmos --setup` |
| Updates | **System → Atmos → Update**, or re-run the command | `omarchy update` |

Both need an [Omarchy](https://omarchy.org) system (Hyprland) and `~/.local/bin` on `PATH`. The script also needs `git`, `python3`, `cargo`, and `quickshell`, and checks for them. From a clone, `./install.sh` does the same thing as the curl command.

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
