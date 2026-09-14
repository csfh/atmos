import QtQuick
import "../../components"
import "../../services"
import "../../services/LiveStats.js" as LiveStatsJs
import "../../services/Monitor.js" as MonitorJs
import "../../services/Processes.js" as ProcessesJs

PrefsPage {
  id: root
  hubId: "monitor/disk"
  title: "Disk I/O"
  description: "Read and write rates from /proc/diskstats. Partitions are skipped. Capacity lives on Disks."

  property string diskFilter: ""
  property string diskSort: "io"

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var disks: {
    var src = root.latest && root.latest.disks ? root.latest.disks : []
    var q = String(root.diskFilter || "").toLowerCase()
    var out = []
    var i
    var row
    for (i = 0; i < src.length; i++) {
      row = src[i]
      if (!row) continue
      if (q && String(row.name).toLowerCase().indexOf(q) === -1) continue
      out.push(row)
    }
    out.sort(function (a, b) {
      var ar = (a.readBps || 0) + (a.writeBps || 0)
      var br = (b.readBps || 0) + (b.writeBps || 0)
      if (root.diskSort === "read") return (b.readBps || 0) - (a.readBps || 0)
      if (root.diskSort === "write") return (b.writeBps || 0) - (a.writeBps || 0)
      if (root.diskSort === "name") return String(a.name).localeCompare(String(b.name))
      return br - ar
    })
    return out
  }
  readonly property var ioProcs: ProcessesJs.list(
    root.latest && root.latest.processes ? root.latest.processes : [],
    "",
    "io",
    { scope: "all", uid: root.latest ? root.latest.uid : null, cap: 12 }
  )

  function waiting() {
    return LiveStatsStore.waiting ? "waiting for samples" : "unknown"
  }

  function diskLine(row) {
    var r = LiveStatsJs.formatBps(row && row.readBps)
    var w = LiveStatsJs.formatBps(row && row.writeBps)
    if (!r && !w) return root.waiting()
    return "↓ " + (r || "—") + "  ↑ " + (w || "—")
  }

  PrefsGroup {
    title: "Pressure"
    query: root.query
    detail: "Share of time some tasks stalled on I/O."

    SettingRow {
      label: "I/O stall"
      description: "PSI some avg10 for io."
      hint: "/proc/pressure/io"
      query: root.query
      keywords: ["psi", "stall", "io"]
      valueText: root.latest && root.latest.psi ? LiveStatsJs.formatPsi(root.latest.psi.io) || "unknown" : root.waiting()
    }
  }

  PrefsGroup {
    title: "Devices"
    query: root.query
    wide: true
    detail: "Whole disks only. Dual sparkline is read over write."
    hint: "/proc/diskstats"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      PrefsField {
        width: parent.width
        placeholder: "Filter disks…"
        onEdited: function(value) { root.diskFilter = value }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: [
            { id: "io", label: "I/O" },
            { id: "read", label: "Read" },
            { id: "write", label: "Write" },
            { id: "name", label: "Name" }
          ]

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.diskSort === (modelData && modelData.id ? modelData.id : "")
            onClicked: root.diskSort = modelData.id
          }
        }
      }
    }

    Repeater {
      model: root.disks

      SettingRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "disk"
        description: root.diskLine(modelData)
        hint: "/proc/diskstats"
        query: root.query
        keywords: ["disk", "nvme", "ssd", "io"]
        valueText: root.diskLine(modelData)
        stretchControl: true

        PrefsSparkline {
          width: parent.width
          values: LiveStatsJs.diskSeries(root.history, modelData.name, "readBps")
          valuesB: LiveStatsJs.diskSeries(root.history, modelData.name, "writeBps")
          valueText: root.diskLine(modelData)
          fill: false
        }
      }
    }

    SettingRow {
      available: root.disks.length === 0
      sectionHelp: false
      label: "No disks"
      description: root.latest ? "Nothing matches that filter." : "Waiting for the first sample."
      query: root.query
      keywords: ["empty", "disk"]
    }
  }

  PrefsGroup {
    title: "Busiest tasks"
    query: root.query
    wide: true
    detail: "Process read and write bytes from /proc/pid/io when that file is readable."

    Repeater {
      model: root.ioProcs

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
      available: root.ioProcs.length === 0
      sectionHelp: false
      label: "No I/O samples"
      description: "Process I/O waits for a second sample, and some tasks hide /proc/pid/io."
      query: root.query
      keywords: ["empty", "io"]
    }
  }
}
