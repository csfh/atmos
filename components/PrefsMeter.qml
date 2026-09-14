import QtQuick
import "../services"

Item {
  id: root

  property string label: ""
  property string valueText: ""
  property string caption: ""
  property var values: []
  property var valuesB: []
  property real barValue: 0
  property bool showBar: false
  property bool alert: false
  property bool fill: true

  implicitWidth: 260
  implicitHeight: Math.max(Theme.meterMinHeight, inner.implicitHeight + Theme.pad * 2)
  width: implicitWidth
  height: implicitHeight

  Accessible.role: Accessible.StaticText
  Accessible.name: root.label
  Accessible.description: root.valueText + (root.caption ? ". " + root.caption : "")

  Rectangle {
    anchors.fill: parent
    color: Theme.fill(Theme.normalFill)
    border.width: Theme.borderWidth
    border.color: root.alert ? Theme.urgent : Theme.borderColor()
    radius: Theme.radius
  }

  Column {
    id: inner
    x: Theme.pad
    y: Theme.pad
    width: parent.width - Theme.pad * 2
    spacing: Theme.labelGap

    Text {
      width: parent.width
      text: root.label
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.metaSize
      font.letterSpacing: Theme.sectionTracking
      elide: Text.ElideRight
    }

    Text {
      width: parent.width
      text: root.valueText.length ? root.valueText : "—"
      color: root.alert ? Theme.urgent : Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.embedTitleSize
      font.bold: true
      elide: Text.ElideRight
    }

    PrefsSparkline {
      width: parent.width
      height: Theme.sparklineHeight
      values: root.values
      valuesB: root.valuesB
      valueText: root.valueText
      alert: root.alert
      fill: root.fill
    }

    PrefsProgress {
      width: parent.width
      visible: root.showBar
      from: 0
      to: 100
      value: root.barValue
    }

    Text {
      width: parent.width
      visible: root.caption.length > 0
      text: root.caption
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
      elide: Text.ElideRight
    }
  }
}
