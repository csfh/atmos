import QtQuick
import "../components"
import "../services"
import "../services/Workspaces.js" as WsJs

PrefsPage {
  id: root
  hubId: "workspaces"
  title: "Workspaces"
  description: "How many numbered workspaces stay around, what the bar calls them, and which monitor they live on."

  property string specialDraft: ""
  property string specialError: ""

  readonly property var numbered: {
    var list = Omarchy.workspaces || []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && !list[i].special) out.push(list[i])
    }
    return out
  }

  readonly property var specials: {
    var list = Omarchy.workspaces || []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].special) out.push(list[i])
    }
    return out
  }

  function currentState() {
    return WsJs.clampState({
      count: root.numbered.length || 10,
      wrapSwitch: Omarchy.workspaceWrapSwitch,
      wheelSwitch: Omarchy.workspaceWheelSwitch,
      items: Omarchy.workspaces
    })
  }

  function writeItems(items, wrap, wheel) {
    Omarchy.writeWorkspaces(items, wrap, wheel)
  }

  function setCount(n) {
    n = WsJs.clampCount(n)
    var state = root.currentState()
    state.count = n
    var next = WsJs.clampState(state)
    root.writeItems(next.items, next.wrapSwitch, next.wheelSwitch)
  }

  function patchItem(id, patch) {
    var state = root.currentState()
    var items = []
    for (var i = 0; i < state.items.length; i++) {
      var row = state.items[i]
      if (row.id === id) {
        var merged = {}
        var k
        for (k in row) merged[k] = row[k]
        for (k in patch) merged[k] = patch[k]
        items.push(merged)
      } else items.push(row)
    }
    root.writeItems(items, state.wrapSwitch, state.wheelSwitch)
  }

  function addSpecial() {
    var id = WsJs.sanitizeId("special:" + String(root.specialDraft || "").replace(/^\s+|\s+$/g, ""))
    if (!id || id.indexOf("special:") !== 0) {
      root.specialError = "Use letters, digits, underscore, or hyphen."
      return
    }
    var state = root.currentState()
    for (var i = 0; i < state.items.length; i++) {
      if (state.items[i].id === id) {
        root.specialError = "That scratch workspace already exists."
        return
      }
    }
    state.items.push({ id: id, name: "", persistent: true, special: true, monitor: "", isDefault: false, onCreatedEmpty: "" })
    root.specialError = ""
    root.specialDraft = ""
    root.writeItems(state.items, state.wrapSwitch, state.wheelSwitch)
  }

  PrefsGroup {
    title: "Count and switching"
    query: root.query
    detail: "Omarchy keeps numbered workspaces 1–10. Atmos writes a persistent rule for each one you keep, so Super+N still has somewhere to go when that workspace is empty."
    hint: "~/.config/hypr/atmos.lua"

    SettingRow {
      stretchControl: true
      label: "Number of workspaces"
      description: "How many numbered workspaces Hyprland keeps persistent."
      hint: "hl.workspace_rule"
      query: root.query
      keywords: ["count", "number", "ten"]

      PrefsSlider {
        width: parent.width
        from: 1
        to: 10
        stepSize: 1
        value: root.numbered.length || 10
        valueText: String(root.numbered.length || 10)
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== (root.numbered.length || 10)) root.setCount(next)
        }
      }
    }

    SettingRow {
      label: "Wrap switching"
      description: "The last workspace wraps to the first."
      hint: "workspace e+1"
      query: root.query
      keywords: ["wrap", "cycle", "switch"]

      PrefsToggle {
        checked: Omarchy.workspaceWrapSwitch
        onToggled: function(next) {
          Omarchy.writeWorkspaces(Omarchy.workspaces, next, Omarchy.workspaceWheelSwitch)
        }
      }
    }

    SettingRow {
      label: "Super + mouse wheel"
      description: "The mouse wheel moves between workspaces while Super is held. Atmos writes that as an override."
      hint: "SUPER + mouse_down"
      query: root.query
      keywords: ["wheel", "scroll", "switch"]

      PrefsToggle {
        checked: Omarchy.workspaceWheelSwitch
        onToggled: function(next) {
          Omarchy.writeWorkspaces(Omarchy.workspaces, Omarchy.workspaceWrapSwitch, next)
        }
      }
    }

    SettingRow {
      label: "Open on login"
      description: "Which numbered workspace Hyprland focuses after you log in."
      hint: "hl.workspace_rule · default"
      query: root.query
      keywords: ["default", "login", "start"]

      PrefsSelect {
        value: WsJs.defaultId(root.numbered)
        options: WsJs.defaultOptions(root.numbered)
        onChanged: function(value) {
          if (value === WsJs.defaultId(root.numbered)) return
          var state = root.currentState()
          root.writeItems(WsJs.withDefault(state.items, value), state.wrapSwitch, state.wheelSwitch)
        }
      }
    }
  }

  PrefsGroup {
    title: "Names and monitors"
    query: root.query
    detail: "A name is what the bar shows instead of the number. A monitor pins that workspace to one output. Leave either blank."

    Repeater {
      model: root.numbered

      SettingRow {
        required property var modelData
        label: modelData && modelData.id ? modelData.id : "Workspace"
        description: modelData && modelData.monitor
          ? ("Pinned to " + modelData.monitor + ".")
          : "Number in the bar unless you name it."
        hint: "~/.config/hypr/atmos.lua"
        query: root.query
        keywords: ["name", "monitor", "bar", "label"]

        Row {
          spacing: Theme.space
          PrefsField {
            id: workspaceNameField
            width: 120
            placeholder: "Bar name"
            value: modelData && modelData.name ? modelData.name : ""
            onSubmitted: function(value) { root.patchItem(modelData.id, { name: value }) }
          }
          PrefsButton {
            text: "Set"
            onClicked: root.patchItem(modelData.id, { name: workspaceNameField.currentText() })
          }
          PrefsSelect {
            width: 120
            value: modelData && modelData.monitor ? modelData.monitor : ""
            options: WsJs.monitorOptions(Omarchy.monitors, modelData && modelData.monitor)
            onChanged: function(value) {
              var cur = modelData && modelData.monitor ? modelData.monitor : ""
              if (value !== cur) root.patchItem(modelData.id, { monitor: value })
            }
          }
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Scratch workspaces"
    query: root.query
    detail: "A special workspace stays off the numbered strip until you toggle it."

    SettingRow {
      label: "Add a scratch workspace"
      description: root.specialError.length ? root.specialError : "Name it special:notes or similar."
      hint: "special:name"
      query: root.query
      keywords: ["scratch", "special", "toggle"]

      Row {
        spacing: Theme.space
        PrefsField {
          width: 160
          placeholder: "notes"
          onEdited: function(value) { root.specialDraft = value }
          onSubmitted: function() { root.addSpecial() }
        }
        PrefsButton {
          text: "Add"
          primary: true
          onClicked: root.addSpecial()
        }
      }
    }

    Repeater {
      model: root.specials

      SettingRow {
        required property var modelData
        label: modelData && modelData.id ? modelData.id : "special"
        description: "A scratch workspace. Remove drops the Atmos rule."
        query: root.query
        keywords: ["scratch", "special"]

        PrefsButton {
          text: "Remove"
          danger: true
          onClicked: {
            var state = root.currentState()
            var items = []
            for (var i = 0; i < state.items.length; i++) {
              if (state.items[i].id !== modelData.id) items.push(state.items[i])
            }
            root.writeItems(items, state.wrapSwitch, state.wheelSwitch)
          }
        }
      }
    }
  }
}
