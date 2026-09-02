import QtQuick
import "../services"

Item {
  id: root

  property string value: ""
  property string placeholder: ""
  property bool enabled: true

  signal submitted(string value)

  implicitWidth: 200
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: enabled ? 1 : 0.45

  function currentText() {
    return field.text
  }

  function clear() {
    field.text = ""
  }

  Accessible.role: Accessible.EditableText
  Accessible.passwordEdit: true
  Accessible.name: placeholder
  Accessible.onPressAction: if (enabled) field.forceActiveFocus()

  HoverHandler {
    id: fieldHover
    enabled: root.enabled
  }

  Rectangle {
    anchors.fill: parent
    radius: Theme.radius
    color: field.activeFocus || fieldHover.hovered ? Theme.fill(Theme.hoverFill) : Theme.fill(Theme.normalFill)
    border.width: 1
    border.color: field.activeFocus || fieldHover.hovered ? Theme.accent : Theme.borderColor()

    Behavior on color {
      ColorAnimation { duration: 90 }
    }
    Behavior on border.color {
      ColorAnimation { duration: 90 }
    }

    TextInput {
      id: field
      anchors.fill: parent
      anchors.leftMargin: 10
      anchors.rightMargin: 10
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      clip: true
      enabled: root.enabled
      echoMode: TextInput.Password
      selectByMouse: true
      verticalAlignment: TextInput.AlignVCenter
      activeFocusOnTab: root.enabled
      onAccepted: root.submitted(text)

      Text {
        visible: field.text.length === 0 && !field.activeFocus
        text: root.placeholder
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
      }
    }
  }
}
