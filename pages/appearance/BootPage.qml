import QtQuick
import QtQuick.Dialogs
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "appearance/boot"
  title: I18n.tr("Boot screen")
  description: I18n.tr("What you see while the machine unlocks, including the logo on the Plymouth screen.")

  FileDialog {
    id: plymouthDialog
    title: I18n.tr("Set boot logo")
    nameFilters: ["PNG (*.png)"]
    onAccepted: Omarchy.setPlymouthFromPath(RichUi.pathFromUrl(selectedFile))
  }

  FileDialog {
    id: plymouthPreviewDialog
    title: I18n.tr("Preview boot logo")
    nameFilters: ["PNG (*.png)"]
    onAccepted: Omarchy.previewPlymouthFromPath(RichUi.pathFromUrl(selectedFile))
  }

  PrefsConfirm {
    id: resetPlymouthConfirm
    title: I18n.tr("Reset boot screen")
    message: "Put the stock Omarchy unlock theme and login screen back."
    confirmText: "Reset"
    onConfirmed: Omarchy.resetPlymouth()
  }

  PrefsGroup {
    title: I18n.tr("Plymouth")
    query: root.query
    detail: "The unlock animation before the desktop. A custom logo is a PNG tinted with the current theme. Preview renders it without applying it."

    SettingRow {
      label: "Unlock theme"
      description: I18n.tr("The Plymouth theme used at unlock. Default is the stock Omarchy logo.")
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
      description: I18n.tr("Pick a PNG for the unlock screen. Omarchy tints it with the current theme colors.")
      hint: "omarchy plymouth set"
      query: root.query
      keywords: ["logo", "png", "custom", "unlock"]

      PrefsButton {
        text: I18n.tr("Choose…")
        enabled: !Omarchy.jobBusy
        onClicked: plymouthDialog.open()
      }
    }

    SettingRow {
      label: "Preview"
      description: I18n.tr("Render an unlock screen from a PNG so you can see it before you apply it.")
      hint: "omarchy plymouth preview"
      query: root.query
      keywords: ["preview", "unlock", "plymouth", "logo"]

      PrefsButton {
        text: I18n.tr("Preview…")
        enabled: !Omarchy.jobBusy
        onClicked: plymouthPreviewDialog.open()
      }
    }

    SettingRow {
      label: "Reset boot screen"
      description: I18n.tr("Put the stock Omarchy unlock theme and login screen back.")
      hint: "omarchy plymouth reset"
      query: root.query
      keywords: ["default", "unlock", "sddm", "login"]

      PrefsButton {
        text: I18n.tr("Reset…")
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
