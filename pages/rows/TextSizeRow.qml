import QtQuick
import "../../components"
import "../../services"

SettingRow {
  label: "Text size"
  description: "How large type is in the shell, GTK apps, and terminals. You can pick 9 to 20 pixels."
  hint: "omarchy display text size"
  keywords: ["scale", "size", "type", "font", "a11y"]

  PrefsSliderStepper {
    from: 9
    to: 20
    stepSize: 1
    value: Omarchy.textSize
    valueText: Omarchy.textSize + " px"
    onChanged: function(value) {
      var next = Math.round(value)
      if (next !== Omarchy.textSize) Omarchy.setTextSize(next)
    }
  }
}
