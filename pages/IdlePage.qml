import QtQuick
import "../components"
import "../services"
import "../services/Theme.js" as ThemeJs

PrefsPage {
  id: root
  title: "Idle and lock"
  description: "How long the machine waits before the screensaver and lock. You can also keep it awake or change the screensaver logo."

  PrefsGroup {
    title: "Timings"
    query: root.query
    detail: "Screensaver and lock are separate timers in ~/.config/omarchy/shell.json. Zero on a slider skips that step."

    SettingRow {
      stretchControl: true
      label: "Screensaver"
      description: "How long you can sit still before the screensaver starts. Zero skips the screensaver."
      hint: "~/.config/omarchy/shell.json · idle.screensaver"
      query: root.query
      keywords: ["idle", "timeout", "sleep"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 1800
        stepSize: 60
        value: Omarchy.idleScreensaver
        valueText: ThemeJs.formatSeconds(Omarchy.idleScreensaver)
        formatTick: function(v) { return ThemeJs.formatSeconds(v) }
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.idleScreensaver)
            Omarchy.setIdle(next, Omarchy.idleLock)
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Lock"
      description: "How long you can sit still before the session locks. Zero skips the lock."
      hint: "~/.config/omarchy/shell.json · idle.lock"
      query: root.query
      keywords: ["screen lock", "security"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 3600
        stepSize: 60
        value: Omarchy.idleLock
        valueText: ThemeJs.formatSeconds(Omarchy.idleLock)
        formatTick: function(v) { return ThemeJs.formatSeconds(v) }
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.idleLock)
            Omarchy.setIdle(Omarchy.idleScreensaver, next)
        }
      }
    }
  }

  PrefsGroup {
    title: "Behavior"
    query: root.query
    detail: "Stay awake skips the screensaver and lock timers. Screensaver and Suspend menu stay available while these switches are on."

    SettingRow {
      label: "Stay awake"
      description: "Keep the screen awake and unlocked."
      hint: "omarchy toggle idle"
      query: root.query
      keywords: ["caffeine", "inhibit", "awake", "sleep"]

      PrefsToggle {
        checked: Omarchy.stayAwake
        onToggled: Omarchy.setStayAwake(!Omarchy.stayAwake)
      }
    }

    SettingRow {
      label: "Screensaver"
      description: "The screensaver runs after the idle timeout."
      hint: "omarchy toggle screensaver"
      query: root.query
      keywords: ["screensaver", "allow", "disable", "tte"]

      PrefsToggle {
        checked: Omarchy.screensaverEnabled
        onToggled: Omarchy.setScreensaverEnabled(!Omarchy.screensaverEnabled)
      }
    }

    SettingRow {
      label: "Suspend menu"
      description: "Suspend stays in the system menu."
      hint: "omarchy toggle suspend"
      query: root.query
      keywords: ["sleep", "power", "system menu", "allow", "suspend"]

      PrefsToggle {
        checked: Omarchy.suspendEnabled
        onToggled: Omarchy.setSuspendEnabled(!Omarchy.suspendEnabled)
      }
    }
  }

  PrefsGroup {
    title: "Branding"
    query: root.query
    detail: "ASCII art on the screensaver. Choose a picture to turn into that art. Edit opens the text file if you want to write it yourself. Reset puts the Omarchy logo back."

    SettingRow {
      label: "Screensaver logo"
      description: Omarchy.screensaverBranded
        ? "You are using custom ASCII art on the screensaver."
        : "The stock Omarchy logo. Choose a picture to turn into ASCII. Edit opens the text file."
      hint: "omarchy branding screensaver"
      query: root.query
      keywords: ["ascii", "logo", "branding", "tte"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Choose…"
          onClicked: Omarchy.setScreensaverBranding("image")
        }
        PrefsButton {
          text: "Edit"
          onClicked: Omarchy.setScreensaverBranding("text")
        }
        PrefsButton {
          visible: Omarchy.screensaverBranded
          text: "Reset"
          danger: true
          enabled: Omarchy.screensaverBranded
          onClicked: Omarchy.setScreensaverBranding("reset")
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: Omarchy.isLaptop ? root.query : "."
    detail: "Lid close already locks when the machine is undocked. Omarchy runs that from logind, not from Atmos."

    SettingRow {
      available: Omarchy.isLaptop
      label: "Lid close"
      description: "Closing the lid locks when undocked. A docked lid stays unlocked on the other screen. Change this in logind (omarchy-system-lid-close), not here."
      hint: "omarchy-system-lid-close"
      query: root.query
      keywords: ["lid", "clamshell", "lock", "close"]
      valueText: "On"
    }
  }
}
