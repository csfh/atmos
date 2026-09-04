import QtQuick
import "../services"
import "../services/ShellConfig.js" as ShellConfigJs

Item {
  id: root

  readonly property bool prefsRow: true

  property string label: ""
  property string description: ""
  property string hint: ""
  // Longer copy for the section "?" modal. Leave empty to use description.
  property string detail: ""
  property string query: ""
  property var keywords: []
  property bool stretchControl: false
  property bool available: true
  // List rows (wifi SSIDs, devices) stay out of the section modal.
  property bool sectionHelp: true
  // Find a setting indexes PrefsRow blocks unless this is false.
  property bool catalog: true

  default property alias extra: controlSlot.data

  readonly property string searchHaystack: {
    var parts = [label, description, hint, detail]
    var list = keywords || []
    for (var i = 0; i < list.length; i++) parts.push(list[i])
    return ShellConfigJs.joinSearchHaystack(parts)
  }

  readonly property bool matches: ShellConfigJs.haystackMatches(query, searchHaystack)

  readonly property int maxControlCol: {
    var avail = parent ? parent.width : Theme.controlColumnWidth
    return Math.max(140, avail - 160 - Theme.spaceMd)
  }

  readonly property int controlCol: {
    var wanted = Theme.controlColumnWidth
    var natural = controlSlot.implicitWidth
    if (natural > wanted) wanted = natural
    return Math.min(wanted, maxControlCol)
  }

  visible: available && matches
  width: parent ? parent.width : 640
  implicitWidth: width
  implicitHeight: visible
    ? (stretchControl ? stackedBody.implicitHeight : rowBody.implicitHeight) + Theme.rowPad * 2
    : 0
  height: implicitHeight

  Row {
    id: rowBody
    width: parent.width - Theme.rowPad * 2
    x: Theme.rowPad
    y: Theme.rowPad
    spacing: Theme.spaceMd
    visible: !root.stretchControl

    Item {
      id: inlineCopyHost
      width: controlSlot.children.length > 0
        ? Math.max(160, parent.width - root.controlCol - parent.spacing)
        : parent.width
      implicitWidth: width
      implicitHeight: copyCol.implicitHeight
      height: implicitHeight
    }

    Item {
      id: inlineControl
      visible: controlSlot.children.length > 0
      width: root.controlCol
      implicitWidth: width
      implicitHeight: Math.max(Theme.controlHeight, controlSlot.implicitHeight, inlineCopyHost.implicitHeight)
      height: implicitHeight
    }
  }

  Column {
    id: stackedBody
    width: parent.width - Theme.rowPad * 2
    x: Theme.rowPad
    y: Theme.rowPad
    spacing: 6
    visible: root.stretchControl

    Item {
      id: stackedCopyHost
      width: parent.width
      implicitHeight: copyCol.implicitHeight
      height: implicitHeight
    }

    Item {
      id: stackedControl
      width: parent.width
      implicitHeight: visible ? controlSlot.implicitHeight : 0
      height: implicitHeight
    }
  }

  Column {
    id: copyCol
    parent: root.stretchControl ? stackedCopyHost : inlineCopyHost
    width: parent.width
    spacing: 4

    PrefsText {
      width: parent.width
      visible: root.label.length > 0
      text: root.label
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      font.bold: true
      horizontalAlignment: Text.AlignLeft
    }

    PrefsText {
      width: parent.width
      visible: root.description.length > 0
      text: root.description
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
      horizontalAlignment: Text.AlignLeft
    }
  }

  Item {
    id: controlSlot
    parent: root.stretchControl ? stackedControl : inlineControl
    implicitWidth: children.length > 0 ? children[0].implicitWidth : 0
    implicitHeight: children.length > 0 ? children[0].implicitHeight : 0
    width: root.stretchControl ? stackedControl.width : Math.min(implicitWidth, inlineControl.width)
    height: implicitHeight
    anchors.right: root.stretchControl ? undefined : parent.right
    anchors.verticalCenter: root.stretchControl ? undefined : parent.verticalCenter
  }
}
