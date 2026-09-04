import QtQuick
import "../services"
import "../services/Layout.js" as LayoutJs

Column {
  id: root

  property string title: ""
  property string query: ""
  property string detail: ""
  property string hint: ""
  // Find a setting indexes PrefsGroup blocks unless this is false.
  property bool catalog: true

  width: parent ? parent.width : 640
  spacing: Theme.space
  // The card stays in the layout even when it has no matching rows.
  // Hiding it with visible:false, or collapsing it to height 0 with clip,
  // skips the row layout pass and every group stays a heading with no controls.
  visible: query.length === 0 || rowsColumn.implicitHeight > 0

  function collectHelpRows() {
    var out = []
    var kids = rowsColumn.children
    for (var i = 0; i < kids.length; i++) {
      var kid = kids[i]
      if (!kid || kid.prefsRow !== true) continue
      if (kid.sectionHelp === false) continue
      if (kid.available === false) continue
      if (kid.matches === false) continue
      out.push({
        label: kid.label || "",
        description: kid.description || "",
        detail: kid.detail || "",
        hint: kid.hint || ""
      })
    }
    return out
  }

  readonly property var helpTopics: {
    var _q = root.query
    var _n = rowsColumn.children.length
    return LayoutJs.sectionHelpTopics(root.collectHelpRows())
  }

  readonly property bool showHelp: root.detail.length > 0 || root.hint.length > 0
  readonly property int contentPad: Theme.rowPad

  Item {
    width: parent.width
    implicitHeight: root.title.length > 0 ? Math.max(titleLabel.implicitHeight, groupHelp.implicitHeight) : 0
    height: implicitHeight
    visible: root.title.length > 0

    PrefsText {
      id: titleLabel
      anchors.left: parent.left
      anchors.leftMargin: root.contentPad + Theme.rowPad
      anchors.right: groupHelp.visible ? groupHelp.left : parent.right
      anchors.rightMargin: groupHelp.visible ? Theme.space : root.contentPad + Theme.rowPad
      anchors.verticalCenter: parent.verticalCenter
      text: root.title
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      font.bold: true
    }

    PrefsHelp {
      id: groupHelp
      anchors.right: parent.right
      anchors.rightMargin: root.contentPad + Theme.rowPad
      anchors.verticalCenter: parent.verticalCenter
      title: root.title
      body: root.detail
      command: root.hint
      topics: root.showHelp ? root.helpTopics : []
    }
  }

  Rectangle {
    width: parent.width
    implicitHeight: rowsWrap.implicitHeight
    height: implicitHeight
    color: Theme.fill(Theme.normalFill)
    border.width: 1
    border.color: Theme.borderColor()
    radius: Theme.radius

    Item {
      id: rowsWrap
      width: parent.width
      implicitHeight: rowsColumn.implicitHeight + root.contentPad * 2
      height: implicitHeight

      Column {
        id: rowsColumn
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: root.contentPad
        spacing: Theme.spaceMd
      }
    }
  }

  default property alias extra: rowsColumn.data
}
