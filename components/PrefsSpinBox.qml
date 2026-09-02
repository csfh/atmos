import QtQuick
import QtQuick.Controls
import "../services"

Item {
  id: root

  property int value: 0
  property int from: 0
  property int to: 100
  property int stepSize: 1
  property bool enabled: true

  signal changed(int value)

  implicitWidth: 108
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: enabled ? 1 : 0.45

  Accessible.role: Accessible.SpinBox
  Accessible.name: String(value)

  SpinBox {
    id: spin
    anchors.fill: parent
    from: root.from
    to: root.to
    stepSize: root.stepSize
    value: root.value
    enabled: root.enabled
    editable: true
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontSize
    leftPadding: down.indicator ? down.indicator.width : 22
    rightPadding: up.indicator ? up.indicator.width : 22
    topPadding: 0
    bottomPadding: 0
    onValueModified: root.changed(value)

    background: Rectangle {
      color: spin.activeFocus || spin.hovered ? Theme.fill(Theme.hoverFill) : Theme.fill(Theme.normalFill)
      border.width: 1
      border.color: spin.activeFocus || spin.hovered ? Theme.accent : Theme.borderColor()
      radius: Theme.radius
    }

    contentItem: TextInput {
      width: spin.availableWidth
      height: spin.availableHeight
      text: spin.displayText
      font: spin.font
      color: Theme.foreground
      selectionColor: Theme.accentFill(0.35)
      selectedTextColor: Theme.foreground
      horizontalAlignment: TextInput.AlignHCenter
      verticalAlignment: TextInput.AlignVCenter
      readOnly: !spin.editable
      validator: spin.validator
      inputMethodHints: Qt.ImhDigitsOnly
    }

    up.indicator: Rectangle {
      x: spin.mirrored ? 0 : parent.width - width
      height: parent.height
      implicitWidth: 22
      color: spin.up.pressed || spin.up.hovered ? Theme.fill(Theme.hoverFill) : "transparent"
      Text {
        anchors.centerIn: parent
        text: "+"
        color: Theme.foreground
        font.pixelSize: Theme.fontSize
      }
    }

    down.indicator: Rectangle {
      x: spin.mirrored ? parent.width - width : 0
      height: parent.height
      implicitWidth: 22
      color: spin.down.pressed || spin.down.hovered ? Theme.fill(Theme.hoverFill) : "transparent"
      Text {
        anchors.centerIn: parent
        text: "−"
        color: Theme.foreground
        font.pixelSize: Theme.fontSize
      }
    }
  }

  Binding {
    target: spin
    property: "value"
    value: root.value
    when: !spin.activeFocus
  }
}
