import QtQuick
import "../services"

// A chamfered panel: corners cut at 45 degrees rather than rounded.
//
// Theme.qml is explicit that cards must not be rounded. This respects that
// and is not a workaround for it -- the chamfer is the shape language
// Omarchy already uses in its own brand marks, so it reads as house style
// rather than as decoration. Hairline border, no shadow, no gradient.
//
// Canvas rather than QtQuick.Shapes so there is no extra module dependency,
// and it repaints only when geometry or colour actually changes, never per
// frame.

Item {
  id: root

  property color fillColor: "transparent"
  property color strokeColor: "transparent"
  property int strokeWidth: Theme.borderWidth
  // Which corners to cut. Top-left and bottom-right by default: a diagonal
  // pair reads as deliberate, all four reads as an octagon.
  property bool cutTopLeft: true
  property bool cutTopRight: false
  property bool cutBottomRight: true
  property bool cutBottomLeft: false
  property int cut: Theme.chamfer

  onFillColorChanged: canvas.requestPaint()
  onStrokeColorChanged: canvas.requestPaint()
  onStrokeWidthChanged: canvas.requestPaint()
  onCutChanged: canvas.requestPaint()
  onCutTopLeftChanged: canvas.requestPaint()
  onCutTopRightChanged: canvas.requestPaint()
  onCutBottomRightChanged: canvas.requestPaint()
  onCutBottomLeftChanged: canvas.requestPaint()

  Canvas {
    id: canvas
    anchors.fill: parent
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var w = width
      var h = height
      if (w <= 0 || h <= 0) return

      // Inset by half the stroke so the hairline lands on the pixel grid
      // instead of straddling it and rendering as a soft two-pixel smear.
      var s = root.strokeWidth
      var o = s > 0 ? s / 2 : 0
      var c = Math.max(0, Math.min(root.cut, Math.min(w, h) / 2 - o))

      var l = o
      var t = o
      var r = w - o
      var b = h - o

      ctx.beginPath()
      ctx.moveTo(l + (root.cutTopLeft ? c : 0), t)
      ctx.lineTo(r - (root.cutTopRight ? c : 0), t)
      if (root.cutTopRight) ctx.lineTo(r, t + c)
      ctx.lineTo(r, b - (root.cutBottomRight ? c : 0))
      if (root.cutBottomRight) ctx.lineTo(r - c, b)
      ctx.lineTo(l + (root.cutBottomLeft ? c : 0), b)
      if (root.cutBottomLeft) ctx.lineTo(l, b - c)
      ctx.lineTo(l, t + (root.cutTopLeft ? c : 0))
      ctx.closePath()

      if (Qt.colorEqual(root.fillColor, "transparent") === false) {
        ctx.fillStyle = root.fillColor
        ctx.fill()
      }
      if (s > 0 && Qt.colorEqual(root.strokeColor, "transparent") === false) {
        ctx.lineWidth = s
        ctx.strokeStyle = root.strokeColor
        ctx.stroke()
      }
    }
  }
}
