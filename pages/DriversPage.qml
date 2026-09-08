import QtQuick
import "../components"
import "../services"
import "../services/Hardware.js" as HardwareJs

PrefsPage {
  id: root
  title: "Drivers"
  description: "Bound GPU drivers, hybrid switching, and firmware updates through fwupd."

  readonly property var hw: HardwareJs.normalize(Omarchy.hardware)
  readonly property bool hasGraphics: !!(root.hw.gpus.length || Omarchy.hwNvidia || Omarchy.hwVulkan || Omarchy.hybridGpuAvailable)

  PrefsConfirm {
    id: hybridGpuConfirm
    title: "Switch GPU mode"
    message: Omarchy.hybridGpuMode === "Integrated"
      ? "Turn the dedicated GPU on (hybrid) and reboot."
      : "Use only the integrated GPU and reboot."
    confirmText: "Switch and reboot"
    onConfirmed: Omarchy.toggleHybridGpu()
  }

  PrefsConfirm {
    id: firmwareConfirm
    title: "Firmware update"
    message: "Ask fwupd to install available firmware. You may need to reboot afterward."
    confirmText: "Update"
    onConfirmed: Omarchy.updateFirmware()
  }

  Component.onCompleted: {
    hybridGpuConfirm.parent = root.prefsOverlay
    firmwareConfirm.parent = root.prefsOverlay
  }

  function hasText() {
    for (var i = 0; i < arguments.length; i++) {
      if (String(arguments[i] || "").length) return true
    }
    return false
  }

  function copyField(text) {
    Omarchy.copyText(String(text || ""))
  }

  PrefsGroup {
    framed: true
    title: "Graphics"
    query: root.query
    detail: "PCI display devices, plus the DRM driver when the kernel bound one. Active is NVIDIA when that GPU is present, otherwise Vulkan. Hybrid switching reboots."

    SettingRow {
      available: !root.hasGraphics
      label: "Graphics"
      description: "No GPUs reported."
      hint: "lspci"
      query: root.query
      keywords: ["gpu", "graphics", "vga", "drm"]
    }

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
    title: "Firmware"
    query: root.query
    detail: "Identity is on Hardware. Update asks fwupd to install vendor firmware."

    SettingRow {
      available: root.hasText(root.hw.bios.vendor, root.hw.bios.version, root.hw.bios.date) || root.hw.bios.uefi
      label: "BIOS"
      description: HardwareJs.biosSummary(root.hw.bios) || (root.hw.bios.uefi ? "UEFI firmware." : "")
      hint: "/sys/class/dmi/id/bios_version"
      query: root.query
      keywords: ["bios", "uefi", "efi"]

      PrefsButton {
        text: "Copy"
        enabled: root.hasText(HardwareJs.biosSummary(root.hw.bios), root.hw.bios.version)
        onClicked: root.copyField(HardwareJs.biosSummary(root.hw.bios) || root.hw.bios.version)
      }
    }

    SettingRow {
      label: "Firmware"
      description: Omarchy.jobKind === "update-firmware" && Omarchy.jobBusy
        ? "Updating firmware…"
        : "Install firmware updates through fwupd when the vendor ships them."
      hint: "omarchy update firmware"
      query: root.query
      keywords: ["firmware", "fwupd", "bios", "update"]

      PrefsButton {
        text: "Update…"
        enabled: !Omarchy.jobBusy
        onClicked: firmwareConfirm.ask()
      }
    }

    SettingRow {
      label: "Refresh"
      description: "Read the units again after a firmware or GPU change."
      hint: "snapshot"
      query: root.query
      keywords: ["reload", "rescan"]

      PrefsButton {
        text: "Refresh"
        onClicked: Omarchy.refresh()
      }
    }
  }
}
