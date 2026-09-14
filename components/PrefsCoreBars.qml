import QtQuick
import "../services"
import "../services/Monitor.js" as MonitorJs

Item {
  id: root

  property var values: []
  property string valueText: ""

  implicitWidth: 260
  implicitHeight: Theme.coreBarHeight
  width: implicitWidth
  height: implicitHeight

  Accessible.role: Accessible.StaticText
  Accessible.name: root.valueText.length ? root.valueText : "Per-core load"

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
    antialiasing: false

    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var rects = MonitorJs.barRects(root.values, width, height, 1)
      var i
      var r
      ctx.fillStyle = root.cssColor(Theme.fill(Theme.normalFill))
      ctx.fillRect(0, 0, width, height)
      for (i = 0; i < rects.length; i++) {
        r = rects[i]
        ctx.fillStyle = root.cssColor(r.alert === "hot" ? Theme.urgent : Theme.accent)
        if (r.h > 0) ctx.fillRect(r.x, r.y, Math.max(1, r.w), r.h)
      }
    }
  }

  onValuesChanged: canvas.requestPaint()
}
