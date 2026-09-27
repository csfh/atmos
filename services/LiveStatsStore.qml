pragma Singleton
import QtQuick
import Quickshell.Io
import "LiveStats.js" as LiveStatsJs
import "Monitor.js" as MonitorJs

QtObject {
  id: root

  property var history: []
  property int intervalMs: 2000
  property int userIntervalMs: 2000
  property var intervalHolds: ({})
  property bool paused: false
  property string lastError: ""

  readonly property var latest: LiveStatsJs.latest(root.history)
  readonly property int sampleCount: root.history.length
  readonly property bool waiting: root.history.length === 0

  function poll() {
    if (root.paused) return
    if (statsProc.running) return
    statsProc.running = true
  }

  function adoptSample(text) {
    var parsed = LiveStatsJs.parse(text)
    if (!parsed) {
      root.lastError = "Could not parse a live sample."
      return
    }
    root.lastError = ""
    root.history = LiveStatsJs.pushSample(root.history, parsed, Date.now())
  }

  function applyInterval() {
    root.intervalMs = MonitorJs.applySampleInterval(root.userIntervalMs, root.intervalHolds)
  }

  function setIntervalId(id) {
    var ms = MonitorJs.intervalMs(id)
    root.userIntervalMs = ms
    root.intervalHolds = MonitorJs.syncIntervalHolds(root.intervalHolds, ms)
    root.applyInterval()
  }

  function setPaused(on) {
    root.paused = !!on
    if (!root.paused) root.poll()
  }

  function togglePaused() {
    root.setPaused(!root.paused)
  }

  Component.onCompleted: root.poll()

  property Timer pollTimer: Timer {
    interval: root.intervalMs
    running: !root.paused && FrameClock.holds > 0
    repeat: true
    onTriggered: root.poll()
  }

  property Process statsProc: Process {
    command: ["python3", Omarchy.liveStatsScript]
    stdout: StdioCollector {
      id: statsOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: statsErr
      waitForEnd: true
    }
    onExited: function(code) {
      if (code === 0) root.adoptSample(statsOut.text)
      else root.lastError = String(statsErr.text || "live-stats.py failed").replace(/^\s+|\s+$/g, "")
    }
  }
}
