import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Power"
  description: Omarchy.isLaptop
    ? "How hard the machine works, and what the battery is doing."
    : "How hard the machine works while it is plugged in."

  readonly property var profileLabels: ({
    "power-saver": "Power saver",
    balanced: "Balanced",
    performance: "Performance"
  })

  readonly property var profileOptions: {
    var out = []
    var list = Omarchy.powerProfiles || []
    for (var i = 0; i < list.length; i++) {
      var id = String(list[i])
      out.push({ value: id, label: root.profileLabels[id] || id })
    }
    return out
  }

  PrefsGroup {
    title: "Profile"
    query: root.query
    detail: "These are power-profiles-daemon modes. Performance uses more power. Power saver stretches battery. On a laptop, AC and battery each remember their own pick."

    SettingRow {
      label: "Power profile"
      description: Omarchy.isLaptop
        ? "How hard the machine works right now. On a laptop this follows the AC or battery pick below."
        : "How hard the machine works right now."
      hint: "omarchy powerprofiles set"
      query: root.query
      keywords: ["performance", "balanced", "power saver", "battery"]

      PrefsSelect {
        value: Omarchy.powerProfile
        options: root.profileOptions
        enabled: root.profileOptions.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.powerProfile) Omarchy.setPowerProfile(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.isLaptop
      label: "On AC"
      description: "The profile to use while the charger is plugged in."
      hint: "omarchy powerprofiles set ac"
      query: root.query
      keywords: ["plugged", "mains", "adapter", "charger"]

      PrefsSelect {
        value: Omarchy.powerProfileAc
        options: root.profileOptions
        enabled: root.profileOptions.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.powerProfileAc) Omarchy.setPowerProfileAc(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.isLaptop
      label: "On battery"
      description: "The profile to use while you are on battery."
      hint: "omarchy powerprofiles set battery"
      query: root.query
      keywords: ["unplugged", "discharging", "laptop"]

      PrefsSelect {
        value: Omarchy.powerProfileBattery
        options: root.profileOptions
        enabled: root.profileOptions.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.powerProfileBattery) Omarchy.setPowerProfileBattery(value)
        }
      }
    }

  }

  PrefsGroup {
    title: "Presentation"
    query: root.query
    detail: "Stay awake, silence notifications, and stop the screensaver. This is session state, not a saved prefs file."

    SettingRow {
      label: "Presentation Mode"
      description: Omarchy.presentationMode
        ? "Idle, lock, and notifications are held off until you turn this off."
        : "A timed stay-awake plus do-not-disturb for talks and sharing a screen."
      hint: "~/.local/state/omarchy/atmos-presentation.json"
      query: root.query
      keywords: ["presentation", "stay awake", "dnd", "caffeine"]

      PrefsToggle {
        checked: Omarchy.presentationMode
        onToggled: function(next) { Omarchy.setPresentationMode(next) }
      }
    }
  }

  PrefsGroup {
    title: "Hardware"
    query: (Omarchy.powerGovernor || Omarchy.amdPstate || Omarchy.chargeLimitAvailable) ? root.query : "."
    detail: "Governor and energy preference are what power-profiles-daemon is using. Atmos does not write CPU sysfs while PPD is running."

    SettingRow {
      available: Omarchy.powerGovernor.length > 0
      label: "CPU governor"
      description: "What power-profiles-daemon selected. Atmos does not write this."
      hint: "scaling_governor"
      query: root.query
      keywords: ["governor", "cpu", "schedutil"]
      valueText: Omarchy.powerGovernor
    }

    SettingRow {
      available: Omarchy.amdPstate.length > 0
      label: "Energy preference"
      description: "The energy_performance_preference power-profiles-daemon selected."
      hint: "energy_performance_preference"
      query: root.query
      keywords: ["amd", "pstate", "epp"]
      valueText: Omarchy.amdPstate
    }

    SettingRow {
      available: Omarchy.chargeLimitAvailable
      stretchControl: true
      label: "Charge limit"
      description: "Stop charging past this percent. Only on hardware that exposes a threshold."
      hint: "charge_control_end_threshold"
      query: root.query
      keywords: ["charge", "limit", "battery"]

      PrefsSlider {
        width: parent.width
        from: 50
        to: 100
        stepSize: 5
        value: Omarchy.chargeLimit || 100
        valueText: (Omarchy.chargeLimit || 100) + "%"
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.chargeLimit) Omarchy.setChargeLimit(next)
        }
      }
    }
  }

  PrefsGroup {
    title: "Battery"
    query: (Omarchy.batteryPresent || (Omarchy.powerPresent && Omarchy.isLaptop)) ? root.query : "."
    detail: "A one-shot notification with charge and draw. The percentage toggle is the number next to the bar's power icon."

    SettingRow {
      available: Omarchy.batteryPresent
      label: "Battery status"
      description: "Pop a notification with the current charge and how much power you are drawing."
      hint: "omarchy notification battery"
      query: root.query
      keywords: ["charge", "notify", "draw", "capacity"]

      PrefsButton {
        text: "Show battery"
        enabled: Omarchy.batteryPresent
        onClicked: Omarchy.showBatteryNotification()
      }
    }

    SettingRow {
      available: Omarchy.powerPresent && Omarchy.isLaptop
      label: "Battery percentage"
      description: "The charge number next to the power icon on the bar."
      hint: "omarchy bar set omarchy.power showPercentage"
      query: root.query
      keywords: ["battery", "percent", "charge", "laptop"]

      PrefsToggle {
        checked: Omarchy.powerShowPercentage
        enabled: Omarchy.powerPresent && Omarchy.isLaptop
        onToggled: Omarchy.setPowerShowPercentage(!Omarchy.powerShowPercentage)
      }
    }
  }
}
