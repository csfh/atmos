import QtQuick
import "../../components"
import "../../services"
import "../../services/Hardware.js" as HardwareJs

// Glanceable dashboard for this computer. Diagnostics is the report you
// copy when something is wrong. This page answers what the machine is and
// how it is doing, from snapshot and hardware inventory Atmos already has.
// Unknown stays unknown: a missing charge is not shown as 0%.

PrefsPage {
  id: root
  hubId: "system/machine"
  title: "Machine"
  description: "What this computer is, and how it is doing right now."

  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)
  readonly property string chargeLimitNote: Omarchy.chargeLimitAvailable && Omarchy.chargeLimit > 0
    ? "Charge limited to " + Omarchy.chargeLimit + "%."
    : ""

  function orUnknown(text) {
    var s = String(text || "").replace(/^\s+|\s+$/g, "")
    return s.length > 0 ? s : "unknown"
  }

  function batteryValue(item) {
    return HardwareJs.batterySummary(item) || "unknown"
  }

  function batteryHasCapacity(item) {
    return !!(item && item.capacity)
  }

  readonly property string modelLine: {
    var vendor = String(Omarchy.dmiVendor || "").replace(/^\s+|\s+$/g, "")
    var product = String(Omarchy.dmiProduct || "").replace(/^\s+|\s+$/g, "")
    if (!vendor && !product) return "unknown"
    if (vendor && product && product.toLowerCase().indexOf(vendor.toLowerCase()) === 0)
      return product
    return (vendor + " " + product).replace(/^\s+|\s+$/g, "")
  }

  PrefsGroup {
    title: "This computer"
    query: root.query
    detail: "Identity as the firmware reports it. Hardware has the full DMI dump."

    SettingRow {
      label: "Model"
      description: "Vendor and product from DMI."
      hint: "/sys/class/dmi/id"
      query: root.query
      keywords: ["machine", "model", "dmi", "vendor", "product"]
      valueText: root.modelLine
    }

    SettingRow {
      label: "Family"
      description: "The line this machine belongs to, when firmware names one."
      hint: "/sys/class/dmi/id"
      query: root.query
      keywords: ["family", "dmi"]
      available: String(Omarchy.dmiFamily || "").length > 0
      valueText: root.orUnknown(Omarchy.dmiFamily)
    }

    SettingRow {
      label: "Processor"
      description: "What is doing the work."
      hint: "lscpu"
      query: root.query
      keywords: ["cpu", "processor"]
      valueText: root.orUnknown(Omarchy.cpuIdentity)
    }

    SettingRow {
      label: "Graphics"
      description: "The GPU the compositor is drawing on."
      hint: "drm"
      query: root.query
      keywords: ["gpu", "graphics"]
      valueText: root.orUnknown(Omarchy.gpuIdentity)
    }

    SettingRow {
      label: "Neural engine"
      description: "Only shown when the machine actually has one."
      query: root.query
      keywords: ["npu", "neural"]
      available: String(Omarchy.npuIdentity || "").length > 0
      valueText: root.orUnknown(Omarchy.npuIdentity)
    }
  }

  PrefsGroup {
    title: "Right now"
    query: root.query
    detail: root.chargeLimitNote
      ? ("Live from the same snapshot the rest of Atmos reads. " + root.chargeLimitNote)
      : "Live from the same snapshot the rest of Atmos reads."

    SettingRow {
      label: "Processor load"
      description: "As the kernel reports it."
      hint: "/proc/stat"
      query: root.query
      keywords: ["load", "cpu", "usage"]
      available: String(Omarchy.cpuStat || "").length > 0
      valueText: root.orUnknown(Omarchy.cpuStat)
    }

    SettingRow {
      label: "Memory"
      description: "In use against what is fitted."
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["memory", "ram"]
      available: String(Omarchy.memoryStat || "").length > 0
      valueText: root.orUnknown(Omarchy.memoryStat)
    }

    Repeater {
      model: root.hw.batteries

      SettingRow {
        required property var modelData
        stretchControl: true
        label: (modelData && modelData.name) || "Battery"
        description: root.chargeLimitNote || "Charge from sysfs. Profiles stay on Power."
        hint: "/sys/class/power_supply"
        query: root.query
        keywords: ["battery", "charge", "capacity"]
        valueText: root.batteryValue(modelData)

        PrefsProgress {
          width: parent.width
          visible: root.batteryHasCapacity(modelData)
          from: 0
          to: 100
          value: modelData && modelData.capacity ? modelData.capacity : 0
        }
      }
    }

    SettingRow {
      label: "Battery"
      description: root.chargeLimitNote || "The inventory has not filled charge or status yet."
      hint: "/sys/class/power_supply"
      query: root.query
      keywords: ["battery", "charge", "capacity"]
      available: Omarchy.batteryPresent && root.hw.batteries.length === 0
      valueText: "unknown"
    }
  }

  PrefsGroup {
    title: "Storage"
    query: root.query
    detail: "Every mounted filesystem Atmos can see. Disks has the device detail."

    Repeater {
      model: Array.isArray(Omarchy.disks) ? Omarchy.disks : []

      SettingRow {
        required property var modelData
        stretchControl: true
        sectionHelp: false
        label: String(modelData && modelData.name ? modelData.name : "disk")
        description: String(modelData && modelData.mount ? modelData.mount : "")
        hint: "df"
        query: root.query
        keywords: ["disk", "storage", "df", "filesystem"]

        PrefsUsageBar {
          width: parent.width
          used: modelData && modelData.used ? Number(modelData.used) : 0
          size: modelData && modelData.size ? Number(modelData.size) : 0
          avail: modelData && modelData.avail ? Number(modelData.avail) : 0
        }
      }
    }

    SettingRow {
      label: "No filesystems reported"
      description: "The snapshot has not answered yet, or nothing is mounted that Atmos tracks."
      query: root.query
      keywords: ["disk", "storage"]
      available: !Array.isArray(Omarchy.disks) || Omarchy.disks.length === 0
    }
  }
}
