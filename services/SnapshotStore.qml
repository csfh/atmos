pragma Singleton
import QtQuick
import "Requests.js" as Requests
import "SnapshotGroups.js" as SnapshotGroups

// Asks the daemon to read settings topics again (docs/adr/0011). What a
// topic shows is held and pushed by the daemon (SnapshotFeed); this is the
// Refresh button's side of it: it batches the asks of a moment into one.
QtObject {
  id: root

  property var pending: []

  function scheduleRefresh(group) {
    root.pending = root.pending.concat([SnapshotGroups.normalizeGroup(group)])
    refreshTimer.restart()
  }

  property Timer refreshTimer: Timer {
    interval: 180
    repeat: false
    onTriggered: {
      var topics = SnapshotGroups.topicsForGroups(root.pending)
      root.pending = []
      Backend.request(Requests.snapshotInvalidate(topics), function() {})
    }
  }
}
