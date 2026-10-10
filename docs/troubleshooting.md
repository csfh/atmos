# Troubleshooting

**`atmos` is not found.** The wrapper lives in `~/.local/bin`. Add that directory to `PATH`, or run `~/.local/share/atmos/bin/atmos`.

**Hyprland config-reload warnings.** After a look, input, monitor, or workspace write, Atmos runs `hyprctl reload`. A failed reload does not undo the write. If Hyprland already has config errors, those writers print `hyprctl configerrors` and the apply can look failed even though the file changed. **System → Diagnostics** lists the errors. A dirty config still runs; the errors are why a bind or monitor rule did not apply.

**`hypr.atmos` is missing / the window does not tile.** Install writes the drop-in and the require. Diagnostics reports when the require is absent. Re-run `install.sh`, or restore Hyprland defaults from System (the Atmos require is written back).

**Display or input change snapped back.** Mode, scale, rotation, refresh, keyboard layout, and turning the touchpad off revert after 12 seconds unless you press Keep. Escape reverts too. Quitting during those 12 seconds leaves the new value.

**Install says a tool is missing.** `install.sh` needs `git`, `python3`, `cargo`, and `quickshell`. `atmos mcp` also needs `node`.

**Need a report.** **System → Diagnostics → Copy report** puts an Atmos summary plus `omarchy-debug --no-sudo --print` on the clipboard.
