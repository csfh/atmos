# Agent notes

Atmos is a standalone Quickshell preferences app for Omarchy. Do not import `qs.Ui` or `qs.Commons` from `$OMARCHY_PATH/shell`.

## Run

- `./bin/atmos` — launch or focus
- `./tests/run` — parser tests plus a live snapshot check when `omarchy` is present

## Rules

- Mutations go through `omarchy` commands, `scripts/set-idle.sh` which sources `omarchy-shell-config`, or the Hyprland sentinel writers `scripts/set-hypr-look.sh`, `scripts/set-hypr-input.sh`, `scripts/set-hypr-autostart.sh`, `scripts/set-hypr-bindings.sh`, `scripts/set-hypr-windows.sh`, and `scripts/set-hyprsunset.sh`.
- Do not write a private prefs store.
- Hyprland drop-in is `~/.config/hypr/atmos.lua` required as `hypr.atmos` next to `hypr.omafetch`, before `default.hypr.toggles`. Sentinel blocks are `-- atmos:look|input|autostart|bindings|windows begin/end`.
- Theme colors come from `~/.local/state/omarchy/current/theme/{colors,shell}.toml` and `~/.config/omarchy/shell.toml`.
- Keep parsers in `services/*.js` so Node can test them without Quickshell.
- Do not import `qs.Ui`. Restyle Qt Quick Controls through `Prefs*` wrappers.
- Do not launch floating terminals for settings work. Long jobs use `Omarchy.runJob` or an in-page `Process`.
