pragma Singleton
import QtQuick
import "LiveStats.js" as LiveStatsJs
import "Monitor.js" as MonitorJs

QtObject {
  id: root

  property var history: []
  property int intervalMs: 2000
  property int userIntervalMs: 2000
  property var intervalHolds: ({})
  property bool paused: false
  // Pages that read samples without drawing a chart hold the feed open here,
  // instead of retaining FrameClock, which would also run the vsync tick.
  property int pollHolds: 0
  property string lastError: ""
  // The newest sample taken in, by the time the daemon began collecting it.
  property double lastAt: 0
  // The first sample is taken once even when nothing holds the feed, so the
  // store is not empty when a page opens.
  property bool primed: false

  readonly property var latest: LiveStatsJs.latest(root.history)
  readonly property int sampleCount: root.history.length
  readonly property bool waiting: root.history.length === 0

  // The feed runs while a chart is in view or a page holds it, and not while
  // paused. The daemon samples on the quickest holder's interval and pushes;
  // nothing here asks.
  readonly property bool wanted: !root.paused && (FrameClock.holds > 0 || root.pollHolds > 0 || !root.primed)

  function syncFeed() {
    Backend.subscribe("live", root.wanted ? { live: { intervalMs: root.intervalMs } } : ({}))
  }

  onWantedChanged: root.syncFeed()
  onIntervalMsChanged: root.syncFeed()

  // A frame is { kind, sampledAt, result } or { kind, stale, error }. Rates
  // run on sampledAt, the time the daemon began collecting, so a late or
  // bunched frame is still measured correctly.
  function adoptFrame(kind, frame) {
    if (kind !== "live" || !frame) return
    if (frame.error) {
      root.lastError = String(frame.error)
      return
    }
    var at = Number(frame.sampledAt)
    // The same sample twice would read as a rate of zero.
    if (!(at > root.lastAt)) return
    // A frame crossed a signal, which hands its arrays back wrapped; the text
    // round trip restores real ones, as the reply callback used to give.
    var parsed = LiveStatsJs.parse(JSON.stringify(frame.result))
    if (!parsed) {
      root.lastError = "Could not parse a live sample."
      return
    }
    root.lastError = ""
    root.lastAt = at
    root.history = LiveStatsJs.pushSample(root.history, parsed, at)
    root.primed = true
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

  function holdPoll() {
    root.pollHolds += 1
  }

  function releasePoll() {
    if (root.pollHolds > 0) root.pollHolds -= 1
  }

  function setPaused(on) {
    root.paused = !!on
  }

  function togglePaused() {
    root.setPaused(!root.paused)
  }

  Component.onCompleted: {
    Backend.sample.connect(root.adoptFrame)
    root.syncFeed()
  }
}
