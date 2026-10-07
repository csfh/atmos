import QtQuick
import "../../components"
import "../../services"

PrefsPage {
  id: root
  hubId: "applications/startup"
  title: I18n.tr("Startup")
  description: I18n.tr("Programs Hyprland launches at login. Disable comments the line out. Delay prefixes sleep N &&. Remove only deletes a row Atmos added.")

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
    title: I18n.tr("Remove startup command")
    message: I18n.tr("Remove {command} from the Atmos autostart block?", { command: root.pendingAutostart })
    confirmText: "Remove"
    onConfirmed: Omarchy.removeAutostart(root.pendingAutostart)
  }

  PrefsGroup {
    framed: true
    title: I18n.tr("Launch on start")
    query: root.query
    detail: "Writes o.launch_on_start in ~/.config/hypr/autostart.lua. Lines you typed yourself stay."
    hint: "~/.config/hypr/autostart.lua"

    SettingRow {
      label: "Add a command"
      description: I18n.tr("A program name or command. Delay waits that many seconds after login.")
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
          text: I18n.tr("Add")
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
        description: (modelData && modelData.enabled === false ? I18n.tr("Disabled. ") : "")
        + (modelData && modelData.delay ? I18n.tr("Starts after {seconds}s. ", { seconds: modelData.delay }) : "")
        + (modelData && modelData.managed ? I18n.tr("Atmos manages this line.") : I18n.tr("You wrote this line."))
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
            text: I18n.tr("Remove…")
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
    title: I18n.tr("Failures")
    query: root.failed.length ? root.query : "."
    detail: "User or system units that failed this boot. Startup commands that never launched often show up here."

    SettingRow {
      available: root.failed.length === 0
      label: "Startup failures"
      description: I18n.tr("No failed units reported.")
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
