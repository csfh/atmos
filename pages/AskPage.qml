import QtQuick
import "../components"
import "../services"
import "../services/Ask.js" as AskJs

// Ask for what you want, in plain words, without leaving.
//
// A page rather than an overlay, and the first thing in the sidebar. An
// overlay has to be summoned by someone who already knows it is there, and
// a settings app is opened rarely enough that nobody memorises its keys --
// so the one feature meant to answer "I do not know where this lives"
// cannot itself be something you have to know about.
//
// Nothing here shells out. An earlier version handed hard questions to the
// user's coding agent, which meant a terminal opening over the window you
// were working in and the answer arriving somewhere else entirely. That is
// a settings app giving up. Everything is answered here, in the window, or
// it is honestly reported as unanswered.
//
// The boundary is unchanged and it is the point: this page can take you to
// a setting and explain one, and it has no path that changes one.

PrefsPage {
  id: root
  hubId: "ask"
  title: "Ask"
  description: "Describe what you want in plain words. Atmos will take you to the setting, and explain it here. It will not change anything on its own."

  readonly property var hubs: AskJs.hubsFrom(Omarchy.hubCatalogue)
  property string asked: ""

  readonly property var result: root.asked.length > 0 ? AskJs.resolve(root.hubs, root.asked) : null
  readonly property var matches: root.result && root.result.matches ? root.result.matches : []
  readonly property var agentRead: Omarchy.agentAnswer.length > 0
    ? AskJs.readAnswer(Omarchy.agentAnswer, root.hubs)
    : null

  signal goToHub(string hubId)

  Component.onCompleted: Omarchy.probeAgent()

  function ask(text) {
    var q = String(text || "").replace(/^\s+|\s+$/g, "")
    root.asked = q
    Omarchy.agentAnswer = ""
    Omarchy.agentError = ""
  }

  // The machine's configured agent, not a second AI of Atmos's choosing.
  // Omarchy already holds a default; asking anything else would mean a
  // dependency the user never opted into.
  function askAgent() {
    var titles = []
    for (var i = 0; i < root.hubs.length; i++) {
      if (root.hubs[i] && root.hubs[i].title) titles.push(root.hubs[i].title)
    }
    Omarchy.askAgent(AskJs.agentPrompt(root.asked, titles))
  }

  PrefsGroup {
    title: "What do you want to change?"
    query: root.query
    detail: "Answered from the list of settings pages Atmos already has. Nothing is sent anywhere unless you press the local model button, and that stays on this machine."

    PrefsRow {
      label: "Ask"
      description: "Plain words are fine. \"my bluetooth mouse keeps dropping\" works as well as \"bluetooth\"."
      query: root.query
      stretchControl: true
      keywords: ["ask", "search", "help", "find", "natural", "language"]

      PrefsField {
        placeholder: "Ask…"
        onSubmitted: function (value) { root.ask(value) }
        onEdited: function (value) { if (value.length === 0) root.ask("") }
      }
    }
  }

  PrefsGroup {
    title: "Where it lives"
    query: root.query
    visible: root.asked.length > 0
    detail: "Matched against every page title, description and keyword in Atmos. This runs in the window and needs nothing installed."

    PrefsRow {
      label: "Nothing matched"
      description: "No page title, description or keyword lines up with that. Try a plainer word, or ask the local model below if one is running."
      query: root.query
      available: root.asked.length > 0 && root.matches.length === 0
    }

    Repeater {
      model: root.matches

      delegate: PrefsRow {
        required property var modelData
        required property int index
        label: modelData.hub.title
        description: modelData.hub.description || ""
        query: root.query
        valueText: index === 0 && root.result && root.result.kind === "go" ? "best match" : ""

        // House rule: an action names where it goes. "Open" alone tells you
        // nothing on a page that is a list of destinations.
        PrefsButton {
          text: "Open " + modelData.hub.title
          primary: index === 0
          onClicked: root.goToHub(modelData.hub.id)
        }
      }
    }
  }

  PrefsGroup {
    title: "Ask your agent"
    query: root.query
    visible: root.asked.length > 0
    detail: "Atmos hands the question to the coding agent this machine is already set up with, and prints the answer here. It never opens a terminal, and it cannot change a setting on your behalf."

    PrefsRow {
      label: "Your agent"
      description: {
        if (Omarchy.agentBusy)
          return "Asking " + Omarchy.agentName + "… " + Omarchy.agentSeconds + "s so far."
        if (Omarchy.agentError.length > 0)
          return Omarchy.agentError
        return "Configured default: " + Omarchy.agentName + ". It reads the question and says which page it belongs on."
      }
      query: root.query
      keywords: ["agent", "ai", "claude", "codex", "grok", "llm"]

      PrefsButton {
        text: Omarchy.agentBusy ? "Thinking…" : "Ask it"
        primary: true
        enabled: !Omarchy.agentBusy && root.asked.length > 0
        onClicked: root.askAgent()
      }
    }

    PrefsRow {
      label: "What it said"
      description: root.agentRead && root.agentRead.what ? root.agentRead.what : Omarchy.agentAnswer
      query: root.query
      available: Omarchy.agentAnswer.length > 0
      detail: "Treat the page as the answer and the sentence as a hint. Nothing here is turned into a command."

      PrefsButton {
        text: root.agentRead && root.agentRead.target ? "Open " + root.agentRead.target.title : "No page named"
        primary: !!(root.agentRead && root.agentRead.target)
        enabled: !!(root.agentRead && root.agentRead.target)
        onClicked: if (root.agentRead && root.agentRead.target) root.goToHub(root.agentRead.target.id)
      }
    }
  }
}
