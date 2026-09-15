import QtQuick
import "../components"
import "../services"
import "../services/Charts.js" as ChartsJs
import "../services/LiveStats.js" as LiveStatsJs
import "../services/Monitor.js" as MonitorJs

PrefsPage {
  id: root
  hubId: "dashboard"
  expandContent: true
  title: "Dashboard"
  description: "Every live chart on one grid. While this hub is open, samples run at 100ms and marks tween between them."

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var intervals: MonitorJs.intervalChips()
  readonly property int tileMin: 260
  readonly property int tileGap: Theme.spaceMd
  readonly property int gridInnerWidth: Math.max(240, root.pageColumnWidth - Theme.copyInset * 2)
  readonly property int chartColumns: {
    var n = Math.floor((root.gridInnerWidth + root.tileGap) / (root.tileMin + root.tileGap))
    if (n < 2) n = 2
    if (n > 4) n = 4
    return n
  }
  readonly property int tileWidth: Math.max(
    160,
    Math.floor((root.gridInnerWidth - root.tileGap * (root.chartColumns - 1)) / root.chartColumns)
  )
  readonly property int tileChartHeight: Math.max(Theme.chartHeight, Math.round(root.tileWidth * 0.62))
  readonly property var irqRates: {
    var h = root.history
    if (!h || h.length < 2) return []
    return ChartsJs.irqRateMatrix(h[h.length - 2].interrupts, h[h.length - 1].interrupts)
  }
  readonly property var psiRidges: ChartsJs.psiRidges(root.history)
  readonly property var freqViolins: ChartsJs.freqViolins(root.history)
  readonly property var softirqWedges: {
    var h = root.history
    if (!h || h.length < 2) return []
    return ChartsJs.softirqWedges(h[h.length - 2].softirqs, h[h.length - 1].softirqs)
  }
  readonly property var buddyValues: ChartsJs.buddySeries(root.history)
  readonly property var rssTree: ChartsJs.rssTree(root.latest && root.latest.processes)
  readonly property var memSankey: ChartsJs.meminfoSankey(root.latest)
  readonly property var cgroupTree: ChartsJs.cgroupTree(root.latest && root.latest.cgroups)
  readonly property var slabTree: ChartsJs.slabTree(root.latest && root.latest.slabs)
  readonly property var diskStream: ChartsJs.diskStream(root.history)
  readonly property var tcpBees: ChartsJs.tcpBees(root.latest)
  readonly property var netRadar: ChartsJs.netdevRadar(root.latest && root.latest.ifaces)
  readonly property var radarAxes: ChartsJs.radarAxes()
  readonly property var thermalDays: ChartsJs.thermalDays(root.history)
  readonly property var raplSteps: {
    var h = root.history
    if (!h || h.length < 2) return []
    return ChartsJs.raplSteps(h[h.length - 2], h[h.length - 1])
  }
  readonly property var parallelRows: ChartsJs.processParallel(root.latest && root.latest.processes)
  readonly property var parallelAxes: ChartsJs.parallelAxes()
  readonly property var tiles: [
    { kind: "heatmap", title: "IRQ land", hint: "/proc/interrupts", model: "irqRates" },
    { kind: "ridgeline", title: "PSI ridgeline", hint: "/proc/pressure", model: "psiRidges" },
    { kind: "violin", title: "Core frequency", hint: "scaling_cur_freq", model: "freqViolins" },
    { kind: "rose", title: "Softirq rose", hint: "/proc/softirqs", model: "softirqWedges" },
    { kind: "horizon", title: "Buddy horizon", hint: "/proc/buddyinfo", model: "buddyValues" },
    { kind: "treemap", title: "RSS treemap", hint: "/proc/*/status", model: "rssTree" },
    { kind: "sankey", title: "Meminfo Sankey", hint: "/proc/meminfo", model: "memSankey" },
    { kind: "sunburst", title: "Cgroup sunburst", hint: "memory.current", model: "cgroupTree" },
    { kind: "icicle", title: "Slab icicle", hint: "/proc/slabinfo", model: "slabTree" },
    { kind: "streamgraph", title: "Disk streamgraph", hint: "/proc/diskstats", model: "diskStream" },
    { kind: "beeswarm", title: "TCP beeswarm", hint: "/proc/net/tcp", model: "tcpBees" },
    { kind: "radar", title: "Netdev radar", hint: "/proc/net/dev", model: "netRadar" },
    { kind: "calendar", title: "Thermal calendar", hint: "temp*_input", model: "thermalDays" },
    { kind: "waterfall", title: "RAPL waterfall", hint: "energy_uj", model: "raplSteps" },
    { kind: "parallel", title: "Process coordinates", hint: "/proc/*/stat", model: "parallelRows" }
  ]

  function waiting() {
    return LiveStatsStore.waiting ? "waiting for samples" : "unknown"
  }

  function chartModel(id) {
    if (id === "irqRates") return root.irqRates
    if (id === "psiRidges") return root.psiRidges
    if (id === "freqViolins") return root.freqViolins
    if (id === "softirqWedges") return root.softirqWedges
    if (id === "buddyValues") return root.buddyValues
    if (id === "rssTree") return root.rssTree
    if (id === "memSankey") return root.memSankey
    if (id === "cgroupTree") return root.cgroupTree
    if (id === "slabTree") return root.slabTree
    if (id === "diskStream") return root.diskStream
    if (id === "tcpBees") return root.tcpBees
    if (id === "netRadar") return root.netRadar
    if (id === "thermalDays") return root.thermalDays
    if (id === "raplSteps") return root.raplSteps
    if (id === "parallelRows") return root.parallelRows
    return null
  }

  function chartModelB(kind) {
    if (kind === "radar") return root.radarAxes
    if (kind === "parallel") return root.parallelAxes
    return null
  }

  Component.onCompleted: {
    LiveStatsStore.requestInterval("dashboard", 100)
    LiveStatsStore.poll()
  }

  Component.onDestruction: LiveStatsStore.releaseInterval("dashboard")

  PrefsGroup {
    title: "Sampling"
    query: root.query
    wide: true
    detail: "Dashboard holds the poll at 100ms. A slower chip is a floor only after you leave this hub."

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
          onClicked: LiveStatsStore.togglePaused()
        }
      }

      Text {
        width: parent.width
        text: LiveStatsStore.waiting
          ? root.waiting()
          : ("live " + LiveStatsStore.intervalMs + "ms  ·  " + LiveStatsStore.sampleCount + " samples  ·  " + root.chartColumns + " columns")
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
      }
    }
  }

  PrefsGroup {
    title: "Charts"
    query: root.query
    wide: true
    catalog: true
    detail: "IRQ heatmap, PSI ridgeline, core-freq violin, softirq rose, buddy horizon, RSS treemap, meminfo Sankey, cgroup sunburst, slab icicle, disk streamgraph, TCP beeswarm, netdev radar, thermal calendar, RAPL waterfall, and process parallel coordinates."
    hint: "/proc"

    Grid {
      id: chartGrid
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      columns: root.chartColumns
      columnSpacing: root.tileGap
      rowSpacing: root.tileGap

      Repeater {
        model: root.tiles

        Rectangle {
          required property var modelData
          width: root.tileWidth
          height: inner.implicitHeight + Theme.pad * 2
          color: Theme.fill(Theme.normalFill)
          border.width: Theme.borderWidth
          border.color: Theme.borderColor()
          radius: Theme.radius

          Column {
            id: inner
            x: Theme.pad
            y: Theme.pad
            width: parent.width - Theme.pad * 2
            spacing: Theme.labelGap

            Text {
              width: parent.width
              text: modelData && modelData.title ? modelData.title : ""
              color: Theme.muted
              font.family: Theme.fontFamily
              font.pixelSize: Theme.metaSize
              font.letterSpacing: Theme.sectionTracking
              elide: Text.ElideRight
            }

            Text {
              width: parent.width
              visible: !!(modelData && modelData.hint)
              text: modelData && modelData.hint ? modelData.hint : ""
              color: Theme.muted
              opacity: Theme.metaOpacity
              font.family: Theme.fontFamily
              font.pixelSize: Theme.captionSize
              elide: Text.ElideRight
            }

            PrefsChart {
              width: parent.width
              height: root.tileChartHeight
              kind: modelData && modelData.kind ? modelData.kind : ""
              model: root.chartModel(modelData && modelData.model)
              modelB: root.chartModelB(modelData && modelData.kind)
              valueText: modelData && modelData.title ? modelData.title : ""
            }
          }
        }
      }
    }
  }
}
