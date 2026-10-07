import QtQuick
import QtQuick.Dialogs
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "appearance/background"
  title: I18n.tr("Background")
  description: I18n.tr("The picture behind your windows. These controls apply to the current theme.")

  FileDialog {
    id: wallpaperDialog
    title: I18n.tr("Set background")
    nameFilters: ["Images (*.jpg *.jpeg *.png *.gif *.webp *.bmp)"]
    onAccepted: Omarchy.setBackgroundPath(RichUi.pathFromUrl(selectedFile))
  }

  PrefsGroup {
    title: I18n.tr("Wallpaper")
    query: root.query
    detail: "These pictures belong to the current theme. Next walks the set. Choose an image if you want a file from disk. Extra files in the theme folder join the cycle."

    SettingRow {
      label: Omarchy.background.length ? RichUi.fileBasename(Omarchy.background) : "Current wallpaper"
      description: Omarchy.background.length
        ? Omarchy.background
        : "No wallpaper is set for this theme yet. Choose a file below, or Next if the theme ships images."
      hint: "omarchy theme bg current"
      query: root.query
      keywords: ["wallpaper", "image", "path"]

      Row {
        spacing: Theme.space

        Image {
          visible: Omarchy.background.length > 0
          width: 72
          height: 48
          fillMode: Image.PreserveAspectCrop
          asynchronous: true
          source: Omarchy.background.length ? ("file://" + encodeURI(Omarchy.background)) : ""
        }

        PrefsButton {
          text: I18n.tr("Copy path")
          enabled: Omarchy.background.length > 0
          onClicked: Omarchy.copyText(Omarchy.background)
        }
      }
    }

    SettingRow {
      label: "Next wallpaper"
      description: I18n.tr("Move to the next image this theme ships. Keep pressing if you want to browse the set.")
      hint: "omarchy theme bg next"
      query: root.query
      keywords: ["wallpaper", "cycle"]

      PrefsButton {
        text: I18n.tr("Next")
        onClicked: Omarchy.nextBackground()
      }
    }

    SettingRow {
      label: "Choose an image"
      description: I18n.tr("Use a JPG, PNG, WebP, or similar file from disk as the wallpaper.")
      hint: "omarchy theme bg set"
      query: root.query
      keywords: ["wallpaper", "file", "image", "path"]

      PrefsButton {
        text: I18n.tr("Choose…")
        onClicked: wallpaperDialog.open()
      }
    }

    SettingRow {
      label: "Theme wallpaper folder"
      description: I18n.tr("Drop extra images in this theme's folder if you want them in the cycle.")
      hint: "omarchy theme bg install"
      query: root.query
      keywords: ["wallpaper", "folder", "add", "user"]

      PrefsButton {
        text: I18n.tr("Open folder")
        onClicked: Omarchy.openBackgroundFolder()
      }
    }

    SettingRow {
      label: "Rebuild thumbnails"
      description: I18n.tr("Regenerate the small previews the wallpaper switcher shows for this theme.")
      hint: "omarchy theme bg cache"
      query: root.query
      keywords: ["cache", "thumbnail", "preview", "wallpaper"]

      PrefsButton {
        text: I18n.tr("Rebuild")
        onClicked: Omarchy.cacheBackgrounds()
      }
    }

    SettingRow {
      available: Omarchy.hasAether
      label: "Aether"
      description: I18n.tr("Open Aether if you want to pull a palette from the current wallpaper.")
      hint: "aether"
      query: root.query
      keywords: ["palette", "extract"]

      PrefsButton {
        text: I18n.tr("Open Aether")
        onClicked: Omarchy.openAether()
      }
    }
  }
}
