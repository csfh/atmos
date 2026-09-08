import QtQuick
import "../../components"
import "../../services"
import "../../services/EnvPrefs.js" as EnvJs

PrefsPage {
  id: root
  title: "Environment"
  description: "Detected session values are read-only. Extra variables and a PATH prepend write ~/.config/environment.d/10-atmos.conf and apply on the next login."

  property string varKeyDraft: ""
  property string varValueDraft: ""
  property string varError: ""
  property string pathPrependDraft: Omarchy.envPathPrepend

  readonly property var detected: EnvJs.detected(Omarchy.envDetected)

  Connections {
    target: Omarchy
    function onEnvPathPrependChanged() { root.pathPrependDraft = Omarchy.envPathPrepend }
  }

  function addVar() {
    var key = EnvJs.sanitizeKey(root.varKeyDraft)
    var value = EnvJs.sanitizeValue(root.varValueDraft)
    if (!key) {
      root.varError = "Use a shell variable name. PATH is set with PATH prepend."
      return
    }
    var vars = []
    var list = Omarchy.envVars || []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].key === key) continue
      vars.push(list[i])
    }
    vars.push({ key: key, value: value })
    root.varError = ""
    Omarchy.setEnvVars(vars, Omarchy.envPathPrepend)
  }

  PrefsGroup {
    title: "Detected"
    query: root.query
    detail: "What this session is already using. Defaults for browser, terminal, and editor are on Defaults."

    SettingRow {
      label: "Session"
      description: "This login's session type and desktop."
      query: root.query
      keywords: ["wayland", "session", "desktop"]
      valueText: (root.detected.sessionType || "unknown") + (root.detected.desktop ? " · " + root.detected.desktop : "")
    }

    SettingRow {
      label: "Shell"
      description: "The login shell for this account."
      query: root.query
      keywords: ["shell", "zsh", "bash"]
      valueText: root.detected.shell || "not set"
    }

    SettingRow {
      label: "Editor / terminal / browser"
      description: "Environment values for those tools. Defaults still live on Defaults."
      query: root.query
      keywords: ["editor", "terminal", "browser"]
      valueText: [root.detected.editor, root.detected.terminal, root.detected.browser].filter(function (v) { return v }).join(" · ") || "not set"
    }

    SettingRow {
      stretchControl: true
      label: "PATH"
      description: root.detected.path || "not set"
      query: root.query
      keywords: ["path"]
    }
  }

  PrefsGroup {
    framed: true
    title: "Overlay"
    query: root.query
    detail: "Written to ~/.config/environment.d/10-atmos.conf. systemd user sessions pick it up on the next login."
    hint: "~/.config/environment.d/10-atmos.conf"

    SettingRow {
      stretchControl: true
      label: "PATH prepend"
      description: "Absolute directories, colon-separated, added in front of PATH."
      hint: "PATH=…:$PATH"
      query: root.query
      keywords: ["path", "prepend"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: pathPrependField
          width: parent.width - pathPrependSetBtn.width - parent.spacing
          placeholder: "/opt/bin"
          value: root.pathPrependDraft
          onEdited: function(value) { root.pathPrependDraft = value }
          onSubmitted: function(value) {
            root.pathPrependDraft = value
            Omarchy.setEnvVars(Omarchy.envVars, value)
          }
        }

        PrefsButton {
          id: pathPrependSetBtn
          text: "Set"
          primary: true
          enabled: root.pathPrependDraft !== Omarchy.envPathPrepend
          onClicked: Omarchy.setEnvVars(Omarchy.envVars, root.pathPrependDraft)
        }
      }
    }

    SettingRow {
      label: "Add a variable"
      description: root.varError.length ? root.varError : "A name and a value. PATH is the prepend field above."
      query: root.query
      keywords: ["environment", "variable", "export"]

      Row {
        spacing: Theme.space
        PrefsField {
          width: 120
          placeholder: "EDITOR"
          onEdited: function(value) { root.varKeyDraft = value }
        }
        PrefsField {
          width: 160
          placeholder: "nvim"
          onEdited: function(value) { root.varValueDraft = value }
        }
        PrefsButton {
          text: "Add"
          primary: true
          onClicked: root.addVar()
        }
      }
    }

    Repeater {
      model: Omarchy.envVars

      SettingRow {
        required property var modelData
        label: modelData && modelData.key ? modelData.key : "VAR"
        description: modelData && modelData.value ? modelData.value : ""
        query: root.query
        keywords: ["environment"]

        PrefsButton {
          text: "Remove"
          danger: true
          onClicked: {
            var vars = []
            var list = Omarchy.envVars || []
            for (var i = 0; i < list.length; i++) {
              if (list[i] && list[i].key !== modelData.key) vars.push(list[i])
            }
            Omarchy.setEnvVars(vars, Omarchy.envPathPrepend)
          }
        }
      }
    }
  }
}
