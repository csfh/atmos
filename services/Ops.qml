pragma Singleton
import QtQuick
import "Failure.js" as FailureJs
import "Snapshot.js" as SnapshotJs

// Typed writes. The daemon runs each one on its own lane, in order with the
// writes and reads around it (docs/architecture.md, the command bus), and the
// reply carries the state the machine is now in. The window shows that state;
// it never patches its own snapshot with a guess, which is what once emptied
// the monitor list.
//
// run(opts): opts.op is a body from Requests.js, opts.key names what is being
// written. Calls for one key may overlap, as a dragged slider's do; only the
// newest one's reply is applied, since the daemon collapses the rest into it.
QtObject {
  id: root

  property var sent: ({})

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
      if (snapshot && root.sent[key] === seq) Omarchy.applySnapshot(SnapshotJs.plain(snapshot))
    })
  }
}
