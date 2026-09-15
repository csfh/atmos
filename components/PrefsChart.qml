import QtQuick
import "../services"
import "../services/Charts.js" as ChartsJs

Item {
  id: root

  property string kind: ""
  property var model: null
  property var modelB: null
  property string valueText: ""
  property int bandCount: 3
  property bool tween: true
  property int tweenMs: LiveStatsStore.intervalMs
  property real tweenT: 1
  property var fromScene: null
  property var toScene: null

  implicitWidth: 260
  implicitHeight: Theme.chartHeight
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

  function ink(alpha) {
    var a = Number(alpha)
    if (!isFinite(a) || a < 0) a = 0.2
    if (a > 1) a = 1
    return root.cssColor(Theme.fill(0.1 + a * 0.7))
  }

  function layout() {
    var k = String(root.kind || "")
    var m = root.model
    var w = canvas.width
    var h = canvas.height
    if (k === "heatmap") return { cells: ChartsJs.heatmapCells(m, w, h) }
    if (k === "ridgeline") return { ridges: ChartsJs.ridgelinePaths(m, w, h) }
    if (k === "horizon") return { bands: ChartsJs.horizonBands(m, w, h, root.bandCount) }
    if (k === "treemap") return { rects: ChartsJs.treemapRects(m, w, h) }
    if (k === "violin") return { violins: ChartsJs.violinPaths(m, w, h) }
    if (k === "beeswarm") return { points: ChartsJs.beeswarmPoints(m, w, h) }
    if (k === "rose") return { wedges: ChartsJs.nightingaleWedges(m, w, h) }
    if (k === "sankey") {
      var sk = m && typeof m === "object" ? m : { nodes: [], links: [] }
      return ChartsJs.sankeyLayout(sk.nodes, sk.links, w, h)
    }
    if (k === "parallel") return { polylines: ChartsJs.parallelPolylines(m, root.modelB, w, h) }
    if (k === "calendar") return { cells: ChartsJs.calendarCells(m, w, h) }
    if (k === "streamgraph") return { layers: ChartsJs.streamgraphLayers(m, w, h) }
    if (k === "sunburst") return { arcs: ChartsJs.sunburstArcs(m, w, h) }
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

  function paintedScene() {
    return ChartsJs.lerpScene(root.fromScene, root.toScene, root.tweenT)
  }

  function retarget() {
    var next = root.layout()
    if (root.tween && root.hasMarks(root.toScene) && root.hasMarks(next)) {
      root.fromScene = ChartsJs.lerpScene(root.fromScene, root.toScene, root.tweenT)
      root.toScene = next
      root.tweenT = 0
      tweenAnim.restart()
    } else {
      root.fromScene = next
      root.toScene = next
      root.tweenT = 1
    }
    canvas.requestPaint()
  }

  NumberAnimation {
    id: tweenAnim
    target: root
    property: "tweenT"
    from: 0
    to: 1
    duration: Math.max(80, root.tweenMs)
    easing.type: Easing.InOutCubic
  }

  onTweenTChanged: canvas.requestPaint()

  Canvas {
    id: canvas
    anchors.fill: parent
    antialiasing: true

    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      ctx.fillStyle = root.cssColor(Theme.fill(Theme.normalFill))
      ctx.fillRect(0, 0, width, height)
      var scene = root.paintedScene()
      if (!root.hasMarks(scene)) return
      var i
      var j
      var item
      var pts
      if (scene.cells) {
        for (i = 0; i < scene.cells.length; i++) {
          item = scene.cells[i]
          ctx.fillStyle = root.ink(item.fill)
          ctx.fillRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
        }
      }
      if (scene.rects) {
        for (i = 0; i < scene.rects.length; i++) {
          item = scene.rects[i]
          ctx.fillStyle = root.ink(0.25 + (item.depth || 0) * 0.12)
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
          ctx.fillStyle = root.ink(item.fill)
          ctx.fill()
        }
      }
      if (scene.ridges) {
        for (i = 0; i < scene.ridges.length; i++) {
          item = scene.ridges[i]
          root.strokePoly(ctx, item.points, false)
          ctx.strokeStyle = root.cssColor(Theme.accent)
          ctx.globalAlpha = 0.45 + i * 0.2
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
          ctx.globalAlpha = 1
        }
      }
      if (scene.violins) {
        for (i = 0; i < scene.violins.length; i++) {
          root.strokePoly(ctx, scene.violins[i].points, true)
          ctx.fillStyle = root.ink(0.45)
          ctx.fill()
          ctx.strokeStyle = root.cssColor(Theme.accent)
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
        }
      }
      if (scene.points) {
        ctx.fillStyle = root.cssColor(Theme.accent)
        for (i = 0; i < scene.points.length; i++) {
          item = scene.points[i]
          ctx.fillRect(item.x - 1, item.y - 1, 2, 2)
        }
      }
      if (scene.wedges) {
        for (i = 0; i < scene.wedges.length; i++) {
          item = scene.wedges[i]
          ctx.beginPath()
          ctx.moveTo(item.cx, item.cy)
          ctx.arc(item.cx, item.cy, item.r, item.start, item.end, false)
          ctx.closePath()
          ctx.fillStyle = root.ink(0.3 + (i % 4) * 0.15)
          ctx.fill()
          ctx.strokeStyle = root.cssColor(Theme.background)
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
        }
      }
      if (scene.nodes) {
        for (i = 0; i < scene.nodes.length; i++) {
          item = scene.nodes[i]
          ctx.fillStyle = root.ink(0.5)
          ctx.fillRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
        }
      }
      if (scene.links) {
        ctx.strokeStyle = root.cssColor(Theme.accent)
        ctx.globalAlpha = 0.45
        for (i = 0; i < scene.links.length; i++) {
          item = scene.links[i]
          ctx.beginPath()
          ctx.moveTo(item.x0, item.y0)
          ctx.bezierCurveTo((item.x0 + item.x1) / 2, item.y0, (item.x0 + item.x1) / 2, item.y1, item.x1, item.y1)
          ctx.lineWidth = Math.max(1, item.width)
          ctx.stroke()
        }
        ctx.globalAlpha = 1
      }
      if (scene.polylines) {
        ctx.strokeStyle = root.cssColor(Theme.accent)
        ctx.globalAlpha = 0.35
        ctx.lineWidth = Theme.borderWidth
        for (i = 0; i < scene.polylines.length; i++) {
          root.strokePoly(ctx, scene.polylines[i].points, false)
          ctx.stroke()
        }
        ctx.globalAlpha = 1
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
          ctx.fillStyle = root.ink(0.25 + (i % 4) * 0.15)
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
          ctx.fillStyle = root.ink(0.2 + (item.depth || 0) * 0.12)
          ctx.fill()
          ctx.strokeStyle = root.cssColor(Theme.background)
          ctx.lineWidth = Theme.borderWidth
          ctx.stroke()
        }
      }
      if (scene.axes) {
        ctx.strokeStyle = root.cssColor(Theme.muted)
        ctx.lineWidth = Theme.borderWidth
        ctx.beginPath()
        ctx.moveTo(scene.axes[0].x, scene.axes[0].y)
        for (i = 1; i < scene.axes.length; i++) ctx.lineTo(scene.axes[i].x, scene.axes[i].y)
        ctx.closePath()
        ctx.stroke()
      }
      if (scene.polygons) {
        ctx.strokeStyle = root.cssColor(Theme.accent)
        ctx.lineWidth = Theme.borderWidth
        for (i = 0; i < scene.polygons.length; i++) {
          root.strokePoly(ctx, scene.polygons[i].points, true)
          ctx.globalAlpha = 0.25
          ctx.fillStyle = root.cssColor(Theme.accent)
          ctx.fill()
          ctx.globalAlpha = 1
          ctx.stroke()
        }
      }
      if (scene.bars) {
        for (i = 0; i < scene.bars.length; i++) {
          item = scene.bars[i]
          ctx.fillStyle = item.value < 0 ? root.cssColor(Theme.urgent) : root.ink(0.55)
          ctx.fillRect(item.x, item.y, Math.max(1, item.w), Math.max(1, item.h))
        }
      }
    }
  }

  onKindChanged: root.retarget()
  onModelChanged: root.retarget()
  onModelBChanged: root.retarget()
  onWidthChanged: root.retarget()
  onHeightChanged: root.retarget()
  Component.onCompleted: root.retarget()
}
