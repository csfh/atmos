import QtQuick
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Applications"
  description: "Launchers you added yourself under ~/.local/share/applications. Remove deletes that desktop file. Packages from the repos stay on the system."

  property string pendingKind: ""
  property string pendingId: ""
  property string pendingName: ""
  property string addKind: ""
  property string addError: ""
  property string addWindowStyle: "tile"
  property string autostartDraft: ""
  property string pendingAutostart: ""

  readonly property var windowStyleOptions: [
    { value: "tile", label: "Tiled" },
    { value: "float", label: "Floating" }
  ]

  function askRemove(kind, id, name) {
    root.pendingKind = kind
    root.pendingId = id
    root.pendingName = name
    removeAppConfirm.ask()
  }

  function kindLabel(kind) {
    if (kind === "web") return "web app"
    if (kind === "tui") return "terminal app"
    return "desktop app"
  }

  function addTitle() {
    if (root.addKind === "web") return "Add a web app"
    if (root.addKind === "tui") return "Add a terminal app"
    return "Add a desktop app"
  }

  function addBlurb() {
    if (Omarchy.jobBusy && (Omarchy.jobKind === "desktop-install" || Omarchy.jobKind === "tui-install" || Omarchy.jobKind === "webapp-install"))
      return "Creating the launcher…"
    if (root.addKind === "web")
      return "A site in its own window. Leave the icon blank to use the site's favicon."
    if (root.addKind === "tui")
      return "Opens in your default terminal. Icon can be a name, a PNG path, or a URL."
    return "A launcher for a command on this machine. Icon can be a name, a PNG path, or a URL."
  }

  function openAdd(kind) {
    root.addKind = kind
    root.addError = ""
    root.addWindowStyle = "tile"
    addNameField.clear()
    addCommandField.clear()
    addUrlField.clear()
    addIconField.setText(kind === "tui" ? "utilities-terminal" : "")
    addDialog.open()
  }

  function submitAdd() {
    var name = RichUi.parseLauncherName(addNameField.currentText())
    if (!name) {
      root.addError = "Name cannot be empty or contain a slash."
      return
    }
    if (root.addKind === "web") {
      var url = RichUi.parseWebAppUrl(addUrlField.currentText())
      if (!url) {
        root.addError = "Enter an http or https URL."
        return
      }
      root.addError = ""
      Omarchy.installWebApp(name, url, addIconField.currentText())
      addDialog.close()
      return
    }
    var command = String(addCommandField.currentText() || "").replace(/^\s+|\s+$/g, "")
    if (!command) {
      root.addError = "Enter the command to run."
      return
    }
    if (root.addKind === "tui") {
      var icon = String(addIconField.currentText() || "").replace(/^\s+|\s+$/g, "")
      if (!icon) icon = "utilities-terminal"
      if (!RichUi.isTuiWindowStyle(root.addWindowStyle)) {
        root.addError = "Pick tiled or floating."
        return
      }
      root.addError = ""
      Omarchy.installTui(name, command, root.addWindowStyle, icon)
      addDialog.close()
      return
    }
    var desktopIcon = String(addIconField.currentText() || "").replace(/^\s+|\s+$/g, "")
    if (!desktopIcon) desktopIcon = "application-x-executable"
    root.addError = ""
    Omarchy.installDesktopApp(name, command, desktopIcon)
    addDialog.close()
  }

  Component.onCompleted: {
    removeAppConfirm.parent = root.prefsOverlay
    addDialog.parent = root.prefsOverlay
    removeAutostartConfirm.parent = root.prefsOverlay
  }

  PrefsConfirm {
    id: removeAppConfirm
    title: "Remove " + root.kindLabel(root.pendingKind)
    message: "Remove " + root.pendingName + " from the launcher list? The desktop file under ~/.local/share/applications goes with it."
    confirmText: "Remove"
    onConfirmed: {
      if (root.pendingKind === "web") Omarchy.removeWebApp(root.pendingId)
      else if (root.pendingKind === "tui") Omarchy.removeTui(root.pendingId)
      else Omarchy.removeDesktopApp(root.pendingId, root.pendingName)
    }
  }

  PrefsDialog {
    id: addDialog
    title: root.addTitle()

    PrefsText {
      width: parent.width
      text: root.addBlurb()
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: addNameField
      width: parent.width
      placeholder: "Name"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsField {
      id: addCommandField
      width: parent.width
      visible: root.addKind !== "web"
      placeholder: root.addKind === "tui" ? "lazydocker" : "Command"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsField {
      id: addUrlField
      width: parent.width
      visible: root.addKind === "web"
      placeholder: "https://example.com"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAdd() }
    }

    PrefsSelect {
      visible: root.addKind === "tui"
      width: parent.width
      value: root.addWindowStyle
      options: root.windowStyleOptions
      enabled: !Omarchy.jobBusy
      onChanged: function(value) { root.addWindowStyle = value }
    }

    PrefsField {
      id: addIconField
      width: parent.width
      placeholder: root.addKind === "web" ? "Icon URL or name (optional)" : "Icon URL or name"
      enabled: !Omarchy.jobBusy
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
        enabled: !Omarchy.jobBusy
        onClicked: root.submitAdd()
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Advanced"
    query: root.query
    detail: "Autostart writes a managed block at the end of ~/.config/hypr/autostart.lua. Lines you typed yourself stay. Remove only deletes a line Atmos added."
    hint: "~/.config/hypr/autostart.lua"

    SettingRow {
      label: "Add a command"
      description: "A program name or command Omarchy should launch on start. Same form as o.launch_on_start."
      hint: "~/.config/hypr/autostart.lua"
      query: root.query
      keywords: ["autostart", "startup", "launch", "hypr"]

      Row {
        spacing: Theme.space
        PrefsField {
          width: 180
          placeholder: "hyprsunset"
          onEdited: function(value) { root.autostartDraft = value }
          onSubmitted: function(value) {
            root.autostartDraft = value
            Omarchy.addAutostart(value)
          }
        }
        PrefsButton {
          text: "Add"
          primary: true
          enabled: root.autostartDraft.length > 0
          onClicked: Omarchy.addAutostart(root.autostartDraft)
        }
      }
    }

    SettingRow {
      available: Omarchy.autostart.length === 0
      sectionHelp: false
      label: "Launch on start"
      description: "No launch-on-start commands."
      query: root.query
      keywords: ["autostart", "empty"]
    }

    Repeater {
      model: Omarchy.autostart

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.command ? modelData.command : "command"
        description: modelData && modelData.managed
          ? "Atmos wrote this line."
          : "This line is outside the Atmos block, so Remove stays off."
        hint: "~/.config/hypr/autostart.lua"
        query: root.query
        keywords: ["autostart", "startup"]

        PrefsButton {
          visible: !!(modelData && modelData.managed)
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

  PrefsGroup {
    title: "Add"
    query: root.query
    detail: "Desktop writes a .desktop file for a command. Terminal uses omarchy tui install. Web uses omarchy webapp install and can fetch the site icon."
    hint: "omarchy tui install · omarchy webapp install"

    SettingRow {
      label: "Desktop"
      description: "A launcher for a command on this machine."
      hint: "add-desktop-launcher.sh"
      query: root.query
      keywords: ["add", "install", "create", "desktop", "launcher"]

      PrefsButton {
        text: "Add…"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.openAdd("desktop")
      }
    }

    SettingRow {
      label: "Terminal"
      description: "A TUI that opens in your default terminal."
      hint: "omarchy tui install"
      query: root.query
      keywords: ["add", "install", "create", "tui", "terminal"]

      PrefsButton {
        text: "Add…"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.openAdd("tui")
      }
    }

    SettingRow {
      label: "Web"
      description: "A site in its own window."
      hint: "omarchy webapp install"
      query: root.query
      keywords: ["add", "install", "create", "webapp", "web"]

      PrefsButton {
        text: "Add…"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.openAdd("web")
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Desktop"
    query: root.query
    detail: "Regular desktop launchers in ~/.local/share/applications. Web apps and terminal UIs have their own sections below. Remove deletes that .desktop file."
    hint: "omarchy remove launcher entry"

    SettingRow {
      available: Omarchy.desktopApps.length === 0
      sectionHelp: false
      label: "Desktop launchers"
      description: "No desktop launchers."
      query: root.query
      keywords: ["empty", "desktop"]
    }

    Repeater {
      model: Omarchy.desktopApps

      CollectionRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "App"
        description: modelData && modelData.detail ? modelData.detail : "Desktop launcher you added."
        hint: "omarchy remove launcher entry"
        query: root.query
        keywords: ["desktop", "app", "launcher", "uninstall"]
        dangerAction: "Remove…"
        dangerEnabled: !!(modelData && modelData.id)
        onDangered: root.askRemove("desktop", modelData.id, modelData.name)
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Terminal"
    query: root.query
    detail: "Terminal UIs you installed with omarchy tui install. They open in your default terminal. Remove deletes the launcher."
    hint: "omarchy tui remove"

    SettingRow {
      available: Omarchy.tuiApps.length === 0
      sectionHelp: false
      label: "Terminal launchers"
      description: "No terminal launchers."
      query: root.query
      keywords: ["empty", "tui"]
    }

    Repeater {
      model: Omarchy.tuiApps

      CollectionRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "TUI"
        description: modelData && modelData.detail ? modelData.detail : "Opens in your default terminal."
        hint: "omarchy tui remove"
        query: root.query
        keywords: ["tui", "terminal", "console"]
        dangerAction: "Remove…"
        dangerEnabled: !!(modelData && modelData.name)
        onDangered: root.askRemove("tui", modelData.id, modelData.name)
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Web"
    query: root.query
    detail: "Site wrappers you installed with omarchy webapp install. Remove deletes the launcher. The site itself stays online."
    hint: "omarchy webapp remove"

    SettingRow {
      available: Omarchy.webApps.length === 0
      sectionHelp: false
      label: "Web apps"
      description: "No web apps."
      query: root.query
      keywords: ["empty", "webapp"]
    }

    Repeater {
      model: Omarchy.webApps

      CollectionRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "Web app"
        description: modelData && modelData.detail ? modelData.detail : "Opens this site in its own window."
        hint: "omarchy webapp remove"
        query: root.query
        keywords: ["webapp", "web", "browser", "pwa"]
        dangerAction: "Remove…"
        dangerEnabled: !!(modelData && modelData.name)
        onDangered: root.askRemove("web", modelData.id, modelData.name)
      }
    }
  }

  PrefsConfirm {
    id: removeAutostartConfirm
    title: "Remove autostart"
    message: "Stop launching " + root.pendingAutostart + " at login?"
    confirmText: "Remove"
    onConfirmed: Omarchy.removeAutostart(root.pendingAutostart)
  }
}
