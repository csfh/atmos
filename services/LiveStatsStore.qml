pragma Singleton
import QtQuick
import "Failure.js" as FailureJs
import "LiveStats.js" as LiveStatsJs
import "Monitor.js" as MonitorJs
import "Requests.js" as Requests

QtObject {
  id: root

  property var history: []
  property int intervalMs: 2000
  property int userIntervalMs: 2000
  property var intervalHolds: ({})
  property bool paused: false
  property string lastError: ""
  // When the sample in flight was started, so a wedged one can be noticed.
  property double startedAt: 0
  property bool inFlight: false
  // Each sample carries a number. A stalled sample is dropped by moving on to
  // the next number, so its late answer cannot overwrite the stall message.
  property int sampleToken: 0
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
    if (root.inFlight) return
    root.inFlight = true
    root.startedAt = Date.now()
    root.sampleToken += 1
    var token = root.sampleToken
    Backend.request(Requests.displayGet("live"), function(env) {
      if (token !== root.sampleToken) return
      root.inFlight = false
      root.startedAt = 0
      if (env && env.ok === true && env.result) root.adoptSample(JSON.stringify(env.result))
      else root.lastError = FailureJs.errorText(env && env.error) || "live-stats.py failed"
    })
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

  // Without this, one sample that never finishes stops the dashboard for good:
  // poll() returns early on every later tick, the charts hold the last good
  // numbers, and lastError stays empty because it is only written from
  // an answer lands. The realistic cause is a blocking read under hwmon.
  // Freezing is acceptable; freezing silently is not. The backend gives the
  // sampler its own time limit; this drops the request on our side and says so.
  property Timer stallTimer: Timer {
    interval: 1000
    running: !root.paused
    repeat: true
    onTriggered: {
      if (!root.inFlight || root.startedAt <= 0) return
      if (Date.now() - root.startedAt < root.stallAfterMs) return
      root.lastError = "A live sample stopped responding and was dropped."
      root.startedAt = 0
      root.inFlight = false
      root.sampleToken += 1
    }
  }

  property Timer pollTimer: Timer {
    interval: root.intervalMs
    running: !root.paused && FrameClock.holds > 0
    repeat: true
    onTriggered: root.poll()
  }
}
