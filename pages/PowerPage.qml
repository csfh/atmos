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

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
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
    title: "Battery"
    query: (Omarchy.batteryPresent || (Omarchy.powerPresent && Omarchy.isLaptop)) ? root.query : "."
    detail: "A one-shot notification with charge and draw. The percentage toggle is the number next to the bar's power icon."

    PrefsRow {
      available: Omarchy.batteryPresent
      label: "Battery status"
      description: "Pop a notification with the current charge and how much power you are drawing."
      hint: "omarchy notification battery"
      query: root.query
      keywords: ["charge", "notify", "draw", "capacity"]

      PrefsButton {
        text: "Show"
        enabled: Omarchy.batteryPresent
        onClicked: Omarchy.showBatteryNotification()
      }
    }

    PrefsRow {
      available: Omarchy.powerPresent && Omarchy.isLaptop
      label: "Battery percentage"
      description: "Show the charge number next to the power icon on the bar."
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
