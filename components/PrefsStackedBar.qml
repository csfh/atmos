import QtQuick
import "../services"
import "../services/Monitor.js" as MonitorJs
import "../services/RichUi.js" as RichUi

Column {
  id: root

  property var parts: []
  property string valueText: ""

  width: parent ? parent.width : 260
  spacing: Theme.labelGap

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

  function fillFor(id, index) {
    var alphas = [0.88, 0.55, 0.32, 0.14]
    var a = alphas[index] !== undefined ? alphas[index] : 0.2
    if (id === "used") return Theme.accent
    if (id === "free") return Theme.fill(Theme.normalFill)
    return Theme.fill(a)
  }

  Canvas {
    id: canvas
    width: parent.width
    height: Theme.stackedBarHeight
    antialiasing: false

    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var rects = MonitorJs.stackedRects(root.parts, width, height)
      var i
      var r
      ctx.fillStyle = root.cssColor(Theme.fill(Theme.normalFill))
      ctx.fillRect(0, 0, width, height)
      for (i = 0; i < rects.length; i++) {
        r = rects[i]
        ctx.fillStyle = root.cssColor(root.fillFor(r.id, i))
        if (r.w > 0) ctx.fillRect(r.x, r.y, r.w, r.h)
      }
      ctx.strokeStyle = root.cssColor(Theme.borderColor())
      ctx.lineWidth = Theme.borderWidth
      ctx.strokeRect(0.5, 0.5, Math.max(0, width - 1), Math.max(0, height - 1))
    }
  }

  Flow {
    width: parent.width
    spacing: Theme.spaceMd

    Repeater {
      model: root.parts

      Text {
        required property var modelData
        text: (modelData && modelData.label ? modelData.label : "") + " " + RichUi.formatBytes(((modelData && modelData.kb) || 0) * 1024)
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
      }
    }
  }

  onPartsChanged: canvas.requestPaint()
}
