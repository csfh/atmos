import QtQuick
import QtQuick.Dialogs
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Background"
  description: "The picture behind your windows. These controls apply to the current theme."

  FileDialog {
    id: wallpaperDialog
    title: "Set background"
    nameFilters: ["Images (*.jpg *.jpeg *.png *.gif *.webp *.bmp)"]
    onAccepted: Omarchy.setBackgroundPath(RichUi.pathFromUrl(selectedFile))
  }

  PrefsGroup {
    title: "Wallpaper"
    query: root.query
    detail: "These pictures belong to the current theme. Next walks the set. Choose an image if you want a file from disk. Extra files in the theme folder join the cycle."

    PrefsRow {
      label: "Current wallpaper"
      description: Omarchy.background.length ? Omarchy.background : "No wallpaper is set for this theme yet."
      hint: "omarchy theme bg current"
      query: root.query
      keywords: ["wallpaper", "image", "path"]
    }

    PrefsRow {
      label: "Next wallpaper"
      description: "Move to the next image this theme ships. Keep pressing if you want to browse the set."
      hint: "omarchy theme bg next"
      query: root.query
      keywords: ["wallpaper", "cycle"]

      PrefsButton {
        text: "Next"
        enabled: !Omarchy.busy
        onClicked: Omarchy.nextBackground()
      }
    }

    PrefsRow {
      label: "Choose an image"
      description: "Use a JPG, PNG, WebP, or similar file from disk as the wallpaper."
      hint: "omarchy theme bg set"
      query: root.query
      keywords: ["wallpaper", "file", "image", "path"]

      PrefsButton {
        text: "Choose file…"
        enabled: !Omarchy.busy
        onClicked: wallpaperDialog.open()
      }
    }

    PrefsRow {
      label: "Theme wallpaper folder"
      description: "Drop extra images in this theme's folder if you want them in the cycle."
      hint: "omarchy theme bg install"
      query: root.query
      keywords: ["wallpaper", "folder", "add", "user"]

      PrefsButton {
        text: "Open folder"
        enabled: !Omarchy.busy
        onClicked: Omarchy.openBackgroundFolder()
      }
    }

    PrefsRow {
      label: "Rebuild thumbnails"
      description: "Regenerate the small previews the wallpaper switcher shows for this theme."
      hint: "omarchy theme bg cache"
      query: root.query
      keywords: ["cache", "thumbnail", "preview", "wallpaper"]

      PrefsButton {
        text: "Cache"
        enabled: !Omarchy.busy
        onClicked: Omarchy.cacheBackgrounds()
      }
    }

    PrefsRow {
      available: Omarchy.hasAether
      label: "Aether"
      description: "Open Aether if you want to pull a palette from the current wallpaper."
      hint: "aether"
      query: root.query
      keywords: ["palette", "extract"]

      PrefsButton {
        text: "Open Aether"
        enabled: !Omarchy.busy
        onClicked: Omarchy.openAether()
      }
    }
  }
}
