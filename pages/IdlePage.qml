import QtQuick
import "../components"
import "../services"
import "../services/Theme.js" as ThemeJs

PrefsPage {
  id: root
  hubId: "idle"
  title: I18n.tr("Idle")
  description: I18n.tr("How long the machine waits before the screensaver and lock. You can also keep it awake or change the screensaver logo.")

  PrefsGroup {
    title: I18n.tr("Timings")
    query: root.query
    detail: "Screensaver and lock are separate timers in ~/.config/omarchy/shell.json. Zero on a slider skips that step."

    SettingRow {
      stretchControl: true
      label: "Screensaver"
      description: I18n.tr("How long you can sit still before the screensaver starts. Zero skips the screensaver.")
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
      description: I18n.tr("How long you can sit still before the session locks. Zero skips the lock.")
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
    title: I18n.tr("Behavior")
    query: root.query
    detail: "Stay awake skips the screensaver and lock timers. Screensaver and Suspend menu stay available while these switches are on."

    SettingRow {
      label: "Stay awake"
      description: I18n.tr("Keep the screen awake and unlocked.")
      hint: "omarchy toggle idle"
      query: root.query
      keywords: ["caffeine", "inhibit", "awake", "sleep"]

      PrefsToggle {
        checked: Omarchy.stayAwake
        onToggled: Omarchy.set("stayAwake", !Omarchy.stayAwake)
      }
    }

    SettingRow {
      label: "Screensaver"
      description: I18n.tr("The screensaver runs after the idle timeout.")
      hint: "omarchy toggle screensaver-off"
      query: root.query
      keywords: ["screensaver", "allow", "disable"]

      PrefsToggle {
        checked: Omarchy.screensaverEnabled
        onToggled: Omarchy.set("screensaverEnabled", !Omarchy.screensaverEnabled)
      }
    }

    SettingRow {
      label: "Suspend menu"
      description: I18n.tr("Suspend stays in the system menu.")
      hint: "omarchy toggle suspend-off"
      query: root.query
      keywords: ["sleep", "power", "system menu", "allow", "suspend"]

      PrefsToggle {
        checked: Omarchy.suspendEnabled
        onToggled: Omarchy.set("suspendEnabled", !Omarchy.suspendEnabled)
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Branding")
    query: root.query
    detail: "ASCII art on the screensaver. Choose a picture to turn into that art. Edit opens the text file if you want to write it yourself. Reset puts the Omarchy logo back."

    SettingRow {
      label: "Screensaver logo"
      description: Omarchy.screensaverBranded
        ? "You are using custom ASCII art on the screensaver."
        : "The stock Omarchy logo. Choose a picture to turn into ASCII. Edit opens the text file."
      hint: "omarchy branding screensaver"
      query: root.query
      keywords: ["ascii", "logo", "branding"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: I18n.tr("Choose…")
          onClicked: Omarchy.setScreensaverBranding("image")
        }
        PrefsButton {
          text: I18n.tr("Edit")
          onClicked: Omarchy.setScreensaverBranding("text")
        }
        PrefsButton {
          visible: Omarchy.screensaverBranded
          text: I18n.tr("Reset")
          danger: true
          enabled: Omarchy.screensaverBranded
          onClicked: Omarchy.setScreensaverBranding("reset")
        }
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Advanced")
    advanced: true
    query: Omarchy.isLaptop ? root.query : "."
    detail: "Lid close already locks when the machine is undocked. Omarchy runs that from logind, not from Atmos."

    SettingRow {
      available: Omarchy.isLaptop
      label: "Lid close"
      description: I18n.tr("Closing the lid locks when undocked. A docked lid stays unlocked on the other screen. Change this in logind (omarchy-system-lid-close), not here.")
      hint: "omarchy-system-lid-close"
      query: root.query
      keywords: ["lid", "clamshell", "lock", "close"]
      valueText: "On"
    }
  }
}
