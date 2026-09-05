import QtQuick
import "../services"

Item {
  id: root

  property string value: ""
  property string placeholder: ""
  property bool enabled: true
  property bool invalid: false
  property int horizontalAlignment: TextInput.AlignLeft

  signal submitted(string value)
  signal edited(string value)

  implicitWidth: Theme.fieldWidth
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: Theme.controlOpacity(enabled)

  function currentText() {
    return field.text
  }

  function clear() {
    field.text = ""
  }

  function setText(next) {
    field.text = String(next || "")
  }

  function replaceText(next) {
    var pos = field.cursorPosition
    var old = field.text
    if (next === old) return
    field.text = next
    field.cursorPosition = Math.max(0, Math.min(next.length, pos + (next.length - old.length)))
  }

  function applyValue() {
    if (!field.activeFocus)
      field.text = root.value
  }

  onValueChanged: applyValue()
  Component.onCompleted: field.text = root.value

  Accessible.role: Accessible.EditableText
  Accessible.name: field.text.length ? field.text : root.placeholder
  Accessible.onPressAction: if (enabled) field.forceActiveFocus()

  HoverHandler {
    id: fieldHover
    enabled: root.enabled
  }

  Rectangle {
    anchors.fill: parent
    radius: Theme.radius
    color: field.activeFocus || fieldHover.hovered ? Theme.fill(Theme.hoverFill) : Theme.fill(Theme.normalFill)
    border.width: Theme.borderWidth
    border.color: root.invalid ? Theme.urgent : (field.activeFocus || fieldHover.hovered ? Theme.accent : Theme.borderColor())

    Behavior on color {
      ColorAnimation { duration: Theme.motionFast }
    }
    Behavior on border.color {
      ColorAnimation { duration: Theme.motionFast }
    }

    TextInput {
      id: field
      anchors.fill: parent
      anchors.leftMargin: Theme.fieldInset
      anchors.rightMargin: Theme.fieldInset
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      clip: true
      enabled: root.enabled
      selectByMouse: true
      horizontalAlignment: root.horizontalAlignment
      verticalAlignment: TextInput.AlignVCenter
      activeFocusOnTab: root.enabled
      onAccepted: root.submitted(text)
      onTextEdited: root.edited(text)

      Text {
        anchors.fill: parent
        visible: field.text.length === 0 && !field.activeFocus
        text: root.placeholder
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        horizontalAlignment: root.horizontalAlignment
        verticalAlignment: Text.AlignVCenter
      }
    }
  }
}
