import QtQuick
import "../components"
import "../services"
import "../services/Hardware.js" as HardwareJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Hardware"
  description: "What this machine is made of. Processor, memory, chipset, firmware, graphics, NPU, and the rest of the units the kernel can see."

  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)

  PrefsConfirm {
    id: hybridGpuConfirm
    title: "Switch GPU mode"
    message: Omarchy.hybridGpuMode === "Integrated"
      ? "Turn the dedicated GPU on (hybrid) and reboot."
      : "Use only the integrated GPU and reboot."
    confirmText: "Switch and reboot"
    onConfirmed: Omarchy.toggleHybridGpu()
  }

  Component.onCompleted: {
    hybridGpuConfirm.parent = root.prefsOverlay
  }

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

  function copyField(text) {
    Omarchy.copyText(String(text || ""))
  }

  PrefsGroup {
    title: "Machine"
    query: root.objectQuery(root.hw.machine, ["vendor", "name", "family", "chassis", "version", "serial", "sku"])
    detail: "Name and chassis from DMI. Refresh reads the machine again, including memory use."

    SettingRow {
      available: root.hasText(root.hw.machine.vendor, root.hw.machine.name, root.hw.machine.family, root.hw.machine.chassis)
      label: root.hw.machine.name || root.hw.machine.family || "This machine"
      description: HardwareJs.machineSummary(root.hw.machine) || "The firmware did not name this chassis."
      hint: "/sys/class/dmi/id"
      query: root.query
      keywords: ["machine", "system", "product", "chassis", "laptop", "desktop", "dmi", "smbios"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(HardwareJs.machineSummary(root.hw.machine), root.hw.machine.name)
        onClicked: root.copyField(HardwareJs.machineSummary(root.hw.machine) || root.hw.machine.name)
      }
    }

    SettingRow {
      available: root.hasText(root.hw.machine.serial, root.hw.machine.sku)
      label: "Identity"
      description: root.hasText(root.hw.machine.serial)
        ? ("Serial " + root.hw.machine.serial + (root.hw.machine.sku ? (". SKU " + root.hw.machine.sku) : "") + ".")
        : ("SKU " + root.hw.machine.sku + ".")
      hint: "/sys/class/dmi/id/product_serial"
      query: root.query
      keywords: ["serial", "sku", "service tag"]

      PrefsButton {
        text: root.hasText(root.hw.machine.serial) ? "Copy serial" : "Copy SKU"
        enabled: root.hasText(root.hw.machine.serial, root.hw.machine.sku)
        onClicked: root.copyField(root.hw.machine.serial || root.hw.machine.sku)
      }
    }

    SettingRow {
      label: "Refresh"
      description: "Read the units again. Memory use and temperatures change while the machine runs."
      hint: "snapshot"
      query: root.query
      keywords: ["reload", "rescan", "inventory"]

      PrefsButton {
        text: "Refresh"
        onClicked: Omarchy.refresh()
      }
    }
  }

  PrefsGroup {
    title: "Motherboard"
    query: root.objectQuery(root.hw.board, ["vendor", "name", "version"])
    detail: "The board DMI names. Chipset is the host bridge on that board, listed next."

    SettingRow {
      available: root.hasText(root.hw.board.vendor, root.hw.board.name)
      label: root.hw.board.name || "Board"
      description: HardwareJs.boardSummary(root.hw.board)
      hint: "/sys/class/dmi/id/board_name"
      query: root.query
      keywords: ["motherboard", "mainboard", "board", "baseboard"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(HardwareJs.boardSummary(root.hw.board))
        onClicked: root.copyField(HardwareJs.boardSummary(root.hw.board))
      }
    }
  }

  PrefsGroup {
    title: "Chipset"
    query: root.objectQuery(root.hw.chipset, ["name", "vendor", "pciId", "southbridge"])
    detail: "The PCI host bridge, and the ISA or LPC bridge when the kernel names one. That is the chipset the CPU talks to."

    SettingRow {
      available: root.hasText(root.hw.chipset.name, root.hw.chipset.vendor, root.hw.chipset.pciId)
      label: root.hw.chipset.name || "Host bridge"
      description: HardwareJs.chipsetSummary(root.hw.chipset)
      hint: "lspci"
      query: root.query
      keywords: ["chipset", "northbridge", "southbridge", "host bridge", "isa", "lpc", "pch", "pci"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(HardwareJs.chipsetSummary(root.hw.chipset))
        onClicked: root.copyField(HardwareJs.chipsetSummary(root.hw.chipset))
      }
    }
  }

  PrefsGroup {
    title: "Firmware"
    query: root.firmwareQuery()
    detail: "BIOS or UEFI from DMI, plus TPM and Secure Boot when the firmware exposes them."

    SettingRow {
      available: root.hasText(root.hw.bios.vendor, root.hw.bios.version, root.hw.bios.date) || root.hw.bios.uefi
      label: "BIOS"
      description: HardwareJs.biosSummary(root.hw.bios) || (root.hw.bios.uefi ? "UEFI firmware." : "")
      hint: "/sys/class/dmi/id/bios_version"
      query: root.query
      keywords: ["bios", "uefi", "firmware", "efi"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(HardwareJs.biosSummary(root.hw.bios), root.hw.bios.version)
        onClicked: root.copyField(HardwareJs.biosSummary(root.hw.bios) || root.hw.bios.version)
      }
    }

    SettingRow {
      available: root.hw.secureBoot.available
      label: "Secure Boot"
      description: root.hw.secureBoot.enabled
        ? "The firmware is verifying boot loaders. Change this in UEFI setup, not here."
        : "The firmware is not verifying boot loaders. Change this in UEFI setup, not here."
      hint: "/sys/firmware/efi"
      query: root.query
      keywords: ["secure boot", "efi", "mok"]
      valueText: root.hw.secureBoot.enabled ? "On" : "Off"
    }

    SettingRow {
      available: root.hw.tpm.present
      label: "TPM"
      description: HardwareJs.tpmSummary(root.hw.tpm)
      hint: "/sys/class/tpm"
      query: root.query
      keywords: ["tpm", "trusted platform"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(HardwareJs.tpmSummary(root.hw.tpm))
        onClicked: root.copyField(HardwareJs.tpmSummary(root.hw.tpm))
      }
    }
  }

  PrefsGroup {
    title: "Processor"
    query: root.objectQuery(root.hw.cpu, ["model", "vendor", "arch", "cores"])
    detail: "Cores and threads from the kernel. Flags are the ones that matter for guests and SIMD."

    SettingRow {
      available: root.hasText(root.hw.cpu.model, root.hw.cpu.vendor)
      label: root.hw.cpu.model || "CPU"
      description: HardwareJs.cpuSummary(root.hw.cpu)
      hint: "lscpu"
      query: root.query
      keywords: ["cpu", "processor", "core", "thread", "avx", "intel", "amd", "arm"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(root.hw.cpu.model)
        onClicked: root.copyField(root.hw.cpu.model)
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Memory"
    query: root.hw.memory.total > 0 || root.hw.memory.modules.length > 0 ? root.query : "."
    detail: "Use comes from /proc/meminfo. Modules are SMBIOS type 17 when the firmware table is readable without root."

    SettingRow {
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

    SettingRow {
      available: root.hw.memory.swapTotal > 0
      stretchControl: true
      label: "Swap"
      description: ""
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["swap", "zram"]

      PrefsUsageBar {
        width: parent.width
        used: root.hw.memory.swapUsed
        size: root.hw.memory.swapTotal
        avail: Math.max(0, root.hw.memory.swapTotal - root.hw.memory.swapUsed)
      }
    }

    Repeater {
      model: root.hw.memory.modules

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: (modelData && modelData.locator) || "DIMM"
        description: HardwareJs.moduleSummary(modelData)
        hint: "dmidecode -t memory"
        query: root.query
        keywords: ["dimm", "sodimm", "ddr4", "ddr5", "module", "bank"]

        PrefsButton {
          text: "Copy"
          enabled: !!(modelData && HardwareJs.moduleSummary(modelData))
          onClicked: root.copyField(HardwareJs.moduleSummary(modelData))
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Graphics"
    query: (root.hw.gpus.length || Omarchy.hwNvidia || Omarchy.hwVulkan || Omarchy.hybridGpuAvailable) ? root.query : "."
    detail: "PCI display devices, plus the DRM driver when the kernel bound one. Active is NVIDIA when that GPU is present, otherwise Vulkan. Hybrid switching reboots."

    Repeater {
      model: root.hw.gpus

      SettingRow {
        required property var modelData
        label: (modelData && modelData.name) || "GPU"
        description: HardwareJs.gpuSummary(modelData)
        hint: "lspci"
        query: root.query
        keywords: ["gpu", "graphics", "vga", "nvidia", "amd", "intel", "drm"]

        PrefsButton {
          text: "Copy"
          enabled: !!(modelData && (modelData.name || HardwareJs.gpuSummary(modelData)))
          onClicked: root.copyField(HardwareJs.gpuSummary(modelData) || (modelData && modelData.name) || "")
        }
      }
    }

    SettingRow {
      available: Omarchy.hwNvidia || Omarchy.hwVulkan
      label: "Active stack"
      description: Omarchy.hwNvidia
        ? (Omarchy.hwNvidiaGsp
          ? "NVIDIA, with GSP firmware (Turing or newer)."
          : (Omarchy.hwNvidiaWithoutGsp
            ? "NVIDIA, without GSP firmware (Maxwell, Pascal, or Volta)."
            : "NVIDIA."))
        : "Vulkan."
      hint: Omarchy.hwNvidia ? "omarchy hw nvidia" : "omarchy hw vulkan"
      query: root.query
      keywords: ["vulkan", "nvidia", "gsp", "turing", "cuda", "api"]

      PrefsButton {
        text: "Copy"
        onClicked: root.copyField(Omarchy.hwNvidia
          ? (Omarchy.hwNvidiaGsp ? "NVIDIA GSP" : (Omarchy.hwNvidiaWithoutGsp ? "NVIDIA without GSP" : "NVIDIA"))
          : "Vulkan")
      }
    }

    SettingRow {
      available: Omarchy.hybridGpuAvailable
      label: "Hybrid GPU"
      description: Omarchy.hybridGpuMode === "Integrated"
        ? "Using the integrated GPU only. Switch to hybrid if you want the dedicated GPU."
        : (Omarchy.hybridGpuMode === "Hybrid"
          ? "Hybrid mode. The dedicated GPU can wake for a game or CUDA."
          : "This machine can switch between integrated-only and hybrid.")
      hint: "omarchy toggle hybrid gpu"
      query: root.query
      keywords: ["hybrid", "supergfx", "igpu"]

      PrefsButton {
        text: "Switch…"
        enabled: !Omarchy.jobBusy && Omarchy.hybridGpuAvailable
        onClicked: hybridGpuConfirm.ask()
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "NPU"
    query: root.listQuery(root.hw.npus)
    detail: "A neural processor on PCI, such as AMD XDNA, when one is present."

    Repeater {
      model: root.hw.npus

      SettingRow {
        required property var modelData
        label: (modelData && modelData.name) || "NPU"
        description: HardwareJs.npuSummary(modelData)
        hint: "lspci"
        query: root.query
        keywords: ["npu", "xdna", "neural", "ai", "tpu", "accelerator"]

        PrefsButton {
          text: "Copy"
          enabled: !!(modelData && (modelData.name || HardwareJs.npuSummary(modelData)))
          onClicked: root.copyField(HardwareJs.npuSummary(modelData) || (modelData && modelData.name) || "")
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Network adapters"
    query: root.listQuery(root.hw.nics)
    detail: "Physical interfaces the kernel registered. Virtual bridges and containers stay off this list."

    Repeater {
      model: root.hw.nics

      SettingRow {
        required property var modelData
        label: (modelData && (modelData.iface || modelData.name)) || "NIC"
        description: HardwareJs.nicSummary(modelData)
        hint: "/sys/class/net"
        query: root.query
        keywords: ["nic", "ethernet", "wifi", "wlan", "adapter", "mac"]

        PrefsButton {
          text: modelData && modelData.mac ? "Copy MAC" : "Copy"
          enabled: !!(modelData && (modelData.mac || modelData.iface || modelData.name))
          onClicked: root.copyField((modelData && (modelData.mac || modelData.iface || modelData.name)) || "")
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Audio"
    query: root.listQuery(root.hw.audio)
    detail: "Sound cards from ALSA. Volume and sinks stay on the Sound page."

    Repeater {
      model: root.hw.audio

      SettingRow {
        required property var modelData
        label: (modelData && modelData.name) || "Audio"
        description: modelData && modelData.driver ? ("ALSA " + modelData.driver + ". Volume and sinks are on Sound.") : "ALSA card. Volume and sinks are on Sound."
        hint: "/proc/asound/cards"
        query: root.query
        keywords: ["audio", "sound", "alsa", "card"]

        PrefsButton {
          text: "Copy"
          enabled: !!(modelData && modelData.name)
          onClicked: root.copyField((modelData && modelData.name) || "")
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "USB"
    query: root.listQuery(root.hw.usb)
    detail: "Devices on the USB buses that published a product name."

    Repeater {
      model: root.hw.usb

      SettingRow {
        required property var modelData
        label: (modelData && modelData.name) || "USB"
        description: [
          modelData && modelData.vendor ? modelData.vendor : "",
          modelData && modelData.speed ? (modelData.speed + " Mb/s") : ""
        ].filter(function(bit) { return bit.length }).join(". ") + ((modelData && (modelData.vendor || modelData.speed)) ? "." : "")
        hint: "/sys/bus/usb/devices"
        query: root.query
        keywords: ["usb", "hub", "keyboard", "mouse", "storage"]

        PrefsButton {
          text: "Copy"
          enabled: !!(modelData && modelData.name)
          onClicked: root.copyField((modelData && modelData.name) || "")
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Battery"
    query: root.listQuery(root.hw.batteries)
    detail: "Charge from sysfs. Profiles and the bar percentage stay on the Power page."

    Repeater {
      model: root.hw.batteries

      SettingRow {
        required property var modelData
        stretchControl: true
        label: (modelData && modelData.name) || "Battery"
        description: {
          var s = HardwareJs.batterySummary(modelData)
          return s ? (s + " Charge profiles stay on Power.") : "Charge profiles stay on Power."
        }
        hint: "/sys/class/power_supply"
        query: root.query
        keywords: ["battery", "charge", "capacity"]

        Column {
          width: parent.width
          spacing: Theme.space

          PrefsProgress {
            width: parent.width
            visible: !!(modelData && modelData.capacity)
            from: 0
            to: 100
            value: modelData && modelData.capacity ? modelData.capacity : 0
            valueText: (modelData && modelData.capacity ? modelData.capacity : 0) + "%"
          }

          Row {
            spacing: Theme.space
            PrefsButton {
              text: "Show battery"
              enabled: Omarchy.batteryPresent
              onClicked: Omarchy.showBatteryNotification()
            }
            PrefsButton {
              text: "Copy"
              enabled: !!HardwareJs.batterySummary(modelData)
              onClicked: root.copyField(HardwareJs.batterySummary(modelData))
            }
          }
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Thermal"
    query: root.listQuery(root.hw.thermals)
    detail: "Zones the kernel exported. Refresh if you want a newer reading."

    Repeater {
      model: root.hw.thermals

      SettingRow {
        required property var modelData
        label: (modelData && (modelData.name || modelData.type)) || "Sensor"
        description: HardwareJs.thermalSummary(modelData)
        hint: "/sys/class/thermal"
        query: root.query
        keywords: ["thermal", "temperature", "sensor", "heat"]

        PrefsButton {
          text: "Copy"
          enabled: !!HardwareJs.thermalSummary(modelData)
          onClicked: root.copyField(HardwareJs.thermalSummary(modelData))
        }
      }
    }
  }

  PrefsGroup {
    title: "Virtualization"
    query: root.objectQuery(root.hw.virtualization, ["hypervisor", "guest", "kvm"])
    detail: "Whether this OS is a guest, and whether KVM can run guests here."

    SettingRow {
      available: root.hw.virtualization.guest || root.hw.virtualization.kvm || root.hasText(root.hw.virtualization.hypervisor)
      label: root.hw.virtualization.guest ? "Guest" : "Host"
      description: HardwareJs.virtSummary(root.hw.virtualization) || "No hypervisor was reported."
      hint: "lscpu"
      query: root.query
      keywords: ["kvm", "qemu", "hypervisor", "vm", "virtual", "guest"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(HardwareJs.virtSummary(root.hw.virtualization))
        onClicked: root.copyField(HardwareJs.virtSummary(root.hw.virtualization))
      }
    }
  }
}
