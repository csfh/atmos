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
  opacity: enabled ? 1 : 0.45
  color: {
    if (mouse.containsMouse && enabled) return Theme.fill(Theme.hoverFill)
    if (primary) return Theme.accentFill(0.22)
    return Theme.fill(Theme.normalFill)
  }
  border.width: 1
  border.color: {
    if (danger) return Theme.urgent
    if (primary || (mouse.containsMouse && enabled)) return Theme.accent
    return Theme.borderColor()
  }

  Behavior on color {
    ColorAnimation { duration: 90 }
  }
  Behavior on border.color {
    ColorAnimation { duration: 90 }
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
    font.pixelSize: Theme.fontSize
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
