import QtQuick
import "../services"
import "../services/Charts.js" as ChartsJs
import "../services/LiveStats.js" as LiveStatsJs

Item {
  id: root

  property var values: []
  property var valuesB: []
  property string valueText: ""
  property bool alert: false
  property bool fill: true
  property bool tween: true
  property int tweenMs: LiveStatsStore.intervalMs
  property real tweenT: 1
  property real tweenAt: 0
  property var fromValues: []
  property var toValues: []
  property var fromValuesB: []
  property var toValuesB: []
  readonly property bool inView: view.inView

  ChartViewport {
    id: view
    target: root
  }

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

    onWidthChanged: root.paintCanvas()
    onHeightChanged: root.paintCanvas()

    onPaint: {
      if (!root.inView) return
      var ctx = getContext("2d")
      ctx.reset()
      var shown = root.tween
        ? ChartsJs.lerpSeries(root.fromValues, root.toValues, root.tweenT)
        : root.values
      var shownB = root.tween
        ? ChartsJs.lerpSeries(root.fromValuesB, root.toValuesB, root.tweenT)
        : root.valuesB
      var pts = LiveStatsJs.sparklinePoints(shown, width, height, 1)
      var ptsB = LiveStatsJs.sparklinePoints(shownB, width, height, 1)
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
      var seriesB = Theme.chartSwatches && Theme.chartSwatches.length > 1
        ? Theme.chartSwatches[1]
        : Theme.muted
      strokePts(pts, stroke)
      strokePts(ptsB, seriesB)
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
      root.fromValuesB = ChartsJs.lerpSeries(root.fromValuesB, root.toValuesB, root.tweenT)
      root.toValues = root.values
      root.toValuesB = root.valuesB
      root.tweenT = 0
      root.tweenAt = Date.now()
    } else {
      root.fromValues = root.values
      root.toValues = root.values
      root.fromValuesB = root.valuesB
      root.toValuesB = root.valuesB
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
  onValuesBChanged: root.retarget()
  onAlertChanged: root.paintCanvas()
  onFillChanged: root.paintCanvas()
  onInViewChanged: if (root.inView) root.retarget()
  Component.onCompleted: root.retarget()
}
