import QtQuick
import "../services"

// One scrolling behaviour for every scrolling surface in Atmos.
//
// Left to itself a Flickable scrolls a touchpad about a pixel per event,
// which reads as the view being stuck rather than slow. Measuring it on a
// laptop showed the events arriving with a pixel delta of five to eleven,
// so the view was moving a fraction of what the fingers asked for.
//
// The wheel is handled here instead. Note that this is a MouseArea on the
// Flickable viewport and not a WheelHandler: a WheelHandler placed as a
// child of this Flickable never received a single event, while a MouseArea
// receives them.
//
// The handler sits on the viewport (parent: root), not contentItem. Flickable
// reparents children onto contentItem, so a z: -1 MouseArea there only gets
// the wheel where nothing else wants it. Nav rows, search hits, toggles,
// and sliders all have their own MouseAreas that would swallow the event.
//
// scrollFactor defaults to 1 so Input → Scroll speed remains the only
// speed control. pixelDelta already includes compositor scroll_factor.
//
// The nav list already pinned the flick direction and the content pane did
// not, so the two panes of one window scrolled differently. Pinning it here
// is what makes that consistent.
Flickable {
  id: root

  // How far one mouse-wheel notch travels.
  property real wheelStep: Theme.rowHeight * 3
  // How much of the reported distance a touchpad drag actually moves.
  // 1 keeps Input → Scroll speed as the only speed control; pixelDelta
  // already includes compositor scroll_factor.
  property real scrollFactor: 1

  // Coasting after the fingers lift. Setting contentY directly, which is
  // what stops the view lagging the fingers, also means it stops dead the
  // moment the events do. That reads as correct but tight: a touchpad
  // throw should carry.
  //
  // The original bug was Flickable inventing momentum from every single
  // event, so successive invented flicks fought each other. This is the
  // other thing: the velocity actually in the gesture, handed to Flickable
  // once, when the gesture is over. Tracking stays one to one while the
  // fingers are down.
  //
  // Set to 0 to keep the view tight.
  property real momentum: 1
  // A gesture is over when the events stop. Wayland does send a scroll-end,
  // but not every device and driver produces one, so the quiet gap is what
  // is actually reliable.
  property int gestureGapMs: 60
  // Below this a throw was really a nudge, and coasting would overshoot the
  // row someone was aiming at.
  property real minFlickVelocity: 120

  // Recent (timestamp, distance) pairs, newest last.
  property var _samples: []

  contentWidth: width
  flickableDirection: Flickable.VerticalFlick
  boundsBehavior: Flickable.StopAtBounds
  // A pane that already fits should not swallow a drag and rubber-band
  // against its own bounds.
  interactive: contentHeight > height

  // Distance travelled recently, for working out how hard it was thrown.
  function noteSample(dy) {
    if (root.momentum <= 0) return
    var now = Date.now()
    var kept = []
    var recent = root._samples
    for (var i = 0; i < recent.length; i++) {
      if (now - recent[i].t <= 120) kept.push(recent[i])
    }
    kept.push({ t: now, dy: dy })
    root._samples = kept
    coastTimer.restart()
  }

  // Pixels per second across the samples still in the window, which is what
  // Flickable's flick() wants.
  function gestureVelocity() {
    var recent = root._samples
    if (recent.length < 2) return 0
    var span = recent[recent.length - 1].t - recent[0].t
    if (span <= 0) return 0
    var travelled = 0
    // Distance after the first sample: span starts at recent[0].t, so
    // including that dy would treat it as occurring in zero time.
    for (var i = 1; i < recent.length; i++) travelled += recent[i].dy
    return (travelled / span) * 1000
  }

  Timer {
    id: coastTimer
    interval: root.gestureGapMs
    onTriggered: {
      var v = root.gestureVelocity() * root.momentum
      root._samples = []
      if (Math.abs(v) < root.minFlickVelocity) return
      // Flickable's own physics from here, so it decelerates and stops at
      // the bounds the same way a drag does.
      root.flick(0, v)
    }
  }

  // On the viewport, not contentItem. NoButton means it never takes a
  // press, so every control on the page still works. Disabled when the
  // Flickable is not the scroller so an embedded page does not take the
  // wheel on its way to the pane that does.
  MouseArea {
    parent: root
    // Flickable still resolves anchors against contentItem, so filling
    // the viewport that way warns and leaves this 0×0. Bind size instead.
    width: root.width
    height: root.height
    acceptedButtons: Qt.NoButton
    enabled: root.interactive

    onWheel: function(wheel) {
      var max = Math.max(0, root.contentHeight - root.height)
      if (max <= 0) {
        wheel.accepted = false
        return
      }
      // A touchpad reports a distance. A wheel reports notches and no
      // distance at all.
      var dy = wheel.pixelDelta.y * root.scrollFactor
      if (wheel.pixelDelta.y === 0) dy = (wheel.angleDelta.y / 120) * root.wheelStep
      if (dy === 0) {
        wheel.accepted = false
        return
      }
      root.cancelFlick()
      root.contentY = Math.max(0, Math.min(max, root.contentY - dy))
      if (wheel.pixelDelta.y !== 0)
        root.noteSample(dy)
      wheel.accepted = true
    }
  }
}
