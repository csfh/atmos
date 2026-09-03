import QtQuick
import "../components"
import "../services"
import "../services/Software.js" as Soft

PrefsPage {
  id: root
  title: "Software"
  description: "Optional stacks Omarchy can install. Remove asks first. Gaming remove also deletes libraries those launchers keep."

  property var pendingItem: null
  property string pendingKind: ""
  property string pendingId: ""
  property string pendingLabel: ""
  property string pendingHint: ""
  property bool pendingWipe: false
  property var pendingArgv: []
  property string devLang: "node"
  property string dockerDb: "PostgreSQL"

  readonly property var bags: ({
    browsers: Omarchy.browsers,
    terminals: Omarchy.terminals,
    editors: Omarchy.editors,
    services: Omarchy.services,
    gaming: Omarchy.gaming,
    extras: Omarchy.extras
  })

  function installed(item) {
    return Soft.presentIn(item, root.bags)
  }

  function askInstall(item) {
    if (!item || !item.install) return
    root.pendingKind = "install"
    root.pendingItem = item
    root.pendingId = item.id
    root.pendingLabel = item.label
    root.pendingHint = item.hint || ""
    root.pendingWipe = false
    root.pendingArgv = item.install
    softwareConfirm.ask()
  }

  function askRemove(item) {
    if (!item || !item.remove) return
    root.pendingKind = "remove"
    root.pendingItem = item
    root.pendingId = item.id
    root.pendingLabel = item.label
    root.pendingHint = item.hint || ""
    root.pendingWipe = item.wipe === true
    root.pendingArgv = item.remove
    softwareConfirm.ask()
  }

  function askDev(kind) {
    root.pendingKind = kind
    root.pendingItem = null
    root.pendingId = root.devLang
    root.pendingLabel = root.devLang
    root.pendingHint = "omarchy " + kind + " dev env"
    root.pendingWipe = false
    root.pendingArgv = []
    softwareConfirm.ask()
  }

  function runPending() {
    if (root.pendingItem) {
      Omarchy.runSoftware(root.pendingArgv, root.pendingKind === "remove" ? "software-remove" : "software-install")
      return
    }
    if (root.pendingKind === "remove") Omarchy.removeDevEnv(root.pendingId)
    else Omarchy.installDevEnv(root.pendingId)
  }

  function confirmTitle() {
    if (root.pendingKind === "remove") return "Remove " + root.pendingLabel
    return "Install " + root.pendingLabel
  }

  function confirmMessage() {
    if (root.pendingKind === "remove" && root.pendingWipe)
      return "Remove " + root.pendingLabel + "? That also deletes the libraries and prefixes this launcher keeps."
    if (root.pendingKind === "remove")
      return "Remove " + root.pendingLabel + " and the packages Omarchy added for it."
    return "Install " + root.pendingLabel + " with the Omarchy installer."
  }

  PrefsConfirm {
    id: softwareConfirm
    title: root.confirmTitle()
    message: root.confirmMessage()
    confirmText: root.pendingKind === "remove" ? "Remove" : "Install"
    onConfirmed: root.runPending()
  }

  Component.onCompleted: softwareConfirm.parent = root.prefsOverlay

  PrefsGroup {
    title: "Browsers"
    query: root.query
    detail: "Install an extra browser, then set the default on Defaults if you want Omarchy to open it."
    hint: "omarchy install browser"

    Repeater {
      model: Soft.groupItems("browsers")
      delegate: softwareDelegate
    }
  }

  PrefsGroup {
    title: "Terminals"
    query: root.query
    detail: "Omarchy can install another terminal. There is no matching remove command, so these rows only install."
    hint: "omarchy install terminal"

    Repeater {
      model: Soft.groupItems("terminals")
      delegate: softwareDelegate
    }
  }

  PrefsGroup {
    title: "Editors"
    query: root.query
    detail: "Optional editors with Omarchy theme wiring. Remove is not a separate Omarchy command here."
    hint: "omarchy install editor"

    Repeater {
      model: Soft.groupItems("editors")
      delegate: softwareDelegate
    }
  }

  PrefsGroup {
    title: "Services"
    query: root.query
    detail: "Optional apps Omarchy packages as services. Tailscale also has a shortcut on Network."
    hint: "omarchy install service"

    Repeater {
      model: Soft.groupItems("services")
      delegate: softwareDelegate
    }
  }

  PrefsGroup {
    title: "Gaming"
    query: root.query
    detail: "Launchers and a couple of cloud clients. Remove for Steam and the others wipes their libraries."
    hint: "omarchy install gaming"

    Repeater {
      model: Soft.groupItems("gaming")
      delegate: softwareDelegate
    }
  }

  PrefsGroup {
    title: "Development"
    query: root.query
    detail: "Language toolchains through mise, a Docker database, and the ChatGPT desktop app."

    PrefsRow {
      label: "Language toolchain"
      description: "Install or remove a dev environment Omarchy knows."
      hint: "omarchy install dev env"
      query: root.query
      keywords: ["dev", "mise", "ruby", "node", "python", "rust"]

      Flow {
        width: parent.width
        spacing: 8
        PrefsSelect {
          width: 140
          value: root.devLang
          options: Soft.devEnvs()
          enabled: !Omarchy.busy && !Omarchy.jobBusy
          onChanged: function(value) { root.devLang = value }
        }
        PrefsButton {
          text: "Install…"
          primary: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy
          onClicked: root.askDev("install")
        }
        PrefsButton {
          text: "Remove…"
          danger: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy
          onClicked: root.askDev("remove")
        }
      }
    }

    PrefsRow {
      available: Omarchy.extras && Omarchy.extras.docker === true
      label: "Docker database"
      description: "Start a supported database in Docker."
      hint: "omarchy install docker dbs"
      query: root.query
      keywords: ["docker", "postgres", "mysql", "redis", "mongo"]

      Flow {
        width: parent.width
        spacing: 8
        PrefsSelect {
          width: 140
          value: root.dockerDb
          options: Soft.dockerDbs()
          enabled: !Omarchy.busy && !Omarchy.jobBusy
          onChanged: function(value) { root.dockerDb = value }
        }
        PrefsButton {
          text: "Install"
          primary: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy
          onClicked: Omarchy.installDockerDb(root.dockerDb)
        }
      }
    }

    Repeater {
      model: Soft.groupItems("development")
      delegate: softwareDelegate
    }
  }

  Component {
    id: softwareDelegate
    PrefsRow {
      required property var modelData
      sectionHelp: false
      label: modelData && modelData.label ? modelData.label : ""
      description: root.installed(modelData)
        ? (modelData.remove ? "Installed. Remove asks before it runs." : "Installed.")
        : (modelData.wipe ? "Install this launcher. Remove later also deletes its libraries." : "Not installed.")
      hint: modelData && modelData.hint ? modelData.hint : ""
      query: root.query
      keywords: [modelData && modelData.id ? modelData.id : "", modelData && modelData.group ? modelData.group : ""]

      Row {
        spacing: 8
        PrefsButton {
          visible: !root.installed(modelData)
          text: "Install…"
          primary: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy && !!(modelData && modelData.install)
          onClicked: root.askInstall(modelData)
        }
        PrefsButton {
          visible: root.installed(modelData) && !!(modelData && modelData.remove)
          text: "Remove…"
          danger: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy && !!(modelData && modelData.remove)
          onClicked: root.askRemove(modelData)
        }
        PrefsText {
          visible: root.installed(modelData) && !(modelData && modelData.remove)
          text: "Installed"
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.captionSize
        }
      }
    }
  }
}
