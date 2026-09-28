import QtQuick
import "../services"
import "../services/ChartCopy.js" as ChartCopy
import "../services/Charts.js" as ChartsJs

Item {
  id: root

  property string kind: ""
  property var model: null
  property var modelB: null
  property string valueText: ""
  property string explainer: ""
  readonly property string resolvedExplainer: root.explainer.length
    ? root.explainer
    : ChartCopy.explainer(root.kind)
  readonly property string resolvedLegend: ChartCopy.legend(root.kind)
  readonly property color chartThemeAccent: Theme.accent
  readonly property color chartThemeFg: Theme.foreground
  readonly property var chartThemeSwatches: Theme.chartSwatches
  property bool expandable: true
  property int bandCount: 3
  property bool tween: true
  property int tweenMs: LiveStatsStore.intervalMs
  property real tweenT: 1
  property real tweenAt: 0
  property var fromScene: null
  property var toScene: null
  property string treemapFocus: ""
  property real viewScale: 1
  property real viewPanX: 0
  property real viewPanY: 0
  property bool viewPanning: false
  property real viewDragX: 0
  property real viewDragY: 0
  readonly property bool inView: view.inView

  ChartViewport {
    id: view
    target: root
  }

  implicitWidth: 260
  implicitHeight: Theme.chartHeight
  width: implicitWidth
  height: implicitHeight

  Accessible.role: root.expandable ? Accessible.Button : Accessible.StaticText
  Accessible.name: root.valueText
  Accessible.onPressAction: root.expandIntoPage()

  function findPrefsPage() {
    var p = parent
    while (p) {
      if (p.prefsPage === true) return p
      p = p.parent
    }
    return null
  }

  function expandIntoPage() {
    if (!root.expandable) return
    var page = root.findPrefsPage()
    if (!page || !page.expandChart) return
    page.expandChart(root)
  }

  function resetView() {
    root.viewScale = 1
    root.viewPanX = 0
    root.viewPanY = 0
  }

  function clampView() {
    var next = ChartsJs.clampViewPan(
      root.viewScale,
      root.viewPanX,
      root.viewPanY,
      canvas.width,
      canvas.height
    )
    root.viewScale = next.scale
    root.viewPanX = next.x
    root.viewPanY = next.y
  }

  function displayScene() {
    var scene = root.paintedScene()
    if (root.kind !== "treemap") return scene
    return ChartsJs.mapViewScene(scene, root.viewScale, root.viewPanX, root.viewPanY)
  }

  function setTreemapFocus(id) {
    var next = ChartsJs.treemapFocusId(id)
    if (String(next || "") === String(root.treemapFocus || "")) return false
    root.treemapFocus = next
    root.resetView()
    root.retarget()
    return true
  }

  function zoomOut() {
    if (root.kind !== "treemap" || !root.treemapFocus) return false
    root.setTreemapFocus(ChartsJs.treemapParentId(root.model, root.treemapFocus))
    return true
  }

  function handleTreemapClick(mx, my) {
    if (root.kind !== "treemap") return false
    var scene = root.displayScene()
    var hit = ChartsJs.treemapHit(scene && scene.rects, mx, my)
    if (hit && hit.id) {
      var id = ChartsJs.treemapFocusId(hit.id)
      if (String(id) === String(root.treemapFocus || "")) return root.zoomOut()
      return root.setTreemapFocus(id)
    }
    if (root.treemapFocus) return root.zoomOut()
    return false
  }

  function zoomAt(mx, my, factor) {
    var next = ChartsJs.zoomView(
      root.viewScale,
      root.viewPanX,
      root.viewPanY,
      mx,
      my,
      factor,
      canvas.width,
      canvas.height
    )
    root.viewScale = next.scale
    root.viewPanX = next.x
    root.viewPanY = next.y
    root.paintCanvas()
  }

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

  function ink(alpha) {
    var a = Number(alpha)
    if (!isFinite(a) || a < 0) a = 0.2
    if (a > 1) a = 1
    return root.cssColor(Theme.fill(0.1 + a * 0.7))
  }

  function cssHex(hex, alpha) {
    var s = String(hex || "")
    var m = s.match(/^#?([0-9A-Fa-f]{6})$/)
    if (!m) return root.cssColor(Theme.accent)
    var n = parseInt(m[1], 16)
    var r = (n >> 16) & 255
    var g = (n >> 8) & 255
    var b = n & 255
    var a = Number(alpha)
    if (!isFinite(a)) a = 1
    if (a < 0) a = 0
    if (a > 1) a = 1
    return "rgba(" + r + ", " + g + ", " + b + ", " + a + ")"
  }

  function tone(i, alpha) {
    var list = Theme.chartSwatches
    var a = alpha == null ? 0.88 : alpha
    if (!list || !list.length) return root.cssColor(Theme.accentFill(a))
    var idx = Math.abs(Math.floor(Number(i) || 0))
    return root.cssHex(list[idx % list.length], a)
  }

  function toneId(id, alpha) {
    var s = String(id || "")
    var h = 0
    var n
    for (n = 0; n < s.length; n++) h = (h * 33 + s.charCodeAt(n)) >>> 0
    return root.tone(h, alpha)
  }

  function heatTone(t, alpha) {
    var a = alpha == null ? 0.92 : alpha
    var hex = Theme.heatHex(t)
    if (!hex) return root.ink(t)
    return root.cssHex(hex, a)
  }

  function layout() {
    var k = String(root.kind || "")
    var m = root.model
    var w = canvas.width
    var h = canvas.height
    if (k === "heatmap") return ChartsJs.heatmapLayout(m, w, h)
    if (k === "ridgeline") return ChartsJs.ridgelineLayout(m, w, h)
    if (k === "horizon") return ChartsJs.horizonLayout(m, w, h, root.bandCount)
    if (k === "treemap") return ChartsJs.treemapLayout(m, w, h, root.treemapFocus)
    if (k === "violin") return { violins: ChartsJs.violinPaths(m, w, h) }
    if (k === "beeswarm") return ChartsJs.beeswarmLayout(m, w, h)
    if (k === "rose") return { wedges: ChartsJs.nightingaleWedges(m, w, h) }
    if (k === "sankey") {
      var sk = m && typeof m === "object" ? m : { nodes: [], links: [] }
      return ChartsJs.sankeyLayout(sk.nodes, sk.links, w, h)
    }
    if (k === "parallel") return ChartsJs.parallelLayout(m, root.modelB, w, h)
    if (k === "calendar") return ChartsJs.calendarLayout(m, w, h)
    if (k === "streamgraph") return { layers: ChartsJs.streamgraphLayers(m, w, h) }
    if (k === "sunburst") return ChartsJs.sunburstLayout(m, w, h)
    if (k === "radar") return ChartsJs.radarPolygons(m, root.modelB, w, h)
    if (k === "waterfall") return { bars: ChartsJs.waterfallBars(m, w, h) }
    if (k === "icicle") return { rects: ChartsJs.icicleRects(m, w, h) }
    return {}
  }

  function hasMarks(scene) {
    if (!scene) return false
    if (scene.cells && scene.cells.length) return true
    if (scene.ridges && scene.ridges.length) return true
    if (scene.bands && scene.bands.length) return true
    if (scene.rects && scene.rects.length) return true
    if (scene.violins && scene.violins.length) return true
    if (scene.points && scene.points.length) return true
    if (scene.wedges && scene.wedges.length) return true
    if (scene.links && scene.links.length) return true
    if (scene.polylines && scene.polylines.length) return true
    if (scene.layers && scene.layers.length) return true
    if (scene.arcs && scene.arcs.length) return true
    if (scene.polygons && scene.polygons.length) return true
    if (scene.bars && scene.bars.length) return true
    return false
  }

  function strokePoly(ctx, pts, close) {
    if (!pts || pts.length < 2) return
    ctx.beginPath()
    ctx.moveTo(pts[0][0], pts[0][1])
    var i
    for (i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1])
    if (close) ctx.closePath()
  }

  function paintLabelGroup(ctx, items, width, height) {
    if (!items || !items.length) return
    var i
    var item
    var label
    var tx
    var ty
    var align
    ctx.font = Theme.labelSize + "px " + Theme.fontFamily
    ctx.lineJoin = "round"
    ctx.miterLimit = 2
    ctx.lineWidth = Math.max(3, Theme.borderWidth * 3)
    ctx.strokeStyle = root.cssColor(Theme.background)
    ctx.fillStyle = root.cssColor(Theme.foreground)
    for (i = 0; i < items.length; i++) {
      item = items[i]
      label = item && item.label != null ? String(item.label) : ""
      if (!label) continue
      tx = item.lx != null ? item.lx : item.x
      ty = item.ly != null ? item.ly : item.y
      if (tx == null || ty == null) continue
      if (tx < 0) tx = 0
      if (ty < 0) ty = 0
      if (tx > width) tx = width
      if (ty > height) ty = height
      align = item.align || "center"
      ctx.textAlign = align
      ctx.textBaseline = item.baseline || "middle"
      ctx.strokeText(label, tx, ty)
      ctx.fillText(label, tx, ty)
    }
  }

  function paintedScene() {
    return ChartsJs.lerpScene(root.fromScene, root.toScene, root.tweenT)
  }

  function paintCanvas() {
    if (!root.inView) return
    canvas.requestPaint()
  }

  function retarget() {
    if (!root.inView) return
    var next = root.layout()
    if (root.tween && root.hasMarks(root.toScene) && root.hasMarks(next)) {
      root.fromScene = ChartsJs.lerpScene(root.fromScene, root.toScene, root.tweenT)
      root.toScene = next
      root.tweenT = 0
      root.tweenAt = Date.now()
    } else {
      root.fromScene = next
      root.toScene = next
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
      ctx.fillStyle = root.cssColor(Theme.fill(Theme.normalFill))
      ctx.fillRect(0, 0, width, height)
      var scene = root.displayScene()
      if (!root.hasMarks(scene)) return
      var i
      var j
      var item
      var pts
      var markR
      if (scene.cells) {
        for (i = 0; i < scene.cells.length; i++) {
          item = scene.cells[i]
          ctx.fillStyle = root.heatTone(item.fill)
          ctx.fillRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
        }
      }
      if (scene.rects) {
        for (i = 0; i < scene.rects.length; i++) {
          item = scene.rects[i]
          ctx.fillStyle = root.toneId(item.id, 0.55 + (item.depth || 0) * 0.1)
          ctx.fillRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
          ctx.strokeStyle = root.cssColor(Theme.background)
          ctx.lineWidth = Theme.borderWidth
          ctx.strokeRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
        }
      }
      if (scene.bands) {
        for (i = 0; i < scene.bands.length; i++) {
          item = scene.bands[i]
          pts = item.points
          if (!pts || pts.length < 2) continue
          ctx.beginPath()
          ctx.moveTo(pts[0][0], height)
          for (j = 0; j < pts.length; j++) ctx.lineTo(pts[j][0], pts[j][1])
          ctx.lineTo(pts[pts.length - 1][0], height)
          ctx.closePath()
          ctx.fillStyle = root.heatTone(item.fill, 0.55 + (item.layer || 0) * 0.12)
          ctx.fill()
        }
      }
      if (scene.ridges) {
        for (i = 0; i < scene.ridges.length; i++) {
          item = scene.ridges[i]
          root.strokePoly(ctx, item.points, false)
          ctx.strokeStyle = root.tone(i, 1)
          ctx.globalAlpha = 0.85
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
          ctx.globalAlpha = 1
        }
      }
      if (scene.violins) {
        for (i = 0; i < scene.violins.length; i++) {
          root.strokePoly(ctx, scene.violins[i].points, true)
          ctx.fillStyle = root.tone(i, 0.45)
          ctx.fill()
          ctx.strokeStyle = root.tone(i, 1)
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
        }
      }
      if (scene.points) {
        ctx.strokeStyle = root.cssColor(Theme.background)
        ctx.lineWidth = Theme.borderWidth
        for (i = 0; i < scene.points.length; i++) {
          item = scene.points[i]
          markR = item.r > 0 ? item.r : 2
          ctx.fillStyle = root.toneId(item.group || item.id, 0.95)
          ctx.fillRect(item.x - markR, item.y - markR, markR * 2, markR * 2)
          ctx.strokeRect(item.x - markR, item.y - markR, markR * 2, markR * 2)
        }
      }
      if (scene.wedges) {
        for (i = 0; i < scene.wedges.length; i++) {
          item = scene.wedges[i]
          ctx.beginPath()
          ctx.moveTo(item.cx, item.cy)
          ctx.arc(item.cx, item.cy, item.r, item.start, item.end, false)
          ctx.closePath()
          ctx.fillStyle = root.tone(i, 0.82)
          ctx.fill()
          ctx.strokeStyle = root.cssColor(Theme.background)
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
        }
      }
      if (scene.nodes) {
        for (i = 0; i < scene.nodes.length; i++) {
          item = scene.nodes[i]
          ctx.fillStyle = root.tone(i, 0.85)
          ctx.fillRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
        }
      }
      if (scene.links) {
        for (i = 0; i < scene.links.length; i++) {
          item = scene.links[i]
          ctx.beginPath()
          ctx.moveTo(item.x0, item.y0)
          ctx.bezierCurveTo((item.x0 + item.x1) / 2, item.y0, (item.x0 + item.x1) / 2, item.y1, item.x1, item.y1)
          ctx.strokeStyle = root.toneId(item.source != null ? item.source : i, 0.55)
          ctx.lineWidth = Math.max(1, item.width)
          ctx.stroke()
        }
      }
      if (scene.rails) {
        ctx.strokeStyle = root.cssColor(Theme.muted)
        ctx.lineWidth = Theme.borderWidth
        ctx.beginPath()
        for (i = 0; i < scene.rails.length; i++) {
          item = scene.rails[i]
          ctx.moveTo(item.x0, item.y0)
          ctx.lineTo(item.x1, item.y1)
        }
        ctx.stroke()
      }
      if (scene.polylines) {
        ctx.lineWidth = Theme.borderWidth
        for (i = 0; i < scene.polylines.length; i++) {
          ctx.strokeStyle = root.tone(i, 0.8)
          root.strokePoly(ctx, scene.polylines[i].points, false)
          ctx.stroke()
        }
      }
      if (scene.layers) {
        for (i = 0; i < scene.layers.length; i++) {
          item = scene.layers[i]
          ctx.beginPath()
          if (!item.top || !item.bottom || item.top.length < 2) continue
          ctx.moveTo(item.top[0][0], item.top[0][1])
          for (j = 1; j < item.top.length; j++) ctx.lineTo(item.top[j][0], item.top[j][1])
          for (j = item.bottom.length - 1; j >= 0; j--) ctx.lineTo(item.bottom[j][0], item.bottom[j][1])
          ctx.closePath()
          ctx.fillStyle = root.tone(i, 0.7)
          ctx.fill()
        }
      }
      if (scene.arcs) {
        for (i = 0; i < scene.arcs.length; i++) {
          item = scene.arcs[i]
          ctx.beginPath()
          ctx.arc(item.cx, item.cy, item.outerR, item.start, item.end, false)
          ctx.arc(item.cx, item.cy, item.innerR, item.end, item.start, true)
          ctx.closePath()
          ctx.fillStyle = root.toneId(item.id, 0.55 + (item.depth || 0) * 0.12)
          ctx.fill()
          ctx.strokeStyle = root.cssColor(Theme.background)
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
        }
      }
      if (scene.axes && scene.axes.length) {
        ctx.strokeStyle = root.cssColor(Theme.muted)
        ctx.lineWidth = Theme.borderWidth
        ctx.beginPath()
        for (i = 0; i < scene.axes.length; i++) {
          item = scene.axes[i]
          ctx.moveTo(item.cx != null ? item.cx : width / 2, item.cy != null ? item.cy : height / 2)
          ctx.lineTo(item.x, item.y)
        }
        ctx.stroke()
        ctx.beginPath()
        ctx.moveTo(scene.axes[0].x, scene.axes[0].y)
        for (i = 1; i < scene.axes.length; i++) ctx.lineTo(scene.axes[i].x, scene.axes[i].y)
        ctx.closePath()
        ctx.stroke()
      }
      if (scene.polygons) {
        ctx.lineWidth = Theme.borderWidth
        for (i = 0; i < scene.polygons.length; i++) {
          root.strokePoly(ctx, scene.polygons[i].points, true)
          ctx.fillStyle = root.tone(i, 0.35)
          ctx.fill()
          ctx.strokeStyle = root.tone(i, 1)
          ctx.stroke()
        }
      }
      if (scene.bars) {
        for (i = 0; i < scene.bars.length; i++) {
          item = scene.bars[i]
          ctx.fillStyle = item.value < 0 ? root.cssColor(Theme.urgent) : root.tone(i, 0.88)
          ctx.fillRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
        }
      }
      root.paintLabelGroup(ctx, scene.axes, width, height)
      root.paintLabelGroup(ctx, scene.rails, width, height)
      root.paintLabelGroup(ctx, scene.labels, width, height)
      root.paintLabelGroup(ctx, scene.nodes, width, height)
      root.paintLabelGroup(ctx, scene.wedges, width, height)
      root.paintLabelGroup(ctx, scene.bars, width, height)
      root.paintLabelGroup(ctx, scene.violins, width, height)
      root.paintLabelGroup(ctx, scene.rects, width, height)
      root.paintLabelGroup(ctx, scene.layers, width, height)
    }
  }

  MouseArea {
    anchors.fill: parent
    enabled: root.expandable || root.kind === "treemap"
    hoverEnabled: true
    cursorShape: {
      if (root.kind === "treemap" && root.viewPanning) return Qt.ClosedHandCursor
      if (root.kind === "treemap" && root.viewScale > 1) return Qt.OpenHandCursor
      if (root.expandable || root.kind === "treemap") return Qt.PointingHandCursor
      return Qt.ArrowCursor
    }
    onPressed: function (mouse) {
      if (root.kind !== "treemap") return
      root.viewPanning = false
      root.viewDragX = mouse.x
      root.viewDragY = mouse.y
    }
    onPositionChanged: function (mouse) {
      if (root.kind !== "treemap" || !pressed) return
      var dx = mouse.x - root.viewDragX
      var dy = mouse.y - root.viewDragY
      if (!root.viewPanning && dx * dx + dy * dy > 16) root.viewPanning = true
      if (!root.viewPanning) return
      var next = ChartsJs.clampViewPan(
        root.viewScale,
        root.viewPanX + dx,
        root.viewPanY + dy,
        canvas.width,
        canvas.height
      )
      root.viewScale = next.scale
      root.viewPanX = next.x
      root.viewPanY = next.y
      root.viewDragX = mouse.x
      root.viewDragY = mouse.y
      root.paintCanvas()
    }
    onReleased: function (mouse) {
      if (root.kind === "treemap" && !root.viewPanning) {
        if (root.handleTreemapClick(mouse.x, mouse.y)) {
          mouse.accepted = true
          root.viewPanning = false
          return
        }
      }
      root.viewPanning = false
      if (root.kind === "treemap") return
      root.expandIntoPage()
    }
  }

  WheelHandler {
    enabled: root.kind === "treemap"
    onWheel: function (event) {
      if (event.angleDelta.y > 0) {
        root.zoomAt(event.x, event.y, 1.12)
        event.accepted = true
        return
      }
      if (root.viewScale > 1) {
        root.zoomAt(event.x, event.y, 1 / 1.12)
        event.accepted = true
      }
    }
  }

  // paintCanvas reads Theme inside a function, which does not subscribe.
  // These properties exist so a theme change repaints the marks.
  readonly property color chartThemeAccent: Theme.accent
  readonly property color chartThemeFg: Theme.foreground
  readonly property var chartThemeSwatches: Theme.chartSwatches
  onChartThemeAccentChanged: root.paintCanvas()
  onChartThemeFgChanged: root.paintCanvas()
  onChartThemeSwatchesChanged: root.paintCanvas()
  onKindChanged: root.retarget()
  onModelChanged: {
    if (root.treemapFocus && !ChartsJs.findTreeNode(root.model, root.treemapFocus)) {
      root.treemapFocus = ""
      root.resetView()
    }
    root.retarget()
  }
  onModelBChanged: root.retarget()
  onWidthChanged: {
    root.clampView()
    root.retarget()
  }
  onHeightChanged: {
    root.clampView()
    root.retarget()
  }
  onInViewChanged: if (root.inView) root.retarget()
  Component.onCompleted: root.retarget()
  Component.onDestruction: {
    var page = root.findPrefsPage()
    if (page && page.expandedChart === root)
      page.collapseChart()
  }
}
