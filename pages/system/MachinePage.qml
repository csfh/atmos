import QtQuick
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

// The machine, at a glance.
//
// Diagnostics already exists and is good, but it is a report -- you go there
// when something is wrong and you want text to hand someone. This is the
// other thing: the page you open because you want to know what this laptop
// is and how it is doing, and it answers in one screen without you reading a
// single line of output.
//
// It is also, frankly, the page people screenshot. For a project growing by
// word of mouth that is not a vanity concern, it is distribution.
//
// Everything here is already in the snapshot. No new probe, no new process,
// no polling loop -- if a value is unknown the row says so rather than
// inventing a zero, because a dashboard that displays a confident 0% for
// something it failed to read is worse than one that admits the gap.

PrefsPage {
  id: root
  title: "Machine"
  description: "What this computer is, and how it is doing right now."

  function orUnknown(text) {
    var s = String(text || "").replace(/^\s+|\s+$/g, "")
    return s.length > 0 ? s : "unknown"
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

    PrefsRow {
      label: "Model"
      description: "Vendor and product from DMI."
      query: root.query
      valueText: root.modelLine
    }

    PrefsRow {
      label: "Family"
      description: "The line this machine belongs to, when firmware names one."
      query: root.query
      available: String(Omarchy.dmiFamily || "").length > 0
      valueText: root.orUnknown(Omarchy.dmiFamily)
    }

    PrefsRow {
      label: "Processor"
      description: "What is doing the work."
      query: root.query
      valueText: root.orUnknown(Omarchy.cpuIdentity)
    }

    PrefsRow {
      label: "Graphics"
      description: "The GPU the compositor is drawing on."
      query: root.query
      valueText: root.orUnknown(Omarchy.gpuIdentity)
    }

    PrefsRow {
      label: "Neural engine"
      description: "Only shown when the machine actually has one."
      query: root.query
      available: String(Omarchy.npuIdentity || "").length > 0
      valueText: root.orUnknown(Omarchy.npuIdentity)
    }
  }

  PrefsGroup {
    title: "Right now"
    query: root.query
    detail: "Live from the same snapshot the rest of Atmos reads."

    PrefsRow {
      label: "Processor load"
      description: "As the kernel reports it."
      query: root.query
      available: String(Omarchy.cpuStat || "").length > 0
      valueText: root.orUnknown(Omarchy.cpuStat)
    }

    PrefsRow {
      label: "Memory"
      description: "In use against what is fitted."
      query: root.query
      available: String(Omarchy.memoryStat || "").length > 0
      valueText: root.orUnknown(Omarchy.memoryStat)
    }

    PrefsRow {
      label: "Battery"
      description: "Only present on a machine that has one."
      query: root.query
      available: Omarchy.batteryPresent
      valueText: Omarchy.chargeLimitAvailable && Omarchy.chargeLimit > 0
        ? "charge limited to " + Omarchy.chargeLimit + "%"
        : "present"
    }
  }

  // Storage gets its own section with real bars. A number tells you the
  // figure; a bar tells you whether to care, which is the job of a dashboard.
  PrefsGroup {
    title: "Storage"
    query: root.query
    detail: "Every mounted filesystem Atmos can see. Disks has the device detail."

    Repeater {
      model: Array.isArray(Omarchy.disks) ? Omarchy.disks : []

      delegate: PrefsRow {
        required property var modelData
        label: String(modelData && modelData.name ? modelData.name : "disk")
        description: String(modelData && modelData.mount ? modelData.mount : "")
        query: root.query
        stretchControl: true

        PrefsUsageBar {
          used: modelData && modelData.used ? Number(modelData.used) : 0
          size: modelData && modelData.size ? Number(modelData.size) : 0
          avail: modelData && modelData.avail ? Number(modelData.avail) : 0
        }
      }
    }

    PrefsRow {
      label: "No filesystems reported"
      description: "The snapshot has not answered yet, or nothing is mounted that Atmos tracks."
      query: root.query
      available: !Array.isArray(Omarchy.disks) || Omarchy.disks.length === 0
    }
  }
}
