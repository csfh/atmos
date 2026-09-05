import QtQuick
import "../../components"
import "../../services"

SettingRow {
  label: "Hide cursor while typing"
  description: "The pointer disappears when you start typing."
  hint: "~/.config/hypr/looknfeel.lua · cursor.hide_on_key_press"
  keywords: ["cursor", "pointer", "hide", "type"]

  PrefsToggle {
    checked: Omarchy.hyprCursorHideOnKey
    onToggled: Omarchy.setHyprCursorHideOnKey(!Omarchy.hyprCursorHideOnKey)
  }
}
