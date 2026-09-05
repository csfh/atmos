import QtQuick
import QtQuick.Controls
import "../services"
import "../services/RichUi.js" as RichUi

Popup {
  id: root

  property string title: ""
  property string message: ""
  property string confirmText: "Confirm"
  property string cancelText: "Cancel"
  // Install/Apply/Update stay primary. Remove/Forget/Reset keep the urgent
  // border. Pages can still set this explicitly.
  property bool destructive: RichUi.confirmIsDestructive(confirmText)

  signal confirmed()
  signal canceled()

  property bool _accepted: false

  modal: true
  focus: true
  padding: Theme.pad * 1.5
  closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
  anchors.centerIn: Overlay.overlay
  width: Math.min(Theme.confirmWidth, Overlay.overlay ? Overlay.overlay.width - Theme.overlayInset : Theme.confirmWidth)

  background: Rectangle {
    color: Theme.background
    border.width: Theme.borderWidth
    border.color: Theme.borderColor()
    radius: Theme.radius
  }

  Overlay.modal: Rectangle {
    color: Qt.rgba(0, 0, 0, Theme.scrimAlpha)
  }

  // Declared inside PrefsPage's Flickable. Live on Overlay so clip cannot
  // crop Forget/Remove/LUKS dialogs that never reparent themselves.
  function attachOverlay() {
    var overlay = Overlay.overlay
    if (overlay)
      parent = overlay
  }

  function ask() {
    _accepted = false
    attachOverlay()
    open()
  }

  onAboutToShow: attachOverlay()
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
      spacing: Theme.space

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
