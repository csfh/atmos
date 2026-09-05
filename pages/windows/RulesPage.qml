import QtQuick
import "../../components"
import "../../services"
import "../../services/WindowRules.js" as RuleJs

PrefsPage {
  id: root
  title: "Window rules"
  description: "These write a managed block at the end of ~/.config/hypr/atmos.lua. The Atmos window rules stay above that block. Lines in hyprland.lua stay there too."

  property string matchDraft: ""
  property string placementDraft: "float"
  property bool centerDraft: true
  property int widthDraft: 0
  property int heightDraft: 0
  property string workspaceDraft: ""
  property string addError: ""
  property string pendingMatch: ""

  readonly property var placementOptions: [
    { value: "", label: "Leave" },
    { value: "float", label: "Float" },
    { value: "tile", label: "Tile" }
  ]

  readonly property var ruleRows: {
    var list = Omarchy.windowRules || []
    var out = []
    for (var i = 0; i < list.length; i++) {
      var row = list[i]
      if (!row || !row.match) continue
      if (row.managed !== true && row.match === "dev.csfh.atmos") continue
      out.push(row)
    }
    return out
  }

  function openAdd() {
    root.matchDraft = ""
    root.placementDraft = "float"
    root.centerDraft = true
    root.widthDraft = 0
    root.heightDraft = 0
    root.workspaceDraft = ""
    root.addError = ""
    matchField.setText("")
    workspaceField.setText("")
    addDialog.open()
  }

  function useFocused() {
    if (!Omarchy.focusedClass) return
    root.matchDraft = Omarchy.focusedClass
    matchField.setText(Omarchy.focusedClass)
  }

  function submitAdd() {
    var match = RuleJs.sanitizeMatch(matchField.currentText())
    if (!match) {
      root.addError = "Enter a window class, or a class regex."
      return
    }
    var workspace = RuleJs.sanitizeWorkspace(workspaceField.currentText())
    if (workspaceField.currentText().replace(/^\s+|\s+$/g, "").length > 0 && !workspace) {
      root.addError = "Workspace can use letters, digits, colon, underscore, or hyphen."
      return
    }
    var width = root.widthDraft
    var height = root.heightDraft
    if ((width > 0 || height > 0) && !(width >= 100 && height >= 100)) {
      root.addError = "Size needs both width and height, 100 to 4000."
      return
    }
    if (!root.placementDraft && !root.centerDraft && !(width && height) && !workspace) {
      root.addError = "Pick float or tile, turn on center, set a size, or set a workspace."
      return
    }
    root.addError = ""
    Omarchy.addWindowRule(match, root.placementDraft, root.centerDraft, width, height, workspace)
    addDialog.close()
  }

  Component.onCompleted: {
    addDialog.parent = root.prefsOverlay
    removeConfirm.parent = root.prefsOverlay
  }

  PrefsGroup {
    framed: true
    title: "Your rules"
    query: root.query
    detail: "Match is a window class, or a regex Hyprland applies to class. Use focused copies the class of the window that is focused right now."
    hint: "~/.config/hypr/atmos.lua"

    SettingRow {
      label: "Add a rule"
      description: "Float, tile, center, size, or pin a class to a workspace."
      hint: "~/.config/hypr/atmos.lua"
      query: root.query
      keywords: ["window", "rule", "float", "tile", "class"]

      PrefsButton {
        text: "Add…"
        primary: true
        onClicked: root.openAdd()
      }
    }

    SettingRow {
      available: root.ruleRows.length === 0
      sectionHelp: false
      label: "Rules"
      description: "No personal window rules."
      query: root.query
      keywords: ["empty", "rules"]
    }

    Repeater {
      model: root.ruleRows

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.match ? modelData.match : "class"
        description: (RuleJs.describe(modelData) || "A window rule.") + (modelData && modelData.managed
          ? ""
          : " Remove stays off — this line is outside the Atmos block.")
        hint: "~/.config/hypr/atmos.lua"
        query: root.query
        keywords: ["window", "rule", "float"]

        PrefsButton {
          text: "Remove…"
          danger: true
          enabled: modelData && modelData.managed
          onClicked: {
            root.pendingMatch = modelData.match
            removeConfirm.ask()
          }
        }
      }
    }
  }

  PrefsDialog {
    id: addDialog
    title: "Add a window rule"

    PrefsText {
      width: parent.width
      text: "Match is the window class Hyprland sees. Regex is fine. Leave size at 0 to skip it."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Row {
      width: parent.width
      spacing: Theme.space

      PrefsField {
        id: matchField
        width: parent.width - focusedBtn.width - parent.spacing
        placeholder: "firefox"
        onEdited: function(value) { root.matchDraft = value }
        onSubmitted: function() { root.submitAdd() }
      }

      PrefsButton {
        id: focusedBtn
        text: "Use focused"
        enabled: Omarchy.focusedClass.length > 0
        onClicked: root.useFocused()
      }
    }

    PrefsRadioGroup {
      width: parent.width
      wrap: true
      value: root.placementDraft
      options: root.placementOptions
      onChanged: function(value) { root.placementDraft = value }
    }

    SettingRow {
      sectionHelp: false
      label: "Center"
      description: "A floating window sits in the middle of the screen."
      query: ""

      PrefsToggle {
        checked: root.centerDraft
        onToggled: root.centerDraft = !root.centerDraft
      }
    }

    Row {
      width: parent.width
      spacing: Theme.space

      PrefsSpinBox {
        from: 0
        to: 4000
        stepSize: 10
        value: root.widthDraft
        onChanged: function(value) { root.widthDraft = value }
      }

      PrefsSpinBox {
        from: 0
        to: 4000
        stepSize: 10
        value: root.heightDraft
        onChanged: function(value) { root.heightDraft = value }
      }
    }

    PrefsField {
      id: workspaceField
      width: parent.width
      placeholder: "Workspace (optional)"
      onEdited: function(value) { root.workspaceDraft = value }
      onSubmitted: function() { root.submitAdd() }
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
        enabled: root.matchDraft.length > 0
        onClicked: root.submitAdd()
      }
    }
  }

  PrefsConfirm {
    id: removeConfirm
    title: "Remove window rule"
    message: "Remove the Atmos rule for " + root.pendingMatch + "?"
    confirmText: "Remove"
    onConfirmed: Omarchy.removeWindowRule(root.pendingMatch)
  }
}
