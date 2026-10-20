pragma Singleton
import QtQuick
import "SnapshotGroups.js" as SnapshotGroups

// The settings snapshot is held, not read (docs/adr/0011). The daemon collects
// each topic when a window first holds it and again when something that shows
// in it changes, and pushes it. This applies what arrives, except for keys the
// window wrote after the sample was taken (Ops.dropStale), and keeps what a
// producer could not read as the value it had, with the reason.
QtObject {
  id: root

  property bool started: false
  property var heard: ({})
  // Why each key a producer could not read is stale, by topic.
  property var failedByTopic: ({})

  function start() {
    if (root.started) return
    root.started = true
    Backend.sample.connect(root.adopt)
    var topics = {}
    var i
    for (i = 0; i < SnapshotGroups.TOPICS.length; i++) topics[SnapshotGroups.TOPICS[i]] = {}
    Backend.subscribe("settings", topics)
  }

  function adopt(kind, frame) {
    if (kind.indexOf("settings.") !== 0 || !frame) return
    if (frame.error) {
      // The topic could not be collected: what is shown stays, and the
      // Diagnostics page says why (an error dialog on every retry would be noise).
      var failedTopics = root.failedByTopic
      var why = {}
      why[kind] = String(frame.error)
      failedTopics[kind] = why
      root.failedByTopic = failedTopics
      Omarchy.snapshotFailed = root.merged()
      root.noteHeard(kind)
      return
    }
    // A frame crossed a signal, which hands its arrays back wrapped.
    var doc = JSON.parse(JSON.stringify(frame.result))
    var failed = doc._failed || ({})
    delete doc._failed
    var all = root.failedByTopic
    all[kind] = failed
    root.failedByTopic = all
    Omarchy.snapshotFailed = root.merged()
    Omarchy.applySnapshot(Ops.dropStale(doc, Number(frame.sampledAt)))
    root.noteHeard(kind)
  }

  function merged() {
    var out = {}
    var topic, key
    for (topic in root.failedByTopic)
      for (key in root.failedByTopic[topic]) out[key] = root.failedByTopic[topic][key]
    return out
  }

  function noteHeard(kind) {
    var seen = root.heard
    seen[kind] = true
    root.heard = seen
    var i
    for (i = 0; i < SnapshotGroups.TOPICS.length; i++) if (!seen[SnapshotGroups.TOPICS[i]]) return
    Omarchy.snapshotReady = true
  }
}
