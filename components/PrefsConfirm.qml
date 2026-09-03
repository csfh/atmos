import QtQuick
import QtQuick.Controls
import "../services"

Popup {
  id: root

  property string title: ""
  property string message: ""
  property string confirmText: "Confirm"
  property string cancelText: "Cancel"
  property bool destructive: true

  signal confirmed()
  signal canceled()

  property bool _accepted: false

  modal: true
  focus: true
  padding: Theme.pad * 1.5
  closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
  anchors.centerIn: Overlay.overlay
  width: Math.min(420, Overlay.overlay ? Overlay.overlay.width - 48 : 420)

  background: Rectangle {
    color: Theme.background
    border.width: 1
    border.color: Theme.borderColor()
    radius: Theme.radius
  }

  Overlay.modal: Rectangle {
    color: Qt.rgba(0, 0, 0, 0.55)
  }

  function ask() {
    _accepted = false
    open()
  }

  onClosed: if (!_accepted) root.canceled()

  Column {
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

    PrefsText {
      width: parent.width
      text: root.message
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
    }

    Row {
      anchors.right: parent.right
      spacing: 8

      PrefsButton {
        text: root.cancelText
        onClicked: root.close()
      }

      PrefsButton {
        text: root.confirmText
        primary: !root.destructive
        danger: root.destructive
        onClicked: {
          root._accepted = true
          root.confirmed()
          root.close()
        }
      }
    }
  }
}
