import QtQuick
import "../../components"
import "../../services"

PrefsRow {
  label: "Text size"
  description: "How large type is in the shell, GTK apps, and terminals. You can pick 9 to 20 pixels."
  hint: "omarchy display text size"
  keywords: ["scale", "size", "type", "font", "a11y"]

  Item {
    implicitWidth: sizeSlider.width + 10 + sizeSpin.width
    implicitHeight: Math.max(sizeSlider.implicitHeight, sizeSpin.implicitHeight)

    PrefsSlider {
      id: sizeSlider
      width: 162
      anchors.verticalCenter: parent.verticalCenter
      from: 9
      to: 20
      stepSize: 1
      value: Omarchy.textSize
      valueText: Omarchy.textSize + " px"
      showValue: false
      showTicks: false
      onChanged: function(value) {
        var next = Math.round(value)
        if (next !== Omarchy.textSize) Omarchy.setTextSize(next)
      }
    }

    PrefsSpinBox {
      id: sizeSpin
      anchors.left: sizeSlider.right
      anchors.leftMargin: 10
      anchors.verticalCenter: parent.verticalCenter
      from: 9
      to: 20
      value: Omarchy.textSize
      onChanged: function(value) {
        if (value !== Omarchy.textSize) Omarchy.setTextSize(value)
      }
    }
  }
}
