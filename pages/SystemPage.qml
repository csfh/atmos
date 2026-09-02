import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "System"
  description: "This machine's name, language, and clock. Printers, weather, crash capture, and Omarchy updates are further down."

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
    message: "Replace the Hyprland Lua files in ~/.config/hypr with the shipped Omarchy copies. Your current files are backed up first. The Preferences drop-in require is written back so this window still floats."
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

  property string pendingChannel: ""

  Component.onCompleted: {
    channelConfirm.parent = root.prefsOverlay
    updateConfirm.parent = root.prefsOverlay
    firmwareConfirm.parent = root.prefsOverlay
    orphanConfirm.parent = root.prefsOverlay
    pruneConfirm.parent = root.prefsOverlay
    refreshHyprConfirm.parent = root.prefsOverlay
    refreshShellConfirm.parent = root.prefsOverlay
  }

  PrefsGroup {
    title: "Machine"
    query: root.query
    detail: "Hostname is how this computer shows up on the network and in prompts. Full name is the account's real name."

    PrefsRow {
      stretchControl: true
      label: "Hostname"
      description: "How this computer shows up on the network and in your shell prompt."
      hint: "hostnamectl set-hostname"
      query: root.query
      keywords: ["hostname", "computer", "machine", "device", "name"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: hostnameField
          width: parent.width - hostnameSetBtn.width - parent.spacing
          value: Omarchy.hostname
          placeholder: "hostname"
          enabled: !Omarchy.busy
          onSubmitted: function(value) { Omarchy.setHostname(value) }
        }

        PrefsButton {
          id: hostnameSetBtn
          text: "Set"
          primary: true
          enabled: !Omarchy.busy
          onClicked: Omarchy.setHostname(hostnameField.currentText())
        }
      }
    }

    PrefsRow {
      stretchControl: true
      label: "Full name"
      description: "The name on this account. Login screens and user lists show it."
      hint: "chfn --full-name"
      query: root.query
      keywords: ["user", "gecos", "real name", "display name", "account"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: fullNameField
          width: parent.width - fullNameSetBtn.width - parent.spacing
          value: Omarchy.fullName
          placeholder: "Your name"
          enabled: !Omarchy.busy
          onSubmitted: function(value) { Omarchy.setFullName(value) }
        }

        PrefsButton {
          id: fullNameSetBtn
          text: "Set"
          primary: true
          enabled: !Omarchy.busy
          onClicked: Omarchy.setFullName(fullNameField.currentText())
        }
      }
    }
  }

  PrefsGroup {
    title: "Keyboard"
    query: root.query
    detail: "XKB layout for typing. Hyprland reads it from vconsole.conf after you change it."

    PrefsRow {
      label: "Layout"
      description: "Key positions for typing. Hyprland picks this up from vconsole.conf."
      hint: "localectl set-x11-keymap"
      query: root.query
      keywords: ["keyboard", "layout", "keymap", "xkb", "qwerty", "language", "input"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.keyboardLayout
        options: Omarchy.keyboardLayouts
        enabled: !Omarchy.busy && Omarchy.keyboardLayouts.length > 0
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

    PrefsRow {
      label: "Locale"
      description: "Language for the system and apps. Open a new session after you change it."
      hint: "localectl set-locale"
      query: root.query
      keywords: ["locale", "lang", "language", "utf-8", "i18n", "translation"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.locale
        options: Omarchy.locales
        enabled: !Omarchy.busy && Omarchy.locales.length > 0
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

    PrefsRow {
      label: "Timezone"
      description: "The zone the clock, logs, and timestamps use."
      hint: "timedatectl set-timezone"
      query: root.query
      keywords: ["timezone", "tz", "utc", "region", "city", "date", "time", "zoneinfo"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.timezone
        options: Omarchy.timezones
        enabled: !Omarchy.busy && Omarchy.timezones.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.timezone) Omarchy.setTimezone(value)
        }
      }
    }

    PrefsRow {
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
        enabled: !Omarchy.busy && Omarchy.ntpAvailable
        onToggled: Omarchy.setNtp(!Omarchy.ntp)
      }
    }
  }

  PrefsGroup {
    title: "Updates"
    query: root.query
    detail: "Channel picks which Omarchy package stream you follow. Update runs the usual omarchy update job."

    PrefsRow {
      label: "Version"
      description: Omarchy.omarchyVersion.length
        ? ("This machine is on Omarchy " + Omarchy.omarchyVersion + (Omarchy.omarchyChannel ? (" (" + Omarchy.omarchyChannel + ")") : "") + ".")
        : "Omarchy version was not readable."
      hint: "omarchy version"
      query: root.query
      keywords: ["version", "release", "omarchy"]
    }

    PrefsRow {
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
        enabled: !Omarchy.busy && !Omarchy.jobBusy && Omarchy.omarchyChannel.length > 0
        onChanged: function(value) {
          if (value === Omarchy.omarchyChannel) return
          root.pendingChannel = value
          channelConfirm.ask()
        }
      }
    }

    PrefsRow {
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
        spacing: 8
        PrefsButton {
          text: "Check"
          enabled: !Omarchy.busy && !Omarchy.jobBusy
          onClicked: Omarchy.checkOmarchyUpdate()
        }
        PrefsButton {
          text: "Update…"
          primary: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy
          onClicked: updateConfirm.ask()
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Firmware through fwupd, leftover packages, the pacman download cache, and restore for Hyprland Lua or shell.json."

    PrefsRow {
      label: "Firmware"
      description: Omarchy.jobKind === "update-firmware" && Omarchy.jobBusy
        ? "Updating firmware…"
        : "Install firmware updates through fwupd when the vendor ships them."
      hint: "omarchy update firmware"
      query: root.query
      keywords: ["firmware", "fwupd", "bios"]

      PrefsButton {
        text: "Update…"
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: firmwareConfirm.ask()
      }
    }

    PrefsRow {
      label: "Orphan packages"
      description: "Remove packages that nothing else depends on."
      hint: "omarchy update orphan pkgs"
      query: root.query
      keywords: ["orphan", "unused", "pacman"]

      PrefsButton {
        text: "Remove…"
        danger: true
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: orphanConfirm.ask()
      }
    }

    PrefsRow {
      label: "Package cache"
      description: "Delete old downloaded packages to free disk."
      hint: "omarchy update pkg prune"
      query: root.query
      keywords: ["prune", "cache", "pacman"]

      PrefsButton {
        text: "Prune…"
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: pruneConfirm.ask()
      }
    }

    PrefsRow {
      label: "Restart shell"
      description: "Reload the bar and notifications without touching shell.json."
      hint: "omarchy restart shell"
      query: root.query
      keywords: ["restart", "reload", "bar", "quickshell"]

      PrefsButton {
        text: "Restart"
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: Omarchy.restartShell()
      }
    }

    PrefsRow {
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
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: refreshHyprConfirm.ask()
      }
    }

    PrefsRow {
      label: "Restore shell"
      description: "Put the shipped shell.json back. Your copy is backed up. The bar restarts."
      hint: "omarchy refresh shell"
      query: root.query
      keywords: ["refresh", "shell", "restore", "defaults", "bar"]

      PrefsButton {
        text: "Restore…"
        danger: true
        enabled: !Omarchy.busy && !Omarchy.jobBusy
        onClicked: refreshShellConfirm.ask()
      }
    }
  }

  PrefsGroup {
    title: "Printers"
    query: root.query
    detail: "CUPS is the print service on this machine. Setup opens the usual printer window. The web UI is the CUPS admin page on this computer."

    PrefsRow {
      label: "Printers"
      description: Omarchy.cupsActive
        ? "CUPS is running. Setup opens the printer window. The web UI is http://127.0.0.1:631."
        : "CUPS is not running. You can still open the admin page if you start the service."
      hint: "system-config-printer"
      query: root.query
      keywords: ["printer", "cups", "print", "ipp"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Setup"
          primary: true
          enabled: !Omarchy.busy
          onClicked: Omarchy.openPrinters()
        }
        PrefsButton {
          text: "CUPS"
          enabled: !Omarchy.busy
          onClicked: Omarchy.openCupsAdmin()
        }
      }
    }
  }

  PrefsGroup {
    title: "Packages"
    query: root.query
    detail: "How many packages pacman fetches at once. Higher can finish a big upgrade sooner on a fast link."

    PrefsRow {
      label: "Parallel downloads"
      description: "How many packages pacman fetches at once. Bump this if updates feel slow on a good connection."
      hint: "/etc/pacman.conf · ParallelDownloads"
      query: root.query
      keywords: ["pacman", "downloads", "parallel", "mirrors", "aur", "speed"]

      PrefsSpinBox {
        from: 1
        to: 20
        value: Omarchy.parallelDownloads
        enabled: !Omarchy.busy
        onChanged: function(value) {
          if (value !== Omarchy.parallelDownloads) Omarchy.setParallelDownloads(value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Branding"
    query: root.query
    detail: "ASCII art on the About screen. Image turns a picture into that art. Edit opens the text file if you want to write it yourself. Reset puts the Omarchy icon back."

    PrefsRow {
      label: "About logo"
      description: Omarchy.aboutBranded
        ? "You are using custom ASCII art on the About screen."
        : "The stock Omarchy icon. Image turns a picture into ASCII. Edit opens the text file."
      hint: "omarchy branding about"
      query: root.query
      keywords: ["ascii", "logo", "fastfetch", "about"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Image…"
          enabled: !Omarchy.busy
          onClicked: Omarchy.setAboutBranding("image")
        }
        PrefsButton {
          text: "Edit"
          enabled: !Omarchy.busy
          onClicked: Omarchy.setAboutBranding("text")
        }
        PrefsButton {
          visible: Omarchy.aboutBranded
          text: "Reset"
          danger: true
          enabled: !Omarchy.busy && Omarchy.aboutBranded
          onClicked: Omarchy.setAboutBranding("reset")
        }
      }
    }
  }

  PrefsGroup {
    title: "Weather"
    query: root.query
    detail: "The city the weather widget and notifications use. Auto guesses from your IP. Coordinates pin the forecast when a city name is ambiguous."

    PrefsRow {
      stretchControl: true
      label: "Location"
      description: Omarchy.weatherAuto
        ? "Guessing the city from your IP for the weather widget and notifications."
        : "The city the weather widget and notifications use."
      hint: "omarchy weather location"
      query: root.query
      keywords: ["weather", "city", "forecast", "wttr"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: weatherField
          width: parent.width - weatherSetBtn.width - weatherAutoBtn.width - parent.spacing * 2
          value: Omarchy.weatherAuto ? "" : Omarchy.weatherLocation
          placeholder: "City name"
          enabled: !Omarchy.busy
          onSubmitted: function(value) { Omarchy.setWeatherLocation(value) }
        }

        PrefsButton {
          id: weatherSetBtn
          text: "Set"
          primary: true
          enabled: !Omarchy.busy
          onClicked: Omarchy.setWeatherLocation(weatherField.currentText())
        }

        PrefsButton {
          id: weatherAutoBtn
          text: "Auto"
          enabled: !Omarchy.busy && !Omarchy.weatherAuto
          onClicked: Omarchy.clearWeatherLocation()
        }
      }
    }

    PrefsRow {
      available: !Omarchy.weatherAuto && Omarchy.weatherLocation.length > 0
      stretchControl: true
      label: "Coordinates"
      description: "Optional latitude and longitude if the city name is ambiguous. Use lat,lon."
      hint: "omarchy weather location --set name lat,lon"
      query: root.query
      keywords: ["latitude", "longitude", "gps", "coords"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: weatherCoordsField
          width: parent.width - weatherCoordsSetBtn.width - parent.spacing
          value: Omarchy.weatherCoords
          placeholder: "lat,lon"
          enabled: !Omarchy.busy
          onSubmitted: function(value) { Omarchy.setWeatherCoordinates(value) }
        }

        PrefsButton {
          id: weatherCoordsSetBtn
          text: "Set"
          primary: true
          enabled: !Omarchy.busy
          onClicked: Omarchy.setWeatherCoordinates(weatherCoordsField.currentText())
        }
      }
    }

    PrefsRow {
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
        enabled: !Omarchy.busy && Omarchy.weatherPresent
        onChanged: function(value) {
          if (value !== Omarchy.weatherUnit) Omarchy.setWeatherUnit(value)
        }
      }
    }

    PrefsRow {
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
        enabled: !Omarchy.busy && Omarchy.weatherPresent
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

    PrefsRow {
      label: "Crash capture"
      description: "Notify you when a process crashes, so a coding agent can look at the dump."
      hint: "omarchy toggle crash capture"
      query: root.query
      keywords: ["crash", "coredump", "agent", "watch"]

      PrefsToggle {
        checked: Omarchy.crashCapture
        enabled: !Omarchy.busy
        onToggled: Omarchy.setCrashCapture(!Omarchy.crashCapture)
      }
    }
  }
}
