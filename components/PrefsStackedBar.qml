import QtQuick
import "../services"
import "../services/Charts.js" as ChartsJs
import "../services/Monitor.js" as MonitorJs
import "../services/RichUi.js" as RichUi

Column {
  id: root

  property var parts: []
  property string valueText: ""
  property bool tween: true
  property int tweenMs: LiveStatsStore.intervalMs
  property real tweenT: 1
  property real tweenAt: 0
  property var fromParts: []
  property var toParts: []
  readonly property bool inView: view.inView

  ChartViewport {
    id: view
    target: root
  }

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

    onWidthChanged: root.paintCanvas()
    onHeightChanged: root.paintCanvas()

    onPaint: {
      if (!root.inView) return
      var ctx = getContext("2d")
      ctx.reset()
      var shown = root.tween
        ? ChartsJs.lerpKeyed(root.fromParts, root.toParts, root.tweenT, ["kb"])
        : root.parts
      var rects = MonitorJs.stackedRects(shown, width, height)
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

  function paintCanvas() {
    if (!root.inView) return
    canvas.requestPaint()
  }

  function retarget() {
    if (!root.inView) return
    if (root.tween && root.toParts && root.toParts.length) {
      root.fromParts = ChartsJs.lerpKeyed(root.fromParts, root.toParts, root.tweenT, ["kb"])
      root.toParts = root.parts
      root.tweenT = 0
      root.tweenAt = Date.now()
    } else {
      root.fromParts = root.parts
      root.toParts = root.parts
      root.tweenT = 1
    }
    root.paintCanvas()
  }

  function stepFrame() {
    if (!root.inView || !root.tween || root.tweenT >= 1) return
    var t = ChartsJs.tweenProgress(root.tweenAt, FrameClock.nowMs, Math.max(80, root.tweenMs))
    if (t === root.tweenT) return
    root.tweenT = t
    root.paintCanvas()
  }

  Connections {
    target: FrameClock
    enabled: root.inView && root.tween && root.tweenT < 1
    function onFrameChanged() { root.stepFrame() }
  }
  onPartsChanged: root.retarget()
  onInViewChanged: if (root.inView) root.retarget()
  Component.onCompleted: root.retarget()
}
