import QtQuick
import "../../components"
import "../../services"

PrefsRow {
  label: "Animations"
  description: "Window open, close, and fade motion."
  hint: "~/.config/hypr/looknfeel.lua · animations.enabled"
  keywords: ["animation", "motion", "reduce", "a11y"]

  PrefsToggle {
    checked: Omarchy.hyprAnimations
    onToggled: Omarchy.setHyprAnimations(!Omarchy.hyprAnimations)
  }
}
