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
  property bool _holding: false
  property int _heldValue: 0

  signal changed(int value)

  implicitWidth: Theme.spinWidth
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: Theme.controlOpacity(enabled)

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
    onValueModified: {
      root._heldValue = value
      root._holding = true
      root.changed(value)
      // Writers apply or reject in this call. A rejected birth year (or
      // any other ignored number) must not keep Binding off forever.
      root._holding = false
    }

    background: Rectangle {
      color: spin.activeFocus || spin.hovered ? Theme.fill(Theme.hoverFill) : Theme.fill(Theme.normalFill)
      border.width: Theme.borderWidth
      border.color: spin.activeFocus || spin.hovered ? Theme.accent : Theme.borderColor()
      radius: Theme.radius

      Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
      }
      Behavior on border.color {
        ColorAnimation { duration: Theme.motionFast }
      }
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

  onValueChanged: {
    if (_holding && value === _heldValue)
      _holding = false
  }

  Binding {
    target: spin
    property: "value"
    value: root.value
    when: !spin.activeFocus && !_holding
  }
}
