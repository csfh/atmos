import QtQuick
import "../services"

Item {
  id: root

  property string title: ""
  property string description: ""
  property string query: ""
  property bool embed: false
  default property alias extra: sections.data
  readonly property alias prefsOverlay: overlayLayer
  readonly property bool hasSections: sections.implicitHeight > 0

  width: parent ? parent.width : 640
  implicitWidth: width
  implicitHeight: root.embed
    ? (root.hasSections ? pageColumn.implicitHeight + (root.query.length > 0 ? 0 : Theme.pageMargin) : 0)
    : 0
  height: root.embed ? implicitHeight : (parent ? parent.height : 400)
  visible: !root.embed || root.hasSections

  PrefsFlickable {
    id: flick
    anchors.fill: parent
    clip: !root.embed
    interactive: !root.embed && contentHeight > height
    contentHeight: pageColumn.implicitHeight + (root.embed ? 0 : Theme.pageMargin * 2)

    Column {
      id: pageColumn
      width: Theme.contentColumnWidth(flick.width)
      x: Theme.contentColumnX(flick.width, width)
      y: root.embed ? 0 : Theme.pageMargin
      spacing: Theme.spaceLg

      Column {
        width: parent.width - Theme.copyInset * 2
        x: Theme.copyInset
        spacing: Theme.titleGap
        visible: (root.title.length > 0 || root.description.length > 0) && (root.query.length === 0 || root.hasSections)

        PrefsText {
          width: parent.width
          visible: root.title.length > 0
          text: root.title
          color: Theme.foreground
          font.family: Theme.fontFamily
          font.pixelSize: root.embed ? Theme.embedTitleSize : Theme.pageTitleSize
          font.bold: true
        }

        PrefsText {
          width: parent.width
          visible: root.description.length > 0 && root.query.length === 0
          text: root.description
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.pageDescriptionSize
        }
      }

      Column {
        id: sections
        width: parent.width
        spacing: Theme.sectionSpacing
      }
    }
  }

  Item {
    id: overlayLayer
    anchors.fill: parent
    z: 20
  }
}
