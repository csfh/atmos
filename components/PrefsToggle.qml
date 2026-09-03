import QtQuick
import "../services"

Item {
  id: root

  property bool checked: false
  property bool enabled: true

  signal toggled()

  implicitWidth: 44
  implicitHeight: 24
  width: implicitWidth
  height: implicitHeight
  opacity: enabled ? 1 : 0.45
  activeFocusOnTab: enabled

  Accessible.role: Accessible.CheckBox
  Accessible.checkable: true
  Accessible.checked: checked
  Accessible.onPressAction: if (enabled) root.toggled()

  Keys.onReturnPressed: if (enabled) root.toggled()
  Keys.onSpacePressed: if (enabled) root.toggled()

  HoverHandler {
    id: toggleHover
    enabled: root.enabled
  }

  readonly property bool highlight: toggleHover.hovered || root.activeFocus

  Rectangle {
    anchors.fill: parent
    radius: height / 2
    color: root.checked ? Theme.accentFill(root.highlight ? 1 : 0.85) : Theme.fill(root.highlight ? Theme.hoverFill : 0.12)
    border.width: 1
    border.color: root.checked || root.highlight ? Theme.accent : Theme.borderColor()

    Behavior on color {
      ColorAnimation { duration: 90 }
    }
    Behavior on border.color {
      ColorAnimation { duration: 90 }
    }

    Rectangle {
      width: 16
      height: 16
      radius: 8
      anchors.verticalCenter: parent.verticalCenter
      x: root.checked ? parent.width - width - 4 : 4
      color: Theme.foreground

      Behavior on x { NumberAnimation { duration: 120 } }
    }
  }

  MouseArea {
    anchors.fill: parent
    enabled: root.enabled
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: root.toggled()
  }
}
