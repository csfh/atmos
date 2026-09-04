import QtQuick
import "../../components"
import "../../services"

PrefsRow {
  stretchControl: true
  label: "Cursor size"
  description: "How large the pointer is."
  hint: "~/.config/hypr/looknfeel.lua · HYPRCURSOR_SIZE"
  keywords: ["cursor", "pointer", "size", "a11y"]

  PrefsSlider {
    width: parent.width
    from: 8
    to: 64
    stepSize: 2
    value: Omarchy.hyprCursorSize
    valueText: Omarchy.hyprCursorSize + " px"
    onChanged: function(value) {
      var next = Math.round(value)
      if (next !== Omarchy.hyprCursorSize)
        Omarchy.setHyprCursorSize(next)
    }
  }
}
