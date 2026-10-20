pragma Singleton
import QtQuick

// The five inventories are held, not asked for: the daemon pushes each one
// when it is collected and when a window first holds it, and a failed
// collection arrives as an error frame that leaves what is shown.
QtObject {
  id: root

  property bool started: false

  function start() {
    if (root.started) return
    root.started = true
    Backend.sample.connect(root.adopt)
    Backend.subscribe("displays", { hardware: {}, disks: {}, services: {}, software: {}, diagnostics: {} })
  }

  function adopt(kind, frame) {
    if (kind === "live" || !frame || frame.error) return
    // A frame crossed a signal, which hands its arrays back wrapped.
    var one = {}
    one[kind] = JSON.parse(JSON.stringify(frame.result))
    Omarchy.applyDisplays(one)
  }
}
