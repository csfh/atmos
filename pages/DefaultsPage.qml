import QtQuick
import "../components"
import "../services"
import "../services/Defaults.js" as DefaultsJs

PrefsPage {
  id: root
  hubId: "defaults"
  title: "Defaults"
  description: "The programs Omarchy opens when something asks for a default."

  // The logic is in Defaults.js, which the Rust window's Defaults page is held
  // to (frontend/tests/defaults_parity.rs).
  readonly property var browserOptions: DefaultsJs.browserOptions()
  readonly property var terminalOptions: DefaultsJs.terminalOptions()
  readonly property var editorOptions: DefaultsJs.editorOptions()

  PrefsGroup {
    title: "Applications"
    query: root.query
    detail: "Omarchy writes these as the XDG defaults. Only programs that are installed show up in the lists."

    SettingRow {
      label: "Browser"
      description: "The browser that opens links and web apps. Only installed browsers show up."
      hint: "omarchy default browser"
      query: root.query
      keywords: ["web", "chrome", "firefox"]

      PrefsSelect {
        value: Omarchy.browser
        options: Omarchy.installedOptions(root.browserOptions, Omarchy.browsers)
        onChanged: function(value) { if (value !== Omarchy.browser) Omarchy.set("browser", value) }
      }
    }

    SettingRow {
      label: "Terminal"
      description: "The terminal other apps launch when they need a console. Only installed terminals show up."
      hint: "omarchy default terminal"
      query: root.query
      keywords: ["shell", "console"]

      PrefsSelect {
        value: Omarchy.terminal
        options: Omarchy.installedOptions(root.terminalOptions, Omarchy.terminals)
        onChanged: function(value) { if (value !== Omarchy.terminal) Omarchy.set("terminal", value) }
      }
    }

    SettingRow {
      label: "Editor"
      description: "The editor Omarchy opens when you ask to edit a file. Only installed editors show up."
      hint: "omarchy default editor"
      query: root.query
      keywords: ["nvim", "code", "zed"]

      PrefsSelect {
        value: DefaultsJs.editorValue(Omarchy.editor)
        options: Omarchy.installedOptions(root.editorOptions, DefaultsJs.editorsInstalled(Omarchy.editors))
        onChanged: function(value) {
          if (value !== DefaultsJs.editorValue(Omarchy.editor)) Omarchy.set("editor", value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    advanced: true
    query: root.query
    detail: "XDG defaults for PDFs, images, and video. Browser, terminal, and editor stay in the group above."

    SettingRow {
      label: "PDF"
      description: "The program that opens PDF files."
      hint: "xdg-mime default application/pdf"
      query: root.query
      keywords: ["pdf", "mime", "document", "evince", "zathura"]

      PrefsSelect {
        value: Omarchy.mimePdf
        options: DefaultsJs.mimeOptions(Omarchy.mimePdf, Omarchy.mimePdfOptions)
        enabled: DefaultsJs.mimeOptions(Omarchy.mimePdf, Omarchy.mimePdfOptions).length > 0
        onChanged: function(value) {
          if (value !== Omarchy.mimePdf) Omarchy.setMimeDefault("pdf", value)
        }
      }
    }

    SettingRow {
      label: "Images"
      description: "The program that opens pictures."
      hint: "xdg-mime default image/png"
      query: root.query
      keywords: ["image", "png", "jpeg", "mime", "imv"]

      PrefsSelect {
        value: Omarchy.mimeImage
        options: DefaultsJs.mimeOptions(Omarchy.mimeImage, Omarchy.mimeImageOptions)
        enabled: DefaultsJs.mimeOptions(Omarchy.mimeImage, Omarchy.mimeImageOptions).length > 0
        onChanged: function(value) {
          if (value !== Omarchy.mimeImage) Omarchy.setMimeDefault("image", value)
        }
      }
    }

    SettingRow {
      label: "Video"
      description: "The program that opens videos."
      hint: "xdg-mime default video/mp4"
      query: root.query
      keywords: ["video", "mp4", "mime", "mpv", "vlc"]

      PrefsSelect {
        value: Omarchy.mimeVideo
        options: DefaultsJs.mimeOptions(Omarchy.mimeVideo, Omarchy.mimeVideoOptions)
        enabled: DefaultsJs.mimeOptions(Omarchy.mimeVideo, Omarchy.mimeVideoOptions).length > 0
        onChanged: function(value) {
          if (value !== Omarchy.mimeVideo) Omarchy.setMimeDefault("video", value)
        }
      }
    }
  }
}
