import QtQuick
import QtQuick.Dialogs
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Bar"
  description: "Where the bar sits and what it shows. The clock format and a few widgets live further down."

  FolderDialog {
    id: syncDirDialog
    title: "Agent usage sync folder"
    onAccepted: Omarchy.setAgentsSyncDir(RichUi.pathFromUrl(selectedFolder))
  }

  readonly property var horizontalClockFormats: [
    { value: "dddd HH:mm", label: "Monday 14:05" },
    { value: "dddd h:mm AP", label: "Monday 2:05 PM" },
    { value: "HH:mm", label: "14:05" },
    { value: "h:mm AP", label: "2:05 PM" },
    { value: "ddd d MMM HH:mm", label: "Mon 1 Sep 14:05" },
    { value: "ddd d MMM h:mm AP", label: "Mon 1 Sep 2:05 PM" },
    { value: "d MMMM 'W'ww yyyy", label: "1 September W36 2026" },
    { value: "yyyy-MM-dd HH:mm", label: "2026-09-01 14:05" }
  ]
  readonly property var verticalClockFormats: [
    { value: "HH\n—\nmm", label: "14 — 05" },
    { value: "h\n—\nmm\nAP", label: "2 — 05 PM" },
    { value: "dd\nMMM\n'W'ww\n''yy", label: "01 Sep W36 '26" },
    { value: "HH\nmm", label: "14 05" }
  ]
  readonly property var clockFormatOptions: {
    var vertical = Omarchy.barPosition === "left" || Omarchy.barPosition === "right"
    var list = vertical ? root.verticalClockFormats : root.horizontalClockFormats
    var current = Omarchy.clockFormat
    if (!current || current.length === 0) return list
    for (var i = 0; i < list.length; i++) {
      if (list[i].value === current) return list
    }
    return list.concat([{ value: current, label: current }])
  }
  readonly property var clockFormatAltOptions: {
    var vertical = Omarchy.barPosition === "left" || Omarchy.barPosition === "right"
    var list = vertical ? root.verticalClockFormats : root.horizontalClockFormats
    var current = Omarchy.clockFormatAlt
    if (!current || current.length === 0) return list
    for (var i = 0; i < list.length; i++) {
      if (list[i].value === current) return list
    }
    return list.concat([{ value: current, label: current }])
  }
  readonly property var weekStartOptions: {
    var list = [
      { value: "sunday", label: "Sunday" },
      { value: "monday", label: "Monday" },
      { value: "tuesday", label: "Tuesday" },
      { value: "wednesday", label: "Wednesday" },
      { value: "thursday", label: "Thursday" },
      { value: "friday", label: "Friday" },
      { value: "saturday", label: "Saturday" }
    ]
    if (!Omarchy.clockWeekStart || Omarchy.clockWeekStart.length === 0)
      return [{ value: "", label: "Locale default" }].concat(list)
    return list
  }
  readonly property var indicatorOptions: [
    { value: "Dictation", label: "Dictation" },
    { value: "ScreenRecording", label: "Screen recording" },
    { value: "Reminder", label: "Reminder" },
    { value: "NightLight", label: "Night light" },
    { value: "Dnd", label: "Do not disturb" },
    { value: "StayAwake", label: "Stay awake" }
  ]

  function indicatorShown(id) {
    var list = Omarchy.indicatorsItems
    if (!(list instanceof Array) || list.length === 0) return true
    return list.indexOf(id) !== -1
  }

  function trayIdLabel(id) {
    var text = String(id || "")
    var slash = text.lastIndexOf("/")
    return slash !== -1 ? text.substring(slash + 1) : text
  }

  function trayHiddenLabels() {
    var list = Omarchy.trayHidden
    var out = []
    if (!(list instanceof Array)) return out
    for (var i = 0; i < list.length; i++) out.push(root.trayIdLabel(list[i]))
    return out
  }

  function trayPinnedLabels() {
    var list = Omarchy.trayPinned
    var out = []
    if (!(list instanceof Array)) return out
    for (var i = 0; i < list.length; i++) out.push(root.trayIdLabel(list[i]))
    return out
  }

  function toggleIndicator(id) {
    var all = ["Dictation", "ScreenRecording", "Reminder", "NightLight", "Dnd", "StayAwake"]
    var current = Omarchy.indicatorsItems
    var selected = []
    if (!(current instanceof Array) || current.length === 0) {
      selected = all.slice()
    } else {
      for (var i = 0; i < all.length; i++) {
        if (current.indexOf(all[i]) !== -1) selected.push(all[i])
      }
      if (selected.length === 0) selected = all.slice()
    }
    var on = selected.indexOf(id) !== -1
    var next = []
    for (var j = 0; j < all.length; j++) {
      if (all[j] === id) {
        if (!on) next.push(all[j])
      } else if (selected.indexOf(all[j]) !== -1) {
        next.push(all[j])
      }
    }
    if (next.length === 0) return
    Omarchy.setIndicatorsItems(next)
  }

  PrefsGroup {
    title: "Layout"
    query: root.query
    detail: "The bar can sit on any edge. Transparent lets wallpaper show through. Hiding the bar leaves the rest of the shell running."

    SettingRow {
      label: "Position"
      description: "Which edge of the screen the bar sits on. Top is the usual place."
      hint: "omarchy bar position"
      query: root.query
      keywords: ["top", "bottom", "left", "right", "menu bar"]

      PrefsSelect {
        value: Omarchy.barPosition
        options: [
          { value: "top", label: "Top" },
          { value: "bottom", label: "Bottom" },
          { value: "left", label: "Left" },
          { value: "right", label: "Right" }
        ]
        onChanged: function(value) {
          if (value !== Omarchy.barPosition) Omarchy.setBarPosition(value)
        }
      }
    }

    SettingRow {
      label: "Transparent bar"
      description: "The wallpaper shows through the bar."
      hint: "omarchy bar transparent"
      query: root.query
      keywords: ["opacity", "see-through"]

      PrefsToggle {
        checked: Omarchy.barTransparent
        onToggled: Omarchy.setBarTransparent(!Omarchy.barTransparent)
      }
    }

    SettingRow {
      label: "Show bar"
      description: "Keep the bar visible. Turn this off to hide it."
      hint: "omarchy toggle bar"
      query: root.query
      keywords: ["hide", "visible", "autohide"]

      PrefsToggle {
        checked: Omarchy.barVisible
        onToggled: Omarchy.setBarVisible(!Omarchy.barVisible)
      }
    }
  }

  PrefsGroup {
    title: "Spacer"
    query: root.query
    detail: "A blank gap you can put between widgets. Add inserts omarchy.spacer. Remove takes it off the bar. Width is in pixels."

    SettingRow {
      available: !Omarchy.spacerPresent
      label: "Spacer"
      description: "Put a blank gap between widgets. You can set the width after it is there."
      hint: "omarchy bar put omarchy.spacer"
      query: root.query
      keywords: ["gap", "space", "padding", "layout", "missing", "add"]

      PrefsButton {
        text: "Add"
        primary: true
        onClicked: Omarchy.addSpacer()
      }
    }

    SettingRow {
      available: Omarchy.spacerPresent
      stretchControl: true
      label: "Width"
      description: "How wide the blank gap is between the widgets on either side."
      hint: "omarchy bar set omarchy.spacer size"
      query: root.query
      keywords: ["gap", "space", "padding", "layout"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 64
        stepSize: 1
        value: Omarchy.spacerSize
        valueText: Omarchy.spacerSize + " px"
        enabled: Omarchy.spacerPresent
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.spacerSize)
            Omarchy.setSpacerSize(next)
        }
      }
    }

    SettingRow {
      available: Omarchy.spacerPresent
      label: "Remove"
      description: "Take the blank gap off the bar."
      hint: "omarchy plugin disable omarchy.spacer"
      query: root.query
      keywords: ["gap", "space", "padding", "layout", "delete", "remove"]

      PrefsButton {
        text: "Remove"
        enabled: Omarchy.spacerPresent
        onClicked: Omarchy.removeSpacer()
      }
    }
  }

  PrefsGroup {
    title: "Tray"
    query: root.query
    detail: "Hidden icons stay listed so you can show them again. Pinned icons stay visible even when the tray tucks extras away."

    SettingRow {
      available: Omarchy.trayPresent && Omarchy.trayHidden.length === 0
      label: "Hidden icons"
      description: "No hidden tray icons."
      hint: "omarchy bar set omarchy.tray hidden"
      query: root.query
      keywords: ["system tray", "sni", "unhide", "show", "icons", "empty"]
    }

    SettingRow {
      available: Omarchy.trayPresent && Omarchy.trayHidden.length > 0
      label: "Hidden icons"
      description: "Hidden right now: " + root.trayHiddenLabels().join(", ") + ". Show all brings them back."
      hint: "omarchy bar set omarchy.tray hidden"
      query: root.query
      keywords: ["system tray", "sni", "unhide", "show", "icons"]

      PrefsButton {
        text: "Show all"
        enabled: Omarchy.trayPresent
        onClicked: Omarchy.clearTrayHidden()
      }
    }

    SettingRow {
      available: Omarchy.trayPresent && Omarchy.trayPinned.length === 0
      label: "Pinned icons"
      description: "No pinned tray icons."
      hint: "omarchy bar set omarchy.tray pinned"
      query: root.query
      keywords: ["system tray", "sni", "pin", "unpin", "always visible", "empty"]
    }

    SettingRow {
      available: Omarchy.trayPresent && Omarchy.trayPinned.length > 0
      label: "Pinned icons"
      description: "Always visible: " + root.trayPinnedLabels().join(", ") + ". Unpin all lets the tray manage them again."
      hint: "omarchy bar set omarchy.tray pinned"
      query: root.query
      keywords: ["system tray", "sni", "pin", "unpin", "always visible"]

      PrefsButton {
        text: "Unpin all"
        enabled: Omarchy.trayPresent
        onClicked: Omarchy.clearTrayPinned()
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Indicators"
    query: root.query
    detail: "Little status icons for things like dictation, recording, night light, and stay awake. Always show keeps them visible when they are idle."

    SettingRow {
      available: Omarchy.indicatorsPresent
      label: "Always show"
      description: "Keep these status icons on the bar even when they are idle."
      hint: "omarchy bar set omarchy.indicators alwaysShow"
      query: root.query
      keywords: ["status", "icons", "stay awake", "night light", "dnd"]

      PrefsToggle {
        checked: Omarchy.indicatorsAlwaysShow
        enabled: Omarchy.indicatorsPresent
        onToggled: Omarchy.setIndicatorsAlwaysShow(!Omarchy.indicatorsAlwaysShow)
      }
    }

    SettingRow {
      available: Omarchy.indicatorsPresent
      stretchControl: true
      label: "Shown indicators"
      description: "Which status icons this widget may show. Leave them all on if you want the full set."
      hint: "omarchy bar set omarchy.indicators items"
      query: root.query
      keywords: ["dictation", "recording", "reminder", "night light", "dnd", "stay awake"]

      Column {
        width: parent.width
        spacing: Theme.space

        Repeater {
          model: root.indicatorOptions

          Row {
            required property var modelData
            width: parent.width
            spacing: Theme.space

            PrefsText {
              width: parent.width - 52
              text: modelData.label
              color: Theme.foreground
              font.family: Theme.fontFamily
              font.pixelSize: Theme.fontSize
              anchors.verticalCenter: parent.verticalCenter
            }

            PrefsToggle {
              checked: root.indicatorShown(modelData.value)
              enabled: Omarchy.indicatorsPresent
              onToggled: root.toggleIndicator(modelData.value)
            }
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Agents"
    query: root.query
    detail: "Usage for coding agents on this machine. Sync writes a snapshot into a folder you share with other machines."

    SettingRow {
      available: Omarchy.agentsPresent
      stretchControl: true
      label: "Usage refresh"
      description: "How often the widget rebuilds usage numbers for the agents on this machine."
      hint: "omarchy bar set omarchy.agents refreshIntervalSec"
      query: root.query
      keywords: ["claude", "codex", "grok", "interval", "usage"]

      PrefsSlider {
        width: parent.width
        from: 5
        to: 60
        stepSize: 5
        value: Math.max(5, Math.round(Omarchy.agentsRefreshIntervalSec / 60))
        valueText: Math.max(5, Math.round(Omarchy.agentsRefreshIntervalSec / 60)) + " min"
        formatTick: function(v) { return Math.round(v) }
        enabled: Omarchy.agentsPresent
        onChanged: function(value) {
          var next = Math.round(value) * 60
          if (next !== Omarchy.agentsRefreshIntervalSec)
            Omarchy.setAgentsRefreshIntervalSec(next)
        }
      }
    }

    SettingRow {
      available: Omarchy.agentsPresent
      label: "Sync usage"
      description: "This machine's usage goes into the sync folder, and snapshots from other machines fold in."
      hint: "omarchy bar set omarchy.agents syncMode"
      query: root.query
      keywords: ["syncthing", "dropbox", "aggregate", "share"]

      PrefsToggle {
        checked: Omarchy.agentsSync
        enabled: Omarchy.agentsPresent
        onToggled: Omarchy.setAgentsSync(!Omarchy.agentsSync)
      }
    }

    SettingRow {
      available: Omarchy.agentsPresent
      label: "Sync folder"
      description: "A folder you already sync with Syncthing, Dropbox, or rsync. Needed when usage sync is on."
      hint: "omarchy bar set omarchy.agents syncDir"
      query: root.query
      keywords: ["syncthing", "dropbox", "rsync", "folder", "path"]

      Row {
        spacing: Theme.space

        PrefsField {
          id: agentsSyncDirField
          value: Omarchy.agentsSyncDir
          placeholder: "~/Sync/agent-usage"
          enabled: Omarchy.agentsPresent
          onSubmitted: function(value) { Omarchy.setAgentsSyncDir(value) }
        }

        PrefsButton {
          text: "Choose…"
          enabled: Omarchy.agentsPresent
          onClicked: syncDirDialog.open()
        }

        PrefsButton {
          text: "Set"
          enabled: Omarchy.agentsPresent
          onClicked: Omarchy.setAgentsSyncDir(agentsSyncDirField.currentText())
        }

        PrefsButton {
          visible: Omarchy.agentsSyncDir.length > 0
          text: "Clear"
          enabled: Omarchy.agentsPresent && Omarchy.agentsSyncDir.length > 0
          onClicked: Omarchy.setAgentsSyncDir("")
        }
      }
    }

    SettingRow {
      available: Omarchy.agentsPresent
      label: "Snapshot file"
      description: "The filename this machine writes in the sync folder. Leave it blank to use hostname.json."
      hint: "omarchy bar set omarchy.agents syncFileName"
      query: root.query
      keywords: ["snapshot", "filename", "hostname", "json"]

      Row {
        spacing: Theme.space

        PrefsField {
          id: agentsSyncFileField
          value: Omarchy.agentsSyncFileName
          placeholder: "laptop.json"
          horizontalAlignment: TextInput.AlignHCenter
          enabled: Omarchy.agentsPresent
          onSubmitted: function(value) { Omarchy.setAgentsSyncFileName(value) }
        }

        PrefsButton {
          text: "Set"
          enabled: Omarchy.agentsPresent
          onClicked: Omarchy.setAgentsSyncFileName(agentsSyncFileField.currentText())
        }

        PrefsButton {
          visible: Omarchy.agentsSyncFileName.length > 0
          text: "Clear"
          enabled: Omarchy.agentsPresent && Omarchy.agentsSyncFileName.length > 0
          onClicked: Omarchy.setAgentsSyncFileName("")
        }
      }
    }

    SettingRow {
      available: Omarchy.agentsPresent
      label: "Device id"
      description: "How this machine is named inside the synced snapshots. Leave it blank to use the hostname."
      hint: "omarchy bar set omarchy.agents syncDeviceId"
      query: root.query
      keywords: ["device", "hostname", "machine", "id"]

      Row {
        spacing: Theme.space

        PrefsField {
          id: agentsSyncDeviceField
          value: Omarchy.agentsSyncDeviceId
          placeholder: "laptop"
          horizontalAlignment: TextInput.AlignHCenter
          enabled: Omarchy.agentsPresent
          onSubmitted: function(value) { Omarchy.setAgentsSyncDeviceId(value) }
        }

        PrefsButton {
          text: "Set"
          enabled: Omarchy.agentsPresent
          onClicked: Omarchy.setAgentsSyncDeviceId(agentsSyncDeviceField.currentText())
        }

        PrefsButton {
          visible: Omarchy.agentsSyncDeviceId.length > 0
          text: "Clear"
          enabled: Omarchy.agentsPresent && Omarchy.agentsSyncDeviceId.length > 0
          onClicked: Omarchy.setAgentsSyncDeviceId("")
        }
      }
    }
  }

  PrefsGroup {
    title: "Clock"
    query: root.query
    detail: "How the bar clock looks. Right-click cycles the same formats. Birth year is optional and draws a life bar in the calendar popup."

    SettingRow {
      available: Omarchy.clockPresent
      label: "Clock format"
      description: "How the clock reads on the bar. Right-clicking the clock walks through these same presets."
      hint: "omarchy bar set omarchy.clock format"
      query: root.query
      keywords: ["time", "24-hour", "am pm", "date"]

      PrefsSelect {
        implicitWidth: 260
        value: Omarchy.clockFormat
        options: root.clockFormatOptions
        enabled: Omarchy.clockPresent
        onChanged: function(value) {
          if (value !== Omarchy.clockFormat) Omarchy.setClockFormat(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.clockPresent
      label: "Alternate format"
      description: "A second style in the clock's right-click cycle. Handy if you sometimes want the date too."
      hint: "omarchy bar set omarchy.clock formatAlt"
      query: root.query
      keywords: ["date", "week", "cycle", "secondary"]

      PrefsSelect {
        implicitWidth: 260
        value: Omarchy.clockFormatAlt
        options: root.clockFormatAltOptions
        enabled: Omarchy.clockPresent
        onChanged: function(value) {
          if (value !== Omarchy.clockFormatAlt) Omarchy.setClockFormatAlt(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.clockPresent
      label: "Week starts on"
      description: "First day of the week in the calendar popup. Locale default follows the system language."
      hint: "omarchy bar set omarchy.clock weekStartDay"
      query: root.query
      keywords: ["calendar", "week", "sunday", "monday", "start"]

      PrefsSelect {
        value: Omarchy.clockWeekStart
        options: root.weekStartOptions
        enabled: Omarchy.clockPresent
        onChanged: function(value) {
          if (value !== Omarchy.clockWeekStart) Omarchy.setClockWeekStart(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.clockPresent
      label: "Birth year"
      description: Omarchy.clockBirthYear > 0
        ? "The calendar popup draws a life bar from this year to the expectancy below."
        : "Optional. Set a year if you want a life bar in the calendar popup."
      hint: "omarchy bar set omarchy.clock birthYear"
      query: root.query
      keywords: ["age", "life", "calendar", "memento"]

      Row {
        spacing: Theme.space

        PrefsSpinBox {
          id: birthYearField
          implicitWidth: 108
          from: 0
          to: 2100
          value: Omarchy.clockBirthYear
          enabled: Omarchy.clockPresent
          onChanged: function(value) { Omarchy.setClockBirthYear(value) }
        }

        PrefsButton {
          visible: Omarchy.clockBirthYear > 0
          text: "Clear"
          enabled: Omarchy.clockPresent && Omarchy.clockBirthYear > 0
          onClicked: Omarchy.setClockBirthYear("")
        }
      }
    }

    SettingRow {
      available: Omarchy.clockPresent
      label: "Life expectancy"
      description: "How far the calendar life bar runs. Leave it blank to use 90 years."
      hint: "omarchy bar set omarchy.clock lifeExpectancy"
      query: root.query
      keywords: ["age", "life", "span", "memento", "years"]

      Row {
        spacing: Theme.space

        PrefsSpinBox {
          id: lifeExpectancyField
          implicitWidth: 88
          from: 0
          to: 150
          value: Omarchy.clockLifeExpectancy
          enabled: Omarchy.clockPresent
          onChanged: function(value) { Omarchy.setClockLifeExpectancy(value) }
        }

        PrefsButton {
          visible: Omarchy.clockLifeExpectancy > 0
          text: "Clear"
          enabled: Omarchy.clockPresent && Omarchy.clockLifeExpectancy > 0
          onClicked: Omarchy.setClockLifeExpectancy("")
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Shell plugins Omarchy discovered. The bar itself cannot be disabled. Other widgets and services can."

    SettingRow {
      available: Omarchy.plugins.length === 0
      label: "Plugins"
      description: "No plugins listed."
      hint: "omarchy plugin list"
      query: root.query
      keywords: ["plugin", "widget", "shell", "empty"]

      PrefsButton {
        text: "Refresh"
        onClicked: Omarchy.refresh()
      }
    }

    Repeater {
      model: Omarchy.plugins

      SettingRow {
        required property var modelData
        label: String((modelData && (modelData.name || modelData.id)) || "Plugin")
        description: modelData && modelData.canDisable === false
          ? "Always on. This widget cannot be disabled here."
          : (modelData && modelData.firstParty
            ? "A first-party Omarchy plugin."
            : "A third-party shell plugin.")
        hint: "omarchy plugin enable"
        query: root.query
        keywords: ["plugin", "widget", String((modelData && modelData.id) || "")]
        valueText: modelData && modelData.canDisable === false ? "On" : ""

        PrefsToggle {
          visible: !(modelData && modelData.canDisable === false)
          checked: !!(modelData && modelData.enabled)
          enabled: modelData && modelData.canDisable !== false && modelData.id
          onToggled: {
            if (!modelData || !modelData.id) return
            Omarchy.setPluginEnabled(String(modelData.id), !modelData.enabled)
          }
        }
      }
    }
  }
}
