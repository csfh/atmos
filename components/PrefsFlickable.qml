import QtQuick
import "../services"

// One scrolling behaviour for every scrolling surface in Atmos.
//
// Flickable invents momentum. Every scroll event it receives becomes a
// flick with deceleration, so the view keeps sliding after the input has
// stopped. On a touchpad that is the whole problem: the pixel deltas from
// a two-finger drag each start their own flick, the invented momentum from
// one runs into the next, and the result is a view that lags the fingers
// and slides on after they lift. That is the stickiness.
//
// So the wheel is handled here and the flick is cancelled first, which
// leaves the view following the input exactly and stopping when it stops.
//
//   Touchpads report pixelDelta. Those are real distances, so they move
//   the view one to one and the content tracks the fingers.
//
//   Mouse wheels report angleDelta in 120ths of a notch and no pixelDelta,
//   so a notch moves a fixed three rows.
//
// The nav list already pinned the flick direction and the content pane did
// not, so the two panes of one window scrolled differently. Pinning it here
// is what makes that consistent.
Flickable {
  id: root

  // How far one mouse-wheel notch travels.
  property real wheelStep: Theme.rowHeight * 3

  contentWidth: width
  flickableDirection: Flickable.VerticalFlick
  boundsBehavior: Flickable.StopAtBounds
  // A pane that already fits should not swallow a drag and rubber-band
  // against its own bounds.
  interactive: contentHeight > height

  WheelHandler {
    // An embedded page is not the scroller; without this it would eat the
    // wheel on its way to the pane that actually scrolls.
    enabled: root.interactive

    onWheel: function(event) {
      var max = Math.max(0, root.contentHeight - root.height)
      if (max <= 0) return

      // A touchpad sends a distance. A wheel sends notches.
      var dy = event.pixelDelta.y
      if (dy === 0) dy = (event.angleDelta.y / 120) * root.wheelStep
      if (dy === 0) return

      root.cancelFlick()
      root.contentY = Math.max(0, Math.min(max, root.contentY - dy))
      // Without this the Flickable handles the same event again and starts
      // the flick this handler exists to avoid.
      event.accepted = true
    }
  }
}
