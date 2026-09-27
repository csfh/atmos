import QtQuick
import "../services"

Item {
  id: root

  anchors.left: parent.left
  anchors.right: parent.right
  anchors.top: parent.top
  height: Omarchy.guardOpen ? bar.implicitHeight : 0
  clip: true

  Timer {
    interval: 200
    repeat: true
    running: Omarchy.guardOpen
    onTriggered: Omarchy.tickGuard()
  }

  Connections {
    target: Omarchy
    function onGuardArmed() {
      keepBtn.forceActiveFocus()
    }
  }

  Rectangle {
    id: bar
    width: parent.width
    implicitHeight: copy.implicitHeight + Theme.pad * 2
    color: Theme.background
    border.width: Theme.borderWidth
    border.color: Theme.borderColor()

    Row {
      anchors.left: parent.left
      anchors.right: parent.right
      anchors.verticalCenter: parent.verticalCenter
      anchors.leftMargin: Theme.pad * 1.5
      anchors.rightMargin: Theme.pad * 1.5
      spacing: Theme.space

      PrefsText {
        id: copy
        width: Math.max(80, parent.width - actions.width - parent.spacing)
        text: "These settings revert in " + Omarchy.guardSeconds + "s unless you keep them."
        wrapMode: Text.WordWrap
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        Accessible.role: Accessible.StaticText
        Accessible.name: text
      }

      Row {
        id: actions
        spacing: Theme.space

        PrefsButton {
          id: keepBtn
          text: "Keep"
          primary: true
          onClicked: Omarchy.keepGuard()
        }

        PrefsButton {
          text: "Revert"
          onClicked: Omarchy.revertGuard()
        }
      }
    }
  }
}
