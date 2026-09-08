import QtQuick
import "../../components"
import "../../services"
import "../../services/Autostart.js" as AutoJs

PrefsPage {
  id: root
  hubId: "applications/startup"
  title: "Startup"
  description: "Programs Hyprland launches at login. Disable comments the line out. Delay prefixes sleep N &&. Remove only deletes a row Atmos added."

  property string autostartDraft: ""
  property int delayDraft: 0
  property string pendingAutostart: ""

  readonly property var rows: {
    var list = Omarchy.autostart || []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].command) out.push(list[i])
    }
    return out
  }

  readonly property var failed: {
    var units = Omarchy.systemdUnits || []
    var out = []
    for (var i = 0; i < units.length; i++) {
      if (units[i] && (units[i].active === "failed" || units[i].sub === "failed")) out.push(units[i])
    }
    return out
  }

  Component.onCompleted: {
    removeAutostartConfirm.parent = root.prefsOverlay
  }

  PrefsConfirm {
    id: removeAutostartConfirm
    title: "Remove startup command"
    message: "Remove " + root.pendingAutostart + " from the Atmos autostart block?"
    confirmText: "Remove"
    onConfirmed: Omarchy.removeAutostart(root.pendingAutostart)
  }

  PrefsGroup {
    framed: true
    title: "Launch on start"
    query: root.query
    detail: "Writes o.launch_on_start in ~/.config/hypr/autostart.lua. Lines you typed yourself stay."
    hint: "~/.config/hypr/autostart.lua"

    SettingRow {
      label: "Add a command"
      description: "A program name or command. Delay waits that many seconds after login."
      hint: "o.launch_on_start"
      query: root.query
      keywords: ["autostart", "startup", "delay"]

      Row {
        spacing: Theme.space
        PrefsField {
          width: 180
          placeholder: "hyprsunset"
          onEdited: function(value) { root.autostartDraft = value }
          onSubmitted: function(value) {
            Omarchy.addAutostart(value, root.delayDraft)
          }
        }
        PrefsSpinBox {
          from: 0
          to: 600
          value: root.delayDraft
          onChanged: function(value) { root.delayDraft = Math.round(value) }
        }
        PrefsButton {
          text: "Add"
          primary: true
          onClicked: Omarchy.addAutostart(root.autostartDraft, root.delayDraft)
        }
      }
    }

    Repeater {
      model: root.rows

      SettingRow {
        required property var modelData
        label: modelData && modelData.command ? modelData.command : "command"
        description: (modelData && modelData.enabled === false ? "Disabled. " : "") + (modelData && modelData.delay ? ("Starts after " + modelData.delay + "s. ") : "") + (modelData && modelData.managed ? "Atmos manages this line." : "You wrote this line.")
        hint: "~/.config/hypr/autostart.lua"
        query: root.query
        keywords: ["autostart", "enable", "delay"]

        Row {
          spacing: Theme.space
          PrefsToggle {
            checked: modelData && modelData.enabled !== false
            enabled: modelData && modelData.managed
            onToggled: Omarchy.setAutostartEnabled(modelData.command, !(modelData && modelData.enabled !== false))
          }
          PrefsButton {
            text: "Remove…"
            danger: true
            enabled: modelData && modelData.managed
            onClicked: {
              root.pendingAutostart = modelData.command
              removeAutostartConfirm.ask()
            }
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Failures"
    query: root.failed.length ? root.query : "."
    detail: "User or system units that failed this boot. Startup commands that never launched often show up here."

    SettingRow {
      available: root.failed.length === 0
      label: "Startup failures"
      description: "No failed units reported."
      query: root.query
      keywords: ["failed", "empty"]
    }

    Repeater {
      model: root.failed

      SettingRow {
        required property var modelData
        label: modelData && modelData.unit ? modelData.unit : "unit"
        description: modelData && modelData.description ? modelData.description : "Failed this boot."
        query: root.query
        keywords: ["failed", "startup"]
      }
    }
  }
}
