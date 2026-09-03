import QtQuick
import "../services"

// One scrolling behaviour for every scrolling surface in Atmos.
//
// A bare Flickable turns each mouse-wheel notch into its own decelerating
// flick. Roll the wheel a few notches and those animations stack and fight
// each other, which is the sticky, rubbery feel. A notch should move the
// view a fixed distance and stop, so the wheel is handled here and any
// flick still running is cancelled first.
//
// Touchpads are deliberately left alone. They send pixel deltas and want
// the kinetic handling Flickable already does well; taking that over would
// make them worse rather than better.
//
// The nav list already pinned the flick direction and the content pane did
// not, so the two panes of the same window scrolled differently. Pinning it
// here is what makes that consistent.
Flickable {
  id: root

  // How far one wheel notch travels. Three rows is the usual desktop step.
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
    acceptedDevices: PointerDevice.Mouse

    onWheel: function(event) {
      var notches = event.angleDelta.y / 120
      if (notches === 0) return
      var max = Math.max(0, root.contentHeight - root.height)
      if (max <= 0) return
      root.cancelFlick()
      root.contentY = Math.max(0, Math.min(max, root.contentY - notches * root.wheelStep))
    }
  }
}
