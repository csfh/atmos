import QtQuick
import "../../components"
import "../../services"
import "../../services/LiveStats.js" as LiveStatsJs
import "../../services/Monitor.js" as MonitorJs

PrefsPage {
  id: root
  hubId: "monitor/traffic"
  title: "Traffic"
  description: "Per-interface rates from /proc/net/dev and TCP socket counts. Loopback is skipped. Wi-Fi lives on Network."

  property string ifaceFilter: ""
  property string ifaceSort: "io"

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var ifaces: {
    var src = root.latest && root.latest.ifaces ? root.latest.ifaces : []
    var q = String(root.ifaceFilter || "").toLowerCase()
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
      var ar = (a.rxBps || 0) + (a.txBps || 0)
      var br = (b.rxBps || 0) + (b.txBps || 0)
      if (root.ifaceSort === "rx") return (b.rxBps || 0) - (a.rxBps || 0)
      if (root.ifaceSort === "tx") return (b.txBps || 0) - (a.txBps || 0)
      if (root.ifaceSort === "name") return String(a.name).localeCompare(String(b.name))
      return br - ar
    })
    return out
  }
  readonly property var tcpParts: MonitorJs.tcpParts(root.latest && root.latest.tcp)
  readonly property var netRxValues: LiveStatsJs.series(root.history, "rxBps")
  readonly property var netTxValues: LiveStatsJs.series(root.history, "txBps")

  function waiting() {
    return LiveStatsStore.waiting ? "waiting for samples" : "unknown"
  }

  function ifaceLine(row) {
    var r = LiveStatsJs.formatBps(row && row.rxBps)
    var w = LiveStatsJs.formatBps(row && row.txBps)
    if (!r && !w) return root.waiting()
    return "↓ " + (r || "—") + "  ↑ " + (w || "—")
  }

  PrefsGroup {
    title: "Total"
    query: root.query
    detail: "Receive and transmit across every non-loopback interface."

    SettingRow {
      label: "Bandwidth"
      description: "Aggregate of /proc/net/dev, skipping lo."
      hint: "/proc/net/dev"
      query: root.query
      keywords: ["bandwidth", "rx", "tx"]
      valueText: LiveStatsJs.formatNet(root.latest) || root.waiting()
      stretchControl: true

      PrefsSparkline {
        width: parent.width
        values: root.netRxValues
        valuesB: root.netTxValues
        valueText: LiveStatsJs.formatNet(root.latest)
        fill: false
      }
    }
  }

  PrefsGroup {
    title: "Sockets"
    query: root.query
    detail: "IPv4 and IPv6 TCP from /proc/net/tcp and tcp6."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsStackedBar {
        width: parent.width
        parts: root.tcpParts
        valueText: root.latest && root.latest.tcp ? String(root.latest.tcp.total) + " sockets" : ""
      }
    }

    SettingRow {
      label: "Established"
      description: "Active TCP connections."
      hint: "/proc/net/tcp"
      query: root.query
      keywords: ["tcp", "established"]
      valueText: root.latest && root.latest.tcp ? String(root.latest.tcp.established) : root.waiting()
    }

    SettingRow {
      label: "Listen"
      description: "Sockets waiting for a client."
      hint: "st 0A"
      query: root.query
      keywords: ["listen", "port"]
      valueText: root.latest && root.latest.tcp ? String(root.latest.tcp.listen) : root.waiting()
    }

    SettingRow {
      label: "Time-wait"
      description: "Sockets in TIME_WAIT."
      hint: "st 06"
      query: root.query
      keywords: ["time-wait", "tcp"]
      valueText: root.latest && root.latest.tcp ? String(root.latest.tcp.timeWait) : root.waiting()
    }

    SettingRow {
      label: "Close-wait"
      description: "Sockets in CLOSE_WAIT."
      hint: "st 08"
      query: root.query
      keywords: ["close-wait"]
      valueText: root.latest && root.latest.tcp ? String(root.latest.tcp.closeWait) : root.waiting()
    }
  }

  PrefsGroup {
    title: "Interfaces"
    query: root.query
    wide: true
    detail: "One dual sparkline per interface. Receive is the accent stroke. Transmit is muted."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      PrefsField {
        width: parent.width
        placeholder: "Filter interfaces…"
        onEdited: function(value) { root.ifaceFilter = value }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: [
            { id: "io", label: "Total" },
            { id: "rx", label: "Receive" },
            { id: "tx", label: "Transmit" },
            { id: "name", label: "Name" }
          ]

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.ifaceSort === (modelData && modelData.id ? modelData.id : "")
            onClicked: root.ifaceSort = modelData.id
          }
        }
      }
    }

    Repeater {
      model: root.ifaces

      SettingRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "iface"
        description: root.ifaceLine(modelData)
        hint: "/proc/net/dev"
        query: root.query
        keywords: ["iface", "ethernet", "wlan"]
        valueText: root.ifaceLine(modelData)
        stretchControl: true

        PrefsSparkline {
          width: parent.width
          values: LiveStatsJs.ifaceSeries(root.history, modelData.name, "rxBps")
          valuesB: LiveStatsJs.ifaceSeries(root.history, modelData.name, "txBps")
          valueText: root.ifaceLine(modelData)
          fill: false
        }
      }
    }

    SettingRow {
      available: root.ifaces.length === 0
      sectionHelp: false
      label: "No interfaces"
      description: root.latest ? "Nothing matches that filter." : "Waiting for the first sample."
      query: root.query
      keywords: ["empty", "interface"]
    }
  }
}
