import QtQuick
import QtQuick.Dialogs
import "../components"
import "../services"
import "../services/Hooks.js" as HookJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "hooks"
  title: I18n.tr("Hooks")
  description: I18n.tr("Scripts Omarchy runs after a theme or font change, at boot, after an update, before a pacman refresh, and on low battery. Each event is a folder under ~/.config/omarchy/hooks.")

  property string pendingPath: ""
  property string pendingName: ""
  property string pendingType: ""
  property string installType: "theme-set"
  property string installMode: "file"
  property string installFile: ""
  property string nameDraft: ""
  property string commandDraft: ""
  property string addError: ""

  readonly property var hookTypes: HookJs.displayTypes(Omarchy.hooks)
  readonly property var typeOptions: HookJs.options()
  readonly property var modeOptions: [
    { value: "file", label: "Existing file" },
    { value: "command", label: "One-line command" }
  ]

  function itemsFor(type) {
    return HookJs.itemsFor(Omarchy.hooks, type)
  }

  function runArg(type) {
    var kind = HookJs.runArgFor(type)
    if (kind === "theme") return Omarchy.theme
    if (kind === "font") return Omarchy.font
    return ""
  }

  function runHint(type) {
    var arg = root.runArg(type)
    if (arg) return "Runs now with $1 set to " + arg + "."
    if (HookJs.argFor(type)) return "Runs now. $1 stays empty unless the real event supplied it."
    return "Runs every script for this event now."
  }

  function destText() {
    if (root.installMode === "command")
      return HookJs.destHint(root.installType, root.nameDraft) || "Name the script to see the install path."
    if (root.installFile)
      return "~/.config/omarchy/hooks/" + root.installType + ".d/" + RichUi.fileBasename(root.installFile)
    return "Pick a file to copy into the event folder."
  }

  function openAdd(type) {
    if (HookJs.isHookId(type)) root.installType = type
    root.installMode = "file"
    root.installFile = ""
    root.nameDraft = ""
    root.commandDraft = ""
    root.addError = ""
    nameField.setText("")
    commandField.setText("")
    addDialog.open()
  }

  function askRemove(item) {
    if (!item || item.sample) return
    root.pendingPath = item.path
    root.pendingName = item.name
    root.pendingType = item.type
    removeHookConfirm.ask()
  }

  function submitAdd() {
    if (!HookJs.isHookId(root.installType)) {
      root.addError = "Pick an event."
      return
    }
    if (root.installMode === "file") {
      if (!root.installFile) {
        root.addError = "Choose a script file to copy."
        return
      }
      root.addError = ""
      Omarchy.installHook(root.installType, root.installFile)
      addDialog.close()
      return
    }
    var name = HookJs.sanitizeName(nameField.currentText())
    var command = HookJs.sanitizeLine(commandField.currentText())
    if (!name) {
      root.addError = "Name the script. Letters, digits, dot, plus, underscore, hyphen."
      return
    }
    if (!command) {
      root.addError = "Enter one command line. $1 is available on theme, font, and battery events."
      return
    }
    root.addError = ""
    Omarchy.createHook(root.installType, name, command)
    addDialog.close()
  }

  function describeItem(item) {
    if (!item) return ""
    if (item.sample)
      return "Sample. Enable drops .sample so Omarchy runs it."
    if (item.flat)
      return "Runs first, before scripts in the .d folder."
    return item.path || ""
  }

  FileDialog {
    id: hookFileDialog
    title: I18n.tr("Install a hook")
    onAccepted: root.installFile = RichUi.pathFromUrl(selectedFile)
  }

  PrefsConfirm {
    id: removeHookConfirm
    title: I18n.tr("Remove this hook")
    message: I18n.tr("Delete {name} from the {type} hook folder? Sample files stay.", { name: root.pendingName, type: root.pendingType })
    confirmText: "Remove"
    onConfirmed: Omarchy.removeHook(root.pendingPath)
  }

  Component.onCompleted: {
    removeHookConfirm.parent = root.prefsOverlay
    addDialog.parent = root.prefsOverlay
  }

  PrefsGroup {
    title: I18n.tr("Install")
    query: root.query
    detail: "Add copies a file you already have, or writes a one-line script, into ~/.config/omarchy/hooks/<event>.d/. Enable a sample by dropping .sample from its name. Disable puts .sample back so the runner skips it."
    hint: "omarchy hook install"

    SettingRow {
      label: "Add a hook"
      description: I18n.tr("Pick the event, then copy a file or write a one-line command.")
      hint: "omarchy hook install"
      query: root.query
      keywords: ["hook", "install", "script", "add"]

      PrefsButton {
        text: I18n.tr("Add…")
        primary: true
        onClicked: root.openAdd(root.installType)
      }
    }
  }

  Repeater {
    model: root.hookTypes

    PrefsGroup {
    framed: true
      required property var modelData
      readonly property string hookId: modelData && modelData.id ? modelData.id : ""
      title: modelData && modelData.label ? modelData.label : hookId
      query: root.query
      detail: HookJs.eventBlurb(hookId)
      hint: "omarchy hook " + hookId

      SettingRow {
        stretchControl: true
        label: "This event"
        description: HookJs.eventBlurb(hookId) + " " + root.runHint(hookId)
        hint: "omarchy hook " + hookId
        query: root.query
        keywords: ["hook", hookId, modelData && modelData.label ? modelData.label : ""]

        Row {
          spacing: Theme.space
          PrefsButton {
            text: I18n.tr("Add…")
            onClicked: root.openAdd(hookId)
          }
          PrefsButton {
            text: I18n.tr("Run now")
            onClicked: Omarchy.runHook(hookId, root.runArg(hookId))
          }
          PrefsButton {
            text: I18n.tr("Open folder")
            onClicked: Omarchy.openHookFolder(hookId)
          }
        }
      }

      SettingRow {
        available: root.itemsFor(hookId).length === 0
        sectionHelp: false
        label: "Scripts"
        description: I18n.tr("No scripts.")
        query: root.query
        keywords: ["hook", "empty", hookId]
      }

      Repeater {
        model: root.itemsFor(hookId)

        CollectionRow {
          required property var modelData
          stretchControl: false
          label: modelData && modelData.name ? modelData.name : "hook"
          description: root.describeItem(modelData)
          hint: modelData && modelData.path ? modelData.path : ""
          query: root.query
          keywords: ["hook", "script", modelData && modelData.type ? modelData.type : ""]
          action: modelData && modelData.sample ? "Enable" : (modelData && modelData.flat ? "Edit" : "Disable")
          actionPrimary: !!(modelData && modelData.sample)
          actionEnabled: !!(modelData && modelData.path)
          extraAction: modelData && !modelData.sample && !modelData.flat ? "Edit" : ""
          extraEnabled: !!(modelData && modelData.path)
          dangerAction: modelData && !modelData.sample ? "Remove…" : ""
          dangerEnabled: !!(modelData && modelData.path)
          onActioned: {
            if (modelData && modelData.sample) Omarchy.setHookSample(modelData.path, true)
            else if (modelData && modelData.flat) Omarchy.editHook(modelData.path)
            else Omarchy.setHookSample(modelData.path, false)
          }
          onExtraActioned: Omarchy.editHook(modelData.path)
          onDangered: root.askRemove(modelData)
        }
      }
    }
  }

  PrefsDialog {
    id: addDialog
    title: I18n.tr("Add a hook")

    PrefsText {
      width: parent.width
      text: HookJs.eventBlurb(root.installType)
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsSelect {
      width: parent.width
      value: root.installType
      options: root.typeOptions
      onChanged: function(value) { root.installType = value }
    }

    PrefsRadioGroup {
      width: parent.width
      wrap: true
      value: root.installMode
      options: root.modeOptions
      onChanged: function(value) { root.installMode = value }
    }

    PrefsButton {
      visible: root.installMode === "file"
      text: root.installFile ? I18n.tr("File: {name}", { name: RichUi.fileBasename(root.installFile) }) : I18n.tr("Choose…")
      onClicked: hookFileDialog.open()
    }

    PrefsField {
      id: nameField
      width: parent.width
      visible: root.installMode === "command"
      placeholder: "notify.sh"
      onEdited: function(value) { root.nameDraft = value }
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsField {
      id: commandField
      width: parent.width
      visible: root.installMode === "command"
      placeholder: 'omarchy-notification-send -u low "Theme" "$1"'
      onEdited: function(value) { root.commandDraft = value }
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsText {
      width: parent.width
      text: root.destText()
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
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
        text: I18n.tr("Install")
        primary: true
        enabled: (
          (root.installMode === "file" && root.installFile.length > 0) ||
          (root.installMode === "command" && root.nameDraft.length > 0 && root.commandDraft.length > 0)
        )
        onClicked: root.submitAdd()
      }
    }
  }
}
