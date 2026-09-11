import QtQuick
import "../services"

Rectangle {
  id: root

  property string text: ""
  property bool enabled: true
  property bool primary: false
  property bool danger: false

  signal clicked()

  TextMetrics {
    id: labelMetrics
    font.family: Theme.fontFamily
    font.pixelSize: Theme.labelSize
    text: root.text
  }

  implicitWidth: Math.ceil(labelMetrics.width) + Theme.pad * 2
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  radius: Theme.radius
  opacity: Theme.controlOpacity(enabled)
  activeFocusOnTab: enabled

  // The root stays a Rectangle so every existing anchor, size and caller is
  // untouched; it simply stops painting itself and lets Chamfer draw the
  // same fill and the same hairline in a different silhouette.
  readonly property color bodyColor: {
    if ((mouse.containsMouse || root.activeFocus) && enabled) return Theme.fill(Theme.hoverFill)
    if (primary) return Theme.accentFill(Theme.primaryFill)
    return Theme.fill(Theme.normalFill)
  }
  readonly property color edgeColor: {
    if (danger) return Theme.urgent
    if (primary || ((mouse.containsMouse || root.activeFocus) && enabled)) return Theme.accent
    return Theme.borderColor()
  }

  color: "transparent"
  border.width: 0
  border.color: "transparent"

  Chamfer {
    anchors.fill: parent
    z: -1
    fillColor: root.bodyColor
    strokeColor: root.edgeColor
    cut: Theme.chamferSm
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
