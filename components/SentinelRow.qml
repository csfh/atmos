import QtQuick
import "../services"
import "../services/Processes.js" as ProcessesJs
import "../services/ShellConfig.js" as ShellConfigJs

// One Sentinel finding or listening port. `item` comes from Sentinel.js rows().
Item {
  id: root

  readonly property bool prefsRow: true

  property var item: null
  property string query: ""
  property bool split: true
  property bool available: true
  property bool sectionHelp: false
  // One line each until clicked, then the whole description and command, wrapped.
  property bool expanded: false

  signal acted(string action)

  readonly property var proc: item && item.proc ? item.proc : null
  readonly property string label: item ? item.comm : ""
  readonly property string description: item ? item.detail : ""
  readonly property string hint: item && item.cmdline ? item.cmdline : ""
  readonly property bool flagged: !!(item && item.flagged)
  readonly property bool canAct: ProcessesJs.canSignal(root.proc) && root.proc !== null

  readonly property string searchHaystack: ShellConfigJs.joinSearchHaystack([root.label, root.description, root.hint])
  readonly property bool matches: ShellConfigJs.haystackMatches(query, searchHaystack)
  readonly property bool shown: available && matches

  function toggle() { root.expanded = !root.expanded }

  visible: shown
  activeFocusOnTab: true
  Keys.onReturnPressed: root.toggle()
  Keys.onSpacePressed: root.toggle()
  width: parent ? parent.width : 640
  implicitWidth: width
  implicitHeight: visible ? Math.max(copyCol.implicitHeight, actions.implicitHeight) + Theme.rowPad * 2 : 0
  height: implicitHeight

  Accessible.role: Accessible.ListItem
  Accessible.name: root.label
  Accessible.description: root.description + (root.hint.length > 0 ? ". " + root.hint : "")
  Accessible.onPressAction: root.toggle()

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
      width: parent.width
      text: root.label
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
      visible: root.description.length > 0
      text: root.description
      color: root.flagged ? Theme.urgent : Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.descriptionSize
      elide: root.expanded ? Text.ElideNone : Text.ElideRight
      maximumLineCount: root.expanded ? 1000 : 1
      wrapMode: root.expanded ? Text.WrapAnywhere : Text.NoWrap
    }

    Text {
      width: parent.width
      visible: root.hint.length > 0
      text: root.hint
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
      elide: root.expanded ? Text.ElideNone : Text.ElideRight
      maximumLineCount: root.expanded ? 1000 : 1
      wrapMode: root.expanded ? Text.WrapAnywhere : Text.NoWrap
    }
  }

  MouseArea {
    anchors.fill: copyCol
    cursorShape: Qt.PointingHandCursor
    onClicked: root.toggle()
  }

  Row {
    id: actions
    visible: root.canAct
    spacing: Theme.space
    anchors.right: parent.right
    anchors.rightMargin: Theme.copyInset
    anchors.verticalCenter: parent.verticalCenter

    PrefsButton {
      text: ProcessesJs.primaryAction().label
      onClicked: root.acted(ProcessesJs.primaryAction().id)
    }

    PrefsMenu {
      items: ProcessesJs.overflowActions(root.proc)
      onPicked: function(id) { root.acted(id) }
    }
  }
}
