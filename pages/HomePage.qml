import QtQuick
import "../components"
import "../services"
import "../services/Hardware.js" as HardwareJs
import "../services/Hubs.js" as HubsJs
import "../services/LiveStats.js" as LiveStatsJs
import "../services/Processes.js" as ProcessesJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "home"
  title: "Home"
  description: "How this machine is doing right now. Monitor is the full process-manager. Machine under System is the identity page."

  property var navigator: null
  property string procQuery: ""
  property string procSort: "cpu"
  property var pendingProc: null
  property string pendingSignal: ""

  readonly property var history: LiveStatsStore.history
  readonly property var latest: LiveStatsStore.latest
  readonly property var processes: ProcessesJs.list(
    root.latest && root.latest.processes ? root.latest.processes : [],
    root.procQuery,
    root.procSort,
    { scope: "mine", uid: root.latest ? root.latest.uid : null }
  )
  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)
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

  function heroLine() {
    if (root.history.length === 0) return "Sampling live stats…"
    var parts = []
    var cpu = root.latest ? LiveStatsJs.formatPercent(root.latest.cpu) : ""
    if (cpu) parts.push("CPU " + cpu)
    if (root.latest && root.latest.mem != null) {
      var mem = LiveStatsJs.formatPercent(root.latest.mem)
      if (mem) parts.push("MEM " + mem)
    }
    var net = LiveStatsJs.formatNet(root.latest)
    if (net) parts.push(net)
    var temp = LiveStatsJs.formatTemp(root.latest && root.latest.cpuTemp)
    if (temp) parts.push(temp)
    return parts.length > 0 ? parts.join(" · ") : "Sampling live stats…"
  }

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

  // Starred settings first, then the pages most people open first.
  readonly property var quickLinks: {
    var out = []
    var favs = Omarchy.favoriteItems || []
    var i
    for (i = 0; i < favs.length && out.length < 5; i++) {
      var f = favs[i]
      if (!f || !f.hub || !f.label) continue
      out.push({ hub: String(f.hub), label: String(f.label), description: String(f.hubTitle || HubsJs.hubTitle(f.hub) || ""), starred: true })
    }
    if (out.length > 0) return out
    var defaults = ["appearance", "display", "network", "sound", "system"]
    for (i = 0; i < defaults.length; i++) {
      var title = HubsJs.hubTitle(defaults[i])
      if (title) out.push({ hub: defaults[i], label: title, description: "", starred: false })
    }
    return out
  }

  PrefsGroup {
    title: "Quick access"
    query: root.query
    lede: "Star a setting on any page and it appears here."

    Repeater {
      model: root.quickLinks

      PrefsLink {
        required property var modelData
        available: !!(root.navigator && root.navigator.go)
        label: modelData.label
        description: modelData.description
        query: root.query
        keywords: ["quick", "favorite", "shortcut"]
        onClicked: root.navigator.go(modelData.hub, modelData.starred ? modelData.label : "")
      }
    }
  }

  PrefsGroup {
    title: "Activity"
    query: root.query
    detail: "Samples stay in this window. Monitor keeps the same poller and adds per-core, disk, traffic, and sensors. Nothing is written to disk."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.labelGap

      PrefsText {
        width: parent.width
        text: root.heroLine()
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.embedTitleSize
        font.bold: true
      }

      PrefsSkeleton {
        width: parent.width
        visible: root.history.length === 0
        active: visible
      }
    }

    SettingRow {
      label: "Processor"
      description: "Share of time the CPUs were busy, from /proc/stat."
      hint: "/proc/stat"
      query: root.query
      keywords: ["cpu", "load", "sparkline"]
      valueText: root.cpuCaption()
      stretchControl: true

      PrefsSparkline {
        id: cpuSpark
        width: parent.width
        height: Math.round(Theme.sparklineHeight * 1.6)
        values: {
          if (!cpuSpark.inView) return []
          return LiveStatsJs.series(root.history, "cpu")
        }
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
          id: memSpark
          width: parent.width
          height: Math.round(Theme.sparklineHeight * 1.6)
          values: {
            if (!memSpark.inView) return []
            return LiveStatsJs.series(root.history, "mem")
          }
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
        id: netSpark
        width: parent.width
        height: Math.round(Theme.sparklineHeight * 1.6)
        values: {
          if (!netSpark.inView) return []
          return LiveStatsJs.series(root.history, "rxBps")
        }
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
        id: cpuTempSpark
        width: parent.width
        values: {
          if (!cpuTempSpark.inView) return []
          return LiveStatsJs.series(root.history, "cpuTemp")
        }
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
          id: gpuSpark
          width: parent.width
          values: {
            if (!gpuSpark.inView) return []
            return LiveStatsJs.gpuSeries(root.history, modelData)
          }
          valueText: root.gpuCaption(modelData)
        }
      }
    }
  }

  PrefsGroup {
    title: "Processes"
    query: root.query
    wide: true
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

    PrefsEmpty {
      available: root.processes.length === 0
      sectionHelp: false
      label: "No matching processes"
      description: root.latest ? "Nothing matches that search." : "Waiting for the first sample."
      query: root.query
      keywords: ["empty", "process"]
    }

    SettingRow {
      available: !!(root.navigator && root.navigator.go)
      label: "Monitor"
      description: "Per-core load, memory composition, disk I/O, traffic, sensors, and a full process table."
      query: root.query
      keywords: ["monitor", "htop", "btop", "process"]

      PrefsButton {
        text: "Configure…"
        primary: true
        onClicked: root.navigator.go("monitor")
      }
    }
  }
}
