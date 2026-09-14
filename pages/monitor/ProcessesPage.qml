import QtQuick
import "../../components"
import "../../services"
import "../../services/LiveStats.js" as LiveStatsJs
import "../../services/Processes.js" as ProcessesJs

PrefsPage {
  id: root
  hubId: "monitor/processes"
  title: "Processes"
  description: "Search, filter, sort, and signal tasks. End sends SIGTERM. Force quit sends SIGKILL. Pid 1 and Atmos itself are refused."

  property string procQuery: ""
  property string procSort: "cpu"
  property string procScope: "mine"
  property string procState: "all"
  property real minCpu: 0
  property real minRssKb: 0
  property int procCap: 80
  property bool tree: false
  property bool reverse: false
  property var pendingProc: null
  property string pendingSignal: ""

  readonly property var latest: LiveStatsStore.latest
  readonly property var processes: ProcessesJs.list(
    root.latest && root.latest.processes ? root.latest.processes : [],
    root.procQuery,
    root.procSort,
    {
      scope: root.procScope,
      state: root.procState,
      uid: root.latest ? root.latest.uid : null,
      minCpu: root.minCpu,
      minRssKb: root.minRssKb,
      cap: root.procCap,
      tree: root.tree,
      reverse: root.reverse
    }
  )
  readonly property int rawCount: root.latest && root.latest.processes ? root.latest.processes.length : 0

  function actOn(row, action) {
    if (!row || !action) return
    if (action === "copy") {
      Omarchy.copyText(String(row.pid))
      return
    }
    if (action === "copyCmd") {
      Omarchy.copyText(String(row.cmdline || row.comm))
      return
    }
    if (action !== "term" && action !== "kill") return
    root.pendingProc = row
    root.pendingSignal = action === "kill" ? "KILL" : "TERM"
    signalConfirm.title = action === "kill" ? "Force quit this process" : "End this process"
    signalConfirm.message = action === "kill"
      ? "Send SIGKILL to " + row.comm + " (" + row.pid + "). It will not get a chance to exit cleanly."
      : "Send SIGTERM to " + row.comm + " (" + row.pid + ")."
    signalConfirm.confirmText = action === "kill" ? "Force quit" : "End"
    signalConfirm.destructive = true
    signalConfirm.ask()
  }

  function runPending() {
    var row = root.pendingProc
    var sig = root.pendingSignal
    root.pendingProc = null
    root.pendingSignal = ""
    if (!row || !sig) return
    Omarchy.signalProcess(row.pid, sig)
  }

  Component.onCompleted: {
    signalConfirm.parent = root.prefsOverlay
  }

  PrefsConfirm {
    id: signalConfirm
    onConfirmed: root.runPending()
    onCanceled: {
      root.pendingProc = null
      root.pendingSignal = ""
    }
  }

  PrefsGroup {
    title: ""
    query: root.query
    framed: false
    catalog: false
    detail: "Filters stay in this window. They are not written to disk."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      Text {
        width: parent.width
        text: root.processes.length + " shown  ·  " + root.rawCount + " sampled"
          + (LiveStatsStore.paused ? "  ·  paused" : "")
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
      }

      PrefsField {
        width: parent.width
        placeholder: "Search name, command, pid, state…"
        onEdited: function(value) { root.procQuery = value }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: ProcessesJs.scopeChips()

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.procScope === (modelData && modelData.id ? modelData.id : "")
            onClicked: root.procScope = modelData.id
          }
        }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: ProcessesJs.stateChips()

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.procState === (modelData && modelData.id ? modelData.id : "")
            onClicked: root.procState = modelData.id
          }
        }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: ProcessesJs.sortChips()

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.procSort === (modelData && modelData.id ? modelData.id : "")
            onClicked: root.procSort = modelData.id
          }
        }

        PrefsButton {
          text: root.reverse ? "Low first" : "High first"
          onClicked: root.reverse = !root.reverse
        }

        PrefsButton {
          text: root.tree ? "Tree" : "Flat"
          primary: root.tree
          onClicked: root.tree = !root.tree
        }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: ProcessesJs.minCpuChips()

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.minCpu === (modelData && modelData.value != null ? modelData.value : -1)
            onClicked: root.minCpu = modelData.value
          }
        }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: ProcessesJs.minRssChips()

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.minRssKb === (modelData && modelData.value != null ? modelData.value : -1)
            onClicked: root.minRssKb = modelData.value
          }
        }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: ProcessesJs.capChips()

          PrefsButton {
            required property var modelData
            text: (modelData && modelData.label ? modelData.label : "") + " rows"
            primary: root.procCap === (modelData && modelData.value ? modelData.value : 0)
            onClicked: root.procCap = modelData.value
          }
        }
      }
    }

    Repeater {
      model: root.processes

      ProcessRow {
        required property var modelData
        procRow: modelData
        query: root.query
        onActed: function(action) { root.actOn(modelData, action) }
      }
    }

    SettingRow {
      available: root.processes.length === 0
      sectionHelp: false
      label: "No matching processes"
      description: root.latest ? "Nothing matches that search or filter." : "Waiting for the first sample."
      query: root.query
      keywords: ["empty", "process", "filter"]
    }
  }
}
