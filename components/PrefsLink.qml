import QtQuick
import "../services"
import "../services/ShellConfig.js" as ShellConfigJs

Item {
  id: root

  readonly property bool prefsRow: true

  property string label: ""
  property string description: ""
  property string hint: ""
  property string detail: ""
  property string query: ""
  property var keywords: []
  property string valueText: ""
  property bool available: true
  property bool sectionHelp: true

  signal clicked()

  readonly property string searchHaystack: {
    var parts = [label, description, hint, valueText, detail]
    var list = keywords || []
    for (var i = 0; i < list.length; i++) parts.push(list[i])
    return ShellConfigJs.joinSearchHaystack(parts)
  }

  readonly property bool matches: ShellConfigJs.haystackMatches(query, searchHaystack)

  readonly property int controlCol: {
    var avail = parent ? parent.width : Theme.controlColumnWidth
    return Math.min(Theme.controlColumnWidth, Math.max(140, avail - 200))
  }

  readonly property int trailWidth: {
    var gap = Theme.space
    var chev = Math.ceil(chevron.implicitWidth)
    var val = root.valueText.length > 0 ? Math.ceil(valueCol.implicitWidth) : 0
    var needed = val > 0 ? val + gap + chev : chev
    return Math.min(root.controlCol, Math.max(chev, needed))
  }

  visible: available && matches
  width: parent ? parent.width : 640
  implicitWidth: width
  implicitHeight: visible
    ? Math.max(textCol.implicitHeight, trail.implicitHeight) + Theme.rowPad * 2
    : 0
  height: implicitHeight

  activeFocusOnTab: true

  Accessible.role: Accessible.Button
  Accessible.name: label
  Accessible.onPressAction: root.clicked()

  Keys.onReturnPressed: root.clicked()
  Keys.onSpacePressed: root.clicked()

  readonly property bool highlight: linkMouse.containsMouse || root.activeFocus

  Rectangle {
    anchors.fill: parent
    color: root.highlight ? Theme.fill(Theme.hoverFill) : "transparent"
    radius: Theme.radius

    Behavior on color {
      ColorAnimation { duration: 90 }
    }
  }

  Item {
    id: rowBody
    anchors.fill: parent
    anchors.margins: Theme.rowPad

    Column {
      id: textCol
      anchors.left: parent.left
      anchors.right: trail.left
      anchors.rightMargin: Theme.spaceMd
      anchors.verticalCenter: parent.verticalCenter
      spacing: 4

      PrefsText {
        width: parent.width
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
      id: trail
      anchors.right: parent.right
      anchors.verticalCenter: parent.verticalCenter
      width: root.trailWidth
      implicitHeight: Math.max(valueCol.implicitHeight, chevron.implicitHeight)
      height: implicitHeight

      Text {
        id: valueCol
        visible: root.valueText.length > 0
        anchors.right: chevron.left
        anchors.rightMargin: Theme.space
        width: Math.min(implicitWidth, parent.width - chevron.implicitWidth - Theme.space)
        anchors.verticalCenter: parent.verticalCenter
        text: root.valueText
        color: root.highlight ? Theme.foreground : Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        elide: Text.ElideMiddle
        horizontalAlignment: Text.AlignRight

        Behavior on color {
          ColorAnimation { duration: 90 }
        }
      }

      PrefsIcon {
        id: chevron
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        name: Theme.iconChevronRight
        size: Theme.titleSize
        color: root.highlight ? Theme.foreground : Theme.muted
        scale: root.highlight ? 1.08 : 1

        Behavior on color {
          ColorAnimation { duration: 90 }
        }
        Behavior on scale {
          NumberAnimation { duration: 90; easing.type: Easing.OutCubic }
        }
      }
    }
  }

  MouseArea {
    id: linkMouse
    z: 0
    anchors.fill: parent
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: root.clicked()
  }
}
