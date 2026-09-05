import QtQuick
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi

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

    SettingRow {
      label: "Do not disturb"
      description: "Hide ordinary notification toasts. Critical alerts still get through."
      hint: "omarchy toggle notification silencing"
      query: root.query
      keywords: ["dnd", "silent", "mute", "quiet"]

      PrefsToggle {
        checked: Omarchy.doNotDisturb
        onToggled: Omarchy.setDoNotDisturb(!Omarchy.doNotDisturb)
      }
    }
  }

  PrefsGroup {
    title: "Send"
    query: root.query
    detail: "These fire an Omarchy toast right now. Use Test if you want to see that notifications still land."

    SettingRow {
      label: "Test toast"
      description: "Send a sample notification from Atmos."
      hint: "omarchy notification send"
      query: root.query
      keywords: ["test", "toast", "notify", "send"]

      PrefsButton {
        text: "Send test"
        onClicked: Omarchy.sendTestNotification()
      }
    }

    SettingRow {
      label: "Time"
      description: "Show the current time and date as a toast."
      hint: "omarchy notification time"
      query: root.query
      keywords: ["time", "clock", "date", "toast"]

      PrefsButton {
        text: "Show time"
        onClicked: Omarchy.sendTimeNotification()
      }
    }

    SettingRow {
      available: Omarchy.batteryPresent
      label: "Battery"
      description: "Show the current battery toast."
      hint: "omarchy notification battery"
      query: root.query
      keywords: ["battery", "charge", "toast"]

      PrefsButton {
        text: "Show battery"
        enabled: Omarchy.batteryPresent
        onClicked: Omarchy.showBatteryNotification()
      }
    }

    SettingRow {
      available: Omarchy.weatherPresent
      label: "Weather"
      description: "Show the current forecast toast."
      hint: "omarchy notification weather"
      query: root.query
      keywords: ["weather", "forecast", "toast"]

      PrefsButton {
        text: "Show weather"
        enabled: Omarchy.weatherPresent
        onClicked: Omarchy.sendWeatherNotification()
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Reminders"
    query: root.query
    detail: "A reminder is a systemd timer that sends an Omarchy toast. Clear drops every outstanding one."
    hint: "omarchy reminder"

    SettingRow {
      label: "New reminder"
      description: "Minutes from now, and an optional message."
      hint: "omarchy reminder"
      query: root.query
      keywords: ["reminder", "timer", "later", "minutes"]

      Row {
        spacing: Theme.space
        PrefsSpinBox {
          from: 1
          to: 1440
          value: root.reminderMinutes
          onChanged: function(value) { root.reminderMinutes = value }
        }
        PrefsField {
          width: 160
          placeholder: "Message"
          onEdited: function(value) { root.reminderMessage = value }
          onSubmitted: function(value) {
            root.reminderMessage = value
            Omarchy.setReminder(String(root.reminderMinutes), root.reminderMessage)
          }
        }
        PrefsButton {
          text: "Set"
          primary: true
          onClicked: Omarchy.setReminder(String(root.reminderMinutes), root.reminderMessage)
        }
      }
    }

    SettingRow {
      available: Omarchy.reminderActive
      label: "Clear reminders"
      description: Omarchy.reminderCount === 1
        ? "One reminder is waiting."
        : (Omarchy.reminderCount + " reminders are waiting.")
      hint: "omarchy reminder clear"
      query: root.query
      keywords: ["reminder", "clear", "cancel"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Show"
          enabled: Omarchy.reminderActive
          onClicked: Omarchy.showReminders()
        }
        PrefsButton {
          text: "Clear"
          danger: true
          enabled: Omarchy.reminderActive
          onClicked: Omarchy.clearReminders()
        }
      }
    }

    SettingRow {
      available: !Omarchy.reminderActive
      sectionHelp: false
      label: "Reminders"
      description: "No reminders waiting."
      query: root.query
      keywords: ["reminder", "empty"]
    }

    Repeater {
      model: Omarchy.reminders

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.label ? modelData.label : "Reminder"
        description: modelData && modelData.remaining
          ? ("Fires in " + modelData.remaining + (modelData.atTime ? " (" + modelData.atTime + ")" : "") + ". Clear all is above — Omarchy cannot drop a single timer.")
          : "Waiting. Clear all is above — Omarchy cannot drop a single timer."
        hint: "omarchy reminder show --json"
        query: root.query
        keywords: ["reminder", "timer"]

        Row {
          spacing: Theme.space
          PrefsButton {
            text: "Copy"
            enabled: RichUi.reminderCopyText(modelData).length > 0
            onClicked: Omarchy.copyText(RichUi.reminderCopyText(modelData))
          }
          PrefsButton {
            text: "Show"
            onClicked: Omarchy.showReminders()
          }
        }
      }
    }
  }
}
