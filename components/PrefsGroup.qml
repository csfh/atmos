import QtQuick
import "../services"
import "../services/Disclosure.js" as DisclosureJs
import "../services/Layout.js" as LayoutJs
import "../services/WritesFile.js" as WritesFile

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
  // Same fold as SettingRow.advanced. Existing Advanced sections opt in
  // so Simple talks to the heading already on the page, not a second model.
  property bool advanced: false
  // Opt-in write target. writesMode is sentinel (Atmos block), file
  // (Atmos owns the file), or command. Default sentinel so a copied
  // Input section cannot claim whole-file ownership by omission.
  property string writesFile: ""
  property string writesMode: "sentinel"
  property string writesNote: ""
  readonly property string writesCaption: WritesFile.caption(root.writesFile, root.writesMode, {
    note: root.writesNote
  })

  width: parent ? parent.width : 640
  spacing: Theme.headingGap

  readonly property string resolvedHubId: {
    var p = parent
    while (p) {
      if (p.hubId !== undefined && String(p.hubId).length) return String(p.hubId)
      p = p.parent
    }
    return ""
  }

  readonly property var disclosureLabels: {
    var _n = root.childCount(rowsColumn)
    var rows = root.collectPrefsRows({ includeFolded: true })
    var out = []
    var i
    for (i = 0; i < rows.length; i++) out.push(rows[i].label || "")
    return out
  }

  readonly property bool folded: DisclosureJs.groupFolded({
    advanced: root.advanced,
    query: root.query,
    hub: root.resolvedHubId,
    labels: root.disclosureLabels
  }, {
    simple: Disclosure.simple,
    revealedHub: Disclosure.revealedHub,
    revealedLabel: Disclosure.revealedLabel
  })

  // The section stays in the layout even when it has no matching rows.
  // Hiding it with visible:false, or collapsing it to height 0 with clip,
  // skips the row layout pass and every group stays a heading with no controls.
  // Simple may hide an opted-in Advanced section as a unit; children stay
  // instantiated so a search pin can bring the heading back.
  visible: !root.folded && (query.length === 0 || rowsColumn.implicitHeight > 0)

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

  function collectPrefsRows(opts) {
    var includeFolded = !!(opts && opts.includeFolded)
    var out = []
    function walk(node) {
      var kids = node && node.children
      if (!kids) return
      var i
      for (i = 0; i < kids.length; i++) {
        var kid = kids[i]
        if (!kid) continue
        if (kid.prefsRow === true) {
          if (kid.available === false) continue
          if (kid.matches === false) continue
          if (kid.visible === false && !(includeFolded && kid.folded === true)) continue
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
    var rows = root.collectPrefsRows({ includeFolded: true })
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
    var _simple = Disclosure.simple
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

  Item {
    id: headingHost
    width: parent.width
    implicitHeight: headingColumn.implicitHeight
    height: implicitHeight
    visible: root.title.length > 0

    HoverHandler {
      id: headingHover
    }

    Column {
      id: headingColumn
      width: parent.width
      spacing: Theme.titleGap

      Item {
        id: titleRow
        width: parent.width
        implicitHeight: Math.max(titleLabel.implicitHeight, groupHelp.implicitHeight)
        height: implicitHeight

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

      PrefsText {
        id: provenanceLabel
        visible: root.writesCaption.length > 0
        x: root.titleInset
        width: parent.width - root.titleInset * 2
        text: root.writesCaption
        color: Theme.muted
        opacity: Theme.metaOpacity
        font.family: Theme.fontFamily
        font.pixelSize: Theme.metaSize
      }
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
