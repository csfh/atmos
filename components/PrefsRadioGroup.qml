import QtQuick
import QtQuick.Controls
import "../services"
import "../services/RichUi.js" as RichUi

Flow {
  id: root

  property var options: []
  property string value: ""
  property bool enabled: true
  property bool wrap: false
  property string displayLabel: ""

  signal changed(string value)

  width: wrap && parent ? parent.width : implicitWidth
  spacing: wrap ? Theme.pad : 6
  flow: wrap ? Flow.LeftToRight : Flow.TopToBottom
  opacity: enabled ? 1 : 0.45

  Accessible.role: Accessible.List
  Accessible.name: displayLabel

  function optionValue(item) { return RichUi.optionValue(item) }
  function optionLabel(item) { return RichUi.optionLabel(item) }

  function refreshDisplayLabel() {
    var list = options || []
    for (var i = 0; i < list.length; i++) {
      if (optionValue(list[i]) === value) {
        displayLabel = optionLabel(list[i])
        return
      }
    }
    displayLabel = value
  }

  onValueChanged: refreshDisplayLabel()
  onOptionsChanged: refreshDisplayLabel()
  Component.onCompleted: refreshDisplayLabel()

  Repeater {
    model: root.options

    RadioButton {
      id: radio
      required property var modelData
      required property int index

      checked: root.optionValue(modelData) === root.value
      enabled: root.enabled
      text: root.optionLabel(modelData)
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      padding: 2
      spacing: 8
      implicitHeight: Math.max(22, Theme.fontSize + 6)
      onClicked: root.changed(root.optionValue(modelData))

      indicator: Rectangle {
        implicitWidth: 16
        implicitHeight: 16
        x: radio.leftPadding
        y: radio.topPadding + (radio.availableHeight - height) / 2
        radius: width / 2
        color: "transparent"
        border.width: 1
        border.color: radio.checked || radio.hovered ? Theme.accent : Theme.borderColor()

        Rectangle {
          anchors.centerIn: parent
          width: 8
          height: 8
          radius: 4
          visible: radio.checked
          color: Theme.accent
        }
      }

      contentItem: Text {
        text: radio.text
        font: radio.font
        color: Theme.foreground
        leftPadding: radio.indicator.width + radio.spacing
        verticalAlignment: Text.AlignVCenter
      }
    }
  }
}
