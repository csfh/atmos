import QtQuick
import "../../components"
import "../../services"

SettingRow {
  label: "Cursor size"
  description: "How large the pointer is."
  hint: "~/.config/hypr/looknfeel.lua · HYPRCURSOR_SIZE"
  keywords: ["cursor", "pointer", "size", "a11y"]

  PrefsSliderStepper {
    from: 8
    to: 64
    stepSize: 2
    value: Omarchy.hyprLook.cursorSize
    valueText: Omarchy.hyprLook.cursorSize + " px"
    onChanged: function(value) {
      var next = Math.round(value)
      if (next !== Omarchy.hyprLook.cursorSize)
        Omarchy.writeHyprLook({ cursorSize: next })
    }
  }
}
