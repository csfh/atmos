import QtQuick
import Quickshell.Io
import "../components"
import "../services"
import "../services/Hardware.js" as HardwareJs
import "../services/LiveStats.js" as LiveStatsJs
import "../services/Processes.js" as ProcessesJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "home"
  title: "Home"
  description: "How this machine is doing right now. Machine under System is the identity page."

  property var history: []
  property string procQuery: ""
  property string procSort: "cpu"
  property var pendingProc: null
  property string pendingSignal: ""

  readonly property var latest: LiveStatsJs.latest(root.history)
  readonly property var processes: ProcessesJs.list(
    root.latest && root.latest.processes ? root.latest.processes : [],
    root.procQuery,
    root.procSort
  )
  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)
  readonly property var cpuValues: LiveStatsJs.series(root.history, "cpu")
  readonly property var memValues: LiveStatsJs.series(root.history, "mem")
  readonly property var netValues: LiveStatsJs.series(root.history, "rxBps")
  readonly property var cpuTempValues: LiveStatsJs.series(root.history, "cpuTemp")
  readonly property var gpuList: LiveStatsJs.gpuRows(root.hw.gpus, root.latest)

  function cpuCaption() {
    var pct = root.latest ? LiveStatsJs.formatPercent(root.latest.cpu) : ""
    return pct || "waiting for samples"
  }

  function memCaption() {
    if (!root.latest || root.latest.memUsed === null || root.latest.memTotal === null)
      return "waiting for samples"
    var used = RichUi.formatBytes(LiveStatsJs.memBytes(root.latest.memUsed))
    var total = RichUi.formatBytes(LiveStatsJs.memBytes(root.latest.memTotal))
    var pct = LiveStatsJs.formatPercent(root.latest.mem)
    return used + " of " + total + (pct ? " (" + pct + ")" : "")
  }

  function netCaption() {
    var text = LiveStatsJs.formatNet(root.latest)
    return text || "waiting for samples"
  }

  function cpuTempCaption() {
    if (root.history.length === 0) return "waiting for samples"
    return LiveStatsJs.formatTemp(root.latest && root.latest.cpuTemp) || "unknown"
  }

  function gpuName(gpu) {
    return (gpu && gpu.name) || "GPU"
  }

  function gpuDescription(gpu) {
    var summary = HardwareJs.gpuSummary(gpu)
    if (summary) return summary
    var parts = []
    if (gpu && gpu.vendor) parts.push(gpu.vendor)
    if (gpu && gpu.driver) parts.push(gpu.driver)
    return parts.join(" · ")
  }

  function gpuCaption(gpu) {
    if (root.history.length === 0) return "waiting for samples"
    var t = LiveStatsJs.gpuTempAt(root.latest, gpu)
    if (t == null) return "unknown"
    var text = LiveStatsJs.formatTemp(t)
    return LiveStatsJs.gpuOwnTemp(root.latest, gpu) == null ? text + " (package)" : text
  }

  function gpuValues(gpu) {
    return LiveStatsJs.gpuSeries(root.history, gpu)
  }

  function pollStats() {
    if (statsProc.running) return
    statsProc.running = true
  }

  function adoptSample(text) {
    var parsed = LiveStatsJs.parse(text)
    if (!parsed) return
    root.history = LiveStatsJs.pushSample(root.history, parsed, Date.now())
  }

  function actOn(row, action) {
    if (!row || !action) return
    if (action === "copy") {
      Omarchy.copyText(String(row.pid))
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
    root.pollStats()
  }

  Timer {
    interval: 2000
    running: true
    repeat: true
    onTriggered: root.pollStats()
  }

  Process {
    id: statsProc
    command: ["python3", Omarchy.liveStatsScript]
    stdout: StdioCollector {
      id: statsOut
      waitForEnd: true
    }
    onExited: function(code) {
      if (code === 0) root.adoptSample(statsOut.text)
    }
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
    title: "Activity"
    query: root.query
    detail: "Samples stay in this window. Leaving Home stops the poll. Nothing is written to disk."

    SettingRow {
      label: "Processor"
      description: "Share of time the CPUs were busy, from /proc/stat."
      hint: "/proc/stat"
      query: root.query
      keywords: ["cpu", "load", "sparkline"]
      valueText: root.cpuCaption()
      stretchControl: true

      PrefsSparkline {
        width: parent.width
        values: root.cpuValues
        valueText: root.cpuCaption()
      }
    }

    SettingRow {
      label: "Memory"
      description: "In use against what is fitted."
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["memory", "ram"]
      valueText: root.memCaption()
      stretchControl: true

      Column {
        width: parent.width
        spacing: Theme.labelGap

        PrefsSparkline {
          width: parent.width
          values: root.memValues
          valueText: root.memCaption()
        }

        PrefsProgress {
          width: parent.width
          visible: !!(root.latest && root.latest.mem != null)
          from: 0
          to: 100
          value: root.latest && root.latest.mem != null ? root.latest.mem : 0
        }
      }
    }

    SettingRow {
      label: "Network"
      description: "Receive and transmit on non-loopback interfaces."
      hint: "/proc/net/dev"
      query: root.query
      keywords: ["network", "bandwidth", "rx", "tx"]
      valueText: root.netCaption()
      stretchControl: true

      PrefsSparkline {
        width: parent.width
        values: root.netValues
        valueText: root.netCaption()
      }
    }
  }

  PrefsGroup {
    title: "Thermal"
    query: root.query
    detail: "Package and GPU sensors. GPU names come from Hardware. A missing reading stays unknown."

    SettingRow {
      label: "Processor"
      description: "CPU package temperature."
      hint: "/sys/class/hwmon · /sys/class/thermal"
      query: root.query
      keywords: ["temperature", "cpu", "heat", "thermal", "package"]
      valueText: root.cpuTempCaption()
      stretchControl: true

      PrefsSparkline {
        width: parent.width
        values: root.cpuTempValues
        valueText: root.cpuTempCaption()
      }
    }

    Repeater {
      model: root.gpuList

      SettingRow {
        required property var modelData
        label: root.gpuName(modelData)
        description: root.gpuDescription(modelData)
        hint: "/sys/class/drm"
        query: root.query
        keywords: ["temperature", "gpu", "heat", "thermal", "graphics"]
        valueText: root.gpuCaption(modelData)
        stretchControl: true

        PrefsSparkline {
          width: parent.width
          values: root.gpuValues(modelData)
          valueText: root.gpuCaption(modelData)
        }
      }
    }
  }

  PrefsGroup {
    title: "Processes"
    query: root.query
    detail: "This user's processes. End sends SIGTERM. Force quit sends SIGKILL. Pid 1 and Atmos itself are refused."
    hint: "/proc"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      PrefsField {
        width: parent.width
        placeholder: "Search processes…"
        onEdited: function(value) { root.procQuery = value }
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
      description: root.latest ? "Nothing matches that search." : "Waiting for the first sample."
      query: root.query
      keywords: ["empty", "process"]
    }
  }
}
