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
        text: "These settings revert in " + Omarchy.guardSeconds + "s unless you keep them. Esc reverts."
        wrapMode: Text.WordWrap
        color: Omarchy.guardSeconds <= 5 ? Theme.urgent : Theme.foreground
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

    Rectangle {
      id: progressTrack
      anchors.left: parent.left
      anchors.right: parent.right
      anchors.bottom: parent.bottom
      height: 3
      color: Theme.fill(Theme.normalFill)

      Rectangle {
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: parent.width * Math.max(0, Math.min(1, Omarchy.guardSeconds / 12))
        color: Omarchy.guardSeconds <= 5 ? Theme.urgent : Theme.accent

        Behavior on width {
          NumberAnimation { duration: 200; easing.type: Easing.Linear }
        }
        Behavior on color {
          ColorAnimation { duration: Theme.motionFast }
        }
      }
    }
  }
}
