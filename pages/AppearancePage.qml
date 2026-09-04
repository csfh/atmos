import QtQuick
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi
import "appearance" as Look
import "rows"

PrefsPage {
  id: root
  title: "Appearance"
  description: "How the desktop looks. The theme sets colors for the shell and themed apps. Wallpaper and type size live further down this page."

  property var stack: null
  property var navigator: null
  property string extraToRemove: ""
  property string dayDraft: Omarchy.nightlightDay
  property string nightDraft: Omarchy.nightlightNight

  function openSubpage(id) {
    if (id === "theme") return
    if (root.navigator && root.navigator.go) {
      root.navigator.go("appearance/" + id)
      return
    }
    if (!stack) return
    if (id === "background") stack.push(backgroundPage)
    else if (id === "boot") stack.push(bootPage)
  }

  function extraCountText() {
    var n = Omarchy.extraThemes.length
    if (n === 1) return "One extra theme is installed on this machine."
    return n + " extra themes are installed on this machine."
  }

  function syncExtraToRemove() {
    var list = Omarchy.extraThemes || []
    for (var i = 0; i < list.length; i++) {
      if (list[i] === root.extraToRemove) return
    }
    root.extraToRemove = list.length > 0 ? list[0] : ""
  }

  Component.onCompleted: {
    removeThemeConfirm.parent = root.prefsOverlay
    addThemeDialog.parent = root.prefsOverlay
    root.syncExtraToRemove()
  }

  Connections {
    target: Omarchy
    function onExtraThemesChanged() { root.syncExtraToRemove() }
    function onNightlightDayChanged() { root.dayDraft = Omarchy.nightlightDay }
    function onNightlightNightChanged() { root.nightDraft = Omarchy.nightlightNight }
  }

  Component { id: backgroundPage; Look.BackgroundPage {} }
  Component { id: bootPage; Look.BootPage {} }

  PrefsConfirm {
    id: removeThemeConfirm
    title: "Remove this theme"
    message: "Delete " + root.extraToRemove + " from your extra themes? The files under ~/.config/omarchy/themes go with it."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeTheme(root.extraToRemove)
  }

  PrefsDialog {
    id: addThemeDialog
    title: "Add a theme"

    PrefsText {
      width: parent.width
      text: Omarchy.jobKind === "theme-install" && Omarchy.jobBusy
        ? "Cloning the repository and switching to it…"
        : "Paste a git URL for an Omarchy theme. Install clones it into ~/.config/omarchy/themes and switches to it."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: themeUrlField
      width: parent.width
      placeholder: "https://github.com/org/omarchy-theme.git"
      enabled: !Omarchy.jobBusy
      onSubmitted: function(value) {
        Omarchy.installTheme(value)
        addThemeDialog.close()
      }
    }

    Row {
      anchors.right: parent.right
      spacing: Theme.space

      PrefsButton {
        text: "Cancel"
        onClicked: addThemeDialog.close()
      }

      PrefsButton {
        text: "Install"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: {
          Omarchy.installTheme(themeUrlField.currentText())
          addThemeDialog.close()
        }
      }
    }
  }

  PrefsGroup {
    title: "Theme"
    query: root.query
    detail: "A theme is a named palette plus the templates Omarchy writes into the shell, terminals, and a few related apps. Switching themes rewrites those configs from the theme's files. Stock themes live in the Omarchy package. If you edit them in place, the next update puts the packaged copies back."
    hint: "omarchy theme set"

    PrefsRow {
      label: "Current theme"
      description: "The palette in use right now. The shell and themed apps follow this."
      hint: "omarchy theme set"
      query: root.query
      keywords: ["appearance", "color", "style", "palette"]

      PrefsSelect {
        value: Omarchy.theme
        options: Omarchy.themes
        enabled: Omarchy.themes.length > 0
        onChanged: function(value) { if (value !== Omarchy.theme) Omarchy.setTheme(value) }
      }
    }

    PrefsRow {
      label: "Theme files"
      description: "The files behind the current theme. Open the folder if you want to tweak colors or templates by hand."
      hint: "omarchy theme dir"
      query: root.query
      keywords: ["folder", "directory", "files", "path"]

      PrefsButton {
        text: "Open folder"
        enabled: Omarchy.theme.length > 0
        onClicked: Omarchy.openThemeFolder()
      }
    }

    PrefsRow {
      label: "Refresh"
      description: "Rewrite the current theme from its templates. Handy after you edit theme files."
      hint: "omarchy theme refresh"
      query: root.query
      keywords: ["reload", "reapply", "templates"]

      PrefsButton {
        text: "Refresh"
        onClicked: Omarchy.refreshTheme()
      }
    }
  }

  PrefsGroup {
    title: "Additional themes"
    query: root.query
    detail: "Extra themes are git clones in ~/.config/omarchy/themes. Add clones a repository and switches to it. Update all pulls the latest commit on each one. Remove deletes a theme you installed."
    hint: "omarchy theme install"

    PrefsRow {
      available: Omarchy.extraThemes.length === 0
      label: "Installed themes"
      description: "None installed yet. Add a git URL below. Update all and Remove stay disabled until a clone exists."
      hint: "omarchy theme extras"
      query: root.query
      keywords: ["git", "extra", "clone"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Add"
          primary: true
          enabled: !Omarchy.jobBusy
          onClicked: addThemeDialog.open()
        }
        PrefsButton {
          text: "Update all"
          enabled: false
        }
        PrefsButton {
          text: "Remove"
          danger: true
          enabled: false
        }
      }
    }

    PrefsRow {
      available: Omarchy.extraThemes.length > 0
      label: "Installed themes"
      description: root.extraCountText() + " Pick one to remove, or pull the latest commit on all of them."
      hint: "omarchy theme extras · omarchy theme update · omarchy theme remove"
      query: root.query
      keywords: ["git", "extra", "clone", "uninstall", "delete", "pull"]

      Column {
        spacing: Theme.space
        width: Theme.controlColumnWidth

        PrefsSelect {
          width: parent.width
          value: root.extraToRemove
          options: Omarchy.extraThemes
          enabled: Omarchy.extraThemes.length > 0
          onChanged: function(value) { root.extraToRemove = value }
        }

        Row {
          spacing: Theme.space
          anchors.right: parent.right

          PrefsButton {
            text: Omarchy.jobKind === "theme-update" && Omarchy.jobBusy ? "Updating…" : "Update all"
            enabled: !Omarchy.jobBusy && Omarchy.extraThemes.length > 0
            onClicked: Omarchy.updateThemes()
          }

          PrefsButton {
            text: "Remove"
            danger: true
            enabled: root.extraToRemove.length > 0
            onClicked: removeThemeConfirm.ask()
          }
        }
      }
    }

    PrefsRow {
      label: "Add a theme"
      description: Omarchy.jobKind === "theme-install" && Omarchy.jobBusy
        ? "Cloning the repository and switching to it…"
        : "Install a theme from a public git repository."
      hint: "omarchy theme install"
      query: root.query
      keywords: ["git", "extra", "clone", "download", "install"]

      PrefsButton {
        text: "Add"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: addThemeDialog.open()
      }
    }
  }

  PrefsGroup {
    title: "Wallpaper and boot"
    query: root.query
    detail: "Background is the desktop picture for this theme. Boot screen is the Plymouth unlock animation and logo you see before you log in."

    PrefsRow {
      label: "Background"
      description: Omarchy.background.length
        ? ("Current file: " + RichUi.fileBasename(Omarchy.background) + ". Cycle images or pick a file on the next page.")
        : "No wallpaper is set for this theme yet. Open the next page to pick a file or cycle the theme set."
      hint: "omarchy theme bg"
      query: root.query
      keywords: ["wallpaper", "image", "file", "aether", "palette", "cache"]

      PrefsButton {
        text: "Open"
        onClicked: root.openSubpage("background")
      }
    }

    PrefsRow {
      label: "Boot screen"
      description: Omarchy.plymouth.length
        ? ("Unlock theme: " + Omarchy.plymouth + ". Logo and preview are on the next page.")
        : "The unlock animation and logo you see before the desktop."
      hint: "omarchy plymouth"
      query: root.query
      keywords: ["plymouth", "sddm", "login", "unlock", "logo", "png"]

      PrefsButton {
        text: "Open"
        onClicked: root.openSubpage("boot")
      }
    }
  }

  PrefsGroup {
    title: "Text"
    query: root.query
    detail: "Font and size apply together to the shell, GTK apps, and terminals. Reset puts size back to 12 pixels."

    PrefsRow {
      label: "Font"
      description: "The monospace face used by the shell and terminals."
      hint: "omarchy font set"
      query: root.query
      keywords: ["typeface", "monospace"]

      PrefsSelect {
        value: Omarchy.font
        options: Omarchy.fonts
        enabled: Omarchy.fonts.length > 0
        onChanged: function(value) { if (value !== Omarchy.font) Omarchy.setFont(value) }
      }
    }

    TextSizeRow { query: root.query }

    PrefsRow {
      available: Omarchy.textSize !== 12
      label: "Reset text size"
      description: "Put type back to 12 pixels everywhere Omarchy sets it."
      hint: "omarchy display text size reset"
      query: root.query
      keywords: ["default", "scale", "12"]

      PrefsButton {
        text: "Reset"
        enabled: Omarchy.textSize !== 12
        onClicked: Omarchy.resetTextSize()
      }
    }
  }

  PrefsGroup {
    title: "Display"
    query: root.query
    detail: "Night light shifts the screen toward amber. Warmth in Kelvin is under Advanced."

    PrefsRow {
      label: "Night light"
      description: Omarchy.nightlightTemperature > 0
        ? ("Shift the colors toward amber so late work is easier on your eyes. Right now that is " + Omarchy.nightlightTemperature + " K.")
        : "Shift the colors toward amber so late work is easier on your eyes."
      hint: "omarchy toggle nightlight"
      query: root.query
      keywords: ["nightlight", "warmth", "temperature", "blue light", "kelvin"]

      PrefsToggle {
        checked: Omarchy.nightlight
        onToggled: Omarchy.setNightlight(!Omarchy.nightlight)
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Warmth is Kelvin. 6500 is daylight. Lower numbers go amber. This talks to hyprsunset the same way the toggle does."

    PrefsRow {
      stretchControl: true
      label: "Night light warmth"
      description: Omarchy.nightlightTemperature > 0
        ? ("The screen is at " + Omarchy.nightlightTemperature + " K. 4000 is the usual amber. 6500 is daylight.")
        : "Pick a color temperature. 4000 is amber. 6500 is daylight."
      hint: "hyprctl hyprsunset temperature"
      query: root.query
      keywords: ["kelvin", "warmth", "temperature", "amber", "blue light"]

      PrefsSlider {
        width: parent.width
        from: 3000
        to: 6500
        stepSize: 100
        value: Omarchy.nightlightTemperature > 0 ? Omarchy.nightlightTemperature : 4000
        valueText: (Omarchy.nightlightTemperature > 0 ? Omarchy.nightlightTemperature : 4000) + " K"
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.nightlightTemperature)
            Omarchy.setNightlightTemperature(next)
        }
      }
    }

    PrefsRow {
      label: "Night light schedule"
      description: "Left is the hour the day profile starts. Right is when night starts. Apply writes ~/.config/hypr/hyprsunset.conf and restarts hyprsunset."
      hint: "~/.config/hypr/hyprsunset.conf"
      query: root.query
      keywords: ["nightlight", "schedule", "hyprsunset", "sunset", "sunrise"]

      Row {
        spacing: 8
        PrefsField {
          width: 72
          value: root.dayDraft
          placeholder: "07:00"
          onEdited: function(value) { root.dayDraft = value }
          onSubmitted: function(value) { root.dayDraft = value }
        }
        PrefsField {
          width: 72
          value: root.nightDraft
          placeholder: "20:00"
          onEdited: function(value) { root.nightDraft = value }
          onSubmitted: function(value) { root.nightDraft = value }
        }
      }
    }

    PrefsRow {
      label: "Automatic night profile"
      description: "Write a night profile at the time on the right. Off leaves only the daytime identity profile."
      hint: "~/.config/hypr/hyprsunset.conf"
      query: root.query
      keywords: ["nightlight", "schedule", "automatic", "hyprsunset"]

      Row {
        spacing: 8
        PrefsToggle {
          checked: Omarchy.nightlightNightOn
          onToggled: Omarchy.setNightlightSchedule(root.dayDraft, root.nightDraft, !Omarchy.nightlightNightOn)
        }
        PrefsButton {
          text: "Apply times"
          onClicked: Omarchy.setNightlightSchedule(root.dayDraft, root.nightDraft, Omarchy.nightlightNightOn)
        }
      }
    }
  }
}
