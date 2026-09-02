import QtQuick
import QtQuick.Controls
import "../services"

Item {
  id: root

  property real value: 0
  property real from: 0
  property real to: 100
  property bool indeterminate: false
  property string valueText: ""

  implicitWidth: 260
  implicitHeight: (valueText.length > 0 ? Theme.captionSize + 8 : 0) + 8
  width: implicitWidth
  height: implicitHeight

  Column {
    anchors.fill: parent
    spacing: 4

    Text {
      visible: root.valueText.length > 0
      text: root.valueText
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    ProgressBar {
      id: bar
      width: parent.width
      from: root.from
      to: root.to
      value: root.value
      indeterminate: root.indeterminate

      background: Rectangle {
        implicitHeight: 6
        height: 6
        y: (parent.height - height) / 2
        radius: 3
        color: Theme.fill(0.2)
      }

      contentItem: Item {
        implicitHeight: 6
        Rectangle {
          width: root.indeterminate ? parent.width * 0.35 : bar.visualPosition * parent.width
          height: 6
          radius: 3
          color: Theme.accent
          x: root.indeterminate ? (parent.width - width) * 0.5 : 0
        }
      }
    }
  }
}
