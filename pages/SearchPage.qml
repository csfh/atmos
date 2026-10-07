import QtQuick
import Quickshell.Io
import "../components"
import "../services"

Item {
  id: root

  readonly property bool searchPane: true
  property string query: ""
  property var navigator: null
  property var hits: []
  property string searchError: ""

  readonly property bool hasHits: root.hits.length > 0

  property int hitIndex: 0
  property string pendingQuery: ""
  property bool restartForLocale: false

  Connections {
    target: I18n
    function onLocaleChanged() {
      if (searchProc.running) {
        root.restartForLocale = true
        searchProc.running = false
      } else root.runQuery()
    }
  }

  onHitsChanged: {
    if (root.hitIndex >= root.hits.length)
      root.hitIndex = Math.max(0, root.hits.length - 1)
  }

  function moveHit(delta) {
    if (!root.hits.length) return
    var next = root.hitIndex + Number(delta)
    if (next < 0) next = 0
    if (next > root.hits.length - 1) next = root.hits.length - 1
    root.hitIndex = next
  }

  function jumpHit(toEnd) {
    if (!root.hits.length) return
    root.hitIndex = toEnd ? root.hits.length - 1 : 0
  }

  function activateHit() {
    if (!root.hits.length) return
    var hit = root.hits[root.hitIndex]
    if (!hit) return
    if (root.navigator && root.navigator.go)
      root.navigator.go(hit.hub, hit.label)
  }

  function revealHit(item) {
    if (!item) return
    var pos = item.mapToItem(flick.contentItem, 0, 0)
    if (pos.y !== pos.y) return
    var top = pos.y
    var bottom = top + item.height
    var viewH = flick.height
    var maxY = Math.max(0, flick.contentHeight - viewH)
    var y = flick.contentY
    if (top < y)
      flick.contentY = Math.max(0, Math.min(maxY, top))
    else if (bottom > y + viewH)
      flick.contentY = Math.max(0, Math.min(maxY, bottom - viewH))
  }

  function sendQuery(text) {
    searchProc.write(JSON.stringify({ cmd: "query", query: text }) + "\n")
  }

  function runQuery() {
    if (root.query.length === 0) {
      root.hits = []
      root.searchError = ""
      root.pendingQuery = ""
      return
    }
    root.pendingQuery = root.query
    if (!searchProc.running) {
      searchProc.running = true
      return
    }
    root.sendQuery(root.pendingQuery)
  }

  onQueryChanged: {
    root.hitIndex = 0
    searchDebounce.restart()
  }

  Timer {
    id: searchDebounce
    interval: 80
    repeat: false
    onTriggered: root.runQuery()
  }

  Component.onCompleted: root.runQuery()

  Process {
    id: searchProc
    command: ["node", Omarchy.shellDir + "/services/SearchIndex.js", "serve", "--root", Omarchy.shellDir, "--locale", I18n.locale]
    stdinEnabled: true
    stdout: SplitParser {
      onRead: function(line) {
        try {
          var parsed = JSON.parse(String(line || "{}"))
          if (String(parsed.query || "") !== root.query) return
          root.searchError = ""
          root.hits = Array.isArray(parsed.hits) ? parsed.hits : []
        } catch (e) {
          root.hits = []
          root.searchError = I18n.tr("Search returned invalid JSON")
        }
      }
    }
    stderr: StdioCollector {
      id: searchErr
      waitForEnd: false
    }
    onStarted: {
      if (root.pendingQuery.length > 0) root.sendQuery(root.pendingQuery)
    }
    onExited: function(code) {
      if (root.restartForLocale) {
        root.restartForLocale = false
        Qt.callLater(root.runQuery)
        return
      }
      if (code !== 0 && root.query.length > 0) {
        root.hits = []
        root.searchError = String(searchErr.text || "Search failed").replace(/^\s+|\s+$/g, "")
      }
    }
  }

  PrefsFlickable {
    id: flick
    anchors.fill: parent
    clip: true
    contentHeight: pageColumn.implicitHeight + Theme.spaceLg * 2

    Column {
      id: pageColumn
      width: Theme.contentColumnWidth(flick.width)
      x: Theme.contentColumnX(flick.width, width)
      y: Theme.spaceLg
      spacing: Theme.spaceLg

      Column {
        width: parent.width
        spacing: 4

        PrefsText {
          width: parent.width
          text: I18n.tr("Search")
          color: Theme.foreground
          font.family: Theme.fontFamily
          font.pixelSize: Theme.titleSize
          font.bold: true
        }

        PrefsText {
          width: parent.width
          text: root.searchError.length > 0
            ? root.searchError
            : (root.hasHits
              ? I18n.tr("Matching settings across every page for “{query}”.", { query: root.query })
              : I18n.tr("Nothing on any page mentions “{query}”. Try another word — ? shows keyboard shortcuts.", { query: root.query }))
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.fontSize
        }
      }

      Repeater {
        model: root.hits
        delegate: PrefsLink {
          id: hitLink
          required property int index
          required property var modelData
          width: pageColumn.width
          query: ""
          picked: index === root.hitIndex
          label: modelData.label || ""
          description: modelData.description || ""
          hint: modelData.hint || ""
          detail: modelData.detail || ""
          valueText: I18n.tr(modelData.hubTitle || modelData.hub || "")
          onPickedChanged: if (picked) root.revealHit(hitLink)
          Component.onCompleted: if (picked) Qt.callLater(function() { root.revealHit(hitLink) })
          onClicked: {
            // hub is a hub id or a hub/subpage path such as windows/bindings.
            // Pass the label so Simple can pin the landing row after chrome
            // search clears the query.
            root.hitIndex = index
            if (root.navigator && root.navigator.go)
              root.navigator.go(modelData.hub, modelData.label)
          }
        }
      }
    }
  }
}
