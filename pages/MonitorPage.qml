import QtQuick
import "../components"
import "../services"
import "../services/Hardware.js" as HardwareJs
import "../services/LiveStats.js" as LiveStatsJs
import "../services/Monitor.js" as MonitorJs
import "../services/Processes.js" as ProcessesJs
import "../services/RichUi.js" as RichUi
import "monitor" as Mon

PrefsPage {
  id: root
  hubId: "monitor"
  title: "Monitor"
  description: "Live load, memory, disks, traffic, sensors, and processes. Home is the short glance. Samples stay in this window."

  property var stack: null
  property var navigator: null
  property string procQuery: ""

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)
  readonly property var cpuValues: LiveStatsJs.series(root.history, "cpu")
  readonly property var memValues: LiveStatsJs.series(root.history, "mem")
  readonly property var netRxValues: LiveStatsJs.series(root.history, "rxBps")
  readonly property var netTxValues: LiveStatsJs.series(root.history, "txBps")
  readonly property var cpuTempValues: LiveStatsJs.series(root.history, "cpuTemp")
  readonly property var loadValues: LiveStatsJs.series(root.history, "load1")
  readonly property var gpuList: LiveStatsJs.gpuRows(root.hw.gpus, root.latest)
  readonly property var coreValues: LiveStatsJs.corePercents(root.latest)
  readonly property var memSegments: LiveStatsJs.memParts(root.latest)
  readonly property var topProcs: ProcessesJs.list(
    root.latest && root.latest.processes ? root.latest.processes : [],
    root.procQuery,
    "cpu",
    { scope: "mine", uid: root.latest ? root.latest.uid : null, cap: 8 }
  )
  readonly property var pages: MonitorJs.pageCards()
  readonly property var intervals: MonitorJs.intervalChips()
  readonly property int tileWidth: Math.max(240, Math.floor((nowFlow.width - Theme.spaceMd) / 2))

  function openSubpage(id) {
    if (stack) {
      if (id === "processes") stack.push(processesPage)
      else if (id === "cpu") stack.push(cpuPage)
      else if (id === "memory") stack.push(memoryPage)
      else if (id === "disk") stack.push(diskPage)
      else if (id === "traffic") stack.push(trafficPage)
      else if (id === "sensors") stack.push(sensorsPage)
      return
    }
    if (root.navigator && root.navigator.go)
      root.navigator.go("monitor/" + id)
  }

  function waitingText() {
    return LiveStatsStore.waiting ? "waiting for samples" : ""
  }

  function cpuCaption() {
    return LiveStatsJs.formatPercent(root.latest && root.latest.cpu) || root.waitingText() || "unknown"
  }

  function memCaption() {
    if (!root.latest || root.latest.memUsed === null || root.latest.memTotal === null)
      return root.waitingText() || "unknown"
    var used = RichUi.formatBytes(LiveStatsJs.memBytes(root.latest.memUsed))
    var total = RichUi.formatBytes(LiveStatsJs.memBytes(root.latest.memTotal))
    var pct = LiveStatsJs.formatPercent(root.latest.mem)
    return used + " / " + total + (pct ? "  " + pct : "")
  }

  function netCaption() {
    return LiveStatsJs.formatNet(root.latest) || root.waitingText() || "unknown"
  }

  function diskCaption() {
    var list = root.latest && root.latest.disks ? root.latest.disks : []
    var r = 0
    var w = 0
    var i
    var any = false
    for (i = 0; i < list.length; i++) {
      if (list[i].readBps != null) { r += list[i].readBps; any = true }
      if (list[i].writeBps != null) { w += list[i].writeBps; any = true }
    }
    if (!any) return root.waitingText() || "unknown"
    return "↓ " + LiveStatsJs.formatBps(r) + "  ↑ " + LiveStatsJs.formatBps(w)
  }

  function diskReadValues() {
    var list = root.latest && root.latest.disks ? root.latest.disks : []
    if (!list.length) return LiveStatsJs.series(root.history, "rxBps")
    return LiveStatsJs.diskSeries(root.history, list[0].name, "readBps")
  }

  function diskWriteValues() {
    var list = root.latest && root.latest.disks ? root.latest.disks : []
    if (!list.length) return []
    return LiveStatsJs.diskSeries(root.history, list[0].name, "writeBps")
  }

  function tempCaption() {
    return LiveStatsJs.formatTemp(root.latest && root.latest.cpuTemp) || root.waitingText() || "unknown"
  }

  function loadCaption() {
    return LiveStatsJs.formatLoadLine(root.latest) || root.waitingText() || "unknown"
  }

  function coresCaption() {
    if (!root.coreValues.length) return root.waitingText() || "no cores yet"
    return root.coreValues.length + " cores"
  }

  function psiCaption() {
    var psi = root.latest && root.latest.psi ? root.latest.psi : null
    if (!psi) return ""
    var bits = []
    if (psi.cpu != null) bits.push("cpu " + LiveStatsJs.formatPsi(psi.cpu))
    if (psi.memory != null) bits.push("mem " + LiveStatsJs.formatPsi(psi.memory))
    if (psi.io != null) bits.push("io " + LiveStatsJs.formatPsi(psi.io))
    return bits.join("   ")
  }

  function tcpCaption() {
    var tcp = root.latest && root.latest.tcp ? root.latest.tcp : null
    if (!tcp) return ""
    return tcp.established + " established  ·  " + tcp.listen + " listen  ·  " + tcp.total + " sockets"
  }

  function alertOf(pct) {
    return MonitorJs.alertLevel(pct) === "hot"
  }

  Component { id: processesPage; Mon.ProcessesPage {} }
  Component { id: cpuPage; Mon.CpuPage {} }
  Component { id: memoryPage; Mon.MemoryPage {} }
  Component { id: diskPage; Mon.DiskPage {} }
  Component { id: trafficPage; Mon.TrafficPage {} }
  Component { id: sensorsPage; Mon.SensorsPage {} }

  PrefsGroup {
    title: "Sampling"
    query: root.query
    detail: "Samples stay in this window. Pause freezes the graphs. Interval is how often /proc is read."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: root.intervals

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: LiveStatsStore.intervalMs === (modelData && modelData.ms ? modelData.ms : 0)
            onClicked: LiveStatsStore.setIntervalId(modelData.id)
          }
        }

        PrefsButton {
          text: LiveStatsStore.paused ? "Resume" : "Pause"
          primary: LiveStatsStore.paused
          onClicked: LiveStatsStore.togglePaused()
        }
      }

      Text {
        width: parent.width
        text: (LiveStatsStore.sampleCount ? LiveStatsStore.sampleCount + " samples" : "No samples yet")
          + (root.psiCaption() ? "  ·  stall " + root.psiCaption() : "")
          + (LiveStatsStore.lastError ? "  ·  " + LiveStatsStore.lastError : "")
        color: LiveStatsStore.lastError ? Theme.urgent : Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
        wrapMode: Text.WordWrap
      }
    }
  }

  PrefsGroup {
    title: "Now"
    query: root.query
    framed: false
    catalog: false
    wide: true
    detail: "Meters share the live sample. Hot ink is 90% and up."

    Flow {
      id: nowFlow
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.spaceMd

      PrefsMeter {
        width: root.tileWidth
        label: "Processor"
        valueText: root.cpuCaption()
        caption: root.coresCaption()
        values: root.cpuValues
        showBar: root.latest && root.latest.cpu != null
        barValue: root.latest && root.latest.cpu != null ? root.latest.cpu : 0
        alert: root.alertOf(root.latest && root.latest.cpu)
      }

      PrefsMeter {
        width: root.tileWidth
        label: "Memory"
        valueText: root.memCaption()
        caption: root.latest && root.latest.memAvail != null
          ? RichUi.formatBytes(LiveStatsJs.memBytes(root.latest.memAvail)) + " available"
          : ""
        values: root.memValues
        showBar: root.latest && root.latest.mem != null
        barValue: root.latest && root.latest.mem != null ? root.latest.mem : 0
        alert: root.alertOf(root.latest && root.latest.mem)
      }

      PrefsMeter {
        width: root.tileWidth
        label: "Traffic"
        valueText: root.netCaption()
        caption: root.tcpCaption()
        values: root.netRxValues
        valuesB: root.netTxValues
        fill: false
      }

      PrefsMeter {
        width: root.tileWidth
        label: "Disk I/O"
        valueText: root.diskCaption()
        values: root.diskReadValues()
        valuesB: root.diskWriteValues()
        fill: false
      }

      PrefsMeter {
        width: root.tileWidth
        label: "Package"
        valueText: root.tempCaption()
        values: root.cpuTempValues
        alert: root.latest && root.latest.cpuTemp != null && root.latest.cpuTemp >= 90
      }

      PrefsMeter {
        width: root.tileWidth
        label: "Load"
        valueText: root.loadCaption()
        caption: root.coreValues.length ? "1 / 5 / 15  ·  " + root.coreValues.length + " cores" : ""
        values: root.loadValues
      }
    }
  }

  PrefsGroup {
    title: "Cores"
    query: root.query
    wide: true
    detail: "One bar per logical CPU from /proc/stat. Height is busy percent since the last sample."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      PrefsCoreBars {
        width: parent.width
        values: root.coreValues
        valueText: root.coresCaption()
      }

      PrefsStackedBar {
        width: parent.width
        parts: root.memSegments
        valueText: root.memCaption()
      }
    }
  }

  PrefsGroup {
    title: "Pages"
    query: root.query
    detail: "Each page is a process-manager surface: filters, rates, and the same live sample."

    Repeater {
      model: root.pages

      SettingRow {
        required property var modelData
        label: modelData && modelData.title ? modelData.title : ""
        description: modelData && modelData.description ? modelData.description : ""
        query: root.query
        keywords: ["monitor", "process", "cpu", "memory", "disk", "network", "sensor"]

        PrefsButton {
          text: "Configure…"
          onClicked: root.openSubpage(modelData.id)
        }
      }
    }
  }

  PrefsGroup {
    title: "Hottest tasks"
    query: root.query
    wide: true
    detail: "This user's processes, sorted by CPU. The full table is on Processes."
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
    }

    Repeater {
      model: root.topProcs

      ProcessRow {
        required property var modelData
        procRow: modelData
        query: root.query
        onActed: function(action) {
          if (action === "copy") Omarchy.copyText(String(modelData.pid))
          else if (action === "copyCmd") Omarchy.copyText(String(modelData.cmdline || modelData.comm))
        }
      }
    }

    SettingRow {
      available: root.topProcs.length === 0
      sectionHelp: false
      label: "No matching processes"
      description: root.latest ? "Nothing matches that search." : "Waiting for the first sample."
      query: root.query
      keywords: ["empty", "process"]
    }

    SettingRow {
      label: "Full table"
      description: "Every filter, sort, and signal is on Processes."
      query: root.query
      keywords: ["process", "kill", "pid"]

      PrefsButton {
        text: "Configure…"
        primary: true
        onClicked: root.openSubpage("processes")
      }
    }
  }
}
