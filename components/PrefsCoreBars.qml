import QtQuick
import "../services"
import "../services/Charts.js" as ChartsJs
import "../services/Monitor.js" as MonitorJs

Item {
  id: root

  property var values: []
  property string valueText: ""
  property bool tween: true
  property int tweenMs: LiveStatsStore.intervalMs
  property real tweenT: 1
  property real tweenAt: 0
  property var fromValues: []
  property var toValues: []
  readonly property bool inView: view.inView

  ChartViewport {
    id: view
    target: root
  }

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

    onWidthChanged: root.paintCanvas()
    onHeightChanged: root.paintCanvas()

    onPaint: {
      if (!root.inView) return
      var ctx = getContext("2d")
      ctx.reset()
      var shown = root.tween
        ? ChartsJs.lerpSeries(root.fromValues, root.toValues, root.tweenT)
        : root.values
      var rects = MonitorJs.barRects(shown, width, height, 1)
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

  function paintCanvas() {
    if (!root.inView) return
    canvas.requestPaint()
  }

  function retarget() {
    if (!root.inView) return
    if (root.tween && root.toValues && root.toValues.length) {
      root.fromValues = ChartsJs.lerpSeries(root.fromValues, root.toValues, root.tweenT)
      root.toValues = root.values
      root.tweenT = 0
      root.tweenAt = Date.now()
    } else {
      root.fromValues = root.values
      root.toValues = root.values
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
  onValuesChanged: root.retarget()
  onInViewChanged: if (root.inView) root.retarget()
  Component.onCompleted: root.retarget()
}
