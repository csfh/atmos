# Configuration

Atmos does not write a private prefs store. It reads and writes Omarchy and Hyprland files under `~/.config`.

- Hyprland drop-in: `~/.config/hypr/atmos.lua`, required as `hypr.atmos` from `hyprland.lua` next to `hypr.omafetch`, before `default.hypr.toggles`. `hypr.atmos_layout` wraps dwindle `layoutmsg` so scrolling workspaces do not throw.
- Sentinel blocks: `-- atmos:look|input|autostart|bindings|windows|workspaces begin/end` in the matching Lua files, and `-- atmos:monitors begin/end` in `monitors.lua`.
- Theme colors: `~/.local/state/omarchy/current/theme/{colors,shell}.toml` and `~/.config/omarchy/shell.toml`.
- Install channel: `~/.config/atmos/channel` (only `stable` exists; an old `alpha` reads as `stable`).
- **System → Atmos → Reset** strips Atmos-managed Hypr overrides. Theme, wallpaper, and `shell.json` stay as they are.
