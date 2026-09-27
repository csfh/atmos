pragma Singleton
import QtQuick

// One vsync tick for live charts that are on screen. Charts retain while
// in view, then sample FrameClock.nowMs to lerp. Idle charts unbind the frame.
QtObject {
  id: root

  property int holds: 0
  property int frame: 0
  property real nowMs: 0
  property real dtMs: 16.67
  readonly property bool active: root.holds > 0 && !LiveStatsStore.paused

  function retain() {
    root.holds += 1
  }

  function release() {
    if (root.holds > 0) root.holds -= 1
  }

  property FrameAnimation tick: FrameAnimation {
    running: root.active
    onTriggered: {
      root.dtMs = Math.max(1, frameTime * 1000)
      root.nowMs = Date.now()
      root.frame = currentFrame
    }
  }
}
