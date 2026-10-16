import QtQuick
import "../components"
import "../services"
import "../services/Requests.js" as Requests
import "../services/Sentinel.js" as SentinelJs

PrefsPage {
  id: root
  hubId: "sentinel"
  title: "Sentinel"
  description: "Processes that stay busy, hold a lot of memory, or keep a port open for days. Linux does not record when a socket opened, so a port's age is its process's age. Ports owned by root services show without a process."

  readonly property var thresholdKeys: ({
    cpuPercent: "sentinelCpuPercent",
    memoryPercent: "sentinelMemoryPercent",
    portAgeDays: "sentinelPortAgeDays"
  })

  property string kind: "flagged"
  property string filter: ""
  // Stored values by domain; an unset or unreadable one falls back to a default.
  property var stored: ({})
  property var tracked: SentinelJs.track(null, null, null)
  property var pendingProc: null
  property string pendingSignal: ""
  property bool holding: false

  readonly property var latest: LiveStatsStore.latest
  readonly property var thresholds: SentinelJs.thresholds(root.stored)
  readonly property var allRows: SentinelJs.rows(root.latest, root.tracked, root.thresholds)
  readonly property var rows: SentinelJs.list(root.allRows, root.kind, root.filter)
  readonly property var chips: SentinelJs.kindChips(root.allRows)
  readonly property string summary: LiveStatsStore.waiting
    ? "Reading the first sample…"
    : root.chips[0].count + " flagged · " + root.chips[3].count + " listening ports"

  onLatestChanged: root.tracked = SentinelJs.track(root.tracked, root.latest, root.thresholds)

  // Sentinel draws no chart, so it asks the live poll to stay on while it is
  // the page in view, and lets go as soon as it is not.
  function syncHold() {
    var want = root.visible
    if (want === root.holding) return
    root.holding = want
    if (want) LiveStatsStore.holdPoll()
    else LiveStatsStore.releasePoll()
  }

  function loadThresholds() {
    var names = Object.keys(root.thresholdKeys)
    names.forEach(function(name) {
      var domain = root.thresholdKeys[name]
      Backend.request(Requests.settingsGet(domain), function(env) {
        if (!env || env.ok !== true) return
        root.storeValue(domain, env.result)
      })
    })
  }

  function storeValue(domain, value) {
    var next = {}
    for (var k in root.stored) next[k] = root.stored[k]
    next[domain] = value
    root.stored = next
  }

  function saveThreshold(name, raw) {
    var domain = root.thresholdKeys[name]
    var value = SentinelJs.clampThreshold(name, raw)
    if (value === root.thresholds[name]) return
    root.storeValue(domain, value)
    Backend.request(Requests.settingsSet(domain, value), function(env) {
      if (!env || env.ok !== true) root.loadThresholds()
    })
  }

  function actOn(item, action) {
    var proc = item ? item.proc : null
    if (!proc || !action) return
    if (action === "copy") {
      Omarchy.copyText(String(proc.pid))
      return
    }
    if (action === "copyCmd") {
      Omarchy.copyText(String(proc.cmdline || proc.comm))
      return
    }
    if (action !== "term" && action !== "kill") return
    root.pendingProc = proc
    root.pendingSignal = action === "kill" ? "KILL" : "TERM"
    signalConfirm.title = action === "kill" ? "Force quit this process" : "End this process"
    signalConfirm.message = action === "kill"
      ? "Send SIGKILL to " + proc.comm + " (" + proc.pid + "). It will not get a chance to exit cleanly."
      : "Send SIGTERM to " + proc.comm + " (" + proc.pid + ")."
    signalConfirm.confirmText = action === "kill" ? "Force quit" : "End"
    signalConfirm.destructive = true
    signalConfirm.ask()
  }

  function runPending() {
    var proc = root.pendingProc
    var sig = root.pendingSignal
    root.pendingProc = null
    root.pendingSignal = ""
    if (!proc || !sig) return
    Omarchy.signalProcess(proc.pid, sig)
  }

  onVisibleChanged: root.syncHold()
  Component.onCompleted: {
    signalConfirm.parent = root.prefsOverlay
    root.syncHold()
    root.loadThresholds()
  }
  Component.onDestruction: {
    if (root.holding) LiveStatsStore.releasePoll()
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

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      Text {
        width: parent.width
        text: root.summary
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.descriptionSize
      }

      PrefsField {
        width: parent.width
        placeholder: "Search processes and ports…"
        onEdited: function(value) { root.filter = value }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: root.chips

          PrefsButton {
            required property var modelData
            text: modelData.label + " " + modelData.count
            primary: root.kind === modelData.id
            onClicked: root.kind = modelData.id
          }
        }
      }
    }

    Repeater {
      model: root.rows

      SentinelRow {
        required property var modelData
        item: modelData
        query: root.query
        onActed: function(action) { root.actOn(modelData, action) }
      }
    }

    PrefsEmpty {
      available: root.rows.length === 0
      sectionHelp: false
      label: LiveStatsStore.waiting
        ? "Reading the first sample"
        : (root.kind === "port" ? "No listening ports" : "Nothing looks wrong")
      description: LiveStatsStore.waiting
        ? "CPU findings need a few samples, about every two seconds."
        : (root.filter.length > 0
          ? "Nothing matches that search."
          : "No process is over its limits, and no long-running process holds a port open.")
      query: root.query
      keywords: ["empty", "clean", "ok"]
    }
  }

  PrefsGroup {
    title: "Limits"
    query: root.query
    detail: "What counts as rogue. A process has to stay over the CPU limit for a few samples in a row before it is flagged."

    SettingRow {
      stretchControl: true
      catalog: false
      label: "CPU"
      description: "Flag a process using more than this share of one core."
      hint: "shell.json atmos.sentinel.cpuPercent"
      query: root.query
      keywords: ["cpu", "busy", "runaway"]

      PrefsSlider {
        width: parent.width
        from: 50
        to: 100
        stepSize: 5
        value: root.thresholds.cpuPercent
        valueText: root.thresholds.cpuPercent + "%"
        formatTick: function(v) { return Math.round(v) }
        onChanged: function(value) { root.saveThreshold("cpuPercent", value) }
      }
    }

    SettingRow {
      stretchControl: true
      catalog: false
      label: "Memory"
      description: "Flag a process holding more than this share of installed memory."
      hint: "shell.json atmos.sentinel.memoryPercent"
      query: root.query
      keywords: ["memory", "ram", "leak"]

      PrefsSlider {
        width: parent.width
        from: 5
        to: 90
        stepSize: 5
        value: root.thresholds.memoryPercent
        valueText: root.thresholds.memoryPercent + "%"
        formatTick: function(v) { return Math.round(v) }
        onChanged: function(value) { root.saveThreshold("memoryPercent", value) }
      }
    }

    SettingRow {
      stretchControl: true
      catalog: false
      label: "Port age"
      description: "Flag a listening port whose process has been up longer than this."
      hint: "shell.json atmos.sentinel.portAgeDays"
      query: root.query
      keywords: ["port", "listening", "days", "age"]

      PrefsSlider {
        width: parent.width
        from: 1
        to: 90
        stepSize: 1
        value: root.thresholds.portAgeDays
        valueText: root.thresholds.portAgeDays + " d"
        formatTick: function(v) { return Math.round(v) }
        onChanged: function(value) { root.saveThreshold("portAgeDays", value) }
      }
    }
  }
}
