import QtQuick
import "../../components"
import "../../services"

SettingRow {
  label: "Animations"
  description: "Window open, close, and fade motion."
  hint: "~/.config/hypr/looknfeel.lua · animations.enabled"
  keywords: ["animation", "motion", "reduce", "a11y"]

  PrefsToggle {
    checked: Omarchy.hyprLook.animations
    onToggled: Omarchy.writeHyprLook({ animations: !Omarchy.hyprLook.animations })
  }
}
