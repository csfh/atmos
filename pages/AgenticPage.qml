import QtQuick
import "../components"
import "../services"
import "../services/Failure.js" as FailureJs
import "../services/Requests.js" as Requests
import "agentic" as AgentPages

PrefsPage {
  id: root
  hubId: "agentic"
  title: I18n.tr("Agentic")
  description: I18n.tr("The coding agents on this machine, and which one Omarchy opens.")

  property var stack: null
  property var navigator: null
  property var agents: []
  property string loadError: ""

  readonly property int installedCount: {
    var list = root.agents || []
    var n = 0
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && list[i].installed) n += 1
    }
    return n
  }

  readonly property bool knownAgent: {
    var id = Omarchy.agent
    if (!id) return true
    var list = root.agents || []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && list[i].id === id) return true
    }
    return false
  }

  function openSubpage(id) {
    if (stack) {
      if (id === "mcp") stack.push(mcpPage)
      return
    }
    if (root.navigator && root.navigator.go)
      root.navigator.go("agentic/" + id)
  }

  function failureText(env, fallback) {
    return FailureJs.errorText(env && env.error) || fallback
  }

  function reload() {
    Backend.request(Requests.agentsMcpList(), function(env) {
      if (!env || env.ok !== true) {
        root.loadError = root.failureText(env, "Could not read agents")
        return
      }
      root.loadError = ""
      root.agents = env.result || []
    })
  }

  function useAgent(id) {
    if (!id || id === Omarchy.agent) return
    Omarchy.set("agent", id)
  }

  function rowDescription(row) {
    if (Omarchy.agent === row.id) return "Omarchy opens this agent."
    if (row.installed) return "Installed on this machine."
    return "Omarchy can install this agent and open it."
  }

  Component.onCompleted: root.reload()

  Connections {
    target: Omarchy
    function onAgentChanged() { root.reload() }
  }

  Component { id: mcpPage; AgentPages.McpPage {} }

  PrefsGroup {
    title: I18n.tr("Agents")
    query: root.query
    detail: root.loadError.length
      ? root.loadError
      : (root.agents.length
        ? (root.installedCount + " of " + root.agents.length + " installed. Choosing one writes it as the Omarchy default.")
        : "Choosing one writes it as the Omarchy default. If it is missing, Omarchy offers to install it.")

    SettingRow {
      visible: Omarchy.agent.length > 0 && !root.knownAgent
      label: Omarchy.agent
      description: I18n.tr("Omarchy opens this agent.")
      valueText: "Default"
      query: root.query
      keywords: ["agent", "default"]
    }

    Repeater {
      model: root.agents

      SettingRow {
        label: modelData.name || modelData.id
        description: root.rowDescription(modelData)
        valueText: Omarchy.agent === modelData.id ? "Default" : ""
        query: root.query
        keywords: ["agent", String(modelData.id || "")]

        PrefsButton {
          visible: Omarchy.agent !== modelData.id
          text: modelData.installed ? "Use" : "Install"
          onClicked: root.useAgent(modelData.id)
        }
      }
    }
  }

  PrefsGroup {
    title: I18n.tr("Tools")
    query: root.query
    detail: "MCP lets an installed agent read this machine and change settings through Atmos."

    SettingRow {
      label: "MCP"
      description: I18n.tr("Install the Atmos command into Grok, Claude Code, and Cursor.")
      hint: "atmos mcp"
      query: root.query
      keywords: ["mcp", "claude", "grok", "cursor"]

      PrefsButton {
        text: I18n.tr("Open…")
        onClicked: root.openSubpage("mcp")
      }
    }
  }
}
