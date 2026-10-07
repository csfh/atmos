import QtQuick
import "../../components"
import "../../services"
import "../../services/Hardware.js" as HardwareJs
import "../../services/ChartCopy.js" as ChartCopy
import "../../services/Charts.js" as ChartsJs
import "../../services/LiveStats.js" as LiveStatsJs
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "monitor/sensors"
  title: I18n.tr("Sensors")
  description: I18n.tr("hwmon temperatures and fans, plus GPU busy and VRAM when the driver publishes them.")

  property string sensorFilter: ""
  property string sensorKind: "all"

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)
  readonly property var gpuList: LiveStatsJs.gpuRows(root.hw.gpus, root.latest)
  readonly property var sensors: {
    var src = root.latest && root.latest.sensors ? root.latest.sensors : []
    var q = String(root.sensorFilter || "").toLowerCase()
    var kind = String(root.sensorKind || "all")
    var out = []
    var i
    var row
    var hay
    for (i = 0; i < src.length; i++) {
      row = src[i]
      if (!row) continue
      if (kind !== "all" && row.kind !== kind) continue
      hay = (row.chip + " " + row.label + " " + row.kind).toLowerCase()
      if (q && hay.indexOf(q) === -1) continue
      out.push(row)
    }
    out.sort(function (a, b) {
      if (a.kind !== b.kind) return a.kind === "temp" ? -1 : 1
      return String(a.label).localeCompare(String(b.label))
    })
    return out
  }

  function waiting() {
    return LiveStatsStore.waiting ? "waiting for samples" : "unknown"
  }

  function sensorText(row) {
    if (!row) return root.waiting()
    if (row.kind === "fan") return LiveStatsJs.formatRpm(row.value) || "unknown"
    return LiveStatsJs.formatTemp(row.value) || "unknown"
  }

  function gpuVramText(gpu) {
    var live = LiveStatsJs.findGpu(root.latest, gpu) || LiveStatsJs.findGpu(root.latest, gpu && gpu.card)
    if (!live || live.vramUsed == null || live.vramTotal == null) return ""
    return RichUi.formatBytes(live.vramUsed * 1024) + " / " + RichUi.formatBytes(live.vramTotal * 1024)
  }

  PrefsGroup {
    title: I18n.tr("Package")
    query: root.query
    detail: "CPU package temperature, the same reading Home uses."

    SettingRow {
      label: "Processor"
      description: I18n.tr("CPU package from hwmon or thermal_zone.")
      hint: "/sys/class/hwmon"
      query: root.query
      keywords: ["cpu", "temperature", "package"]
      valueText: LiveStatsJs.formatTemp(root.latest && root.latest.cpuTemp) || root.waiting()
      stretchControl: true

      PrefsSparkline {
        id: cpuTempSpark
        width: parent.width
        values: {
          if (!cpuTempSpark.inView) return []
          return LiveStatsJs.series(root.history, "cpuTemp")
        }
        valueText: LiveStatsJs.formatTemp(root.latest && root.latest.cpuTemp)
        alert: root.latest && root.latest.cpuTemp != null && root.latest.cpuTemp >= 90
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Thermal calendar")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("calendar")
    detail: "Week × weekday cells of daily-max °C from hwmon temp*_input in this window's sample ring. No private history store."
    hint: "/sys/class/hwmon/*/temp*_input"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        id: calendarChart
        width: parent.width
        kind: "calendar"
        model: {
          var _n = LiveStatsStore.sampleCount
          if (!calendarChart.inView) return null
          return ChartsJs.thermalDays(root.history)
        }
        valueText: "Hwmon thermal calendar"
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("RAPL waterfall")
    query: root.query
    wide: true
    lede: ChartCopy.blurb("waterfall")
    detail: "Sequential joules since the last sample from powercap energy_uj. This AMD host may have no RAPL nodes; the chart stays empty instead of inventing zeros."
    hint: "/sys/class/powercap/intel-rapl*/energy_uj"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        id: waterfallChart
        width: parent.width
        kind: "waterfall"
        model: {
          var _n = LiveStatsStore.sampleCount
          if (!waterfallChart.inView) return null
          var h = root.history
          if (!h || h.length < 2) return []
          return ChartsJs.raplSteps(h[h.length - 2], h[h.length - 1])
        }
        valueText: "RAPL energy waterfall"
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Graphics")
    query: root.query
    detail: "Busy and VRAM come from the DRM device or nvidia-smi. Missing stays unknown."

    Repeater {
      model: root.gpuList

      SettingRow {
        required property var modelData
        label: (modelData && modelData.name) || "GPU"
        description: {
          var bits = []
          var vram = root.gpuVramText(modelData)
          if (modelData && modelData.driver) bits.push(modelData.driver)
          if (vram) bits.push(vram)
          return bits.join(" · ")
        }
        hint: "/sys/class/drm"
        query: root.query
        keywords: ["gpu", "vram", "busy"]
        valueText: {
          var busy = LiveStatsJs.formatPercent(LiveStatsJs.gpuBusyAt(root.latest, modelData))
          var temp = LiveStatsJs.formatTemp(LiveStatsJs.gpuTempAt(root.latest, modelData))
          if (busy && temp) return busy + "  ·  " + temp
          return busy || temp || root.waiting()
        }
        stretchControl: true

        PrefsSparkline {
          id: gpuSpark
          width: parent.width
          values: {
            if (!gpuSpark.inView) return []
            return LiveStatsJs.gpuBusySeries(root.history, modelData)
          }
          valuesB: {
            if (!gpuSpark.inView) return []
            return LiveStatsJs.gpuSeries(root.history, modelData)
          }
          valueText: LiveStatsJs.formatPercent(LiveStatsJs.gpuBusyAt(root.latest, modelData))
          fill: false
        }
      }
    }

    SettingRow {
      available: root.gpuList.length === 0
      sectionHelp: false
      label: "No GPUs"
      description: I18n.tr("Hardware inventory did not name a graphics device.")
      query: root.query
      keywords: ["empty", "gpu"]
    }
  }

  PrefsGroup {
    title: I18n.tr("hwmon")
    query: root.query
    detail: "Every readable temp*_input and fan*_input. Zero and missing stay off the list."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      PrefsField {
        width: parent.width
        placeholder: "Filter sensors…"
        onEdited: function(value) { root.sensorFilter = value }
      }

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: [
            { id: "all", label: "All" },
            { id: "temp", label: "Temp" },
            { id: "fan", label: "Fan" }
          ]

          PrefsButton {
            required property var modelData
            text: modelData && modelData.label ? modelData.label : ""
            primary: root.sensorKind === (modelData && modelData.id ? modelData.id : "")
            onClicked: root.sensorKind = modelData.id
          }
        }
      }
    }

    Repeater {
      model: root.sensors

      SettingRow {
        required property var modelData
        label: modelData && modelData.label ? modelData.label : "sensor"
        description: modelData && modelData.chip ? modelData.chip : ""
        hint: "/sys/class/hwmon"
        query: root.query
        keywords: ["sensor", "thermal", "fan"]
        valueText: root.sensorText(modelData)
        stretchControl: true

        PrefsSparkline {
          id: sensorSpark
          width: parent.width
          values: {
            if (!sensorSpark.inView) return []
            return LiveStatsJs.sensorSeries(root.history, modelData.id)
          }
          valueText: root.sensorText(modelData)
          alert: modelData && modelData.kind === "temp" && modelData.value >= 90
        }
      }
    }

    SettingRow {
      available: root.sensors.length === 0
      sectionHelp: false
      label: "No sensors"
      description: root.latest ? "Nothing matches that filter." : "Waiting for the first sample."
      query: root.query
      keywords: ["empty", "sensor"]
    }
  }
}
