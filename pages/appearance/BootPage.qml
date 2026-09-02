import QtQuick
import QtQuick.Dialogs
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Boot screen"
  description: "What you see while the machine unlocks, including the logo on the Plymouth screen."

  FileDialog {
    id: plymouthDialog
    title: "Set boot logo"
    nameFilters: ["PNG (*.png)"]
    onAccepted: Omarchy.setPlymouthFromPath(RichUi.pathFromUrl(selectedFile))
  }

  FileDialog {
    id: plymouthPreviewDialog
    title: "Preview boot logo"
    nameFilters: ["PNG (*.png)"]
    onAccepted: Omarchy.previewPlymouthFromPath(RichUi.pathFromUrl(selectedFile))
  }

  PrefsConfirm {
    id: resetPlymouthConfirm
    title: "Reset boot screen"
    message: "Put the stock Omarchy unlock theme and login screen back."
    confirmText: "Reset"
    onConfirmed: Omarchy.resetPlymouth()
  }

  PrefsGroup {
    title: "Plymouth"
    query: root.query
    detail: "The unlock animation before the desktop. A custom logo is a PNG tinted with the current theme. Preview renders it without applying it."

    PrefsRow {
      label: "Unlock theme"
      description: "The Plymouth theme used at unlock. Default is the stock Omarchy logo."
      hint: "omarchy plymouth set by theme"
      query: root.query
      keywords: ["plymouth", "sddm", "login", "unlock"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.plymouth
        options: Omarchy.plymouthThemes
        enabled: !Omarchy.busy && Omarchy.plymouthThemes.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.plymouth) Omarchy.setPlymouth(value)
        }
      }
    }

    PrefsRow {
      label: "Custom logo"
      description: "Pick a PNG for the unlock screen. Omarchy tints it with the current theme colors."
      hint: "omarchy plymouth set"
      query: root.query
      keywords: ["logo", "png", "custom", "unlock"]

      PrefsButton {
        text: "Choose logo…"
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: plymouthDialog.open()
      }
    }

    PrefsRow {
      label: "Preview"
      description: "Render an unlock screen from a PNG so you can see it before you apply it."
      hint: "omarchy plymouth preview"
      query: root.query
      keywords: ["preview", "unlock", "plymouth", "logo"]

      PrefsButton {
        text: "Preview…"
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: plymouthPreviewDialog.open()
      }
    }

    PrefsRow {
      available: Omarchy.plymouth !== "default"
      label: "Reset boot screen"
      description: "Put the stock Omarchy unlock theme and login screen back."
      hint: "omarchy plymouth reset"
      query: root.query
      keywords: ["default", "unlock", "sddm", "login"]

      PrefsButton {
        text: "Reset"
        danger: true
        enabled: !Omarchy.busy && !Omarchy.jobBusy && Omarchy.plymouth !== "default"
        onClicked: resetPlymouthConfirm.ask()
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: Omarchy.directBootAvailable ? root.query : "."
    detail: "Direct boot adds an EFI entry for the Omarchy UKI. Snapshot picking then happens in firmware."

    PrefsRow {
      available: Omarchy.directBootAvailable
      label: "Direct EFI boot"
      description: Omarchy.directBoot
        ? "Firmware has an Omarchy UKI entry. Run this again to remove it."
        : "Add an EFI entry that boots the Omarchy UKI directly."
      hint: "omarchy setup direct boot"
      query: root.query
      keywords: ["efi", "uki", "efibootmgr", "direct", "boot"]

      PrefsButton {
        text: Omarchy.directBoot ? "Remove…" : "Set up…"
        danger: Omarchy.directBoot
        enabled: !Omarchy.busy && !Omarchy.jobBusy && Omarchy.directBootAvailable
        onClicked: directBootConfirm.ask()
      }
    }
  }

  PrefsConfirm {
    id: directBootConfirm
    title: Omarchy.directBoot ? "Remove direct boot" : "Set up direct boot"
    message: Omarchy.directBoot
      ? "Delete the Omarchy EFI entry. The machine will boot through the usual menu again."
      : "Create an EFI entry for the Omarchy UKI. Snapshot booting then goes through firmware."
    confirmText: Omarchy.directBoot ? "Remove" : "Set up"
    onConfirmed: Omarchy.setupDirectBoot()
  }

  Component.onCompleted: {
    resetPlymouthConfirm.parent = root.prefsOverlay
    directBootConfirm.parent = root.prefsOverlay
  }
}
