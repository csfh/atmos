import QtQuick
import "../../components"
import "../../services"
import "../../services/Diagnostics.js" as DiagJs

PrefsPage {
  id: root
  hubId: "system/kernel"
  title: "Kernel"
  description: "The running image, how firmware starts it, and a couple of knobs that change kernel behavior."

  readonly property var diag: DiagJs.normalize(Omarchy.diagnostics)

  PrefsConfirm {
    id: directBootConfirm
    title: Omarchy.directBoot ? "Remove direct boot" : "Set up direct boot"
    message: Omarchy.directBoot
      ? "Delete the Omarchy firmware entry. The next start uses Limine again, so snapshot picking is in the bootloader menu."
      : "Firmware will get an Omarchy entry that loads the UKI in /boot/EFI/Linux. The next start skips Limine and goes to disk unlock. To boot a snapshot after that, open the firmware boot menu, pick Limine, then the snapshot."
    confirmText: Omarchy.directBoot ? "Remove" : "Set up"
    onConfirmed: Omarchy.setupDirectBoot()
  }

  Component.onCompleted: {
    directBootConfirm.parent = root.prefsOverlay
  }

  PrefsGroup {
    title: "This boot"
    query: root.query
    detail: "uname as the kernel reports it. Hardware has firmware and DMI."

    SettingRow {
      label: "Running kernel"
      description: DiagJs.kernelSummary(root.diag) || "Kernel version was not readable."
      hint: "uname -sr"
      query: root.query
      keywords: ["kernel", "uname", "linux", "arch", "release"]

      PrefsButton {
        text: "Copy"
        enabled: DiagJs.kernelSummary(root.diag).length > 0
        onClicked: Omarchy.copyText(DiagJs.kernelSummary(root.diag))
      }
    }
  }

  PrefsGroup {
    title: "Direct EFI boot"
    query: Omarchy.directBootAvailable ? root.query : "."
    detail: "A normal start opens Limine, Omarchy's bootloader. That menu is how you pick this install or an older Snapper snapshot after a bad update. Direct EFI boot writes a firmware entry named Omarchy that loads the Unified Kernel Image, a single EFI file under /boot/EFI/Linux that already contains the kernel, initramfs, and command line. Firmware starts that file. Limine never appears, and you go straight to disk unlock. Do this when you never choose a snapshot at power-on and want that pause gone. Snapshots stay on disk. To boot one afterward, open the firmware boot menu (often F12, F10, or Esc), choose Limine, then the snapshot by date. Run this setup again to delete the firmware entry and return to Limine on every start."

    SettingRow {
      available: Omarchy.directBootAvailable
      label: "Direct EFI boot"
      description: Omarchy.directBoot
        ? "Firmware boots the Omarchy UKI with no Limine stop. Remove the entry to get the snapshot menu back at every boot."
        : "Skip Limine and go straight to disk unlock. Snapshot rollback then starts from the firmware boot list."
      hint: "omarchy setup direct boot"
      query: root.query
      keywords: ["efi", "uki", "efibootmgr", "direct", "boot", "limine", "snapshot"]

      PrefsButton {
        text: Omarchy.directBoot ? "Remove…" : "Set up…"
        danger: Omarchy.directBoot
        enabled: !Omarchy.jobBusy && Omarchy.directBootAvailable
        onClicked: directBootConfirm.ask()
      }
    }
  }

  PrefsGroup {
    title: "Memory"
    query: root.query
    detail: "vm.swappiness. Lower values keep more file cache in RAM and wait longer before using swap."

    SettingRow {
      label: "Lower swappiness"
      description: Omarchy.tweaks && Omarchy.tweaks.swappiness
        ? "vm.swappiness is 10 via /etc/sysctl.d/99-atmos-swappiness.conf."
        : "Write vm.swappiness=10. Reset removes the drop-in."
      hint: "/etc/sysctl.d/99-atmos-swappiness.conf"
      query: root.query
      keywords: ["swap", "swappiness", "sysctl", "vm", "memory"]

      Row {
        spacing: Theme.space
        PrefsToggle {
          checked: !!(Omarchy.tweaks && Omarchy.tweaks.swappiness)
          onToggled: Omarchy.setTweak("swappiness", !(Omarchy.tweaks && Omarchy.tweaks.swappiness))
        }
        PrefsButton {
          text: "Reset"
          onClicked: Omarchy.setTweak("swappiness", false)
        }
      }
    }
  }
}
