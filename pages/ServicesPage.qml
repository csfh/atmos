import QtQuick
import "../components"
import "../services"
import "../services/Systemd.js" as SystemdJs

PrefsPage {
  id: root
  title: "Services"
  description: "Failed units always show. Start, stop, and enable only work for the allowlist Atmos considers safe."

  property string unitFilter: ""

  readonly property var rows: {
    var q = String(root.unitFilter || "").toLowerCase()
    var list = Omarchy.systemdUnits || []
    var out = []
    var i
    for (i = 0; i < list.length; i++) {
      var row = SystemdJs.normalizeUnit(list[i])
      if (!row) continue
      if (q.length && (row.unit + " " + row.description).toLowerCase().indexOf(q) === -1) continue
      out.push(row)
    }
    return out
  }

  readonly property var failedRows: {
    var list = root.rows
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i].active === "failed" || list[i].sub === "failed") out.push(list[i])
    }
    return out
  }

  function unitHint(row) {
    if (!row) return ""
    if (row.allowed) return row.scope === "user" ? "systemctl --user" : "systemctl"
    return "status and logs only"
  }

  PrefsGroup {
    title: "Failed"
    query: root.query
    detail: "Units that failed this boot. Atmos will not mask or edit unit files."

    SettingRow {
      available: root.failedRows.length === 0
      label: "Failed units"
      description: "Nothing failed."
      query: root.query
      keywords: ["failed", "empty"]
    }

    Repeater {
      model: root.failedRows

      SettingRow {
        required property var modelData
        label: modelData && modelData.unit ? modelData.unit : "unit"
        description: (modelData && modelData.description ? modelData.description + ". " : "") + (modelData && modelData.allowed ? "Atmos can restart this one." : "Logs only.")
        hint: root.unitHint(modelData)
        query: root.query
        keywords: ["failed", "systemd"]

        Row {
          spacing: Theme.space
          PrefsButton {
            text: "Restart"
            enabled: modelData && modelData.allowed
            onClicked: Omarchy.systemdAction("restart", modelData.unit, modelData.scope)
          }
          PrefsButton {
            text: "Logs"
            onClicked: Omarchy.runCommand(["journalctl", modelData.scope === "user" ? "--user" : "--system", "-u", modelData.unit, "-n", "40", "--no-pager"], { refresh: "none" })
          }
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Allowlist"
    query: root.query
    detail: "Search the units Atmos listed. Enable and start stay on the allowlist."

    SettingRow {
      stretchControl: true
      label: "Filter"
      description: root.rows.length + " units."
      query: root.query
      keywords: ["search", "filter"]

      PrefsField {
        width: parent.width
        placeholder: "pipewire or bluetooth"
        onEdited: function(value) { root.unitFilter = value }
      }
    }

    Repeater {
      model: root.rows

      SettingRow {
        required property var modelData
        available: !(modelData && (modelData.active === "failed" || modelData.sub === "failed"))
        label: modelData && modelData.unit ? modelData.unit : "unit"
        description: (modelData && modelData.active ? modelData.active : "") + (modelData && modelData.sub ? " / " + modelData.sub : "") + (modelData && modelData.allowed ? "." : ". Status and logs only.")
        hint: root.unitHint(modelData)
        query: root.query
        keywords: ["systemd", "enable", "start"]

        Row {
          spacing: Theme.space
          PrefsButton {
            text: "Start"
            enabled: modelData && modelData.allowed
            onClicked: Omarchy.systemdAction("start", modelData.unit, modelData.scope)
          }
          PrefsButton {
            text: "Stop"
            enabled: modelData && modelData.allowed
            onClicked: Omarchy.systemdAction("stop", modelData.unit, modelData.scope)
          }
          PrefsButton {
            text: "Enable"
            enabled: modelData && modelData.allowed
            onClicked: Omarchy.systemdAction("enable", modelData.unit, modelData.scope)
          }
          PrefsButton {
            text: "Disable"
            enabled: modelData && modelData.allowed
            onClicked: Omarchy.systemdAction("disable", modelData.unit, modelData.scope)
          }
        }
      }
    }
  }
}
