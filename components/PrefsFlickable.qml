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

  contentWidth: width
  flickableDirection: Flickable.VerticalFlick
  boundsBehavior: Flickable.StopAtBounds
  // A pane that already fits should not swallow a drag and rubber-band
  // against its own bounds.
  interactive: contentHeight > height

  // On the viewport, not contentItem. NoButton means it never takes a
  // press, so every control on the page still works. Disabled when the
  // Flickable is not the scroller so an embedded page does not take the
  // wheel on its way to the pane that does.
  MouseArea {
    parent: root
    anchors.fill: root
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
      wheel.accepted = true
    }
  }
}
