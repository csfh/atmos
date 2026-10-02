pragma Singleton
import QtQuick
import "Feedback.js" as FeedbackJs

// Write feedback shared by the header chip and the rows. Omarchy reports
// begin/finish around every write; rows call touch() when the user acts.
Item {
  id: root

  property var fb: FeedbackJs.empty()
  readonly property string phase: fb.phase
  readonly property string rowKey: fb.rowKey
  readonly property string message: fb.message
  readonly property string chip: FeedbackJs.chipLabel(fb)

  function touch(hub, label) {
    root.fb = FeedbackJs.touch(root.fb, FeedbackJs.rowKey(hub, label), Date.now())
  }

  function begin() {
    root.fb = FeedbackJs.begin(root.fb, Date.now())
  }

  function finish(ok, message) {
    root.fb = FeedbackJs.finish(root.fb, ok, message, Date.now())
    fade.restart()
  }

  function dismiss() {
    root.fb = FeedbackJs.dismiss(root.fb)
  }

  function rowStatus(hub, label) {
    return FeedbackJs.rowStatus(root.fb, FeedbackJs.rowKey(hub, label))
  }

  Timer {
    id: fade
    interval: FeedbackJs.SAVED_MS
    onTriggered: root.fb = FeedbackJs.expire(root.fb, Date.now())
  }
}
