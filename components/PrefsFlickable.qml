import QtQuick
import "../services"

// One scrolling behaviour for every scrolling surface in Atmos.
//
// Left to itself a Flickable scrolls a touchpad about a pixel per event,
// which reads as the view being stuck rather than slow. Measuring it on a
// laptop showed the events arriving with a pixel delta of five to eleven,
// so the view was moving a fraction of what the fingers asked for.
//
// The wheel is handled here instead. Note that this is a MouseArea and not
// a WheelHandler: a WheelHandler placed on this Flickable never received a
// single event, while a MouseArea receives all of them.
//
// scrollFactor compensates for Omarchy shipping input.lua with
// scroll_factor = 0.4, which damps the deltas before they ever reach an
// application. Three restores roughly what the fingers moved. It is the
// one number here that is a matter of taste rather than correctness.
//
// The nav list already pinned the flick direction and the content pane did
// not, so the two panes of one window scrolled differently. Pinning it here
// is what makes that consistent.
Flickable {
  id: root

  // How far one mouse-wheel notch travels.
  property real wheelStep: Theme.rowHeight * 3
  // How much of the reported distance a touchpad drag actually moves.
  property real scrollFactor: 3

  contentWidth: width
  flickableDirection: Flickable.VerticalFlick
  boundsBehavior: Flickable.StopAtBounds
  // A pane that already fits should not swallow a drag and rubber-band
  // against its own bounds.
  interactive: contentHeight > height

  // Fills the content item, so it lies under the pointer wherever the view
  // is scrolled to. NoButton means it never takes a press, so every control
  // on the page still works; it is here only for the wheel, which nothing
  // in a settings row consumes.
  MouseArea {
    anchors.fill: parent
    z: -1
    acceptedButtons: Qt.NoButton
    propagateComposedEvents: true
    // An embedded page is not the scroller. Without this it would take the
    // wheel on its way to the pane that actually scrolls.
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
      wheel.accepted = true
    }
  }
}
