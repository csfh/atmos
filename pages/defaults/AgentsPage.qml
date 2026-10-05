import QtQuick
import "../../components"
import "../../services"
import "../../services/Failure.js" as FailureJs
import "../../services/Requests.js" as Requests

PrefsPage {
  id: root
  hubId: "defaults/agents"
  title: "Agent tools"
  description: "Let installed coding agents read this machine and change settings through Atmos."

  property var agents: []
  property var watchSig: ({})
  property string loadError: ""
  property string checkText: ""

  readonly property var installed: {
    var list = root.agents || []
    var out = []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && list[i].installed) out.push(list[i])
    }
    return out
  }

  readonly property int installable: {
    var list = root.installed
    var n = 0
    var i, row
    for (i = 0; i < list.length; i++) {
      row = list[i]
      if (row.writer && (row.state === "absent" || row.state === "ours")) n += 1
    }
    return n
  }

  readonly property string missingNames: {
    var list = root.agents || []
    var names = []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && !list[i].installed) names.push(String(list[i].name || list[i].id || ""))
    }
    return names.join(", ")
  }

  function ownsPath(path) {
    var list = root.agents || []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].path || "") === path) return true
    }
    return false
  }

  function syncWatch() {
    var paths = []
    var list = root.agents || []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && list[i].path) paths.push(String(list[i].path))
    }
    Backend.watch("agents-mcp", { paths: paths })
  }

  function failureText(env, fallback) {
    return FailureJs.errorText(env && env.error) || fallback
  }

  function reload(done) {
    Backend.request(Requests.agentsMcpList(), function(env) {
      if (!env || env.ok !== true) {
        root.loadError = root.failureText(env, "Could not read agents")
        if (done) done(false)
        return
      }
      root.agents = env.result || []
      root.syncWatch()
      if (done) done(true)
      else root.loadError = ""
    })
  }

  function apply(agent, on, replace) {
    Backend.request(Requests.agentsMcpSet(agent, on, replace), function(env) {
      var failed = !env || env.ok !== true
      var text = failed ? root.failureText(env, "Could not update") : ""
      root.reload(function(ok) {
        if (!ok) return
        root.loadError = text
      })
    })
  }

  function installAll() {
    var list = root.installed
    var todo = []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i].writer && (list[i].state === "absent" || list[i].state === "ours"))
        todo.push(String(list[i].id))
    }
    var failed = ""
    function next(index) {
      if (index >= todo.length) {
        root.reload(function(ok) {
          if (ok) root.loadError = failed
        })
        return
      }
      Backend.request(Requests.agentsMcpSet(todo[index], true, false), function(env) {
        if ((!env || env.ok !== true) && !failed)
          failed = root.failureText(env, "Could not update")
        next(index + 1)
      })
    }
    next(0)
  }

  function runCheck() {
    root.checkText = "Checking…"
    Backend.request(Requests.agentsMcpCheck(), function(env) {
      if (!env || env.ok !== true) {
        root.checkText = root.failureText(env, "Check failed")
        return
      }
      var names = env.result && env.result.tools ? env.result.tools : []
      root.checkText = names.length + (names.length === 1 ? " tool" : " tools")
    })
  }

  function rowDescription(row) {
    if (!row.writer) return "Atmos cannot write this agent's MCP config yet."
    if (row.state === "custom") return "This agent already has an Omarchy entry with a different command."
    if (row.state === "stale") return "The Omarchy entry points at an old command."
    return "Reads this machine and changes settings through Atmos."
  }

  Component.onCompleted: root.reload()

  Connections {
    target: Backend
    function onStamp(doc) {
      var items = doc && doc.items ? doc.items : []
      var i, path, sig, prev
      for (i = 0; i < items.length; i++) {
        path = String(items[i].path || "")
        sig = String(items[i].sig || "")
        prev = root.watchSig[path]
        root.watchSig[path] = sig
        if (prev !== undefined && prev !== sig && root.ownsPath(path)) root.reload()
      }
    }
  }

  PrefsGroup {
    title: "Installed agents"
    query: root.query
    detail: root.missingNames.length > 0
      ? ("Not installed: " + root.missingNames + ".")
      : "Every agent Omarchy knows is installed."

    SettingRow {
      label: "Install"
      description: "Write the Atmos command into each installed agent that Atmos knows how to edit."
      caption: root.loadError || root.checkText
      query: root.query
      keywords: ["mcp", "install", "check"]

      Row {
        spacing: Theme.space

        PrefsButton {
          text: "Install for every installed agent"
          enabled: root.installable > 0
          onClicked: root.installAll()
        }

        PrefsButton {
          text: "Check"
          onClicked: root.runCheck()
        }
      }
    }

    SettingRow {
      visible: root.installed.length === 0 && root.loadError.length === 0
      label: "No agents"
      description: "Install a coding agent with omarchy default agent, then come back."
      valueText: "None"
      query: root.query
    }

    Repeater {
      model: root.installed

      SettingRow {
        label: modelData.name || modelData.id
        description: root.rowDescription(modelData)
        hint: modelData.path || (modelData.id === "claude" ? "claude mcp" : "")
        caption: modelData.state === "custom" ? "Custom command" : (modelData.state === "stale" ? "Out of date" : "")
        query: root.query
        keywords: ["mcp", String(modelData.id || "")]

        Row {
          spacing: Theme.space

          PrefsToggle {
            checked: modelData.on === true
            enabled: modelData.writer === true && (modelData.state === "absent" || modelData.state === "ours")
            onToggled: function(next) { root.apply(modelData.id, next, false) }
          }

          PrefsButton {
            visible: modelData.writer === true && (modelData.state === "custom" || modelData.state === "stale")
            text: "Replace"
            onClicked: root.apply(modelData.id, true, true)
          }
        }
      }
    }
  }
}
