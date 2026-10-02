import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../services"

Popup {
  id: root

  property string title: ""
  // Standard footer. Leave primaryText empty to keep a caller-built button row.
  property string primaryText: ""
  property string cancelText: "Cancel"
  property bool primaryEnabled: true
  property bool primaryDanger: false
  default property alias extra: body.data

  signal primaryClicked()

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

  ColumnLayout {
    width: parent.width
    spacing: Theme.pad

    Column {
      id: body
      Layout.fillWidth: true
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

    Row {
      visible: root.primaryText.length > 0
      Layout.alignment: Qt.AlignRight
      spacing: Theme.space

      PrefsButton {
        text: root.cancelText
        onClicked: root.close()
      }

      PrefsButton {
        text: root.primaryText
        enabled: root.primaryEnabled
        primary: !root.primaryDanger
        danger: root.primaryDanger
        onClicked: root.primaryClicked()
      }
    }
  }
}
