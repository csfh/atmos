import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Defaults"
  description: "The programs Omarchy opens when something asks for a default. The coding agent is a separate pick."

  readonly property var browserOptions: [
    { value: "chromium", label: "Chromium" },
    { value: "chrome", label: "Chrome" },
    { value: "brave", label: "Brave" },
    { value: "brave-origin", label: "Brave Origin" },
    { value: "edge", label: "Edge" },
    { value: "firefox", label: "Firefox" },
    { value: "zen", label: "Zen" }
  ]
  readonly property var terminalOptions: [
    { value: "alacritty", label: "Alacritty" },
    { value: "foot", label: "Foot" },
    { value: "ghostty", label: "Ghostty" },
    { value: "kitty", label: "Kitty" }
  ]
  readonly property var editorOptions: [
    { value: "nvim", label: "Neovim" },
    { value: "code", label: "VS Code" },
    { value: "cursor", label: "Cursor" },
    { value: "zed", label: "Zed" },
    { value: "sublime_text", label: "Sublime Text" },
    { value: "helix", label: "Helix" },
    { value: "vim", label: "Vim" },
    { value: "emacs", label: "Emacs" }
  ]
  function mimeOptions(current, list) {
    var out = []
    if (list instanceof Array) {
      for (var i = 0; i < list.length; i++) out.push(list[i])
    }
    var id = String(current || "")
    if (!id) return out
    for (var j = 0; j < out.length; j++) {
      if (out[j] && out[j].value === id) return out
    }
    out.push({ value: id, label: id })
    return out
  }

  readonly property var agentOptions: [
    { value: "claude", label: "Claude" },
    { value: "codex", label: "Codex" },
    { value: "copilot", label: "Copilot" },
    { value: "crush", label: "Crush" },
    { value: "gemini", label: "Gemini" },
    { value: "grok", label: "Grok" },
    { value: "omp", label: "omp" },
    { value: "opencode", label: "OpenCode" },
    { value: "pi", label: "Pi" }
  ]

  PrefsGroup {
    title: "Applications"
    query: root.query
    detail: "Omarchy writes these as the XDG defaults. Only programs that are installed show up in the lists."

    PrefsRow {
      label: "Browser"
      description: "The browser that opens links and web apps. Only installed browsers show up."
      hint: "omarchy default browser"
      query: root.query
      keywords: ["web", "chrome", "firefox"]

      PrefsSelect {
        value: Omarchy.browser
        options: Omarchy.installedOptions(root.browserOptions, Omarchy.browsers)
        onChanged: function(value) { if (value !== Omarchy.browser) Omarchy.setBrowser(value) }
      }
    }

    PrefsRow {
      label: "Terminal"
      description: "The terminal other apps launch when they need a console. Only installed terminals show up."
      hint: "omarchy default terminal"
      query: root.query
      keywords: ["shell", "console"]

      PrefsSelect {
        value: Omarchy.terminal
        options: Omarchy.installedOptions(root.terminalOptions, Omarchy.terminals)
        onChanged: function(value) { if (value !== Omarchy.terminal) Omarchy.setTerminal(value) }
      }
    }

    PrefsRow {
      label: "Editor"
      description: "The editor Omarchy opens when you ask to edit a file. Only installed editors show up."
      hint: "omarchy default editor"
      query: root.query
      keywords: ["nvim", "code", "zed"]

      PrefsSelect {
        value: Omarchy.editor === "zeditor" ? "zed" : Omarchy.editor
        options: Omarchy.installedOptions(root.editorOptions, {
          nvim: Omarchy.editors.nvim,
          code: Omarchy.editors.code,
          cursor: Omarchy.editors.cursor,
          zed: Omarchy.editors.zeditor,
          sublime_text: Omarchy.editors.sublime_text,
          helix: Omarchy.editors.helix,
          vim: Omarchy.editors.vim,
          emacs: Omarchy.editors.emacs
        })
        onChanged: function(value) {
          var current = Omarchy.editor === "zeditor" ? "zed" : Omarchy.editor
          if (value !== current) Omarchy.setEditor(value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Agent"
    query: root.query
    detail: "The coding agent Omarchy treats as the default. If it is missing, Omarchy offers to install it."

    PrefsRow {
      label: "Coding agent"
      description: "Which coding agent Omarchy opens. If it is missing, it offers to install it."
      hint: "omarchy default agent"
      query: root.query
      keywords: ["ai", "claude", "codex", "grok"]

      PrefsSelect {
        value: Omarchy.agent
        options: root.agentOptions
        onChanged: function(value) { if (value !== Omarchy.agent) Omarchy.setAgent(value) }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "XDG defaults for PDFs, images, and video. Browser, terminal, and editor stay in the group above."

    PrefsRow {
      label: "PDF"
      description: "The program that opens PDF files."
      hint: "xdg-mime default application/pdf"
      query: root.query
      keywords: ["pdf", "mime", "document", "evince", "zathura"]

      PrefsSelect {
        value: Omarchy.mimePdf
        options: root.mimeOptions(Omarchy.mimePdf, Omarchy.mimePdfOptions)
        enabled: root.mimeOptions(Omarchy.mimePdf, Omarchy.mimePdfOptions).length > 0
        onChanged: function(value) {
          if (value !== Omarchy.mimePdf) Omarchy.setMimeDefault("pdf", value)
        }
      }
    }

    PrefsRow {
      label: "Images"
      description: "The program that opens pictures."
      hint: "xdg-mime default image/png"
      query: root.query
      keywords: ["image", "png", "jpeg", "mime", "imv"]

      PrefsSelect {
        value: Omarchy.mimeImage
        options: root.mimeOptions(Omarchy.mimeImage, Omarchy.mimeImageOptions)
        enabled: root.mimeOptions(Omarchy.mimeImage, Omarchy.mimeImageOptions).length > 0
        onChanged: function(value) {
          if (value !== Omarchy.mimeImage) Omarchy.setMimeDefault("image", value)
        }
      }
    }

    PrefsRow {
      label: "Video"
      description: "The program that opens videos."
      hint: "xdg-mime default video/mp4"
      query: root.query
      keywords: ["video", "mp4", "mime", "mpv", "vlc"]

      PrefsSelect {
        value: Omarchy.mimeVideo
        options: root.mimeOptions(Omarchy.mimeVideo, Omarchy.mimeVideoOptions)
        enabled: root.mimeOptions(Omarchy.mimeVideo, Omarchy.mimeVideoOptions).length > 0
        onChanged: function(value) {
          if (value !== Omarchy.mimeVideo) Omarchy.setMimeDefault("video", value)
        }
      }
    }
  }
}
