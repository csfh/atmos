import QtQuick
import Quickshell
import Quickshell.Io
import "../components"
import "../services"
import "../services/Systemd.js" as SystemdJs

PrefsPage {
  id: root
  hubId: "services"
  title: "Services"
  description: "Start, stop, and enable stay on the allowlist. Other units are status and logs."

  property string unitFilter: ""
  property string stateFilter: "all"
  property string outputTitle: ""
  property string outputText: ""

  readonly property var allRows: {
    var list = Omarchy.systemdUnits || []
    var out = []
    var i, row
    for (i = 0; i < list.length; i++) {
      row = SystemdJs.normalizeUnit(list[i])
      if (row) out.push(row)
    }
    return out
  }

  readonly property var rows: SystemdJs.listUnits(root.allRows, root.unitFilter, root.stateFilter)
  readonly property var summary: SystemdJs.summarize(root.allRows)
  readonly property var summaryBits: SystemdJs.summaryParts(root.summary)
  readonly property var chips: SystemdJs.filterChips()

  function actOn(row, action) {
    if (!row || !action) return
    if (action === "copy") {
      Omarchy.copyText(row.unit)
      return
    }
    if (action === "status") {
      root.showOutput("Status · " + row.unit, root.statusArgv(row))
      return
    }
    if (action === "logs") {
      root.showOutput("Logs · " + row.unit, root.logsArgv(row))
      return
    }
    if (!row.allowed) return
    Omarchy.systemdAction(action, row.unit, row.scope)
  }

  function statusArgv(row) {
    var argv = ["systemctl"]
    if (row.scope === "user") argv.push("--user")
    argv.push("--no-pager", "--full", "status", row.unit)
    return argv
  }

  function logsArgv(row) {
    var argv = ["journalctl"]
    if (row.scope === "user") argv.push("--user")
    else argv.push("--system")
    argv.push("-u", row.unit, "-n", "80", "--no-pager")
    return argv
  }

  function showOutput(title, argv) {
    root.outputTitle = title
    root.outputText = "Reading…"
    outputProc.command = argv
    outputProc.running = true
    outputDialog.open()
  }

  function selectFilter(id) {
    root.stateFilter = String(id || "all")
  }

  PrefsGroup {
    title: ""
    query: root.query
    framed: false
    catalog: false

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      Flow {
        width: parent.width
        spacing: Theme.spaceMd

        Repeater {
          model: root.summaryBits

          Item {
            required property var modelData
            implicitWidth: bitText.implicitWidth
            implicitHeight: Math.max(Theme.descriptionSize + 4, bitText.implicitHeight)
            width: implicitWidth
            height: implicitHeight
            activeFocusOnTab: !!(modelData && modelData.id === "failed" && modelData.failed)

            Text {
              id: bitText
              anchors.verticalCenter: parent.verticalCenter
              text: modelData && modelData.text ? modelData.text : ""
              color: modelData && modelData.failed ? Theme.urgent : Theme.muted
              font.family: Theme.fontFamily
              font.pixelSize: Theme.descriptionSize
            }

            MouseArea {
              anchors.fill: parent
              enabled: !!(modelData && modelData.id === "failed" && modelData.failed)
              hoverEnabled: enabled
              cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
              onClicked: root.selectFilter("failed")
            }

            Keys.onReturnPressed: {
              if (modelData && modelData.failed) root.selectFilter("failed")
            }
            Keys.onSpacePressed: {
              if (modelData && modelData.failed) root.selectFilter("failed")
            }

            Accessible.role: modelData && modelData.failed ? Accessible.Button : Accessible.StaticText
            Accessible.name: modelData && modelData.text ? modelData.text : ""
            Accessible.onPressAction: {
              if (modelData && modelData.failed) root.selectFilter("failed")
            }
          }
        }
      }

      PrefsField {
        width: parent.width
        placeholder: "Search services…"
        onEdited: function(value) { root.unitFilter = value }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: root.chips

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.stateFilter === (modelData && modelData.id ? modelData.id : "")
            onClicked: root.selectFilter(modelData.id)
          }
        }
      }
    }

    Repeater {
      model: root.rows

      ServiceRow {
        required property var modelData
        unitRow: modelData
        query: root.query
        onActed: function(action) { root.actOn(modelData, action) }
      }
    }

    SettingRow {
      available: root.rows.length === 0
      sectionHelp: false
      label: "No matching services"
      description: root.allRows.length === 0
        ? "No units were reported for this session."
        : "Nothing matches that search or filter."
      query: root.query
      keywords: ["empty", "search", "filter"]
    }
  }

  PrefsDialog {
    id: outputDialog
    title: root.outputTitle

    PrefsFlickable {
      width: parent.width
      height: Math.min(320, Math.max(Theme.rowHeight * 6, Math.min(outputBody.implicitHeight, 320)))
      contentHeight: outputBody.implicitHeight
      clip: true

      PrefsText {
        id: outputBody
        width: parent.width
        text: root.outputText
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
        color: Theme.foreground
      }
    }

    PrefsButton {
      text: "Close"
      onClicked: outputDialog.close()
    }
  }

  Process {
    id: outputProc
    command: ["true"]
    stdout: StdioCollector {
      id: outputOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: outputErr
      waitForEnd: true
    }
    onExited: function(code) {
      var text = String(outputOut.text || "")
      var err = String(outputErr.text || "")
      if (text.replace(/^\s+|\s+$/g, "").length === 0) text = err
      if (text.replace(/^\s+|\s+$/g, "").length === 0)
        text = code === 0 ? "No output." : "Could not read that unit."
      root.outputText = text
    }
  }
}
