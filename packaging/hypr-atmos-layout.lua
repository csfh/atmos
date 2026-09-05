-- Skip dwindle layout messages on scrolling workspaces.
-- Omarchy binds SUPER+J to hl.dsp.layout("togglesplit"). Hyprland throws
-- "no such layoutmsg for scrolling" when that workspace uses scrolling.
-- Load this before default.hypr.omarchy so the wrap is in place when
-- those binds are constructed.
--
-- Install as ~/.config/hypr/atmos_layout.lua and
-- require("hypr.atmos_layout") from hyprland.lua before Omarchy defaults.

if not (hl and hl.dsp and type(hl.dsp.layout) == "function") then
  return
end

local orig = hl.dsp.layout
local dwindle_only = {
  togglesplit = true,
  swapsplit = true,
  preselect = true,
}

hl.dsp.layout = function(msg)
  local inner = orig(msg)
  return function(...)
    local ws = hl.get_active_workspace()
    local name = tostring(msg or ""):match("^%S+") or ""
    if ws and ws.tiled_layout == "scrolling" and dwindle_only[name] then
      return
    end
    if type(inner) == "function" then
      return inner(...)
    end
    return hl.dispatch(inner, ...)
  end
end
