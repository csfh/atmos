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
  // Boxes are for collections, objects, and special operations.
  // Ordinary settings are a heading plus rows.
  property bool framed: false

  // The config file this section writes, and whether Atmos owns it or only
  // edits inside its own sentinel. Opt-in per section: a section that has
  // not declared it renders exactly as it does today.
  property string writesFile: ""
  property bool writesManaged: true

  width: parent ? parent.width : 640
  spacing: Theme.headingGap
  // The section stays in the layout even when it has no matching rows.
  // Hiding it with visible:false, or collapsing it to height 0 with clip,
  // skips the row layout pass and every group stays a heading with no controls.
  visible: query.length === 0 || rowsColumn.implicitHeight > 0

  function childCount(node) {
    var n = 0
    var kids = node && node.children
    if (!kids) return 0
    n = kids.length
    var i
    for (i = 0; i < kids.length; i++) {
      var kid = kids[i]
      if (kid && kid.prefsRow !== true && kid.children) n += kid.children.length
    }
    return n
  }

  function collectPrefsRows() {
    var out = []
    function walk(node) {
      var kids = node && node.children
      if (!kids) return
      var i
      for (i = 0; i < kids.length; i++) {
        var kid = kids[i]
        if (!kid) continue
        if (kid.prefsRow === true) {
          if (kid.visible === false) continue
          if (kid.available === false) continue
          if (kid.matches === false) continue
          out.push(kid)
        } else if (kid.children && kid.children.length > 0) {
          walk(kid)
        }
      }
    }
    walk(rowsColumn)
    return out
  }

  function collectHelpRows() {
    var rows = root.collectPrefsRows()
    var out = []
    var i
    for (i = 0; i < rows.length; i++) {
      var kid = rows[i]
      if (kid.sectionHelp === false) continue
      out.push({
        label: kid.label || "",
        description: kid.description || "",
        detail: kid.detail || "",
        hint: kid.hint || ""
      })
    }
    return out
  }

  readonly property var helpPayload: {
    var _q = root.query
    var _n = root.childCount(rowsColumn)
    var _s = root.splitPass
    return LayoutJs.sectionHelpPayload(root.detail, root.hint, root.collectHelpRows())
  }

  readonly property int splitPass: {
    var _q = root.query
    var _n = root.childCount(rowsColumn)
    var rows = root.collectPrefsRows()
    var first = true
    var i
    for (i = 0; i < rows.length; i++) {
      rows[i].split = !(root.framed && first)
      first = false
    }
    return _n
  }

  readonly property bool showHelp: LayoutJs.sectionHelpOpen(root.helpPayload)
  readonly property int contentPad: Theme.rowPad
  readonly property int titleInset: root.framed ? root.contentPad + Theme.copyInset : Theme.copyInset

  // "What will this touch?" -- the single most trust-building thing a
  // settings panel can say, and Atmos is unusual in already knowing the
  // answer, because it tracks managed versus hand-written config for the
  // sentinel deferral. This only surfaces it.
  Item {
    width: parent.width
    visible: root.writesFile.length > 0 && root.title.length > 0
    implicitHeight: visible ? provenanceLabel.implicitHeight + Theme.titleGap : 0
    height: implicitHeight

    Text {
      id: provenanceLabel
      anchors.left: parent.left
      anchors.leftMargin: root.titleInset
      anchors.right: parent.right
      anchors.rightMargin: root.titleInset
      anchors.bottom: parent.bottom
      text: root.writesFile + (root.writesManaged ? "  ·  managed by Atmos" : "  ·  your file, Atmos edits in place")
      color: Theme.muted
      opacity: Theme.metaOpacity
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
      elide: Text.ElideMiddle
    }
  }

  Item {
    id: headingHost
    width: parent.width
    implicitHeight: root.title.length > 0 ? Math.max(titleLabel.implicitHeight, groupHelp.implicitHeight) : 0
    height: implicitHeight
    visible: root.title.length > 0

    HoverHandler {
      id: headingHover
    }

    PrefsText {
      id: titleLabel
      anchors.left: parent.left
      anchors.leftMargin: root.titleInset
      anchors.right: groupHelp.visible ? groupHelp.left : parent.right
      anchors.rightMargin: groupHelp.visible ? Theme.space : root.titleInset
      anchors.verticalCenter: parent.verticalCenter
      text: root.title.toUpperCase()
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.sectionSize
      font.bold: true
      font.letterSpacing: Theme.sectionTracking
    }

    PrefsHelp {
      id: groupHelp
      anchors.right: parent.right
      anchors.rightMargin: root.titleInset
      anchors.verticalCenter: parent.verticalCenter
      title: root.title
      reveal: headingHover.hovered
      body: root.helpPayload && root.helpPayload.body ? root.helpPayload.body : ""
      command: root.helpPayload && root.helpPayload.command ? root.helpPayload.command : ""
      topics: root.showHelp && root.helpPayload ? root.helpPayload.topics : []
    }
  }

  Item {
    width: parent.width
    implicitHeight: root.framed ? card.implicitHeight : rowsColumn.implicitHeight
    height: implicitHeight

    Rectangle {
      id: card
      visible: root.framed
      width: parent.width
      implicitHeight: rowsWrap.implicitHeight
      height: implicitHeight
      color: Theme.fill(Theme.normalFill)
      border.width: Theme.borderWidth
      border.color: Theme.borderColor()
      radius: Theme.radius
    }

    Item {
      id: rowsWrap
      width: parent.width
      implicitHeight: rowsColumn.implicitHeight + (root.framed ? root.contentPad * 2 : 0)
      height: implicitHeight

      Column {
        id: rowsColumn
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: root.framed ? root.contentPad : 0
        spacing: root.framed ? 0 : Theme.controlGap
      }
    }
  }

  default property alias extra: rowsColumn.data
}
