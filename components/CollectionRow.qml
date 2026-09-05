import QtQuick
import "../services"

SettingRow {
  id: root

  sectionHelp: false
  activeFocusOnTab: true

  // Always-visible action (Install, Select, Enable). Empty means none.
  property string action: ""
  property bool actionPrimary: false
  property bool actionDanger: false
  property bool actionEnabled: true
  property string extraAction: ""
  property bool extraEnabled: true
  // Destructive action. Always visible when set. Hiding it until hover made
  // Remove appear out of nowhere next to Edit.
  property string dangerAction: ""
  property bool dangerEnabled: true

  signal actioned()
  signal extraActioned()
  signal dangered()

  Accessible.role: Accessible.ListItem
  Accessible.name: root.label
  Accessible.description: root.description
  Accessible.onPressAction: root.activate()

  Keys.onReturnPressed: root.activate()
  Keys.onSpacePressed: root.activate()

  function activate() {
    if (root.action.length > 0 && root.actionEnabled) {
      root.actioned()
      return
    }
    if (root.dangerAction.length > 0 && root.dangerEnabled) root.dangered()
  }

  Row {
    spacing: Theme.space
    visible: root.action.length > 0 || root.extraAction.length > 0 || root.dangerAction.length > 0

    PrefsButton {
      visible: root.action.length > 0
      text: root.action
      primary: root.actionPrimary
      danger: root.actionDanger
      enabled: root.actionEnabled
      activeFocusOnTab: visible && enabled
      onClicked: root.actioned()
    }

    PrefsButton {
      visible: root.extraAction.length > 0
      text: root.extraAction
      enabled: root.extraEnabled
      activeFocusOnTab: visible && enabled
      onClicked: root.extraActioned()
    }

    PrefsButton {
      visible: root.dangerAction.length > 0
      text: root.dangerAction
      danger: true
      enabled: root.dangerEnabled
      activeFocusOnTab: visible && enabled
      onClicked: root.dangered()
    }
  }
}
