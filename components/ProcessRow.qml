import QtQuick
import QtQuick.Controls
import "../services"
import "../services/LiveStats.js" as LiveStatsJs
import "../services/Processes.js" as ProcessesJs
import "../services/RichUi.js" as RichUi
import "../services/ShellConfig.js" as ShellConfigJs

Item {
  id: root

  readonly property bool prefsRow: true

  property var procRow: null
  property string query: ""
  property bool split: true
  property bool available: true
  property bool sectionHelp: false
  property bool catalog: true

  signal acted(string action)

  readonly property var row: ProcessesJs.normalize(root.procRow)
  readonly property string comm: row && row.comm ? row.comm : ""
  readonly property string pidText: row ? String(row.pid) : ""
  readonly property string cpuText: row ? LiveStatsJs.formatPercent(row.cpu) : ""
  readonly property string rssText: row && row.rssKb != null ? RichUi.formatBytes(row.rssKb * 1024) : ""
  readonly property string statusText: {
    var bits = []
    if (root.pidText) bits.push(root.pidText)
    if (root.cpuText) bits.push(root.cpuText)
    if (root.rssText) bits.push(root.rssText)
    return bits.join("   ")
  }

  readonly property string label: root.comm
  readonly property string description: root.statusText
  readonly property string hint: row && row.cmdline ? row.cmdline : ""
  readonly property var keywords: ["process", "pid", "cpu", "memory"]

  readonly property string searchHaystack: {
    var parts = [root.label, root.description, root.hint]
    return ShellConfigJs.joinSearchHaystack(parts)
  }
  readonly property bool matches: ShellConfigJs.haystackMatches(query, searchHaystack)
  readonly property bool shown: available && matches

  visible: shown
  width: parent ? parent.width : 640
  implicitWidth: width
  implicitHeight: visible ? Math.max(copyCol.implicitHeight, actions.implicitHeight) + Theme.rowPad * 2 : 0
  height: implicitHeight

  Accessible.role: Accessible.ListItem
  Accessible.name: root.comm
  Accessible.description: root.statusText

  Rectangle {
    width: parent.width
    height: 1
    visible: root.split
    color: Theme.splitColor()
  }

  Column {
    id: copyCol
    x: Theme.copyInset
    y: Theme.rowPad
    width: Math.max(80, parent.width - Theme.copyInset * 2 - actions.width - Theme.spaceMd)
    spacing: Theme.labelGap

    Text {
      id: nameText
      width: parent.width
      text: root.comm
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.labelSize
      font.bold: true
      elide: Text.ElideRight
      maximumLineCount: 1
      wrapMode: Text.NoWrap
    }

    Text {
      width: parent.width
      visible: root.statusText.length > 0
      text: root.statusText
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.descriptionSize
      elide: Text.ElideRight
      maximumLineCount: 1
      wrapMode: Text.NoWrap
    }
  }

  Row {
    id: actions
    spacing: Theme.space
    anchors.right: parent.right
    anchors.rightMargin: Theme.copyInset
    anchors.verticalCenter: parent.verticalCenter

    PrefsButton {
      text: ProcessesJs.primaryAction().label
      onClicked: root.acted(ProcessesJs.primaryAction().id)
    }

    PrefsMenu {
      items: ProcessesJs.overflowActions()
      onPicked: function(id) { root.acted(id) }
    }
  }
}
