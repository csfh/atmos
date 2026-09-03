import QtQuick
import "../components"
import "../services"
import "../services/Hardware.js" as HardwareJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Hardware"
  description: "What this machine is made of. Processor, memory, chipset, firmware, and the rest of the units the kernel can see."

  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)

  function hasText() {
    for (var i = 0; i < arguments.length; i++) {
      if (String(arguments[i] || "").length) return true
    }
    return false
  }

  function listQuery(list) {
    return (list instanceof Array) && list.length > 0 ? root.query : "."
  }

  function objectQuery(obj, keys) {
    if (!obj) return "."
    for (var i = 0; i < keys.length; i++) {
      var value = obj[keys[i]]
      if (value === true) return root.query
      if (typeof value === "number" && value) return root.query
      if (String(value || "").length) return root.query
    }
    return "."
  }

  function firmwareQuery() {
    if (root.objectQuery(root.hw.bios, ["vendor", "version", "date"]) !== ".")
      return root.query
    if (root.hw.bios.uefi || root.hw.tpm.present || root.hw.secureBoot.available)
      return root.query
    return "."
  }

  PrefsGroup {
    title: "Machine"
    query: root.objectQuery(root.hw.machine, ["vendor", "name", "family", "chassis", "version", "serial", "sku"])
    detail: "Name and chassis from DMI. Refresh reads the machine again, including memory use."

    PrefsRow {
      available: root.hasText(root.hw.machine.vendor, root.hw.machine.name, root.hw.machine.family, root.hw.machine.chassis)
      label: root.hw.machine.name || root.hw.machine.family || "This machine"
      description: HardwareJs.machineSummary(root.hw.machine) || "The firmware did not name this chassis."
      hint: "/sys/class/dmi/id"
      query: root.query
      keywords: ["machine", "system", "product", "chassis", "laptop", "desktop", "dmi", "smbios"]
    }

    PrefsRow {
      available: root.hasText(root.hw.machine.serial, root.hw.machine.sku)
      label: "Identity"
      description: root.hasText(root.hw.machine.serial)
        ? ("Serial " + root.hw.machine.serial + (root.hw.machine.sku ? (". SKU " + root.hw.machine.sku) : "") + ".")
        : ("SKU " + root.hw.machine.sku + ".")
      hint: "/sys/class/dmi/id/product_serial"
      query: root.query
      keywords: ["serial", "sku", "service tag"]
    }

    PrefsRow {
      label: "Refresh"
      description: "Read the units again. Memory use and temperatures change while the machine runs."
      hint: "snapshot"
      query: root.query
      keywords: ["reload", "rescan", "inventory"]

      PrefsButton {
        text: "Refresh"
        enabled: !Omarchy.busy
        onClicked: Omarchy.refresh()
      }
    }
  }

  PrefsGroup {
    title: "Motherboard"
    query: root.objectQuery(root.hw.board, ["vendor", "name", "version"])
    detail: "The board DMI names. Chipset is the host bridge on that board, listed next."

    PrefsRow {
      available: root.hasText(root.hw.board.vendor, root.hw.board.name)
      label: root.hw.board.name || "Board"
      description: HardwareJs.boardSummary(root.hw.board)
      hint: "/sys/class/dmi/id/board_name"
      query: root.query
      keywords: ["motherboard", "mainboard", "board", "baseboard"]
    }
  }

  PrefsGroup {
    title: "Chipset"
    query: root.objectQuery(root.hw.chipset, ["name", "vendor", "pciId", "southbridge"])
    detail: "The PCI host bridge, and the ISA or LPC bridge when the kernel names one. That is the chipset the CPU talks to."

    PrefsRow {
      available: root.hasText(root.hw.chipset.name, root.hw.chipset.vendor, root.hw.chipset.pciId)
      label: root.hw.chipset.name || "Host bridge"
      description: HardwareJs.chipsetSummary(root.hw.chipset)
      hint: "lspci"
      query: root.query
      keywords: ["chipset", "northbridge", "southbridge", "host bridge", "isa", "lpc", "pch", "pci"]
    }
  }

  PrefsGroup {
    title: "Firmware"
    query: root.firmwareQuery()
    detail: "BIOS or UEFI from DMI, plus TPM and Secure Boot when the firmware exposes them."

    PrefsRow {
      available: root.hasText(root.hw.bios.vendor, root.hw.bios.version, root.hw.bios.date) || root.hw.bios.uefi
      label: "BIOS"
      description: HardwareJs.biosSummary(root.hw.bios) || (root.hw.bios.uefi ? "UEFI is present." : "")
      hint: "/sys/class/dmi/id/bios_version"
      query: root.query
      keywords: ["bios", "uefi", "firmware", "efi"]
    }

    PrefsRow {
      available: root.hw.secureBoot.available
      label: "Secure Boot"
      description: root.hw.secureBoot.enabled
        ? "Secure Boot is on."
        : "UEFI is present. Secure Boot is off."
      hint: "/sys/firmware/efi"
      query: root.query
      keywords: ["secure boot", "efi", "mok"]
    }

    PrefsRow {
      available: root.hw.tpm.present
      label: "TPM"
      description: HardwareJs.tpmSummary(root.hw.tpm)
      hint: "/sys/class/tpm"
      query: root.query
      keywords: ["tpm", "trusted platform"]
    }
  }

  PrefsGroup {
    title: "Processor"
    query: root.objectQuery(root.hw.cpu, ["model", "vendor", "arch", "cores"])
    detail: "Cores and threads from the kernel. Flags are the ones that matter for guests and SIMD."

    PrefsRow {
      available: root.hasText(root.hw.cpu.model, root.hw.cpu.vendor)
      label: root.hw.cpu.model || "CPU"
      description: HardwareJs.cpuSummary(root.hw.cpu)
      hint: "lscpu"
      query: root.query
      keywords: ["cpu", "processor", "core", "thread", "avx", "intel", "amd", "arm"]
    }
  }

  PrefsGroup {
    title: "Memory"
    query: root.hw.memory.total > 0 || root.hw.memory.modules.length > 0 ? root.query : "."
    detail: "Use comes from /proc/meminfo. Modules are SMBIOS type 17 when the firmware table is readable without root."

    PrefsRow {
      available: root.hw.memory.total > 0
      stretchControl: true
      label: "Installed"
      description: ""
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["ram", "memory", "dimm", "ddr", "swap"]

      PrefsUsageBar {
        width: parent.width
        used: root.hw.memory.used
        size: root.hw.memory.total
        avail: root.hw.memory.available
      }
    }

    PrefsRow {
      available: root.hw.memory.swapTotal > 0
      label: "Swap"
      description: RichUi.formatBytes(root.hw.memory.swapUsed) + " of " + RichUi.formatBytes(root.hw.memory.swapTotal) + " in use."
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["swap", "zram"]
    }

    Repeater {
      model: root.hw.memory.modules

      PrefsRow {
        required property var modelData
        sectionHelp: false
        label: (modelData && modelData.locator) || "DIMM"
        description: HardwareJs.moduleSummary(modelData)
        hint: "dmidecode -t memory"
        query: root.query
        keywords: ["dimm", "sodimm", "ddr4", "ddr5", "module", "bank"]
      }
    }
  }

  PrefsGroup {
    title: "Graphics"
    query: root.listQuery(root.hw.gpus)
    detail: "PCI display devices, plus the DRM driver when the kernel bound one."

    Repeater {
      model: root.hw.gpus

      PrefsRow {
        required property var modelData
        label: (modelData && modelData.name) || "GPU"
        description: HardwareJs.gpuSummary(modelData)
        hint: "lspci"
        query: root.query
        keywords: ["gpu", "graphics", "vga", "nvidia", "amd", "intel", "drm"]
      }
    }
  }

  PrefsGroup {
    title: "Network adapters"
    query: root.listQuery(root.hw.nics)
    detail: "Physical interfaces the kernel registered. Virtual bridges and containers stay off this list."

    Repeater {
      model: root.hw.nics

      PrefsRow {
        required property var modelData
        label: (modelData && (modelData.iface || modelData.name)) || "NIC"
        description: HardwareJs.nicSummary(modelData)
        hint: "/sys/class/net"
        query: root.query
        keywords: ["nic", "ethernet", "wifi", "wlan", "adapter", "mac"]
      }
    }
  }

  PrefsGroup {
    title: "Audio"
    query: root.listQuery(root.hw.audio)
    detail: "Sound cards from ALSA. Volume and sinks stay on the Sound page."

    Repeater {
      model: root.hw.audio

      PrefsRow {
        required property var modelData
        label: (modelData && modelData.name) || "Audio"
        description: modelData && modelData.driver ? ("ALSA " + modelData.driver + ".") : "ALSA card."
        hint: "/proc/asound/cards"
        query: root.query
        keywords: ["audio", "sound", "alsa", "card"]
      }
    }
  }

  PrefsGroup {
    title: "USB"
    query: root.listQuery(root.hw.usb)
    detail: "Devices on the USB buses that published a product name."

    Repeater {
      model: root.hw.usb

      PrefsRow {
        required property var modelData
        label: (modelData && modelData.name) || "USB"
        description: [
          modelData && modelData.vendor ? modelData.vendor : "",
          modelData && modelData.speed ? (modelData.speed + " Mb/s") : ""
        ].filter(function(bit) { return bit.length }).join(". ") + ((modelData && (modelData.vendor || modelData.speed)) ? "." : "")
        hint: "/sys/bus/usb/devices"
        query: root.query
        keywords: ["usb", "hub", "keyboard", "mouse", "storage"]
      }
    }
  }

  PrefsGroup {
    title: "Battery"
    query: root.listQuery(root.hw.batteries)
    detail: "Charge from sysfs. Profiles and the bar percentage stay on the Power page."

    Repeater {
      model: root.hw.batteries

      PrefsRow {
        required property var modelData
        label: (modelData && modelData.name) || "Battery"
        description: HardwareJs.batterySummary(modelData)
        hint: "/sys/class/power_supply"
        query: root.query
        keywords: ["battery", "charge", "capacity"]
      }
    }
  }

  PrefsGroup {
    title: "Thermal"
    query: root.listQuery(root.hw.thermals)
    detail: "Zones the kernel exported. Refresh if you want a newer reading."

    Repeater {
      model: root.hw.thermals

      PrefsRow {
        required property var modelData
        label: (modelData && (modelData.name || modelData.type)) || "Sensor"
        description: HardwareJs.thermalSummary(modelData)
        hint: "/sys/class/thermal"
        query: root.query
        keywords: ["thermal", "temperature", "sensor", "heat"]
      }
    }
  }

  PrefsGroup {
    title: "Virtualization"
    query: root.objectQuery(root.hw.virtualization, ["hypervisor", "guest", "kvm"])
    detail: "Whether this OS is a guest, and whether KVM can run guests here."

    PrefsRow {
      available: root.hw.virtualization.guest || root.hw.virtualization.kvm || root.hasText(root.hw.virtualization.hypervisor)
      label: root.hw.virtualization.guest ? "Guest" : "Host"
      description: HardwareJs.virtSummary(root.hw.virtualization) || "No hypervisor was reported."
      hint: "lscpu"
      query: root.query
      keywords: ["kvm", "qemu", "hypervisor", "vm", "virtual", "guest"]
    }
  }
}
