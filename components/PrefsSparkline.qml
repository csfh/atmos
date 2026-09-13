import QtQuick
import "../services"
import "../services/LiveStats.js" as LiveStatsJs

Item {
  id: root

  property var values: []
  // Epoch-ms per sample, parallel to values. HomePage builds these with
  // LiveStatsJs.timedSeries so labels track the true sample times.
  property var times: []
  property string valueText: ""
  // Unit for the hover readout: "%" (default), "°C", or "B/s".
  property string unit: "%"
  // Metric name for the hover readout, e.g. "CPU" or a GPU name.
  property string metric: ""

  // (timestamp, value) pairs; the hover index, ticks, and spike all address
  // entries so the readout can never drift off the drawn points.
  readonly property var entries: LiveStatsJs.zipSeries(root.values, root.times)
  readonly property var ticks: LiveStatsJs.axisTicks(root.entries)
  readonly property var spike: LiveStatsJs.spikeOf(root.entries)
  readonly property string windowText: LiveStatsJs.formatWindow(LiveStatsJs.windowSpanMs(root.entries))

  property int hoverIndex: -1
  property int pinnedIndex: -1

  implicitWidth: 260
  implicitHeight: windowLabel.height + columnGap + Theme.sparklineHeight + columnGap + axisRow.height
  width: implicitWidth
  height: implicitHeight

  readonly property int columnGap: Theme.labelGap

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

  function formatHoverValue(v) {
    if (root.unit === "B/s") return LiveStatsJs.formatBps(v)
    if (root.unit === "°C") return Math.round(Number(v)) + " °C"
    return LiveStatsJs.formatPercent(v)
  }

  function hoverText(i) {
    var e = root.entries[i]
    if (!e) return ""
    var t = LiveStatsJs.formatClock(e.at, LiveStatsJs.windowSpanMs(root.entries))
    var label = root.metric ? root.metric + " " : ""
    return t + "  ·  " + label + root.formatHoverValue(e.value)
  }

  function hoverDot() {
    var i = root.hoverIndex >= 0 ? root.hoverIndex : root.pinnedIndex
    if (i < 0) return null
    var pts = LiveStatsJs.sparklinePoints(root.values, canvas.width, canvas.height, 1)
    if (i >= pts.length) return null
    return { x: pts[i][0], y: pts[i][1], text: root.hoverText(i) }
  }

  function hoverAt(x) {
    var n = root.entries.length
    if (n < 1 || canvas.width <= 2) {
      root.hoverIndex = -1
      return
    }
    var i = Math.round(((x - 1) / (canvas.width - 2)) * (n - 1))
    if (i < 0) i = 0
    if (i > n - 1) i = n - 1
    root.hoverIndex = i
  }

  Text {
    id: windowLabel
    width: parent.width
    horizontalAlignment: Text.AlignRight
    font.pixelSize: Theme.metaSize
    font.family: Theme.fontFamily
    color: Theme.muted
    text: root.windowText
    visible: root.windowText !== ""
    height: visible ? implicitHeight : 0
  }

  Canvas {
    id: canvas
    width: parent.width
    height: Theme.sparklineHeight
    anchors.top: windowLabel.bottom
    anchors.topMargin: root.columnGap
    antialiasing: true

    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var pts = LiveStatsJs.sparklinePoints(root.values, width, height, 1)
      if (pts.length < 2) return
      var i
      ctx.beginPath()
      ctx.moveTo(pts[0][0], pts[0][1])
      for (i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1])
      ctx.lineTo(pts[pts.length - 1][0], height)
      ctx.lineTo(pts[0][0], height)
      ctx.closePath()
      ctx.fillStyle = root.cssColor(Theme.fill(Theme.normalFill))
      ctx.fill()
      ctx.beginPath()
      ctx.moveTo(pts[0][0], pts[0][1])
      for (i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1])
      ctx.lineWidth = Theme.borderWidth
      ctx.strokeStyle = root.cssColor(Theme.accent)
      ctx.lineJoin = "miter"
      ctx.stroke()
      var spike = root.spike
      if (spike && spike.index >= 0 && spike.index < pts.length) {
        var sx = pts[spike.index][0]
        var sy = pts[spike.index][1]
        ctx.beginPath()
        ctx.arc(sx, sy, 3.5, 0, Math.PI * 2)
        ctx.fillStyle = root.cssColor(Theme.accent)
        ctx.fill()
        var label = LiveStatsJs.formatClock(spike.at, LiveStatsJs.windowSpanMs(root.entries))
        ctx.font = Theme.metaSize + "px " + Theme.fontFamily
        ctx.fillStyle = root.cssColor(Theme.muted)
        var tw = ctx.measureText(label).width
        var lx = sx + 8
        ctx.textAlign = "left"
        if (sx > width * 0.65) {
          lx = sx - 8
          ctx.textAlign = "right"
        }
        if (lx < 0) lx = 0
        if (lx + (ctx.textAlign === "left" ? tw : 0) > width) lx = width - tw
        var ly = sy < 18 ? sy + 16 : sy - 8
        ctx.fillText(label, lx, ly)
      }
    }
  }

  Item {
    id: axisRow
    width: parent.width
    height: Theme.metaSize + 4
    anchors.top: canvas.bottom
    anchors.topMargin: root.columnGap

    Repeater {
      model: root.ticks

      Text {
        required property var modelData
        font.pixelSize: Theme.metaSize
        font.family: Theme.fontFamily
        color: Theme.muted
        text: modelData.label
        x: {
          var f = Number(modelData.frac)
          if (!(f > 0)) return 0
          if (!(f < 1)) return axisRow.width - width
          return f * axisRow.width - width / 2
        }
      }
    }
  }

  MouseArea {
    anchors.fill: canvas
    hoverEnabled: true
    acceptedButtons: Qt.LeftButton
    onPositionChanged: function(mouse) { root.hoverAt(mouse.x) }
    onExited: {
      if (root.pinnedIndex < 0) root.hoverIndex = -1
    }
    onClicked: {
      if (root.hoverIndex >= 0) {
        root.pinnedIndex = root.hoverIndex
        pinTimer.restart()
      } else {
        root.pinnedIndex = -1
      }
    }
  }

  Timer {
    id: pinTimer
    interval: 4000
    onTriggered: root.pinnedIndex = -1
  }

  Item {
    id: hoverLayer
    anchors.fill: canvas
    visible: root.hoverDot() !== null

    Rectangle {
      id: hoverBox
      property var dot: root.hoverDot()
      visible: dot !== null
      width: hoverTextItem.implicitWidth + Theme.gap * 2
      height: hoverTextItem.implicitHeight + Theme.labelGap * 2
      x: {
        if (!dot) return 0
        var bx = dot.x + 10
        if (bx + width > hoverLayer.width) bx = dot.x - width - 10
        if (bx < 0) bx = 0
        return bx
      }
      y: {
        if (!dot) return 0
        var by = dot.y - height - 8
        if (by < 0) by = dot.y + 12
        return by
      }
      radius: Theme.chamferSm
      color: Theme.background
      border.width: 1
      border.color: Theme.muted

      Text {
        id: hoverTextItem
        anchors.centerIn: parent
        font.pixelSize: Theme.metaSize
        font.family: Theme.fontFamily
        color: Theme.foreground
        text: parent.dot ? parent.dot.text : ""
      }
    }

    Rectangle {
      property var dot: root.hoverDot()
      visible: dot !== null
      width: 7
      height: 7
      radius: 3.5
      color: Theme.accent
      x: (dot ? dot.x : 0) - width / 2
      y: (dot ? dot.y : 0) - height / 2
    }
  }

  onValuesChanged: canvas.requestPaint()
  onTimesChanged: canvas.requestPaint()
  onSpikeChanged: canvas.requestPaint()
}
