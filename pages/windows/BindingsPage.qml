import QtQuick
import "../../components"
import "../../services"
import "../../services/Bindings.js" as BindJs
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Keybindings"
  description: "The list is what Hyprland is running now. Add writes a managed block at the end of ~/.config/hypr/bindings.lua. Lines you typed yourself stay. Remove only deletes a row Atmos added."

  property string keysDraft: ""
  property string labelDraft: ""
  property string commandDraft: ""
  property bool unbindOnly: false
  property string addError: ""
  property string catalogFilter: ""
  property string pendingKeys: ""
  property string pendingLabel: ""

  readonly property var overrideRows: {
    var list = Omarchy.bindings || []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].keys) out.push(list[i])
    }
    return out
  }

  readonly property var catalogRows: {
    var q = String(root.catalogFilter || "").toLowerCase()
    var list = Omarchy.keybindings || []
    var out = []
    for (var i = 0; i < list.length; i++) {
      var row = list[i]
      if (!row || !row.keys) continue
      if (q.length === 0) {
        out.push(row)
        continue
      }
      var hay = (String(row.keys) + " " + String(row.action || "")).toLowerCase()
      if (hay.indexOf(q) !== -1) out.push(row)
    }
    return out
  }

  function conflictText() {
    return BindJs.catalogConflict(Omarchy.keybindings, root.keysDraft)
  }

  function openAdd(keys) {
    var chord = BindJs.sanitizeKeys(keys || "")
    root.keysDraft = chord
    root.labelDraft = ""
    root.commandDraft = ""
    root.unbindOnly = false
    root.addError = ""
    keysField.setText(chord)
    labelField.setText("")
    commandField.setText("")
    addDialog.open()
  }

  function submitAdd() {
    var keys = BindJs.sanitizeKeys(keysField.currentText())
    if (!keys) {
      root.addError = "Enter a chord such as SUPER + F."
      return
    }
    var label = BindJs.sanitizeLabel(labelField.currentText())
    var command = BindJs.sanitizeCommand(commandField.currentText())
    var unbind = root.unbindOnly === true
    if (!unbind && !command) {
      root.addError = "Enter a command, or turn on Unbind only."
      return
    }
    if (command && !unbind && BindJs.catalogConflict(Omarchy.keybindings, keys))
      unbind = true
    root.addError = ""
    Omarchy.addBinding(keys, label, command, unbind)
    addDialog.close()
  }

  function describeOverride(row) {
    if (!row) return ""
    if (row.command)
      return (row.label ? row.label + ". " : "") + row.command
    return "Unbinds the default for this chord."
  }

  Component.onCompleted: {
    addDialog.parent = root.prefsOverlay
    removeConfirm.parent = root.prefsOverlay
  }

  PrefsGroup {
    title: "Your overrides"
    query: root.query
    detail: "These lines live in the Atmos block of bindings.lua. Adding a chord that is already taken writes hl.unbind first, then o.bind."
    hint: "~/.config/hypr/bindings.lua"

    PrefsRow {
      label: "Add a binding"
      description: "A chord, a short name, and the command to run. Unbind only turns a default off."
      hint: "~/.config/hypr/bindings.lua"
      query: root.query
      keywords: ["bind", "unbind", "hotkey", "shortcut", "chord"]

      PrefsButton {
        text: "Add…"
        primary: true
        onClicked: root.openAdd()
      }
    }

    PrefsRow {
      available: root.overrideRows.length === 0
      sectionHelp: false
      label: "Overrides"
      description: "No personal o.bind or hl.unbind lines in bindings.lua."
      query: root.query
      keywords: ["empty", "bindings"]

      PrefsButton {
        text: "Add…"
        primary: true
        onClicked: root.openAdd()
      }
    }

    Repeater {
      model: root.overrideRows

      PrefsRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.keys ? modelData.keys : "chord"
        description: root.describeOverride(modelData)
        hint: "~/.config/hypr/bindings.lua"
        query: root.query
        keywords: ["bind", "unbind", "override"]

        PrefsButton {
          text: "Remove"
          danger: true
          enabled: modelData && modelData.managed
          onClicked: {
            root.pendingKeys = modelData.keys
            root.pendingLabel = modelData.label || modelData.keys
            removeConfirm.ask()
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "What is bound"
    query: root.query
    detail: "This is omarchy menu keybindings --print. Filter if you want to find a chord before you override it."
    hint: "omarchy menu keybindings --print"

    PrefsRow {
      stretchControl: true
      label: "Filter"
      description: Omarchy.keybindings.length
        ? (root.catalogRows.length + " of " + Omarchy.keybindings.length + " bindings.")
        : "Hyprland did not report any bindings."
      hint: "omarchy menu keybindings --print"
      query: root.query
      keywords: ["search", "filter", "list"]

      PrefsField {
        width: parent.width
        placeholder: "SUPER + Q or Close window"
        enabled: Omarchy.keybindings.length > 0
        onEdited: function(value) { root.catalogFilter = value }
      }
    }

    Repeater {
      model: root.catalogRows

      PrefsRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.keys ? modelData.keys : "chord"
        description: modelData && modelData.action ? modelData.action : ""
        hint: "omarchy menu keybindings --print"
        query: root.query
        keywords: ["keybinding", "hotkey", "shortcut"]

        Row {
          spacing: 8
          PrefsButton {
            text: "Copy"
            enabled: !!(modelData && modelData.keys)
            onClicked: Omarchy.copyText(RichUi.bindingCopyText(modelData))
          }
          PrefsButton {
            text: "Override…"
            enabled: !!(modelData && modelData.keys)
            onClicked: root.openAdd(modelData.keys)
          }
        }
      }
    }
  }

  PrefsDialog {
    id: addDialog
    title: "Add a binding"

    PrefsText {
      width: parent.width
      text: root.conflictText().length
        ? (root.keysDraft + " already runs “" + root.conflictText() + "”. Add will unbind that first.")
        : "Use the same chord form as bindings.lua, for example SUPER + SHIFT + R."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: keysField
      width: parent.width
      placeholder: "SUPER + F"
      onEdited: function(value) { root.keysDraft = value }
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsField {
      id: labelField
      width: parent.width
      placeholder: "Name (optional)"
      onEdited: function(value) { root.labelDraft = value }
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsField {
      id: commandField
      width: parent.width
      visible: !root.unbindOnly
      placeholder: "nautilus"
      onEdited: function(value) { root.commandDraft = value }
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsRow {
      sectionHelp: false
      label: "Unbind only"
      description: "Turn the default off without adding a replacement."
      query: ""

      PrefsToggle {
        checked: root.unbindOnly
        onToggled: root.unbindOnly = !root.unbindOnly
      }
    }

    PrefsText {
      width: parent.width
      visible: root.addError.length > 0
      text: root.addError
      color: Theme.urgent
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Row {
      anchors.right: parent.right
      spacing: Theme.space

      PrefsButton {
        text: "Cancel"
        onClicked: addDialog.close()
      }

      PrefsButton {
        text: "Add"
        primary: true
        enabled: root.keysDraft.length > 0 && (root.unbindOnly || root.commandDraft.length > 0)
        onClicked: root.submitAdd()
      }
    }
  }

  PrefsConfirm {
    id: removeConfirm
    title: "Remove binding"
    message: "Remove the Atmos override for " + root.pendingKeys + "?"
    confirmText: "Remove"
    onConfirmed: Omarchy.removeBinding(root.pendingKeys)
  }
}
