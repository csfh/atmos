import QtQuick
import "../services"
import "../services/RichUi.js" as RichUi

Item {
  id: root

  property real used: 0
  property real size: 0
  property real avail: 0

  readonly property int percent: RichUi.usagePercent(used, size)

  implicitWidth: 260
  implicitHeight: label.implicitHeight + 10
  width: implicitWidth
  height: implicitHeight

  Column {
    anchors.fill: parent
    spacing: 4

    PrefsText {
      id: label
      width: parent.width
      text: RichUi.formatBytes(root.used) + " of " + RichUi.formatBytes(root.size) + " used (" + root.percent + "%). " + RichUi.formatBytes(root.avail) + " free."
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsProgress {
      width: parent.width
      value: root.percent
      from: 0
      to: 100
    }
  }
}
