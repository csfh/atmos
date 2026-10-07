import QtQuick
import "../../components"
import "../../services"
import "../../services/Bindings.js" as BindJs
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "keybindings"
  title: I18n.tr("Keybindings")
  description: I18n.tr("The list is what Hyprland is running now. Add writes a managed block at the end of ~/.config/hypr/bindings.lua. Lines you typed yourself stay. Remove only deletes a row Atmos added.")

  property string keysDraft: ""
  property string labelDraft: ""
  property string commandDraft: ""
  property bool unbindOnly: false
  property string addError: ""
  property string catalogFilter: ""
  // The catalog holds ~250 rows and every row is a full SettingRow. Render
  // a capped window so switching to this hub does not instantiate them all
  // at once; the count line still reports the full filtered total.
  property int catalogLimit: 100
  property string pendingKeys: ""
  property string pendingLabel: ""
  property bool recordingKeys: false

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
      var cat = BindJs.categoryFromAction(row.action)
      row = { keys: row.keys, action: row.action, category: cat }
      if (q.length === 0) {
        out.push(row)
        continue
      }
      var hay = (String(row.keys) + " " + String(row.action || "") + " " + cat).toLowerCase()
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
    root.recordingKeys = true
    addDialog.open()
  }

  function captureKey(event) {
    if (!root.recordingKeys || !event) return
    var chord = BindJs.recordKeyEvent({
      key: event.key,
      text: event.text,
      modifiers: event.modifiers
    })
    if (!chord) {
      event.accepted = true
      return
    }
    event.accepted = true
    root.keysDraft = chord
    keysField.setText(chord)
    root.recordingKeys = false
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
    framed: true
    title: I18n.tr("Your overrides")
    query: root.query
    detail: "These lines live in the Atmos block of bindings.lua. Adding a chord that is already taken writes hl.unbind first, then o.bind."
    hint: "~/.config/hypr/bindings.lua"

    SettingRow {
      label: "Add a binding"
      description: I18n.tr("A chord, a short name, and the command to run. Unbind only turns a default off.")
      hint: "~/.config/hypr/bindings.lua"
      query: root.query
      keywords: ["bind", "unbind", "hotkey", "shortcut", "chord"]

      PrefsButton {
        text: I18n.tr("Add…")
        primary: true
        onClicked: root.openAdd()
      }
    }

    SettingRow {
      available: root.overrideRows.length === 0
      sectionHelp: false
      label: "Overrides"
      description: I18n.tr("No personal bindings.")
      query: root.query
      keywords: ["empty", "bindings"]
    }

    Repeater {
      model: root.overrideRows

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.keys ? modelData.keys : "chord"
        description: root.describeOverride(modelData)
        hint: "~/.config/hypr/bindings.lua"
        query: root.query
        keywords: ["bind", "unbind", "override"]

        PrefsButton {
          text: I18n.tr("Remove…")
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
    framed: true
    title: I18n.tr("What is bound")
    query: root.query
    detail: "This is omarchy menu keybindings --print. Filter if you want to find a chord before you override it."
    hint: "omarchy menu keybindings --print"

    SettingRow {
      available: Omarchy.keybindings.length === 0
      sectionHelp: false
      label: "Bindings"
      description: I18n.tr("No bindings reported.")
      hint: "omarchy menu keybindings --print"
      query: root.query
      keywords: ["empty", "keybinding"]
    }

    SettingRow {
      available: Omarchy.keybindings.length > 0
      stretchControl: true
      label: "Filter"
      description: I18n.tr("{shown} of {total} bindings.", { shown: root.catalogRows.length, total: Omarchy.keybindings.length })
      hint: "omarchy menu keybindings --print"
      query: root.query
      keywords: ["search", "filter", "list"]

      PrefsField {
        width: parent.width
        placeholder: "SUPER + Q or Close window"
        onEdited: function(value) {
          root.catalogFilter = value
          root.catalogLimit = 100
        }
      }
    }

    SettingRow {
      available: Omarchy.keybindings.length > 0 && root.catalogRows.length === 0
      sectionHelp: false
      label: "Bindings"
      description: I18n.tr("No matching bindings.")
      query: root.query
      keywords: ["empty", "filter"]
    }

    Repeater {
      model: root.catalogRows.slice(0, root.catalogLimit)

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.keys ? modelData.keys : "chord"
        description: (modelData && modelData.category ? modelData.category + " · " : "") + (modelData && modelData.action ? modelData.action : "")
        hint: "omarchy menu keybindings --print"
        query: root.query
        keywords: ["keybinding", "hotkey", "shortcut"]

        Row {
          spacing: Theme.space
          PrefsButton {
            text: I18n.tr("Copy")
            enabled: !!(modelData && modelData.keys)
            onClicked: Omarchy.copyText(RichUi.bindingCopyText(modelData))
          }
          PrefsButton {
            text: I18n.tr("Override…")
            enabled: !!(modelData && modelData.keys)
            onClicked: root.openAdd(modelData.keys)
          }
        }
      }
    }

    SettingRow {
      available: root.catalogRows.length > root.catalogLimit
      sectionHelp: false
      label: "More bindings"
      description: I18n.tr("Showing {shown} of {total} bindings. Refine the filter, or show them all.", { shown: Math.min(root.catalogLimit, root.catalogRows.length), total: root.catalogRows.length })
      hint: "omarchy menu keybindings --print"
      query: root.query
      keywords: ["more", "show", "all", "list"]

      PrefsButton {
        text: I18n.tr("Show all")
        onClicked: root.catalogLimit = root.catalogRows.length
      }
    }
  }

  PrefsDialog {
    id: addDialog
    title: I18n.tr("Add a binding")

    Item {
      id: keyGrab
      width: 1
      height: 1
      focus: root.recordingKeys
      Keys.onPressed: function(event) { root.captureKey(event) }
    }

    PrefsText {
      width: parent.width
      text: root.conflictText().length
        ? (root.keysDraft + " already runs “" + root.conflictText() + "”. Add will unbind that first.")
        : (root.recordingKeys
          ? "Press the shortcut now. Super, Ctrl, Alt, and Shift count as modifiers."
          : "Use the same chord form as bindings.lua, or press Record shortcut.")
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: keysField
      width: parent.width
      placeholder: "SUPER + F"
      enabled: !root.recordingKeys
      onEdited: function(value) { root.keysDraft = value }
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsButton {
      text: root.recordingKeys ? "Listening…" : "Record shortcut"
      primary: root.recordingKeys
      onClicked: {
        root.recordingKeys = true
        keyGrab.forceActiveFocus()
      }
    }

    PrefsText {
      width: parent.width
      text: BindJs.generatedBindText({
        keys: BindJs.sanitizeKeys(root.keysDraft),
        label: root.labelDraft,
        command: root.commandDraft,
        unbind: root.unbindOnly || !!root.conflictText()
      }) || "The Hyprland line appears here once the chord is valid."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
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

    SettingRow {
      sectionHelp: false
      label: "Unbind only"
      description: I18n.tr("The default chord is removed, with no replacement.")
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
        text: I18n.tr("Cancel")
        onClicked: addDialog.close()
      }

      PrefsButton {
        text: I18n.tr("Add")
        primary: true
        enabled: root.keysDraft.length > 0 && (root.unbindOnly || root.commandDraft.length > 0)
        onClicked: root.submitAdd()
      }
    }
  }

  PrefsConfirm {
    id: removeConfirm
    title: I18n.tr("Remove binding")
    message: I18n.tr("Remove the Atmos override for {keys}?", { keys: root.pendingKeys })
    confirmText: "Remove"
    onConfirmed: Omarchy.removeBinding(root.pendingKeys)
  }
}
