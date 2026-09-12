import QtQuick
import QtQuick.Window
import "../services"
import "../services/Chamfer.js" as ChamferJs

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
    antialiasing: true
    readonly property real dpr: {
      var win = Window.window
      if (win && win.devicePixelRatio > 0) return win.devicePixelRatio
      if (typeof Screen !== "undefined" && Screen.devicePixelRatio > 0) return Screen.devicePixelRatio
      return 1
    }
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onDprChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var w = width
      var h = height
      var pts = ChamferJs.pathPoints(w, h, root.cut, {
        cutTopLeft: root.cutTopLeft,
        cutTopRight: root.cutTopRight,
        cutBottomRight: root.cutBottomRight,
        cutBottomLeft: root.cutBottomLeft
      }, root.strokeWidth, dpr)
      if (pts.length === 0) return

      ctx.beginPath()
      ctx.moveTo(pts[0][0], pts[0][1])
      var i
      for (i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1])
      ctx.closePath()
      ctx.lineJoin = "miter"
      ctx.miterLimit = 2

      if (ChamferJs.shouldPaint(root.fillColor)) {
        ctx.fillStyle = ChamferJs.cssColor(root.fillColor)
        ctx.fill()
      }
      var metrics = ChamferJs.strokeMetrics(root.strokeWidth, dpr)
      if (metrics.width > 0 && ChamferJs.shouldPaint(root.strokeColor)) {
        ctx.lineWidth = metrics.width
        ctx.strokeStyle = ChamferJs.cssColor(root.strokeColor)
        ctx.stroke()
      }
    }
  }
}
