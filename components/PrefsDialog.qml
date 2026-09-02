import QtQuick
import QtQuick.Controls
import "../services"

Popup {
  id: root

  property string title: ""
  default property alias extra: body.data

  modal: true
  focus: true
  padding: Theme.pad * 1.5
  closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
  anchors.centerIn: Overlay.overlay
  width: Math.min(520, Overlay.overlay ? Overlay.overlay.width - 48 : 520)

  background: Rectangle {
    color: Theme.background
    border.width: 1
    border.color: Theme.borderColor()
    radius: Theme.radius
  }

  Overlay.modal: Rectangle {
    color: Qt.rgba(0, 0, 0, 0.55)
  }

  Column {
    id: body
    width: parent.width
    spacing: Theme.pad

    PrefsText {
      visible: root.title.length > 0
      width: parent.width
      text: root.title
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      font.bold: true
    }
  }
}
