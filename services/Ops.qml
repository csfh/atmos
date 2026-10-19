pragma Singleton
import QtQuick
import "Failure.js" as FailureJs
import "Requests.js" as Requests
import "Snapshot.js" as SnapshotJs
import "SnapshotGroups.js" as SnapshotGroups

// Typed writes. The daemon runs each one on its own lane, in order with the
// writes and reads around it (docs/architecture.md, the command bus), and the
// reply carries the state the machine is now in. The window shows that state;
// it never patches its own snapshot with a guess, which is what once emptied
// the monitor list.
//
// run(opts): opts.op is a body from Requests.js (Requests.write for the named
// writes in docs/protocol.md), opts.key names what is being written. Calls for one key may overlap, as a dragged slider's do; only the
// newest one's reply is applied, since the daemon collapses the rest into it.
QtObject {
  id: root

  property var sent: ({})
  // When each snapshot key was last set from a write's answer.
  property var wrote: ({})

  // A full read that began before a write can finish after it, carrying the
  // value from before. These keys were written since the read began, so they
  // are not taken from it.
  function dropStale(values, since) {
    var out = {}
    var key
    for (key in values) if (!(root.wrote[key] > since)) out[key] = values[key]
    return out
  }

  function noteWritten(values) {
    var now = Date.now()
    var key
    for (key in values) root.wrote[key] = now
  }

  function run(opts) {
    var key = String(opts.key || opts.op.op)
    var seq = (root.sent[key] || 0) + 1
    root.sent[key] = seq
    Feedback.begin()
    Backend.request(opts.op, function(env) {
      if (!env || env.ok !== true) {
        var text = FailureJs.errorText(env && env.error) || "Command failed"
        Omarchy.lastError = text
        Feedback.finish(false, text)
        return
      }
      Feedback.finish(true, "")
      var snapshot = env.result && env.result.snapshot
      // A newer write for this key is on its way; its reply is the one to show.
      if (snapshot && root.sent[key] === seq) {
        root.noteWritten(snapshot)
        Omarchy.applySnapshot(SnapshotJs.plain(snapshot))
        // The input file may carry a gesture the snapshot keys do not say.
        if (snapshot.hyprInput !== undefined) Omarchy.applyHyprWorkspaceGestureFromFile()
      }
      // The machine had not shown the change when the backend stopped waiting
      // (a radio or a device can take longer): read the group again, so the
      // answer that comes later still gets to the window.
      if (env.result && env.result.settled === false) {
        var late = snapshot ? SnapshotGroups.groupForKey(Object.keys(snapshot)[0]) : ""
        if (late) IoQueue.enqueueRead(late)
      }
      var refresh = Omarchy.snapshotRefreshGroup(opts.refresh)
      if (refresh !== "none") IoQueue.enqueueRead(refresh)
    })
  }

  // Show what the machine reports now for some keys, once a write the window
  // did not run itself (an interactive one) has finished. spec is
  // { keys, expect }: expect names values to wait for, as a write's settle does.
  function read(spec) {
    if (!spec || !(spec.keys instanceof Array) || spec.keys.length === 0) return
    Backend.request(Requests.snapshotKeys(spec.keys, spec.expect), function(env) {
      if (env && env.ok === true && env.result && env.result.values) {
        root.noteWritten(env.result.values)
        Omarchy.applySnapshot(SnapshotJs.plain(env.result.values))
      }
    })
  }
}
