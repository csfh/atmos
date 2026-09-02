import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Notifications"
  description: "Do not disturb, a test toast, and reminders that already live in Omarchy."

  property int reminderMinutes: 5
  property string reminderMessage: ""

  PrefsGroup {
    title: "Quiet"
    query: root.query
    detail: "Do not disturb hides ordinary toasts. Critical alerts still get through."

    PrefsRow {
      label: "Do not disturb"
      description: "Hide ordinary notification toasts. Critical alerts still get through."
      hint: "omarchy toggle notification silencing"
      query: root.query
      keywords: ["dnd", "silent", "mute", "quiet"]

      PrefsToggle {
        checked: Omarchy.doNotDisturb
        enabled: !Omarchy.busy
        onToggled: Omarchy.setDoNotDisturb(!Omarchy.doNotDisturb)
      }
    }
  }

  PrefsGroup {
    title: "Send"
    query: root.query
    detail: "These fire an Omarchy toast right now. Use Test if you want to see that notifications still land."

    PrefsRow {
      label: "Test toast"
      description: "Send a sample notification from Preferences."
      hint: "omarchy notification send"
      query: root.query
      keywords: ["test", "toast", "notify", "send"]

      PrefsButton {
        text: "Send test"
        enabled: !Omarchy.busy
        onClicked: Omarchy.sendTestNotification()
      }
    }

    PrefsRow {
      label: "Time"
      description: "Show the current time and date as a toast."
      hint: "omarchy notification time"
      query: root.query
      keywords: ["time", "clock", "date", "toast"]

      PrefsButton {
        text: "Show time"
        enabled: !Omarchy.busy
        onClicked: Omarchy.sendTimeNotification()
      }
    }

    PrefsRow {
      available: Omarchy.batteryPresent
      label: "Battery"
      description: "Show the current battery toast."
      hint: "omarchy notification battery"
      query: root.query
      keywords: ["battery", "charge", "toast"]

      PrefsButton {
        text: "Show battery"
        enabled: !Omarchy.busy && Omarchy.batteryPresent
        onClicked: Omarchy.showBatteryNotification()
      }
    }

    PrefsRow {
      available: Omarchy.weatherPresent
      label: "Weather"
      description: "Show the current forecast toast."
      hint: "omarchy notification weather"
      query: root.query
      keywords: ["weather", "forecast", "toast"]

      PrefsButton {
        text: "Show weather"
        enabled: !Omarchy.busy && Omarchy.weatherPresent
        onClicked: Omarchy.sendWeatherNotification()
      }
    }
  }

  PrefsGroup {
    title: "Reminders"
    query: root.query
    detail: "A reminder is a systemd timer that sends an Omarchy toast. Clear drops every outstanding one."
    hint: "omarchy reminder"

    PrefsRow {
      label: "New reminder"
      description: "Minutes from now, and an optional message."
      hint: "omarchy reminder"
      query: root.query
      keywords: ["reminder", "timer", "later", "minutes"]

      Row {
        spacing: 8
        PrefsSpinBox {
          from: 1
          to: 1440
          value: root.reminderMinutes
          enabled: !Omarchy.busy
          onChanged: function(value) { root.reminderMinutes = value }
        }
        PrefsField {
          width: 160
          placeholder: "Message"
          enabled: !Omarchy.busy
          onEdited: function(value) { root.reminderMessage = value }
          onSubmitted: function(value) {
            root.reminderMessage = value
            Omarchy.setReminder(String(root.reminderMinutes), root.reminderMessage)
          }
        }
        PrefsButton {
          text: "Set"
          primary: true
          enabled: !Omarchy.busy
          onClicked: Omarchy.setReminder(String(root.reminderMinutes), root.reminderMessage)
        }
      }
    }

    PrefsRow {
      available: Omarchy.reminderActive
      label: "Clear reminders"
      description: Omarchy.reminderCount === 1
        ? "One reminder is waiting."
        : (Omarchy.reminderCount + " reminders are waiting.")
      hint: "omarchy reminder clear"
      query: root.query
      keywords: ["reminder", "clear", "cancel"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Show"
          enabled: !Omarchy.busy && Omarchy.reminderActive
          onClicked: Omarchy.showReminders()
        }
        PrefsButton {
          text: "Clear"
          danger: true
          enabled: !Omarchy.busy && Omarchy.reminderActive
          onClicked: Omarchy.clearReminders()
        }
      }
    }

    PrefsRow {
      available: !Omarchy.reminderActive
      sectionHelp: false
      label: "None waiting"
      description: "Set a reminder above if you want a toast later."
      query: root.query
      keywords: ["reminder", "empty"]
    }

    Repeater {
      model: Omarchy.reminders

      PrefsRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.label ? modelData.label : "Reminder"
        description: modelData && modelData.remaining
          ? ("Fires in " + modelData.remaining + (modelData.atTime ? " (" + modelData.atTime + ")" : "") + ".")
          : "Waiting."
        hint: "omarchy reminder show --json"
        query: root.query
        keywords: ["reminder", "timer"]
      }
    }
  }
}
