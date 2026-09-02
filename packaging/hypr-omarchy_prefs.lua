-- Float and center the standalone Omarchy preferences window.
-- Install as ~/.config/hypr/omarchy_prefs.lua and require("hypr.omarchy_prefs")
-- from hyprland.lua next to require("hypr.omafetch"), before toggles.
o.window("org.omarchy.prefs", { float = true })
o.window("org.omarchy.prefs", { center = true })
o.window("org.omarchy.prefs", { size = { 960, 680 } })
