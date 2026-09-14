import QtQuick
import "../services"
import "../services/Monitor.js" as MonitorJs

Column {
  id: root

  property var bins: []
  property string valueText: ""

  width: parent ? parent.width : 260
  spacing: Theme.labelGap

  Accessible.role: Accessible.StaticText
  Accessible.name: root.valueText.length ? root.valueText : "CPU histogram"

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
    width: parent.width
    height: Theme.histogramHeight
    antialiasing: false

    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var rects = MonitorJs.histogramRects(root.bins, width, height)
      var i
      var r
      ctx.fillStyle = root.cssColor(Theme.fill(Theme.normalFill))
      ctx.fillRect(0, 0, width, height)
      for (i = 0; i < rects.length; i++) {
        r = rects[i]
        ctx.fillStyle = root.cssColor(r.id === "hot" ? Theme.urgent : Theme.accent)
        if (r.h > 0) ctx.fillRect(r.x, r.y, Math.max(1, r.w), r.h)
      }
    }
  }

  Flow {
    width: parent.width
    spacing: Theme.spaceMd

    Repeater {
      model: root.bins

      Text {
        required property var modelData
        text: (modelData && modelData.label ? modelData.label : "") + " " + (modelData && modelData.count ? modelData.count : 0)
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
      }
    }
  }

  onBinsChanged: canvas.requestPaint()
}
