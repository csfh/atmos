import QtQuick
import "../../components"
import "../../services"
import "../../services/ChartCopy.js" as ChartCopy
import "../../services/Charts.js" as ChartsJs
import "../../services/LiveStats.js" as LiveStatsJs
import "../../services/Monitor.js" as MonitorJs
import "../../services/Processes.js" as ProcessesJs

PrefsPage {
  id: root
  hubId: "monitor/cpu"
  title: I18n.tr("CPU")
  description: I18n.tr("Per-core busy time, frequency, load averages, and pressure stall from /proc.")

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var cores: root.latest && root.latest.cpus ? root.latest.cpus : []
  readonly property var hotProcs: ProcessesJs.list(
    root.latest && root.latest.processes ? root.latest.processes : [],
    "",
    "cpu",
    { scope: "all", uid: root.latest ? root.latest.uid : null, minCpu: 1, cap: 12 }
  )

  function waiting() {
    return LiveStatsStore.waiting ? "waiting for samples" : "unknown"
  }

  PrefsGroup {
    title: I18n.tr("Package")
    query: root.query
    detail: "Aggregate busy percent from /proc/stat. Load is runnable tasks over 1, 5, and 15 minutes."

    SettingRow {
      label: "Busy"
      description: I18n.tr("Share of time every CPU was not idle or in iowait.")
      hint: "/proc/stat"
      query: root.query
      keywords: ["cpu", "load", "busy"]
      valueText: LiveStatsJs.formatPercent(root.latest && root.latest.cpu) || root.waiting()
      stretchControl: true

      PrefsSparkline {
        id: cpuSpark
        width: parent.width
        values: {
          if (!cpuSpark.inView) return []
          return LiveStatsJs.series(root.history, "cpu")
        }
        valueText: LiveStatsJs.formatPercent(root.latest && root.latest.cpu)
        alert: MonitorJs.alertLevel(root.latest && root.latest.cpu) === "hot"
      }
    }

    SettingRow {
      label: "Load"
      description: I18n.tr("1, 5, and 15 minute load averages.")
      hint: "/proc/loadavg"
      query: root.query
      keywords: ["loadavg", "runnable"]
      valueText: LiveStatsJs.formatLoadLine(root.latest) || root.waiting()
      stretchControl: true

      PrefsSparkline {
        id: loadSpark
        width: parent.width
        values: {
          if (!loadSpark.inView) return []
          return LiveStatsJs.series(root.history, "load1")
        }
        valueText: LiveStatsJs.formatLoadLine(root.latest)
      }
    }

    SettingRow {
      label: "Pressure"
      description: I18n.tr("Share of time some tasks stalled on CPU. From PSI when the kernel publishes it.")
      hint: "/proc/pressure/cpu"
      query: root.query
      keywords: ["psi", "stall", "pressure"]
      valueText: root.latest && root.latest.psi ? LiveStatsJs.formatPsi(root.latest.psi.cpu) || "unknown" : root.waiting()
    }
  }

  PrefsGroup {
    title: I18n.tr("IRQ land")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("heatmap")
    detail: "Per-CPU interrupt rate since the last sample. Fill is relative to the hottest cell. Missing /proc/interrupts stays empty."
    hint: "/proc/interrupts"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        id: irqChart
        width: parent.width
        kind: "heatmap"
        model: {
          var _n = LiveStatsStore.sampleCount
          if (!irqChart.inView) return null
          var h = root.history
          if (!h || h.length < 2) return []
          return ChartsJs.irqRateMatrix(h[h.length - 2].interrupts, h[h.length - 1].interrupts)
        }
        valueText: "IRQ land heatmap"
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("PSI ridgeline")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("ridgeline")
    detail: "cpu, memory, and io some avg10 overlapping on one time scale."
    hint: "/proc/pressure"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        id: psiChart
        width: parent.width
        kind: "ridgeline"
        model: {
          var _n = LiveStatsStore.sampleCount
          if (!psiChart.inView) return null
          return ChartsJs.psiRidges(root.history)
        }
        valueText: "PSI stall ridgeline"
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Cores")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("corebars")
    detail: "Each bar is one logical CPU. Frequency is scaling_cur_freq when cpufreq is present."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      PrefsCoreBars {
        id: coreBars
        width: parent.width
        values: {
          if (!coreBars.inView) return []
          return LiveStatsJs.corePercents(root.latest)
        }
        valueText: root.cores.length + " cores"
      }
    }

    Repeater {
      model: root.cores

      SettingRow {
        required property var modelData
        required property int index
        label: "CPU " + (modelData && modelData.id != null ? modelData.id : index)
        description: (modelData && modelData.governor ? modelData.governor : "governor unknown")
        hint: "/sys/devices/system/cpu"
        query: root.query
        keywords: ["core", "frequency", "governor"]
        valueText: {
          var pct = LiveStatsJs.formatPercent(modelData && modelData.cpu)
          var hz = LiveStatsJs.formatMhz(modelData && modelData.freqMhz)
          if (pct && hz) return pct + "  ·  " + hz
          return pct || hz || "unknown"
        }
        stretchControl: true

        PrefsSparkline {
          id: coreSpark
          width: parent.width
          values: {
            if (!coreSpark.inView) return []
            return LiveStatsJs.coreSeries(root.history, modelData.id)
          }
          valueText: LiveStatsJs.formatPercent(modelData && modelData.cpu)
          alert: MonitorJs.alertLevel(modelData && modelData.cpu) === "hot"
        }
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Core frequency")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("violin")
    detail: "Violin of scaling_cur_freq P-state spread per core over the sample window."
    hint: "scaling_cur_freq"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        id: violinChart
        width: parent.width
        kind: "violin"
        model: {
          var _n = LiveStatsStore.sampleCount
          if (!violinChart.inView) return null
          return ChartsJs.freqViolins(root.history)
        }
        valueText: "Core-freq violin"
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Softirq rose")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("rose")
    detail: "Nightingale rose of /proc/softirqs rates. Wedge area encodes the rate."
    hint: "/proc/softirqs"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        id: roseChart
        width: parent.width
        kind: "rose"
        model: {
          var _n = LiveStatsStore.sampleCount
          if (!roseChart.inView) return null
          var h = root.history
          if (!h || h.length < 2) return []
          return ChartsJs.softirqWedges(h[h.length - 2].softirqs, h[h.length - 1].softirqs)
        }
        valueText: "Softirq Nightingale rose"
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Histogram")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("histogram")
    detail: "How process CPU is distributed across the current sample, including other users."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsHistogram {
        id: cpuHist
        width: parent.width
        bins: {
          var _n = LiveStatsStore.sampleCount
          if (!cpuHist.inView) return []
          return MonitorJs.cpuHistogram(root.latest && root.latest.processes ? root.latest.processes : [])
        }
        valueText: "CPU histogram"
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Hottest")
    query: root.query
    wide: true
    detail: "Tasks using at least 1% of a core in the last interval."

    Repeater {
      model: root.hotProcs

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
      available: root.hotProcs.length === 0
      sectionHelp: false
      label: "No hot tasks"
      description: I18n.tr("Nothing is using at least 1% of a core right now.")
      query: root.query
      keywords: ["empty", "cpu"]
    }
  }
}
