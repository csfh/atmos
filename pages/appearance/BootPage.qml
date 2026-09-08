import QtQuick
import QtQuick.Dialogs
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "appearance/boot"
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

    SettingRow {
      label: "Unlock theme"
      description: "The Plymouth theme used at unlock. Default is the stock Omarchy logo."
      hint: "omarchy plymouth set by theme"
      query: root.query
      keywords: ["plymouth", "sddm", "login", "unlock"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.plymouth
        options: Omarchy.plymouthThemes
        enabled: Omarchy.plymouthThemes.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.plymouth) Omarchy.setPlymouth(value)
        }
      }
    }

    SettingRow {
      label: "Custom logo"
      description: "Pick a PNG for the unlock screen. Omarchy tints it with the current theme colors."
      hint: "omarchy plymouth set"
      query: root.query
      keywords: ["logo", "png", "custom", "unlock"]

      PrefsButton {
        text: "Choose…"
        enabled: !Omarchy.jobBusy
        onClicked: plymouthDialog.open()
      }
    }

    SettingRow {
      label: "Preview"
      description: "Render an unlock screen from a PNG so you can see it before you apply it."
      hint: "omarchy plymouth preview"
      query: root.query
      keywords: ["preview", "unlock", "plymouth", "logo"]

      PrefsButton {
        text: "Preview…"
        enabled: !Omarchy.jobBusy
        onClicked: plymouthPreviewDialog.open()
      }
    }

    SettingRow {
      label: "Reset boot screen"
      description: "Put the stock Omarchy unlock theme and login screen back."
      hint: "omarchy plymouth reset"
      query: root.query
      keywords: ["default", "unlock", "sddm", "login"]

      PrefsButton {
        text: "Reset…"
        danger: true
        enabled: !Omarchy.jobBusy && Omarchy.plymouth !== "default"
        onClicked: resetPlymouthConfirm.ask()
      }
    }
  }

  Component.onCompleted: {
    resetPlymouthConfirm.parent = root.prefsOverlay
  }
}
