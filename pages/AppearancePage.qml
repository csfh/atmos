import QtQuick
import "../components"
import "../services"
import "../services/HyprSunset.js" as HyprSunset
import "../services/RichUi.js" as RichUi
import "../services/Theme.js" as ThemeJs
import "appearance" as Look
import "rows"

PrefsPage {
  id: root
  hubId: "appearance"
  title: "Appearance"
  description: "How the desktop looks. The theme sets colors for the shell and themed apps. Wallpaper and the boot screen open from Wallpaper and boot."

  property var stack: null
  property var navigator: null
  property string extraToRemove: ""
  property string themeUrlDraft: ""
  readonly property string themeUrlParsed: RichUi.parseGitUrl(root.themeUrlDraft)
  readonly property bool themeUrlValid: root.themeUrlParsed.length > 0
  property string dayDraft: Omarchy.nightlightDay
  property string nightDraft: Omarchy.nightlightNight
  readonly property string dayParsed: HyprSunset.parseTime(root.dayDraft)
  readonly property string nightParsed: HyprSunset.parseTime(root.nightDraft)
  readonly property bool nightTimesValid: root.dayParsed.length > 0 && root.nightParsed.length > 0

  function openAddTheme() {
    root.themeUrlDraft = ""
    themeUrlField.setText("")
    addThemeDialog.open()
  }

  function submitAddTheme() {
    if (!root.themeUrlValid || Omarchy.jobBusy) return
    Omarchy.installTheme(root.themeUrlParsed)
    addThemeDialog.close()
  }

  function openSubpage(id) {
    if (id === "theme") return
    if (stack) {
      if (id === "background") stack.push(backgroundPage)
      else if (id === "boot") stack.push(bootPage)
      return
    }
    if (root.navigator && root.navigator.go)
      root.navigator.go("appearance/" + id)
  }

  function extraCountText() {
    var n = Omarchy.extraThemes.length
    if (n === 1) return "One extra theme is installed on this machine."
    return n + " extra themes are installed on this machine."
  }

  function applyNightSchedule(nightOn) {
    if (!root.nightTimesValid) return
    Omarchy.setNightlightSchedule(root.dayParsed, root.nightParsed, nightOn === true)
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
        : (root.themeUrlDraft.length > 0 && !root.themeUrlValid
          ? "Need an https, ssh, or git@host:path URL whose last segment can name a theme. Install stays off until it parses."
          : "Paste a git URL for an Omarchy theme. Install clones it into ~/.config/omarchy/themes and switches to it.")
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: themeUrlField
      width: parent.width
      value: root.themeUrlDraft
      placeholder: "https://github.com/org/omarchy-theme.git"
      enabled: !Omarchy.jobBusy
      invalid: root.themeUrlDraft.length > 0 && !root.themeUrlValid
      onEdited: function(value) { root.themeUrlDraft = value }
      onSubmitted: function(value) {
        root.themeUrlDraft = value
        root.submitAddTheme()
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
        enabled: !Omarchy.jobBusy && root.themeUrlValid
        onClicked: root.submitAddTheme()
      }
    }
  }

  PrefsGroup {
    title: "Theme"
    query: root.query
    detail: "A theme is a named palette plus the templates Omarchy writes into the shell, terminals, and a few related apps. Switching themes rewrites those configs from the theme's files. Stock themes live in the Omarchy package. If you edit them in place, the next update puts the packaged copies back."
    hint: "omarchy theme set"

    SettingRow {
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

    SettingRow {
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

    SettingRow {
      label: "Themes"
      description: "The palette in use right now, and every theme you can switch to. The shell and themed apps follow the current one. Hover a card to preview its colors on this window; click to paint the desktop."
      hint: "omarchy theme set"
      query: root.query
      keywords: ["appearance", "current", "theme", "gallery", "swatch", "preview", "color", "style", "palette"]
      available: Omarchy.themes.length > 0
      stretchControl: true

      Flow {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: Omarchy.themes

          delegate: Rectangle {
            required property var modelData
            property var dots: []
            width: 148
            height: cardCol.implicitHeight + Theme.pad * 2
            color: modelData === Omarchy.theme
              ? Theme.accentFill(Theme.primaryFill)
              : Theme.fill(Theme.tileFill)
            border.width: Theme.borderWidth
            border.color: modelData === Omarchy.theme ? Theme.accent : Theme.borderColor()
            radius: Theme.radius

            Accessible.role: Accessible.Button
            Accessible.name: modelData === Omarchy.theme ? modelData + ", current theme" : "Apply " + modelData
            Accessible.onPressAction: apply()

            Component.onCompleted: loadDots()

            function apply() {
              if (modelData !== Omarchy.theme) Omarchy.setTheme(modelData)
              else Theme.restorePreview()
            }

            // Read-only parse of the theme's colors.toml. Never runs
            // omarchy theme set; hover and click own the commit paths.
            function loadDots() {
              var paths = ThemeJs.themeFileCandidates(modelData, "colors.toml", Theme.home)
              var i
              var raw = ""
              for (i = 0; i < paths.length; i++) {
                raw = Theme.readPath(paths[i])
                if (raw) break
              }
              if (!raw) return
              var parsed = ThemeJs.parseColors(raw)
              dots = [parsed.accent, parsed.foreground, parsed.muted, parsed.urgent]
            }

            Behavior on border.color {
              ColorAnimation { duration: Theme.motionFast }
            }

            Column {
              id: cardCol
              anchors.left: parent.left
              anchors.right: parent.right
              anchors.top: parent.top
              anchors.margins: Theme.pad
              spacing: Theme.labelGap

              Row {
                spacing: Theme.sliderTickGap

                Repeater {
                  model: dots

                  Rectangle {
                    required property var modelData
                    width: Theme.swatchSize - 6
                    height: Theme.swatchSize - 6
                    color: modelData
                    border.width: Theme.borderWidth
                    border.color: Theme.borderColor()
                    radius: Theme.radius
                  }
                }
              }

              Text {
                width: parent.width
                text: modelData
                color: Theme.foreground
                font.family: Theme.fontFamily
                font.pixelSize: Theme.labelSize
                font.bold: modelData === Omarchy.theme
                elide: Text.ElideRight
              }
            }

            MouseArea {
              id: chipMouse
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor
              onEntered: {
                if (modelData !== Omarchy.theme) Theme.previewNamedTheme(modelData)
              }
              onExited: Theme.restorePreview()
              onClicked: apply()
            }
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Additional themes"
    query: root.query
    wide: true
    detail: "Extra themes are git clones in ~/.config/omarchy/themes. Add clones a repository and switches to it. Update all pulls the latest commit on each one. Remove deletes a theme you installed."
    hint: "omarchy theme install"

    SettingRow {
      available: Omarchy.extraThemes.length === 0
      label: "Installed themes"
      description: "No extra themes installed."
      hint: "omarchy theme extras"
      query: root.query
      keywords: ["git", "extra", "clone", "empty"]
    }

    SettingRow {
      available: Omarchy.extraThemes.length > 0
      label: "Installed themes"
      description: root.extraCountText() + " Pick one to apply or remove, or pull the latest commit on all of them. Hover a name to preview its colors on this window."
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
          onChanged: function(value) {
            root.extraToRemove = value
            if (value !== Omarchy.theme) Omarchy.setTheme(value)
            else Theme.restorePreview()
          }

          // Same hover path as Current theme: colors.toml only, then
          // restorePreview puts the live chrome back. Click still records
          // extraToRemove for Remove… and commits through setTheme.
          onPreviewed: function(value) {
            if (value.length > 0 && value !== Omarchy.theme)
              Theme.previewNamedTheme(value)
            else
              Theme.restorePreview()
          }
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
            text: "Remove…"
            danger: true
            enabled: root.extraToRemove.length > 0
            onClicked: removeThemeConfirm.ask()
          }
        }
      }
    }

    SettingRow {
      label: "Add a theme"
      description: Omarchy.jobKind === "theme-install" && Omarchy.jobBusy
        ? "Cloning the repository and switching to it…"
        : "Install a theme from a public git repository."
      hint: "omarchy theme install"
      query: root.query
      keywords: ["git", "extra", "clone", "download", "install"]

      PrefsButton {
        text: "Add…"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.openAddTheme()
      }
    }
  }

  PrefsGroup {
    title: "Wallpaper and boot"
    query: root.query
    detail: "Background is the desktop picture for this theme. Boot screen is the Plymouth unlock animation and logo you see before you log in."

    SettingRow {
      label: "Background"
      description: Omarchy.background.length
        ? ("Current file: " + RichUi.fileBasename(Omarchy.background) + ".")
        : "No wallpaper is set for this theme yet."
      hint: "omarchy theme bg"
      query: root.query
      keywords: ["wallpaper", "image", "file", "aether", "palette", "cache"]

      PrefsButton {
        text: "Choose…"
        onClicked: root.openSubpage("background")
      }
    }

    SettingRow {
      label: "Boot screen"
      description: Omarchy.plymouth.length
        ? ("Unlock theme: " + Omarchy.plymouth + ". Logo and preview are on the next page.")
        : "The unlock animation and logo you see before the desktop."
      hint: "omarchy plymouth"
      query: root.query
      keywords: ["plymouth", "sddm", "login", "unlock", "logo", "png"]

      PrefsButton {
        text: "Configure…"
        onClicked: root.openSubpage("boot")
      }
    }
  }

  PrefsGroup {
    title: "Text"
    query: root.query
    detail: "Font and size apply together to the shell, GTK apps, and terminals. Reset puts size back to 12 pixels."

    SettingRow {
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

    SettingRow {
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

    SettingRow {
      label: "Night light"
      description: Omarchy.nightlightTemperature > 0
        ? ("Shift colors toward amber at night. Right now that is " + Omarchy.nightlightTemperature + " K.")
        : "Shift colors toward amber at night."
      hint: "omarchy toggle nightlight"
      query: root.query
      keywords: ["nightlight", "warmth", "temperature", "blue light", "kelvin"]

      PrefsToggle {
        checked: Omarchy.nightlight
        onToggled: Omarchy.set("nightlight", !Omarchy.nightlight)
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    advanced: true
    query: root.query
    detail: "Warmth is Kelvin. 6500 is daylight. Lower numbers go amber. This talks to hyprsunset the same way the toggle does."

    SettingRow {
      stretchControl: true
      label: "Night light warmth"
      description: Omarchy.nightlightTemperature > 0
        ? ("The screen is at " + Omarchy.nightlightTemperature + " K. 4000 is the usual amber. 6500 is daylight.")
        : "Pick a color temperature. 4000 is amber. 6500 is daylight."
      hint: "hyprctl hyprsunset temperature · ~/.config/hypr/hyprsunset.conf"
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

    SettingRow {
      label: "Night light schedule"
      description: root.nightTimesValid
        ? "Left is when daylight starts. Right is when night starts."
        : "Times need to look like 07:00 and 20:00 (hours 0–23)."
      hint: "~/.config/hypr/hyprsunset.conf"
      query: root.query
      keywords: ["nightlight", "schedule", "hyprsunset", "sunset", "sunrise", "time"]

      Row {
        spacing: Theme.space
        PrefsField {
          width: 72
          value: root.dayDraft
          placeholder: "07:00"
          invalid: root.dayDraft.length > 0 && root.dayParsed.length === 0
          onEdited: function(value) { root.dayDraft = value }
          onSubmitted: function(value) {
            root.dayDraft = value
            root.applyNightSchedule(Omarchy.nightlightNightOn)
          }
        }
        PrefsField {
          width: 72
          value: root.nightDraft
          placeholder: "20:00"
          invalid: root.nightDraft.length > 0 && root.nightParsed.length === 0
          onEdited: function(value) { root.nightDraft = value }
          onSubmitted: function(value) {
            root.nightDraft = value
            root.applyNightSchedule(Omarchy.nightlightNightOn)
          }
        }
        PrefsButton {
          text: "Set"
          enabled: root.nightTimesValid
          onClicked: root.applyNightSchedule(Omarchy.nightlightNightOn)
        }
      }
    }

    SettingRow {
      label: "Use schedule"
      description: root.nightTimesValid
        ? "Turn the timed amber profile on."
        : "Needs valid times above."
      hint: "~/.config/hypr/hyprsunset.conf"
      query: root.query
      keywords: ["nightlight", "schedule", "automatic", "hyprsunset", "enable"]

      PrefsToggle {
        checked: Omarchy.nightlightNightOn
        enabled: root.nightTimesValid
        onToggled: root.applyNightSchedule(!Omarchy.nightlightNightOn)
      }
    }
  }
}
