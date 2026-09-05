import QtQuick
import "../services"

Rectangle {
  id: root

  property string text: ""
  property bool enabled: true
  property bool primary: false
  property bool danger: false

  signal clicked()

  implicitWidth: label.implicitWidth + Theme.pad * 2
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  radius: Theme.radius
  opacity: Theme.controlOpacity(enabled)
  activeFocusOnTab: enabled
  color: {
    if ((mouse.containsMouse || root.activeFocus) && enabled) return Theme.fill(Theme.hoverFill)
    if (primary) return Theme.accentFill(Theme.primaryFill)
    return Theme.fill(Theme.normalFill)
  }
  border.width: Theme.borderWidth
  border.color: {
    if (danger) return Theme.urgent
    if (primary || ((mouse.containsMouse || root.activeFocus) && enabled)) return Theme.accent
    return Theme.borderColor()
  }

  Keys.onReturnPressed: if (enabled) root.clicked()
  Keys.onSpacePressed: if (enabled) root.clicked()

  Behavior on color {
    ColorAnimation { duration: Theme.motionFast }
  }
  Behavior on border.color {
    ColorAnimation { duration: Theme.motionFast }
  }

  Accessible.role: Accessible.Button
  Accessible.name: text
  Accessible.onPressAction: if (enabled) root.clicked()

  Text {
    id: label
    anchors.centerIn: parent
    text: root.text
    color: Theme.foreground
    font.family: Theme.fontFamily
    font.pixelSize: Theme.labelSize
  }

  MouseArea {
    id: mouse
    anchors.fill: parent
    enabled: root.enabled
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: root.clicked()
  }
}
