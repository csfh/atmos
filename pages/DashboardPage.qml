import QtQuick
import "../components"
import "../services"
import "../services/Charts.js" as ChartsJs
import "../services/LiveStats.js" as LiveStatsJs
import "../services/Monitor.js" as MonitorJs

PrefsPage {
  id: root
  hubId: "dashboard"
  title: "Dashboard"
  description: "Every live chart on one page. While this hub is open, samples run at 100ms and marks tween between them."

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var intervals: MonitorJs.intervalChips()
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

  function waiting() {
    return LiveStatsStore.waiting ? "waiting for samples" : "unknown"
  }

  Component.onCompleted: {
    LiveStatsStore.requestInterval("dashboard", 100)
    LiveStatsStore.poll()
  }

  Component.onDestruction: LiveStatsStore.releaseInterval("dashboard")

  PrefsGroup {
    title: "Sampling"
    query: root.query
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
            primary: LiveStatsStore.userIntervalMs === (modelData && modelData.ms ? modelData.ms : 0)
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
          : ("live " + LiveStatsStore.intervalMs + "ms  ·  " + LiveStatsStore.sampleCount + " samples")
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
      }
    }
  }

  PrefsGroup {
    title: "IRQ land"
    query: root.query
    detail: "Interrupt rate by vector × core."
    hint: "/proc/interrupts"

    PrefsChart {
      width: parent.width
      kind: "heatmap"
      model: root.irqRates
      valueText: "IRQ land heatmap"
    }
  }

  PrefsGroup {
    title: "PSI ridgeline"
    query: root.query
    detail: "cpu / memory / io stall overlapping on one time scale."
    hint: "/proc/pressure"

    PrefsChart {
      width: parent.width
      kind: "ridgeline"
      model: root.psiRidges
      valueText: "PSI stall ridgeline"
    }
  }

  PrefsGroup {
    title: "Core frequency"
    query: root.query
    detail: "Violin of scaling_cur_freq per core."
    hint: "scaling_cur_freq"

    PrefsChart {
      width: parent.width
      kind: "violin"
      model: root.freqViolins
      valueText: "Core-freq violin"
    }
  }

  PrefsGroup {
    title: "Softirq rose"
    query: root.query
    detail: "Nightingale rose of software IRQ rates."
    hint: "/proc/softirqs"

    PrefsChart {
      width: parent.width
      kind: "rose"
      model: root.softirqWedges
      valueText: "Softirq Nightingale rose"
    }
  }

  PrefsGroup {
    title: "Buddy horizon"
    query: root.query
    detail: "Free pages from buddyinfo, folded into bands."
    hint: "/proc/buddyinfo"

    PrefsChart {
      width: parent.width
      kind: "horizon"
      model: root.buddyValues
      valueText: "Buddy-order horizon"
    }
  }

  PrefsGroup {
    title: "RSS treemap"
    query: root.query
    detail: "Resident set nested by user then comm."
    hint: "/proc/*/status"

    PrefsChart {
      width: parent.width
      kind: "treemap"
      model: root.rssTree
      valueText: "Process RSS treemap"
    }
  }

  PrefsGroup {
    title: "Meminfo Sankey"
    query: root.query
    detail: "MemTotal flowing into kernel buckets."
    hint: "/proc/meminfo"

    PrefsChart {
      width: parent.width
      kind: "sankey"
      model: root.memSankey
      valueText: "Meminfo composition Sankey"
    }
  }

  PrefsGroup {
    title: "Cgroup sunburst"
    query: root.query
    detail: "memory.current by cgroup path."
    hint: "/sys/fs/cgroup/**/memory.current"

    PrefsChart {
      width: parent.width
      kind: "sunburst"
      model: root.cgroupTree
      valueText: "Cgroup memory sunburst"
    }
  }

  PrefsGroup {
    title: "Slab icicle"
    query: root.query
    detail: "Kernel slab occupancy."
    hint: "/proc/slabinfo"

    PrefsChart {
      width: parent.width
      kind: "icicle"
      model: root.slabTree
      valueText: "Slab cache icicle"
    }
  }

  PrefsGroup {
    title: "Disk streamgraph"
    query: root.query
    detail: "Read+write bandwidth stacked about the centerline."
    hint: "/proc/diskstats"

    PrefsChart {
      width: parent.width
      kind: "streamgraph"
      model: root.diskStream
      valueText: "Blockdev I/O streamgraph"
    }
  }

  PrefsGroup {
    title: "TCP beeswarm"
    query: root.query
    detail: "One point per socket, grouped by state."
    hint: "/proc/net/tcp"

    PrefsChart {
      width: parent.width
      kind: "beeswarm"
      model: root.tcpBees
      valueText: "TCP state beeswarm"
    }
  }

  PrefsGroup {
    title: "Netdev radar"
    query: root.query
    detail: "Per-NIC rx/tx, packets, drops, errs."
    hint: "/proc/net/dev"

    PrefsChart {
      width: parent.width
      kind: "radar"
      model: root.netRadar
      modelB: root.radarAxes
      valueText: "Netdev counter radar"
    }
  }

  PrefsGroup {
    title: "Thermal calendar"
    query: root.query
    detail: "Daily-max °C in this window's sample ring."
    hint: "/sys/class/hwmon/*/temp*_input"

    PrefsChart {
      width: parent.width
      kind: "calendar"
      model: root.thermalDays
      valueText: "Hwmon thermal calendar"
    }
  }

  PrefsGroup {
    title: "RAPL waterfall"
    query: root.query
    detail: "Joules since the last sample. Empty when RAPL is missing."
    hint: "/sys/class/powercap/intel-rapl*/energy_uj"

    PrefsChart {
      width: parent.width
      kind: "waterfall"
      model: root.raplSteps
      valueText: "RAPL energy waterfall"
    }
  }

  PrefsGroup {
    title: "Process coordinates"
    query: root.query
    detail: "CPU%, RSS, FDs, threads, and nice."
    hint: "/proc/*/stat"

    PrefsChart {
      width: parent.width
      kind: "parallel"
      model: root.parallelRows
      modelB: root.parallelAxes
      valueText: "Process parallel coordinates"
    }
  }
}
