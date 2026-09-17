pragma Singleton
import QtQuick
import Quickshell.Io
import "LiveStats.js" as LiveStatsJs
import "Monitor.js" as MonitorJs

QtObject {
  id: root

  property var history: []
  property int intervalMs: 2000
  property bool paused: false
  property string lastError: ""
  // When the sample in flight was started, so a wedged one can be noticed.
  property double startedAt: 0
  // A sample that outlives this many intervals is treated as wedged. Generous
  // on purpose: a loaded machine can take several times the interval to walk
  // /proc, and killing a slow-but-working sample would be worse than waiting.
  readonly property int stallFactor: 6
  readonly property int stallAfterMs: Math.max(15000, root.intervalMs * root.stallFactor)

  readonly property var latest: LiveStatsJs.latest(root.history)
  readonly property int sampleCount: root.history.length
  readonly property bool waiting: root.history.length === 0

  function poll() {
    if (root.paused) return
    // Backpressure: a sample still in flight means the last one has not
    // landed, so skipping this tick is right. What is not right is skipping
    // forever -- see stallTimer.
    if (statsProc.running) return
    root.startedAt = Date.now()
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

  function setIntervalId(id) {
    root.intervalMs = MonitorJs.intervalMs(id)
  }

  function setPaused(on) {
    root.paused = !!on
    if (!root.paused) root.poll()
  }

  function togglePaused() {
    root.setPaused(!root.paused)
  }

  Component.onCompleted: root.poll()

  // Without this, one sample that never finishes stops the dashboard for good:
  // poll() returns early on every later tick, the charts hold the last good
  // numbers, and lastError stays empty because it is only written from
  // onExited. The realistic causes are a blocking read under hwmon and
  // Quickshell not always emitting exited() when a process fails to start,
  // both of which leave running stuck true. Freezing is acceptable; freezing
  // silently is not.
  property Timer stallTimer: Timer {
    interval: 1000
    running: !root.paused
    repeat: true
    onTriggered: {
      if (!statsProc.running || root.startedAt <= 0) return
      if (Date.now() - root.startedAt < root.stallAfterMs) return
      root.lastError = "A live sample stopped responding and was dropped."
      root.startedAt = 0
      statsProc.running = false
    }
  }

  property Timer pollTimer: Timer {
    interval: root.intervalMs
    running: !root.paused
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
      root.startedAt = 0
      if (code === 0) root.adoptSample(statsOut.text)
      else root.lastError = String(statsErr.text || "live-stats.py failed").replace(/^\s+|\s+$/g, "")
    }
  }
}
