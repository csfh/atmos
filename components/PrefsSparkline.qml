import QtQuick
import "../services"
import "../services/LiveStats.js" as LiveStatsJs

Item {
  id: root

  property var values: []
  property var valuesB: []
  property string valueText: ""
  property bool alert: false
  property bool fill: true

  implicitWidth: 260
  implicitHeight: Theme.sparklineHeight
  width: implicitWidth
  height: implicitHeight

  Accessible.role: Accessible.StaticText
  Accessible.name: root.valueText

  function cssColor(color) {
    if (color && typeof color === "object" && color.r !== undefined) {
      var r = Math.round(Number(color.r) * 255)
      var g = Math.round(Number(color.g) * 255)
      var b = Math.round(Number(color.b) * 255)
      var a = color.a !== undefined ? Number(color.a) : 1
      if (!isFinite(r) || !isFinite(g) || !isFinite(b)) return "transparent"
      if (!isFinite(a)) a = 1
      return "rgba(" + r + ", " + g + ", " + b + ", " + a + ")"
    }
    return String(color || "transparent")
  }

  Canvas {
    id: canvas
    anchors.fill: parent
    antialiasing: true

    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var pts = LiveStatsJs.sparklinePoints(root.values, width, height, 1)
      var ptsB = LiveStatsJs.sparklinePoints(root.valuesB, width, height, 1)
      var i
      var stroke = root.alert ? Theme.urgent : Theme.accent
      if (root.fill && pts.length >= 2) {
        ctx.beginPath()
        ctx.moveTo(pts[0][0], pts[0][1])
        for (i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1])
        ctx.lineTo(pts[pts.length - 1][0], height)
        ctx.lineTo(pts[0][0], height)
        ctx.closePath()
        ctx.fillStyle = root.cssColor(Theme.fill(Theme.normalFill))
        ctx.fill()
      }
      function strokePts(list, color) {
        if (list.length < 2) return
        ctx.beginPath()
        ctx.moveTo(list[0][0], list[0][1])
        var n
        for (n = 1; n < list.length; n++) ctx.lineTo(list[n][0], list[n][1])
        ctx.lineWidth = Theme.borderWidth
        ctx.strokeStyle = root.cssColor(color)
        ctx.lineJoin = "miter"
        ctx.stroke()
      }
      strokePts(pts, stroke)
      strokePts(ptsB, Theme.muted)
    }
  }

  onValuesChanged: canvas.requestPaint()
  onValuesBChanged: canvas.requestPaint()
  onAlertChanged: canvas.requestPaint()
  onFillChanged: canvas.requestPaint()
}
