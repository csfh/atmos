import QtQuick
import QtQuick.Controls
import "../services"
import "../services/Systemd.js" as SystemdJs
import "../services/ShellConfig.js" as ShellConfigJs

Item {
  id: root

  readonly property bool prefsRow: true

  property var unitRow: null
  property string query: ""
  property bool split: true
  property bool available: true
  property bool sectionHelp: false
  property bool catalog: true

  signal acted(string action)

  readonly property var row: SystemdJs.normalizeUnit(root.unitRow)
  readonly property string unitName: row && row.unit ? row.unit : ""
  readonly property string statusText: SystemdJs.statusLine(row)
  readonly property var primary: SystemdJs.primaryAction(row)
  readonly property var overflow: SystemdJs.overflowActions(row)
  readonly property bool failed: SystemdJs.isFailed(row)

  readonly property string label: root.unitName
  readonly property string description: root.statusText
  readonly property string hint: row && row.allowed
    ? (row.scope === "user" ? "systemctl --user" : "systemctl")
    : "status and logs only"
  readonly property string detail: ""
  readonly property var keywords: ["systemd", "service", "unit"]

  readonly property string searchHaystack: {
    var parts = [root.label, root.description, root.hint]
    var more = SystemdJs.searchHaystack(row)
    if (more) parts.push(more)
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
  Accessible.name: root.unitName
  Accessible.description: root.statusText + (root.primary ? ". " + root.primary.label : "")

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
      text: root.unitName
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.labelSize
      font.bold: true
      elide: Text.ElideRight
      maximumLineCount: 1
      wrapMode: Text.NoWrap
      horizontalAlignment: Text.AlignLeft

      HoverHandler { id: nameHover }

      ToolTip.visible: nameText.truncated && nameHover.hovered
      ToolTip.text: root.unitName
      ToolTip.delay: 400
    }

    Text {
      width: parent.width
      visible: root.statusText.length > 0
      text: root.statusText
      color: root.failed ? Theme.urgent : Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.descriptionSize
      elide: Text.ElideRight
      maximumLineCount: 1
      wrapMode: Text.NoWrap
      horizontalAlignment: Text.AlignLeft
    }
  }

  Row {
    id: actions
    spacing: Theme.space
    anchors.right: parent.right
    anchors.rightMargin: Theme.copyInset
    anchors.verticalCenter: parent.verticalCenter

    PrefsButton {
      visible: !!(root.primary && root.primary.label)
      text: root.primary && root.primary.label ? root.primary.label : ""
      onClicked: root.acted(root.primary.id)
    }

    PrefsMenu {
      items: root.overflow
      onPicked: function(id) { root.acted(id) }
    }
  }
}
