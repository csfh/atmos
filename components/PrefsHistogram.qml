import QtQuick
import "../services"
import "../services/Charts.js" as ChartsJs
import "../services/Monitor.js" as MonitorJs

Column {
  id: root

  property var bins: []
  property string valueText: ""
  property bool tween: true
  property int tweenMs: LiveStatsStore.intervalMs
  property real tweenT: 1
  property real tweenAt: 0
  property var fromBins: []
  property var toBins: []
  readonly property bool inView: view.inView

  ChartViewport {
    id: view
    target: root
  }

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

    onWidthChanged: root.paintCanvas()
    onHeightChanged: root.paintCanvas()

    onPaint: {
      if (!root.inView) return
      var ctx = getContext("2d")
      ctx.reset()
      var shown = root.tween
        ? ChartsJs.lerpKeyed(root.fromBins, root.toBins, root.tweenT, ["count"])
        : root.bins
      var rects = MonitorJs.histogramRects(shown, width, height)
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

  function paintCanvas() {
    if (!root.inView) return
    canvas.requestPaint()
  }

  function retarget() {
    if (!root.inView) return
    if (root.tween && root.toBins && root.toBins.length) {
      root.fromBins = ChartsJs.lerpKeyed(root.fromBins, root.toBins, root.tweenT, ["count"])
      root.toBins = root.bins
      root.tweenT = 0
      root.tweenAt = Date.now()
    } else {
      root.fromBins = root.bins
      root.toBins = root.bins
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
  onBinsChanged: root.retarget()
  onInViewChanged: if (root.inView) root.retarget()
  Component.onCompleted: root.retarget()
}
