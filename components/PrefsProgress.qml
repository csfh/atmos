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
  readonly property bool inView: view.inView

  ChartViewport {
    id: view
    target: root
    retainClock: false
  }

  implicitWidth: 260
  implicitHeight: (valueText.length > 0 ? Theme.captionSize + 8 : 0) + 8
  width: implicitWidth
  height: implicitHeight

  Column {
    anchors.fill: parent
    spacing: Theme.labelGap

    Text {
      visible: root.valueText.length > 0
      text: root.valueText
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.metaSize
    }

    ProgressBar {
      id: bar
      width: parent.width
      from: root.from
      to: root.to
      value: root.value
      indeterminate: root.indeterminate

      Behavior on value {
        enabled: !root.indeterminate && root.inView
        NumberAnimation {
          duration: Math.max(80, LiveStatsStore.intervalMs)
          easing.type: Easing.Linear
        }
      }

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
