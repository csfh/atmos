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

  function runQuery() {
    if (root.query.length === 0) {
      root.hits = []
      root.searchError = ""
      return
    }
    if (searchProc.running) searchProc.running = false
    searchProc.running = true
  }

  onQueryChanged: searchDebounce.restart()

  Timer {
    id: searchDebounce
    interval: 80
    repeat: false
    onTriggered: root.runQuery()
  }

  Component.onCompleted: root.runQuery()

  Process {
    id: searchProc
    command: ["node", Omarchy.shellDir + "/services/SearchIndex.js", "query", root.query, "--root", Omarchy.shellDir]
    stdout: StdioCollector {
      id: searchOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: searchErr
      waitForEnd: true
    }
    onExited: function(code) {
      if (code !== 0) {
        root.hits = []
        root.searchError = String(searchErr.text || "Search failed").replace(/^\s+|\s+$/g, "")
        return
      }
      root.searchError = ""
      try {
        var parsed = JSON.parse(String(searchOut.text || "[]"))
        root.hits = Array.isArray(parsed) ? parsed : []
      } catch (e) {
        root.hits = []
        root.searchError = "Search returned invalid JSON"
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
          text: "Search"
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
              ? "Matching settings across every page for “" + root.query + "”."
              : "Nothing on any page mentions “" + root.query + "”. Try another word.")
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.fontSize
        }
      }

      Repeater {
        model: root.hits
        delegate: PrefsLink {
          required property var modelData
          width: pageColumn.width
          query: ""
          label: modelData.label || ""
          description: modelData.description || ""
          hint: modelData.hint || ""
          detail: modelData.detail || ""
          valueText: modelData.hubTitle || modelData.hub || ""
          onClicked: {
            if (root.navigator && root.navigator.go)
              root.navigator.go(modelData.hub)
          }
        }
      }
    }
  }
}
