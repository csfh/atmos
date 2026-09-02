import QtQuick
import Quickshell.Io
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Speed test"
  description: "A short download, then an upload, on whatever you are connected to now. Opening this page starts a run."

  property bool speedRunning: false
  property string speedPhase: ""
  property string downloadMbps: ""
  property string uploadMbps: ""
  property string speedError: ""
  property bool speedExpectedStop: false

  function startSpeedtest() {
    if (speedProc.running) return
    if (Omarchy.netKind === "disconnected") {
      speedError = "Nothing is online yet. Connect to a network first."
      return
    }
    speedError = ""
    downloadMbps = ""
    uploadMbps = ""
    speedRunning = true
    speedExpectedStop = false
    startSpeedPhase("down")
  }

  function startSpeedPhase(phase) {
    speedPhase = phase
    speedProc.command = ["omarchy", "network", "speedtest", phase]
    speedProc.running = true
    speedTimer.restart()
  }

  function stopSpeedtest() {
    speedTimer.stop()
    speedExpectedStop = true
    speedPhase = ""
    speedRunning = false
    if (speedProc.running) speedProc.running = false
  }

  function finishSpeedPhase() {
    if (speedPhase === "down") {
      startSpeedPhase("up")
      return
    }
    speedPhase = ""
    speedRunning = false
    speedExpectedStop = false
  }

  function phaseLabel() {
    if (speedRunning && speedPhase === "up") return "Measuring upload."
    if (speedRunning) return "Measuring download."
    if (downloadMbps.length > 0 || uploadMbps.length > 0) return "Last run on this page."
    return "Download, then upload. About five seconds each."
  }

  Component.onCompleted: startSpeedtest()
  Component.onDestruction: {
    if (speedProc.running) speedProc.running = false
  }

  Timer {
    id: speedTimer
    interval: 5000
    repeat: false
    onTriggered: {
      if (speedProc.running) {
        speedExpectedStop = true
        speedProc.running = false
      } else {
        root.finishSpeedPhase()
      }
    }
  }

  Process {
    id: speedProc
    stdout: SplitParser {
      onRead: function(line) {
        var n = RichUi.parseMbpsLine(line)
        if (!isFinite(n)) return
        if (root.speedPhase === "down") root.downloadMbps = String(n)
        else if (root.speedPhase === "up") root.uploadMbps = String(n)
      }
    }
    stderr: StdioCollector {
      id: speedErr
      waitForEnd: true
    }
    onExited: function(code) {
      speedTimer.stop()
      if (root.speedExpectedStop) {
        root.speedExpectedStop = false
        if (root.speedRunning) root.finishSpeedPhase()
        return
      }
      if (code !== 0) {
        root.speedError = String(speedErr.text || "Speed test failed").replace(/^\s+|\s+$/g, "")
        root.speedRunning = false
        root.speedPhase = ""
        return
      }
      root.finishSpeedPhase()
    }
  }

  PrefsGroup {
    title: "Run"
    query: root.query
    detail: "Runs a download sample, then an upload sample. Each direction is about five seconds. Results are megabits per second on the default route. Opening this page starts a run."

    PrefsRow {
      label: "Speed test"
      description: root.phaseLabel()
      hint: "omarchy network speedtest"
      detail: "Runs a download sample, then an upload sample. Each direction is about five seconds. Results are megabits per second on the default route. Cancel stops the current process. Opening this page starts a run."
      query: root.query
      keywords: ["bandwidth", "ping", "speedtest"]

      PrefsButton {
        text: root.speedRunning ? "Cancel" : "Run"
        primary: !root.speedRunning
        danger: root.speedRunning
        enabled: root.speedRunning || Omarchy.netKind !== "disconnected"
        onClicked: root.speedRunning ? root.stopSpeedtest() : root.startSpeedtest()
      }
    }

    PrefsRow {
      stretchControl: true
      label: "Progress"
      description: root.speedPhase === "up" ? "Upload" : (root.speedPhase === "down" ? "Download" : "Idle")
      hint: "omarchy network speedtest"
      query: root.query
      keywords: ["progress"]

      PrefsProgress {
        width: parent.width
        value: root.speedPhase === "down" ? 40 : (root.speedPhase === "up" ? 80 : (root.downloadMbps.length ? 100 : 0))
        indeterminate: root.speedRunning
        valueText: root.speedRunning
          ? (root.speedPhase === "up" ? "Upload" : "Download")
          : (root.downloadMbps.length ? "Finished" : "Waiting")
      }
    }

    PrefsRow {
      available: root.speedError.length > 0
      label: "Error"
      description: root.speedError
      query: root.query
      keywords: ["failed"]
    }
  }

  PrefsGroup {
    title: "Results"
    query: root.query
    detail: "Inbound and outbound rates from the last run on this page. They are megabits per second on the default route."

    PrefsRow {
      label: "Download"
      description: "How fast data came in on the last run."
      hint: "omarchy network speedtest down"
      query: root.query
      keywords: ["down", "mbps"]

      Text {
        text: (root.downloadMbps || "—") + " Mbps"
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.titleSize
        font.bold: true
      }
    }

    PrefsRow {
      label: "Upload"
      description: "How fast data went out on the last run."
      hint: "omarchy network speedtest up"
      query: root.query
      keywords: ["up", "mbps"]

      Text {
        text: (root.uploadMbps || "—") + " Mbps"
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.titleSize
        font.bold: true
      }
    }
  }
}
