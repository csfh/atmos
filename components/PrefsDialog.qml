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
  width: Math.min(Theme.dialogWidth, Overlay.overlay ? Overlay.overlay.width - Theme.overlayInset : Theme.dialogWidth)

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
  // crop LUKS and add-item dialogs that never reparent themselves.
  function attachOverlay() {
    var overlay = Overlay.overlay
    if (overlay)
      parent = overlay
  }

  onAboutToShow: attachOverlay()

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
