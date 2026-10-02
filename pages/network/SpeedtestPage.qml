import QtQuick
import "../../components"
import "../../services"
import "../../services/Failure.js" as FailureJs
import "../../services/Requests.js" as Requests
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "network/speedtest"
  title: "Speed test"
  description: "A short download, then an upload, on whatever you are connected to now. Opening this page starts a run."

  property bool speedRunning: false
  property string speedPhase: ""
  property string downloadMbps: ""
  property string uploadMbps: ""
  property string speedError: ""
  property int speedToken: 0
  property bool speedInFlight: false

  function startSpeedtest() {
    if (speedInFlight) return
    if (Omarchy.netKind === "disconnected") {
      speedError = "No default route. Join a network on the Network page, then retry."
      return
    }
    speedError = ""
    downloadMbps = ""
    uploadMbps = ""
    speedRunning = true
    startSpeedPhase("down")
  }

  function startSpeedPhase(phase) {
    speedPhase = phase
    speedToken += 1
    speedInFlight = true
    var token = speedToken
    Backend.request(Requests.speedtestNet(phase), function(env) {
      if (token === root.speedToken) root.speedAnswered(env)
    })
    speedTimer.restart()
  }

  // Cancel stops listening. The measurement ends by itself on the backend.
  function stopSpeedtest() {
    speedTimer.stop()
    speedToken += 1
    speedInFlight = false
    speedPhase = ""
    speedRunning = false
  }

  function speedAnswered(env) {
    speedTimer.stop()
    speedInFlight = false
    var result = env && env.ok === true && env.result ? env.result : null
    if (result) root.adoptSpeed(result.stdout)
    if (!result || Number(result.exit) !== 0) {
      var text = result && result.stderr ? result.stderr : FailureJs.errorText(env && env.error)
      root.speedError = String(text || "Speed test failed").replace(/^\s+|\s+$/g, "")
      root.speedRunning = false
      root.speedPhase = ""
      return
    }
    root.finishSpeedPhase()
  }

  function finishSpeedPhase() {
    if (speedPhase === "down") {
      startSpeedPhase("up")
      return
    }
    speedPhase = ""
    speedRunning = false
  }

  function phaseLabel() {
    if (Omarchy.netKind === "disconnected")
      return "Run stays disabled until this machine has a default route. Join a network on the Network page."
    if (speedRunning && speedPhase === "up") return "Measuring upload."
    if (speedRunning) return "Measuring download."
    if (downloadMbps.length > 0 || uploadMbps.length > 0) return "Last run on this page."
    return "Download, then upload. About five seconds each."
  }

  function rateText(raw) {
    return RichUi.mbpsLabel(raw) || "—"
  }

  function progressDescription() {
    if (root.speedRunning && root.speedPhase === "up")
      return "Measuring outbound on the default route."
    if (root.speedRunning)
      return "Measuring inbound on the default route."
    if (root.downloadMbps.length > 0 || root.uploadMbps.length > 0)
      return "Last run finished. Copy a rate from Results, or Run again."
    return "About five seconds each direction. Opening this page starts a run."
  }

  function progressValueText() {
    if (root.speedRunning && root.speedPhase === "up")
      return "Upload " + (RichUi.mbpsLabel(root.uploadMbps) || "…")
    if (root.speedRunning)
      return "Download " + (RichUi.mbpsLabel(root.downloadMbps) || "…")
    if (root.downloadMbps.length > 0 || root.uploadMbps.length > 0)
      return "Down " + root.rateText(root.downloadMbps) + "   Up " + root.rateText(root.uploadMbps)
    return "Waiting"
  }

  function downloadDescription() {
    if (root.downloadMbps.length > 0)
      return RichUi.mbpsLabel(root.downloadMbps) + " inbound on the last sample."
    if (root.speedRunning && root.speedPhase === "down")
      return "Measuring inbound on the default route."
    if (Omarchy.netKind === "disconnected")
      return "Copy stays disabled until a run finishes. Join a network on the Network page, then Run."
    return "No download sample yet. Run above measures inbound first."
  }

  function uploadDescription() {
    if (root.uploadMbps.length > 0)
      return RichUi.mbpsLabel(root.uploadMbps) + " outbound on the last sample."
    if (root.speedRunning && root.speedPhase === "up")
      return "Measuring outbound on the default route."
    if (root.speedRunning && root.speedPhase === "down")
      return "Upload starts after the download sample. Copy stays disabled until that finishes."
    if (Omarchy.netKind === "disconnected")
      return "Copy stays disabled until a run finishes. Join a network on the Network page, then Run."
    return "No upload sample yet. Run above measures outbound second."
  }

  Component.onCompleted: startSpeedtest()
  Component.onDestruction: {
    speedToken += 1
  }

  Timer {
    id: speedTimer
    interval: 5000
    repeat: false
    onTriggered: {
      // Five seconds is the window for one direction. An answer that has not
      // come by now is dropped and the run moves on.
      root.speedToken += 1
      root.speedInFlight = false
      root.finishSpeedPhase()
    }
  }

  function adoptSpeed(text) {
    var lines = String(text || "").split("\n")
    var i, n
    for (i = 0; i < lines.length; i++) {
      n = RichUi.parseMbpsLine(lines[i])
      if (!isFinite(n)) continue
      if (root.speedPhase === "down") root.downloadMbps = String(n)
      else if (root.speedPhase === "up") root.uploadMbps = String(n)
    }
  }

  PrefsGroup {
    title: "Run"
    query: root.query
    detail: "Runs a download sample, then an upload sample. Each direction is about five seconds. Results are megabits per second on the default route. Opening this page starts a run."

    SettingRow {
      label: "Speed test"
      description: root.phaseLabel()
      hint: "omarchy network speedtest"
      detail: "Runs a download sample, then an upload sample. Each direction is about five seconds. Results are megabits per second on the default route. Cancel stops the current process. Opening this page starts a run."
      query: root.query
      keywords: ["bandwidth", "ping", "speedtest"]

      PrefsButton {
        text: root.speedRunning ? "Cancel" : "Run now"
        primary: !root.speedRunning
        danger: root.speedRunning
        enabled: root.speedRunning || Omarchy.netKind !== "disconnected"
        onClicked: root.speedRunning ? root.stopSpeedtest() : root.startSpeedtest()
      }
    }

    SettingRow {
      stretchControl: true
      label: "Progress"
      description: root.progressDescription()
      hint: "omarchy network speedtest"
      query: root.query
      keywords: ["progress", "mbps"]

      PrefsProgress {
        width: parent.width
        value: root.speedPhase === "down" ? 40 : (root.speedPhase === "up" ? 80 : (root.downloadMbps.length ? 100 : 0))
        indeterminate: root.speedRunning
        valueText: root.progressValueText()
      }
    }

    SettingRow {
      available: root.speedError.length > 0
      label: "Could not run"
      description: root.speedError
      query: root.query
      keywords: ["failed"]

      PrefsButton {
        text: "Retry"
        enabled: !root.speedRunning && Omarchy.netKind !== "disconnected"
        onClicked: root.startSpeedtest()
      }
    }
  }

  PrefsGroup {
    title: "Results"
    query: root.query
    detail: "Inbound and outbound rates from the last run on this page. They are megabits per second on the default route."

    SettingRow {
      label: "Download"
      description: root.downloadDescription()
      hint: "omarchy network speedtest down"
      query: root.query
      keywords: ["down", "mbps"]

      Row {
        spacing: Theme.space
        Text {
          text: root.rateText(root.downloadMbps)
          color: root.downloadMbps.length > 0 ? Theme.foreground : Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.titleSize
          font.bold: true
          verticalAlignment: Text.AlignVCenter
          anchors.verticalCenter: parent.verticalCenter
        }
        PrefsButton {
          text: "Copy"
          enabled: root.downloadMbps.length > 0
          onClicked: Omarchy.copyText(RichUi.mbpsCopyText("Download", root.downloadMbps))
        }
      }
    }

    SettingRow {
      label: "Upload"
      description: root.uploadDescription()
      hint: "omarchy network speedtest up"
      query: root.query
      keywords: ["up", "mbps"]

      Row {
        spacing: Theme.space
        Text {
          text: root.rateText(root.uploadMbps)
          color: root.uploadMbps.length > 0 ? Theme.foreground : Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.titleSize
          font.bold: true
          verticalAlignment: Text.AlignVCenter
          anchors.verticalCenter: parent.verticalCenter
        }
        PrefsButton {
          text: "Copy"
          enabled: root.uploadMbps.length > 0
          onClicked: Omarchy.copyText(RichUi.mbpsCopyText("Upload", root.uploadMbps))
        }
      }
    }
  }
}
