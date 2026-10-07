import QtQuick
import "../services"
import "../services/Disclosure.js" as DisclosureJs
import "../services/Layout.js" as LayoutJs

Item {
  id: root

  property string title: ""
  property string description: ""
  property string query: ""
  property string hubId: ""
  property bool embed: false
  // Grow to the two-column cap even when this page has one full-width block.
  property bool expandContent: false
  default property alias extra: sections.data
  readonly property alias prefsOverlay: overlayLayer
  readonly property bool prefsPage: true
  readonly property bool hasSections: sections.implicitHeight > 0
  property var expandedChart: null
  readonly property bool chartOpen: expandedChart !== null

  function rightPaneHost() {
    var p = parent
    while (p) {
      if (p.atmosRightPane === true) return p
      p = p.parent
    }
    return root
  }

  function expandChart(source) {
    if (root.embed || !source) return
    var host = root.rightPaneHost()
    if (chartExpandLayer.parent !== host)
      chartExpandLayer.parent = host
    root.expandedChart = source
  }

  function collapseChart() {
    if (expandView) expandView.treemapFocus = ""
    root.expandedChart = null
    if (chartExpandLayer.parent !== root)
      chartExpandLayer.parent = root
  }

  function zoomOutChart() {
    return !!(expandView && expandView.zoomOut && expandView.zoomOut())
  }

  Component.onDestruction: root.collapseChart()

  function treeHasAdvanced(node, depth) {
    if (!node || depth > 24) return false
    if (node.advanced === true) return true
    var kids = node.children
    if (!kids) return false
    var i
    for (i = 0; i < kids.length; i++) {
      if (treeHasAdvanced(kids[i], depth + 1)) return true
    }
    return false
  }

  readonly property bool hasAdvanced: {
    var _n = sections.children.length
    return treeHasAdvanced(sections, 0)
  }

  readonly property bool showDisclosure: DisclosureJs.showModeToggle(root.hasAdvanced, {
    embed: root.embed,
    query: root.query
  })

  width: parent ? parent.width : 640
  implicitWidth: width
  implicitHeight: root.embed
    ? (root.hasSections ? pageColumn.implicitHeight + (root.query.length > 0 ? 0 : Theme.pageMargin) : 0)
    : 0
  height: root.embed ? implicitHeight : (parent ? parent.height : 400)
  visible: !root.embed || root.hasSections

  readonly property int visibleGridCount: {
    var _n = sections.children.length
    var _q = root.query
    var _simple = Disclosure.simple
    var kids = []
    var i
    for (i = 0; i < sections.children.length; i++)
      kids.push(sections.children[i])
    return LayoutJs.countGridSections(kids)
  }

  readonly property int pageColumnWidth: LayoutJs.pageContentWidth(flick.width, {
    margin: Theme.pageMargin,
    cap: Theme.contentMaxWidth,
    wideCap: Theme.contentWideMaxWidth,
    minColumn: Theme.sectionMinColumn,
    gap: Theme.sectionSpacing,
    maxColumns: Theme.sectionMaxColumns,
    minWidth: 240,
    itemCount: root.expandContent ? 8 : root.visibleGridCount
  })

  readonly property int sectionColumns: LayoutJs.sectionColumnCount(
    root.pageColumnWidth,
    Theme.sectionMinColumn,
    Theme.sectionSpacing,
    Theme.sectionMaxColumns,
    root.visibleGridCount
  )

  readonly property int sectionColumnWidth: LayoutJs.sectionColumnWidth(
    root.pageColumnWidth,
    root.sectionColumns,
    Theme.sectionSpacing
  )

  readonly property int sectionFullWidth: root.pageColumnWidth

  PrefsFlickable {
    id: flick
    anchors.fill: parent
    clip: !root.embed
    interactive: !root.embed && contentHeight > height && !root.chartOpen
    contentHeight: pageColumn.implicitHeight + (root.embed ? 0 : Theme.pageMargin * 2)

    Column {
      id: pageColumn
      width: root.pageColumnWidth
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
          text: I18n.tr(root.title)
          color: Theme.foreground
          font.family: Theme.fontFamily
          font.pixelSize: root.embed ? Theme.embedTitleSize : Theme.pageTitleSize
          font.bold: true
        }

        PrefsText {
          width: parent.width
          visible: root.description.length > 0 && root.query.length === 0
          text: I18n.tr(root.description)
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.pageDescriptionSize
        }
      }

      Flow {
        id: sections
        width: parent.width
        spacing: Theme.sectionSpacing
      }
    }
  }

  Item {
    id: chartExpandLayer
    anchors.fill: parent
    z: 15
    visible: root.chartOpen
    enabled: root.chartOpen

    Accessible.role: Accessible.Button
    Accessible.name: {
      var title = root.expandedChart && root.expandedChart.valueText
        ? String(root.expandedChart.valueText)
        : "chart"
      return title + ". Click or Escape to close"
    }
    Accessible.onPressAction: root.collapseChart()

    Rectangle {
      anchors.fill: parent
      color: Theme.background
    }

    MouseArea {
      id: expandDismiss
      anchors.fill: parent
      hoverEnabled: true
      cursorShape: Qt.PointingHandCursor
      onClicked: root.collapseChart()
    }

    Item {
      id: expandBody
      anchors.fill: parent
      anchors.margins: Theme.pageMargin

      PrefsText {
        id: expandTitle
        width: parent.width
        text: root.expandedChart && root.expandedChart.valueText
          ? String(root.expandedChart.valueText)
          : (root.expandedChart && root.expandedChart.kind ? String(root.expandedChart.kind) : "")
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.pageTitleSize
        font.bold: true
      }

      PrefsText {
        id: expandBlurb
        y: expandTitle.height + Theme.labelGap
        width: parent.width
        visible: text.length > 0
        text: root.expandedChart && root.expandedChart.resolvedExplainer
          ? String(root.expandedChart.resolvedExplainer)
          : ""
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.descriptionSize
      }

      PrefsText {
        id: expandLegend
        y: expandBlurb.y + (expandBlurb.visible ? expandBlurb.height + Theme.labelGap : 0)
        width: parent.width
        visible: text.length > 0
        text: root.expandedChart && root.expandedChart.resolvedLegend
          ? String(root.expandedChart.resolvedLegend)
          : ""
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.descriptionSize
      }

      PrefsChart {
        id: expandView
        y: expandTitle.height + Theme.headingGap
          + (expandBlurb.visible ? expandBlurb.height + Theme.headingGap : 0)
          + (expandLegend.visible ? expandLegend.height + Theme.headingGap : 0)
        width: parent.width
        height: Math.max(Theme.chartHeight, parent.height - y)
        expandable: false
        kind: root.expandedChart ? root.expandedChart.kind : ""
        model: root.expandedChart ? root.expandedChart.model : null
        modelB: root.expandedChart ? root.expandedChart.modelB : null
        valueText: root.expandedChart ? root.expandedChart.valueText : ""
      }
    }
  }

  Item {
    id: overlayLayer
    anchors.fill: parent
    z: 20
  }
}
