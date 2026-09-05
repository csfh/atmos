import QtQuick
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "System"
  description: "This machine's name, language, and clock. Account name and face are on Accounts. Printers, weather, crash capture, and Omarchy updates are further down."

  PrefsConfirm {
    id: channelConfirm
    title: "Switch channel"
    message: root.pendingChannel === "dev"
      ? "Dev links Omarchy to a source checkout. That is for people working on Omarchy itself. The machine then upgrades against that tree."
      : "Switch the package channel to " + root.pendingChannel + ". That rewrites pacman mirrors and upgrades the system."
    confirmText: "Switch"
    onConfirmed: Omarchy.setOmarchyChannel(root.pendingChannel)
  }

  PrefsConfirm {
    id: atmosUpdateConfirm
    title: "Update Atmos"
    message: "Fetch the alpha branch and replace the installed Atmos files under ~/.local/share/atmos. If the window does not reload, quit Atmos and open it again."
    confirmText: "Update"
    onConfirmed: Omarchy.runAtmosUpdate()
  }

  PrefsConfirm {
    id: updateConfirm
    title: "Update Omarchy"
    message: "Download and install Omarchy and system package updates. This can take a while and may ask for a password."
    confirmText: "Update"
    onConfirmed: Omarchy.runOmarchyUpdate()
  }

  PrefsConfirm {
    id: firmwareConfirm
    title: "Firmware update"
    message: "Ask fwupd to install available firmware. You may need to reboot afterward."
    confirmText: "Update"
    onConfirmed: Omarchy.updateFirmware()
  }

  PrefsConfirm {
    id: orphanConfirm
    title: "Remove orphans"
    message: "Remove packages that nothing else depends on."
    confirmText: "Remove"
    onConfirmed: Omarchy.updateOrphanPkgs()
  }

  PrefsConfirm {
    id: pruneConfirm
    title: "Prune package cache"
    message: "Delete old downloaded packages from the pacman cache."
    confirmText: "Prune"
    onConfirmed: Omarchy.prunePkgCache()
  }

  PrefsConfirm {
    id: refreshHyprConfirm
    title: "Restore Hyprland defaults"
    message: "Replace the Hyprland Lua files in ~/.config/hypr with the shipped Omarchy copies. Your current files are backed up first. The Atmos drop-in require is written back so this window still floats."
    confirmText: "Restore"
    onConfirmed: Omarchy.refreshHyprland()
  }

  PrefsConfirm {
    id: refreshShellConfirm
    title: "Restore shell defaults"
    message: "Replace ~/.config/omarchy/shell.json with the shipped Omarchy copy. Your current file is backed up first. The bar restarts afterward."
    confirmText: "Restore"
    onConfirmed: Omarchy.refreshShell()
  }

  PrefsConfirm {
    id: resetAtmosConfirm
    title: "Reset Atmos"
    message: "Remove Atmos-managed Hyprland overrides (look, input, autostart, bindings, extra window rules) and the search index cache. The Atmos window still floats. Theme, wallpaper, and shell.json stay as they are."
    confirmText: "Reset"
    onConfirmed: Omarchy.resetAtmos()
  }

  property string pendingChannel: ""
  property string hostnameDraft: Omarchy.hostname
  readonly property string hostnameParsed: RichUi.parseHostname(root.hostnameDraft)
  readonly property bool hostnameValid: root.hostnameParsed.length > 0
  property string weatherLocationDraft: Omarchy.weatherAuto ? "" : Omarchy.weatherLocation
  readonly property string weatherLocationParsed: RichUi.parseWeatherLocation(root.weatherLocationDraft)
  readonly property bool weatherLocationValid: root.weatherLocationParsed.length > 0
  property string weatherCoordsDraft: Omarchy.weatherCoords
  readonly property string weatherCoordsParsed: RichUi.parseWeatherCoords(root.weatherCoordsDraft)
  readonly property bool weatherCoordsValid: root.weatherCoordsParsed.length > 0

  Component.onCompleted: {
    channelConfirm.parent = root.prefsOverlay
    atmosUpdateConfirm.parent = root.prefsOverlay
    updateConfirm.parent = root.prefsOverlay
    firmwareConfirm.parent = root.prefsOverlay
    orphanConfirm.parent = root.prefsOverlay
    pruneConfirm.parent = root.prefsOverlay
    refreshHyprConfirm.parent = root.prefsOverlay
    refreshShellConfirm.parent = root.prefsOverlay
    resetAtmosConfirm.parent = root.prefsOverlay
  }

  Connections {
    target: Omarchy
    function onHostnameChanged() { root.hostnameDraft = Omarchy.hostname }
    function onWeatherLocationChanged() {
      if (!Omarchy.weatherAuto) root.weatherLocationDraft = Omarchy.weatherLocation
    }
    function onWeatherAutoChanged() {
      root.weatherLocationDraft = Omarchy.weatherAuto ? "" : Omarchy.weatherLocation
    }
    function onWeatherCoordsChanged() { root.weatherCoordsDraft = Omarchy.weatherCoords }
  }

  PrefsGroup {
    title: "Machine"
    query: root.query
    detail: "Hostname is how this computer shows up on the network and in prompts."

    SettingRow {
      stretchControl: true
      label: "Hostname"
      description: root.hostnameValid || root.hostnameDraft.length === 0
        ? "How this computer shows up on the network and in your shell prompt."
        : "Letters, digits, and hyphens only (labels 1–63 characters). Set stays off until the name is valid."
      hint: "hostnamectl set-hostname"
      query: root.query
      keywords: ["hostname", "computer", "machine", "device", "name"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: hostnameField
          width: parent.width - hostnameSetBtn.width - parent.spacing
          value: root.hostnameDraft
          placeholder: "hostname"
          invalid: root.hostnameDraft.length > 0 && !root.hostnameValid
          onEdited: function(value) { root.hostnameDraft = value }
          onSubmitted: function(value) {
            root.hostnameDraft = value
            if (root.hostnameValid) Omarchy.setHostname(root.hostnameParsed)
          }
        }

        PrefsButton {
          id: hostnameSetBtn
          text: "Set"
          primary: true
          enabled: root.hostnameValid && root.hostnameParsed !== Omarchy.hostname
          onClicked: Omarchy.setHostname(root.hostnameParsed)
        }
      }
    }
  }

  PrefsGroup {
    title: "Keyboard"
    query: root.query
    detail: "XKB layout for typing. Hyprland reads it from vconsole.conf after you change it."

    SettingRow {
      label: "Layout"
      description: "Key positions for typing. Hyprland picks this up from vconsole.conf."
      hint: "localectl set-x11-keymap"
      query: root.query
      keywords: ["keyboard", "layout", "keymap", "xkb", "qwerty", "language", "input"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.keyboardLayout
        options: Omarchy.keyboardLayouts
        enabled: Omarchy.keyboardLayouts.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.keyboardLayout) Omarchy.setKeyboardLayout(value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Language"
    query: root.query
    detail: "The locale apps and the system use for language, dates, and number formats. New sessions pick this up."

    SettingRow {
      label: "Locale"
      description: "Language for the system and apps. Open a new session after you change it."
      hint: "localectl set-locale"
      query: root.query
      keywords: ["locale", "lang", "language", "utf-8", "i18n", "translation"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.locale
        options: Omarchy.locales
        enabled: Omarchy.locales.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.locale) Omarchy.setLocale(value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Date and time"
    query: root.query
    detail: "Timezone is what the clock, logs, and timestamps use. Network time keeps that clock honest over NTP."

    SettingRow {
      label: "Timezone"
      description: "The zone the clock, logs, and timestamps use."
      hint: "timedatectl set-timezone"
      query: root.query
      keywords: ["timezone", "tz", "utc", "region", "city", "date", "time", "zoneinfo"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.timezone
        options: Omarchy.timezones
        enabled: Omarchy.timezones.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.timezone) Omarchy.setTimezone(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.ntpAvailable
      label: "Network time"
      description: Omarchy.ntp && Omarchy.ntpSynchronized
        ? "The clock is in sync over the network."
        : "Keep the clock in sync over the network (NTP)."
      hint: "timedatectl set-ntp"
      query: root.query
      keywords: ["ntp", "timesync", "synchronize", "automatic", "clock"]

      PrefsToggle {
        checked: Omarchy.ntp
        enabled: Omarchy.ntpAvailable
        onToggled: Omarchy.setNtp(!Omarchy.ntp)
      }
    }
  }

  PrefsGroup {
    title: "Updates"
    query: root.query
    detail: "Channel picks which Omarchy package stream you follow. Update runs the usual omarchy update job."

    SettingRow {
      label: Omarchy.omarchyVersion.length ? ("Omarchy " + Omarchy.omarchyVersion) : "Omarchy"
      description: Omarchy.omarchyVersion.length
        ? (Omarchy.omarchyChannel
          ? ("Channel " + Omarchy.omarchyChannel + ". Copy the version string if you need it in a report.")
          : "Copy the version string if you need it in a report.")
        : "Version was not readable. Copy stays disabled until omarchy version works."
      hint: "omarchy version"
      query: root.query
      keywords: ["version", "release", "omarchy"]

      PrefsButton {
        text: "Copy"
        enabled: Omarchy.omarchyVersion.length > 0
        onClicked: Omarchy.copyText("Omarchy " + Omarchy.omarchyVersion + (Omarchy.omarchyChannel ? (" (" + Omarchy.omarchyChannel + ")") : ""))
      }
    }

    SettingRow {
      label: "Channel"
      description: "Stable is the usual stream. rc and edge move faster. Dev is a source checkout."
      hint: "omarchy channel set"
      query: root.query
      keywords: ["channel", "stable", "rc", "edge", "dev", "mirror"]

      PrefsSelect {
        value: Omarchy.omarchyChannel
        options: [
          { value: "stable", label: "Stable" },
          { value: "rc", label: "RC" },
          { value: "edge", label: "Edge" },
          { value: "dev", label: "Dev" }
        ]
        enabled: !Omarchy.jobBusy && Omarchy.omarchyChannel.length > 0
        onChanged: function(value) {
          if (value === Omarchy.omarchyChannel) return
          root.pendingChannel = value
          channelConfirm.ask()
        }
      }
    }

    SettingRow {
      label: "Updates"
      description: Omarchy.jobKind === "omarchy-update" && Omarchy.jobBusy
        ? "Updating…"
        : (Omarchy.updateAvailable
          ? (Omarchy.updateSummary || "Updates are available.")
          : (Omarchy.updateSummary || "Omarchy is up to date."))
      hint: "omarchy update"
      query: root.query
      keywords: ["update", "upgrade", "pacman", "check"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Check"
          enabled: !Omarchy.jobBusy
          onClicked: Omarchy.checkOmarchyUpdate()
        }
        PrefsButton {
          text: "Update…"
          primary: true
          enabled: !Omarchy.jobBusy
          onClicked: updateConfirm.ask()
        }
      }
    }
  }

  PrefsGroup {
    title: "Atmos"
    query: root.query
    detail: "Installed files live under ~/.local/share/atmos. Channel is the git branch Check and Update follow. Only alpha exists yet."

    SettingRow {
      label: Omarchy.atmosRevision.length ? Omarchy.atmosRevision : "Atmos"
      description: Omarchy.atmosRevision.length
        ? "Installed revision under ~/.local/share/atmos. Copy if you need it in a report."
        : (Omarchy.atmosInstalled
          ? "REVISION was not readable. Copy stays disabled until that file exists."
          : "Not installed in XDG data. Copy stays disabled. Run install.sh from the Atmos source tree.")
      hint: "~/.local/share/atmos/REVISION"
      query: root.query
      keywords: ["atmos", "version", "revision", "git"]

      PrefsButton {
        text: "Copy"
        enabled: Omarchy.atmosRevision.length > 0
        onClicked: Omarchy.copyText(Omarchy.atmosRevision)
      }
    }

    SettingRow {
      label: "Channel"
      description: "Alpha tracks the alpha branch. Other channels are not available yet."
      hint: "~/.config/atmos/channel"
      query: root.query
      keywords: ["atmos", "channel", "alpha", "branch"]

      PrefsSelect {
        value: Omarchy.atmosChannel
        options: [
          { value: "alpha", label: "Alpha" }
        ]
        enabled: !Omarchy.jobBusy
        onChanged: function(value) { if (value !== Omarchy.atmosChannel) Omarchy.setAtmosChannel(value) }
      }
    }

    SettingRow {
      label: "Updates"
      description: Omarchy.jobKind === "atmos-update" && Omarchy.jobBusy
        ? "Updating…"
        : (Omarchy.jobKind === "atmos-update-check" && Omarchy.jobBusy
          ? "Checking…"
          : (Omarchy.atmosUpdateSummary
            || (Omarchy.atmosUpdateAvailable ? "A newer Atmos is on alpha." : "Check the alpha branch for a newer copy.")))
      hint: "scripts/update-atmos.sh"
      query: root.query
      keywords: ["atmos", "update", "upgrade", "git", "pull", "alpha"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Check"
          enabled: !Omarchy.jobBusy
          onClicked: Omarchy.checkAtmosUpdate()
        }
        PrefsButton {
          text: "Update…"
          primary: true
          enabled: !Omarchy.jobBusy && Omarchy.atmosUpdateAvailable
          onClicked: atmosUpdateConfirm.ask()
        }
      }
    }

    SettingRow {
      label: "Reset"
      description: Omarchy.jobKind === "reset-atmos" && Omarchy.jobBusy
        ? "Resetting Atmos…"
        : "Strip look, input, autostart, bindings, extra window rules, and the search cache. This window still floats."
      hint: "scripts/reset-atmos.sh"
      query: root.query
      keywords: ["reset", "clear", "sentinel", "overrides", "atmos", "search", "index", "sqlite", "cache"]

      PrefsButton {
        text: "Reset…"
        danger: true
        enabled: !Omarchy.jobBusy
        onClicked: resetAtmosConfirm.ask()
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Advanced"
    query: root.query
    detail: "Firmware through fwupd, leftover packages, the pacman download cache, and restore for Hyprland Lua or shell.json."

    SettingRow {
      label: "Firmware"
      description: Omarchy.jobKind === "update-firmware" && Omarchy.jobBusy
        ? "Updating firmware…"
        : "Install firmware updates through fwupd when the vendor ships them."
      hint: "omarchy update firmware"
      query: root.query
      keywords: ["firmware", "fwupd", "bios"]

      PrefsButton {
        text: "Update…"
        enabled: !Omarchy.jobBusy
        onClicked: firmwareConfirm.ask()
      }
    }

    SettingRow {
      label: "Orphan packages"
      description: "Remove packages that nothing else depends on."
      hint: "omarchy update orphan pkgs"
      query: root.query
      keywords: ["orphan", "unused", "pacman"]

      PrefsButton {
        text: "Remove…"
        danger: true
        enabled: !Omarchy.jobBusy
        onClicked: orphanConfirm.ask()
      }
    }

    SettingRow {
      label: "Package cache"
      description: "Delete old downloaded packages to free disk."
      hint: "omarchy update pkg prune"
      query: root.query
      keywords: ["prune", "cache", "pacman"]

      PrefsButton {
        text: "Prune…"
        enabled: !Omarchy.jobBusy
        onClicked: pruneConfirm.ask()
      }
    }

    SettingRow {
      label: "Restart shell"
      description: "Reload the bar and notifications without touching shell.json."
      hint: "omarchy restart shell"
      query: root.query
      keywords: ["restart", "reload", "bar", "quickshell"]

      PrefsButton {
        text: "Restart"
        enabled: !Omarchy.jobBusy
        onClicked: Omarchy.restartShell()
      }
    }

    SettingRow {
      label: "Restore Hyprland"
      description: Omarchy.jobKind === "refresh-hyprland" && Omarchy.jobBusy
        ? "Restoring Hyprland Lua…"
        : "Put the shipped Hyprland Lua files back. Your copies are backed up."
      hint: "omarchy refresh hyprland"
      query: root.query
      keywords: ["refresh", "hyprland", "restore", "defaults", "bindings", "monitors"]

      PrefsButton {
        text: "Restore…"
        danger: true
        enabled: !Omarchy.jobBusy
        onClicked: refreshHyprConfirm.ask()
      }
    }

    SettingRow {
      label: "Restore shell"
      description: "Put the shipped shell.json back. Your copy is backed up. The bar restarts."
      hint: "omarchy refresh shell"
      query: root.query
      keywords: ["refresh", "shell", "restore", "defaults", "bar"]

      PrefsButton {
        text: "Restore…"
        danger: true
        enabled: !Omarchy.jobBusy
        onClicked: refreshShellConfirm.ask()
      }
    }
  }

  PrefsGroup {
    title: "Printers"
    query: root.query
    detail: "CUPS is the print service on this machine. Set up opens the usual printer window. The web UI is the CUPS admin page on this computer."

    SettingRow {
      label: "Printers"
      description: Omarchy.cupsActive
        ? "CUPS is running. Set up opens the printer window. The web UI is http://127.0.0.1:631."
        : "CUPS is not running. You can still open the admin page if you start the service."
      hint: "system-config-printer"
      query: root.query
      keywords: ["printer", "cups", "print", "ipp"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Set up…"
          primary: true
          onClicked: Omarchy.openPrinters()
        }
        PrefsButton {
          text: "Open CUPS"
          onClicked: Omarchy.openCupsAdmin()
        }
      }
    }
  }

  PrefsGroup {
    title: "Packages"
    query: root.query
    detail: "How many packages pacman fetches at once. Higher can finish a big upgrade sooner on a fast link."

    SettingRow {
      label: "Parallel downloads"
      description: "How many packages pacman fetches at once. Bump this if updates feel slow on a good connection."
      hint: "/etc/pacman.conf · ParallelDownloads"
      query: root.query
      keywords: ["pacman", "downloads", "parallel", "mirrors", "aur", "speed"]

      PrefsSpinBox {
        from: 1
        to: 20
        value: Omarchy.parallelDownloads
        onChanged: function(value) {
          if (value !== Omarchy.parallelDownloads) Omarchy.setParallelDownloads(value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Branding"
    query: root.query
    detail: "ASCII art on the About screen. Choose a picture to turn into that art. Edit opens the text file if you want to write it yourself. Reset puts the Omarchy icon back."

    SettingRow {
      label: "About logo"
      description: Omarchy.aboutBranded
        ? "You are using custom ASCII art on the About screen."
        : "The stock Omarchy icon. Choose a picture to turn into ASCII. Edit opens the text file."
      hint: "omarchy branding about"
      query: root.query
      keywords: ["ascii", "logo", "fastfetch", "about"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Choose…"
          onClicked: Omarchy.setAboutBranding("image")
        }
        PrefsButton {
          text: "Edit"
          onClicked: Omarchy.setAboutBranding("text")
        }
        PrefsButton {
          visible: Omarchy.aboutBranded
          text: "Reset"
          danger: true
          enabled: Omarchy.aboutBranded
          onClicked: Omarchy.setAboutBranding("reset")
        }
      }
    }
  }

  PrefsGroup {
    title: "Weather"
    query: root.query
    detail: "The city the weather widget and notifications use. Auto guesses from your IP. Coordinates pin the forecast when a city name is ambiguous."

    SettingRow {
      stretchControl: true
      label: "Location"
      description: root.weatherLocationValid || root.weatherLocationDraft.length === 0
        ? (Omarchy.weatherAuto
          ? "Guessing the city from your IP for the weather widget and notifications."
          : "The city the weather widget and notifications use.")
        : "A city name, not a flag or a blank line. Set stays off until the name is valid."
      hint: "omarchy weather location"
      query: root.query
      keywords: ["weather", "city", "forecast", "wttr"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: weatherField
          width: parent.width - weatherSetBtn.width - weatherAutoBtn.width - parent.spacing * 2
          value: root.weatherLocationDraft
          placeholder: "City name"
          invalid: root.weatherLocationDraft.length > 0 && !root.weatherLocationValid
          onEdited: function(value) { root.weatherLocationDraft = value }
          onSubmitted: function(value) {
            root.weatherLocationDraft = value
            if (root.weatherLocationValid) Omarchy.setWeatherLocation(root.weatherLocationParsed)
          }
        }

        PrefsButton {
          id: weatherSetBtn
          text: "Set"
          primary: true
          enabled: root.weatherLocationValid && (Omarchy.weatherAuto || root.weatherLocationParsed !== Omarchy.weatherLocation)
          onClicked: Omarchy.setWeatherLocation(root.weatherLocationParsed)
        }

        PrefsButton {
          id: weatherAutoBtn
          text: "Auto"
          enabled: !Omarchy.weatherAuto
          onClicked: Omarchy.clearWeatherLocation()
        }
      }
    }

    SettingRow {
      available: !Omarchy.weatherAuto && Omarchy.weatherLocation.length > 0
      stretchControl: true
      label: "Coordinates"
      description: root.weatherCoordsValid || root.weatherCoordsDraft.length === 0
        ? "Optional latitude and longitude if the city name is ambiguous. Use lat,lon."
        : "Latitude -90 to 90 and longitude -180 to 180, as lat,lon. Spaces around the comma are fine. Set stays off until the pair is valid."
      hint: "omarchy weather location --set name lat,lon"
      query: root.query
      keywords: ["latitude", "longitude", "gps", "coords"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: weatherCoordsField
          width: parent.width - weatherCoordsSetBtn.width - parent.spacing
          value: root.weatherCoordsDraft
          placeholder: "lat,lon"
          invalid: root.weatherCoordsDraft.length > 0 && !root.weatherCoordsValid
          onEdited: function(value) { root.weatherCoordsDraft = value }
          onSubmitted: function(value) {
            root.weatherCoordsDraft = value
            if (root.weatherCoordsValid) Omarchy.setWeatherCoordinates(root.weatherCoordsParsed)
          }
        }

        PrefsButton {
          id: weatherCoordsSetBtn
          text: "Set"
          primary: true
          enabled: root.weatherCoordsValid && root.weatherCoordsParsed !== Omarchy.weatherCoords
          onClicked: Omarchy.setWeatherCoordinates(root.weatherCoordsParsed)
        }
      }
    }

    SettingRow {
      available: Omarchy.weatherPresent
      label: "Units"
      description: "Temperature and wind in the weather widget. Auto follows the location you set above."
      hint: "omarchy bar set omarchy.weather unit"
      query: root.query
      keywords: ["celsius", "fahrenheit", "metric", "imperial", "temperature"]

      PrefsSelect {
        value: Omarchy.weatherUnit
        options: [
          { value: "auto", label: "Auto" },
          { value: "metric", label: "Celsius" },
          { value: "imperial", label: "Fahrenheit" }
        ]
        enabled: Omarchy.weatherPresent
        onChanged: function(value) {
          if (value !== Omarchy.weatherUnit) Omarchy.setWeatherUnit(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.weatherPresent
      stretchControl: true
      label: "Refresh"
      description: "How often the bar pulls a new forecast. Five minutes is chatty. An hour is plenty for most days."
      hint: "omarchy bar set omarchy.weather refreshMinutes"
      query: root.query
      keywords: ["interval", "update", "minutes", "forecast"]

      PrefsSlider {
        width: parent.width
        from: 5
        to: 60
        stepSize: 5
        value: Omarchy.weatherRefreshMinutes
        valueText: Omarchy.weatherRefreshMinutes + " min"
        formatTick: function(v) { return Math.round(v) }
        enabled: Omarchy.weatherPresent
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.weatherRefreshMinutes)
            Omarchy.setWeatherRefreshMinutes(next)
        }
      }
    }
  }

  PrefsGroup {
    title: "Diagnostics"
    query: root.query
    detail: "When a process dumps core, Omarchy can notify you so a coding agent can look at the crash."

    SettingRow {
      label: "Crash capture"
      description: "A notification when a process crashes, so a coding agent can look at the dump."
      hint: "omarchy toggle crash capture"
      query: root.query
      keywords: ["crash", "coredump", "agent", "watch"]

      PrefsToggle {
        checked: Omarchy.crashCapture
        onToggled: Omarchy.setCrashCapture(!Omarchy.crashCapture)
      }
    }
  }

  PrefsGroup {
    title: "Debug"
    catalog: false
    query: root.query
    detail: "Not in Find a setting. Show error opens the error dialog so you can try Copy and Dismiss without failing a real command."

    SettingRow {
      catalog: false
      label: "Error dialog"
      description: "Set lastError without running a failing command. Copy and Dismiss are on the dialog."
      query: root.query
      sectionHelp: false

      PrefsButton {
        text: "Show error…"
        onClicked: Omarchy.showDebugError()
      }
    }
  }
}
