// Geometry for Monitor charts. Node loads this without Quickshell.
// Empty or unreadable input yields empty geometry, never fabricated zeros.

function finiteNumber(v) {
  if (typeof v === "number") return isFinite(v) ? v : null;
  if (v === null || v === undefined || v === "") return null;
  var n = Number(v);
  return isFinite(n) ? n : null;
}

function nonNeg(v) {
  var n = finiteNumber(v);
  if (n === null || n < 0) return null;
  return n;
}

function hash01(s) {
  var h = 2166136261;
  var str = String(s);
  var i;
  for (i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0) / 4294967295;
}

// Slot gutters for heatmap / calendar tiles. Dense grids stay flush so
// cells do not collapse; larger tiles get a 1–2px hairline gap.
function tileMetrics(extent, count) {
  var e = Number(extent);
  var n = Number(count);
  if (!(e > 0) || !(n > 0)) return { size: 0, gap: 0, step: 0 };
  var slot = e / n;
  var gap = 0;
  if (n > 1 && slot >= 4) gap = 1;
  if (n > 1 && slot >= 8) gap = 2;
  var size = n > 1 ? (e - gap * (n - 1)) / n : e;
  if (!(size > 0)) {
    gap = 0;
    size = slot;
  }
  return { size: size, gap: gap, step: size + gap };
}

// Caption band for radar / parallel / sankey / waterfall / ridgeline / violin /
// calendar / sunburst / heatmap / horizon labels. Keeps marks inside the canvas.
function axisLabelBand(extent) {
  var e = Number(extent);
  if (!(e > 0)) return 0;
  return Math.max(14, Math.min(24, Math.round(e * 0.18)));
}

// Highest-scoring indices, original order, at most `limit` entries.
function topIndices(scores, limit) {
  var n = Array.isArray(scores) ? scores.length : 0;
  var out = [];
  var i;
  if (!n) return out;
  var cap = Math.floor(Number(limit));
  if (!(cap > 0)) cap = 1;
  if (n <= cap) {
    for (i = 0; i < n; i++) out.push(i);
    return out;
  }
  var order = [];
  for (i = 0; i < n; i++) order.push(i);
  order.sort(function (a, b) {
    var d = scores[b] - scores[a];
    if (d) return d;
    return a - b;
  });
  out = order.slice(0, cap);
  out.sort(function (a, b) {
    return a - b;
  });
  return out;
}

function heatmapRowScore(vals) {
  var score = null;
  var j;
  var n;
  var src = Array.isArray(vals) ? vals : [];
  for (j = 0; j < src.length; j++) {
    n = finiteNumber(src[j]);
    if (n === null) continue;
    if (score === null || n > score) score = n;
  }
  return score;
}

// Heatmap: 2-D cell grid, fill 0..1 from value.
// Empty rows/cols take no slot. More rows than pixels keeps the hottest.
function heatmapCells(matrix, width, height) {
  var src = Array.isArray(matrix) ? matrix : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var rows = [];
  var rowScore = [];
  var i;
  var j;
  var vals;
  var row;
  var cols = 0;
  var n;
  var score;
  var keep;
  var next;
  for (i = 0; i < src.length; i++) {
    row = src[i];
    vals = Array.isArray(row && row.values) ? row.values : Array.isArray(row) ? row : [];
    score = heatmapRowScore(vals);
    if (score === null) continue;
    if (vals.length > cols) cols = vals.length;
    rows.push({ id: row && row.id != null ? String(row.id) : String(i), values: vals });
    rowScore.push(score);
  }
  if (!rows.length || !(cols > 0)) return [];
  if (rows.length > Math.floor(h)) {
    keep = topIndices(rowScore, Math.floor(h));
    next = [];
    for (i = 0; i < keep.length; i++) next.push(rows[keep[i]]);
    rows = next;
  }
  cols = 0;
  for (i = 0; i < rows.length; i++) {
    if (rows[i].values.length > cols) cols = rows[i].values.length;
  }
  if (!(cols > 0)) return [];
  var colScore = [];
  for (j = 0; j < cols; j++) colScore[j] = null;
  for (i = 0; i < rows.length; i++) {
    for (j = 0; j < cols; j++) {
      n = finiteNumber(rows[i].values[j]);
      if (n === null) continue;
      if (colScore[j] === null || n > colScore[j]) colScore[j] = n;
    }
  }
  keep = [];
  for (j = 0; j < cols; j++) {
    if (colScore[j] !== null) keep.push(j);
  }
  if (keep.length > Math.floor(w)) {
    var keepScore = [];
    for (i = 0; i < keep.length; i++) keepScore.push(colScore[keep[i]]);
    next = topIndices(keepScore, Math.floor(w));
    var picked = [];
    for (i = 0; i < next.length; i++) picked.push(keep[next[i]]);
    keep = picked;
  }
  var cpuCols = keep;
  if (keep.length < cols) {
    for (i = 0; i < rows.length; i++) {
      vals = [];
      for (j = 0; j < keep.length; j++) vals.push(rows[i].values[keep[j]]);
      rows[i] = { id: rows[i].id, values: vals };
    }
    cols = keep.length;
  }
  if (!(cols > 0)) return [];
  var min = null;
  var max = null;
  for (i = 0; i < rows.length; i++) {
    for (j = 0; j < rows[i].values.length; j++) {
      n = finiteNumber(rows[i].values[j]);
      if (n === null) continue;
      if (min === null || n < min) min = n;
      if (max === null || n > max) max = n;
    }
  }
  if (min === null) return [];
  var span = max - min;
  var xTile = tileMetrics(w, cols);
  var yTile = tileMetrics(h, rows.length);
  if (!(xTile.size > 0) || !(yTile.size > 0)) return [];
  var out = [];
  var fill;
  for (i = 0; i < rows.length; i++) {
    for (j = 0; j < cols; j++) {
      n = finiteNumber(rows[i].values[j]);
      if (n === null) continue;
      fill = span === 0 ? 0.5 : (n - min) / span;
      out.push({
        x: j * xTile.step,
        y: i * yTile.step,
        w: xTile.size,
        h: yTile.size,
        value: n,
        fill: fill,
        row: rows[i].id,
        col: cpuCols[j],
      });
    }
  }
  return out;
}

var EMPTY_HEAT = { cells: [], labels: [] };

function heatmapLayout(matrix, width, height) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || !Array.isArray(matrix) || !matrix.length) return EMPTY_HEAT;
  var left = Math.max(axisLabelBand(w), Math.min(88, Math.round(w * 0.34)));
  var top = axisLabelBand(h);
  var plotW = w - left;
  var plotH = h - top;
  if (!(plotW > w * 0.45)) {
    left = axisLabelBand(w);
    plotW = w - left;
  }
  if (!(plotH > h * 0.5)) {
    top = Math.min(top, Math.max(12, Math.round(h * 0.2)));
    plotH = h - top;
  }
  if (!(plotW > 0)) {
    left = 0;
    plotW = w;
  }
  if (!(plotH > 0)) {
    top = 0;
    plotH = h;
  }
  var cells = heatmapCells(matrix, plotW, plotH);
  if (!cells.length) return EMPTY_HEAT;
  var i;
  for (i = 0; i < cells.length; i++) {
    cells[i].x += left;
    cells[i].y += top;
  }
  var rowSeen = {};
  var colSeen = {};
  var labels = [];
  var cell;
  var ly;
  var lx;
  for (i = 0; i < cells.length; i++) {
    cell = cells[i];
    if (!Object.prototype.hasOwnProperty.call(rowSeen, cell.row)) {
      rowSeen[cell.row] = true;
      ly = cell.y + cell.h / 2;
      if (ly < 0) ly = 0;
      if (h > 0 && ly > h) ly = h;
      labels.push({
        id: cell.row,
        label: cell.row,
        x: 0,
        y: ly,
        lx: left > 0 ? left : 0,
        ly: ly,
        align: left > 0 ? "right" : "left",
        baseline: "middle",
        maxW: Math.max(8, left),
      });
    }
    if (!Object.prototype.hasOwnProperty.call(colSeen, String(cell.col))) {
      colSeen[String(cell.col)] = true;
      lx = cell.x + cell.w / 2;
      if (lx < 0) lx = 0;
      if (w > 0 && lx > w) lx = w;
      labels.push({
        id: "cpu" + cell.col,
        label: String(cell.col),
        x: lx,
        y: 0,
        lx: lx,
        ly: 0,
        align: "center",
        baseline: "top",
        maxW: Math.max(8, cell.w),
      });
    }
  }
  return { cells: cells, labels: labels };
}

// Ridgeline: overlapping series sharing one x-scale, offset in y.
function ridgelinePaths(series, width, height) {
  var src = Array.isArray(series) ? series : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var i;
  var j;
  var nums;
  var n;
  var maxLen = 0;
  var maxV = 0;
  var rows = [];
  var vals;
  for (i = 0; i < src.length; i++) {
    nums = [];
    vals = Array.isArray(src[i] && src[i].values) ? src[i].values : [];
    for (j = 0; j < vals.length; j++) {
      n = finiteNumber(vals[j]);
      if (n === null) continue;
      if (n < 0) n = 0;
      nums.push(n);
      if (n > maxV) maxV = n;
    }
    if (nums.length > maxLen) maxLen = nums.length;
    rows.push({ id: src[i] && src[i].id != null ? String(src[i].id) : String(i), values: nums });
  }
  if (maxLen < 2 || !(maxV > 0)) return [];
  var band = Math.max(axisLabelBand(w), Math.min(64, Math.round(w * 0.3)));
  var plotW = w - band;
  if (!(plotW > w * 0.5)) {
    band = axisLabelBand(w);
    plotW = w - band;
  }
  if (!(plotW > 0)) {
    band = 0;
    plotW = w;
  }
  var ridgeH = h / (src.length + 0.35);
  var out = [];
  var pts;
  var x;
  var y0;
  var y;
  for (i = 0; i < rows.length; i++) {
    if (rows[i].values.length < 2) continue;
    y0 = ridgeH * (i + 1);
    pts = [];
    for (j = 0; j < rows[i].values.length; j++) {
      x = band + (plotW * j) / (maxLen - 1);
      y = y0 - (rows[i].values[j] / maxV) * ridgeH * 0.9;
      pts.push([x, y]);
    }
    out.push({ id: rows[i].id, points: pts, baseline: y0, xScale: maxLen });
  }
  return out;
}

function ridgelineLayout(series, width, height) {
  var ridges = ridgelinePaths(series, width, height);
  var h = Number(height);
  var labels = [];
  var i;
  var r;
  var ly;
  var band = 0;
  if (ridges.length && ridges[0].points && ridges[0].points.length) band = ridges[0].points[0][0];
  for (i = 0; i < ridges.length; i++) {
    r = ridges[i];
    ly = r.baseline;
    if (ly < 0) ly = 0;
    if (h > 0 && ly > h) ly = h;
    labels.push({
      id: r.id,
      label: r.id,
      x: 0,
      y: ly,
      lx: 0,
      ly: ly,
      align: "left",
      baseline: "middle",
      maxW: Math.max(8, band),
    });
  }
  return { ridges: ridges, labels: labels };
}

// Horizon: time series folded into stacked bands. Darker layer = larger value.
function horizonBands(values, width, height, bandCount) {
  var src = Array.isArray(values) ? values : [];
  var w = Number(width);
  var h = Number(height);
  var bands = Math.floor(Number(bandCount) || 3);
  if (bands < 1) bands = 3;
  if (!(w > 0) || !(h > 0)) return [];
  var nums = [];
  var i;
  var n;
  var max = 0;
  for (i = 0; i < src.length; i++) {
    n = finiteNumber(src[i]);
    if (n === null) continue;
    if (n < 0) n = 0;
    nums.push(n);
    if (n > max) max = n;
  }
  if (nums.length < 2 || !(max > 0)) return [];
  var bandH = max / bands;
  var out = [];
  var k;
  var j;
  var slice;
  var pts;
  var x;
  var y;
  for (k = 0; k < bands; k++) {
    pts = [];
    for (j = 0; j < nums.length; j++) {
      slice = nums[j] - k * bandH;
      if (slice < 0) slice = 0;
      if (slice > bandH) slice = bandH;
      x = (w * j) / (nums.length - 1);
      y = h - (slice / bandH) * h;
      pts.push([x, y]);
    }
    out.push({
      id: String(k),
      layer: k,
      fill: (k + 1) / bands,
      ceiling: (k + 1) * bandH,
      points: pts,
    });
  }
  return out;
}

function foldCaption(n) {
  if (n == null || !isFinite(n)) return "";
  if (Math.abs(n - Math.round(n)) < 1e-6) return String(Math.round(n));
  return String(n);
}

function horizonLayout(values, width, height, bandCount) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0)) return { bands: [], labels: [] };
  var left = Math.max(axisLabelBand(w), Math.min(88, Math.round(w * 0.34)));
  var plotW = w - left;
  if (!(plotW > w * 0.45)) {
    left = axisLabelBand(w);
    plotW = w - left;
  }
  if (!(plotW > 0)) {
    left = 0;
    plotW = w;
  }
  var bands = horizonBands(values, plotW, h, bandCount);
  if (!bands.length) return { bands: [], labels: [] };
  var i;
  var j;
  var pts;
  for (i = 0; i < bands.length; i++) {
    pts = bands[i].points;
    if (!pts) continue;
    for (j = 0; j < pts.length; j++) pts[j][0] += left;
  }
  var labels = [];
  var ly;
  var n = bands.length;
  var id;
  var label;
  for (i = 0; i < n; i++) {
    ly = h - ((i + 0.5) / n) * h;
    if (ly < 0) ly = 0;
    if (h > 0 && ly > h) ly = h;
    id = bands[i].id != null ? String(bands[i].id) : String(i);
    label = foldCaption(bands[i].ceiling);
    if (!label) label = String(i + 1);
    labels.push({
      id: id,
      label: label,
      x: 0,
      y: ly,
      lx: left > 0 ? left : 0,
      ly: ly,
      align: left > 0 ? "right" : "left",
      baseline: "middle",
      maxW: Math.max(8, left),
    });
  }
  return { bands: bands, labels: labels };
}

function treemapNodeValue(node) {
  if (!node) return 0;
  var v = finiteNumber(node.value);
  if (v !== null && v > 0 && !Array.isArray(node.children)) return v;
  if (Array.isArray(node.children) && node.children.length) {
    var s = 0;
    var i;
    for (i = 0; i < node.children.length; i++) s += treemapNodeValue(node.children[i]);
    if (s > 0) return s;
  }
  return v !== null && v > 0 ? v : 0;
}

function treemapContents(node) {
  if (!node) return [];
  var kids = Array.isArray(node.children) ? node.children.slice() : [];
  var v = finiteNumber(node.value);
  if (kids.length && v !== null && v > 0) {
    kids.push({
      id: String(node.id) + ":self",
      comm: node.comm || String(node.id),
      value: v,
      children: [],
    });
  }
  return kids;
}

function treemapCaption(node, id) {
  if (node && node.comm) return String(node.comm);
  if (id && String(id).slice(-5) === ":self") return String(id).slice(0, -5);
  return id ? String(id) : "";
}

function treemapFocusId(id) {
  var s = id != null ? String(id) : "";
  if (s.slice(-5) === ":self") return s.slice(0, -5);
  return s;
}

function worstAspect(row, side) {
  if (!(side > 0) || !row.length) return Infinity;
  var total = 0;
  var i;
  var mn = Infinity;
  var mx = 0;
  for (i = 0; i < row.length; i++) {
    total += row[i].value;
    if (row[i].value < mn) mn = row[i].value;
    if (row[i].value > mx) mx = row[i].value;
  }
  if (!(total > 0) || !(mn > 0)) return Infinity;
  var s2 = side * side;
  return Math.max((s2 * mx) / (total * total), (total * total) / (s2 * mn));
}

function layoutTreemapRow(row, x, y, w, h, out, depth) {
  var total = 0;
  var i;
  for (i = 0; i < row.length; i++) total += row[i].value;
  if (!(total > 0)) return;
  var horizontal = w >= h;
  var a = 0;
  var rw;
  var rh;
  var nx;
  var ny;
  for (i = 0; i < row.length; i++) {
    if (horizontal) {
      rw = w * (row[i].value / total);
      rh = h;
      nx = x + a;
      ny = y;
      a += rw;
    } else {
      rw = w;
      rh = h * (row[i].value / total);
      nx = x;
      ny = y + a;
      a += rh;
    }
    pushTreemapRect(row[i].node, nx, ny, rw, rh, out, depth);
  }
}

function pushTreemapRect(node, x, y, w, h, out, depth) {
  var value = treemapNodeValue(node);
  var id = node && node.id != null ? String(node.id) : "";
  var caption = treemapCaption(node, id);
  var contents = treemapContents(node);
  var rect = {
    x: x,
    y: y,
    w: w,
    h: h,
    id: id,
    value: value,
    depth: depth,
  };
  // Leaves only: children fill the parent, so parent captions would be covered.
  if (!contents.length && caption && w >= 16 && h >= 10) {
    rect.label = caption;
    rect.lx = x + w / 2;
    rect.ly = y + h / 2;
    rect.align = "center";
    rect.baseline = "middle";
    rect.maxW = Math.max(8, w - 2);
  }
  rect.branch = contents.length > 0;
  out.push(rect);
  if (contents.length) squarify(contents, x, y, w, h, depth + 1, out);
}

function squarify(nodes, x, y, w, h, depth, out) {
  var src = Array.isArray(nodes) ? nodes.slice() : [];
  if (!src.length || !(w > 0) || !(h > 0)) return;
  var items = [];
  var i;
  var v;
  for (i = 0; i < src.length; i++) {
    v = treemapNodeValue(src[i]);
    if (v > 0) items.push({ node: src[i], value: v });
  }
  items.sort(function (a, b) {
    return b.value - a.value;
  });
  if (!items.length) return;
  if (items.length === 1) {
    pushTreemapRect(items[0].node, x, y, w, h, out, depth);
    return;
  }
  var side = Math.min(w, h);
  var row = [];
  var item;
  while (items.length) {
    item = items[0];
    if (!row.length) {
      row.push(item);
      items.shift();
      continue;
    }
    if (worstAspect(row.concat([item]), side) <= worstAspect(row, side)) {
      row.push(item);
      items.shift();
      continue;
    }
    var rowVal = 0;
    for (i = 0; i < row.length; i++) rowVal += row[i].value;
    var restVal = 0;
    for (i = 0; i < items.length; i++) restVal += items[i].value;
    var frac = rowVal / (rowVal + restVal);
    if (w >= h) {
      layoutTreemapRow(row, x, y, w * frac, h, out, depth);
      x += w * frac;
      w -= w * frac;
    } else {
      layoutTreemapRow(row, x, y, w, h * frac, out, depth);
      y += h * frac;
      h -= h * frac;
    }
    row = [];
    side = Math.min(w, h);
  }
  if (row.length) layoutTreemapRow(row, x, y, w, h, out, depth);
}

function findTreeNode(tree, id) {
  if (!tree) return null;
  if (id == null || String(id) === "") return tree;
  if (tree.id != null && String(tree.id) === String(id)) return tree;
  var kids = Array.isArray(tree.children) ? tree.children : [];
  var i;
  var hit;
  for (i = 0; i < kids.length; i++) {
    hit = findTreeNode(kids[i], id);
    if (hit) return hit;
  }
  return null;
}

function findTreeParent(node, id, parentId) {
  if (!node) return null;
  if (node.id != null && String(node.id) === String(id)) return parentId;
  var kids = Array.isArray(node.children) ? node.children : [];
  var i;
  var hit;
  var nid = node.id != null ? String(node.id) : "";
  for (i = 0; i < kids.length; i++) {
    hit = findTreeParent(kids[i], id, nid);
    if (hit !== null) return hit;
  }
  return null;
}

function treemapParentId(tree, id) {
  if (!tree || id == null || String(id) === "") return "";
  var p = findTreeParent(tree, String(id), "");
  if (p === null || p === "") return "";
  if (tree.id != null && String(p) === String(tree.id)) return "";
  return p;
}

function treemapRoot(tree, focusId) {
  if (!tree) return null;
  if (focusId == null || String(focusId) === "") return tree;
  var n = findTreeNode(tree, focusId);
  return n || tree;
}

function treemapRects(tree, width, height, focusId) {
  var w = Number(width);
  var h = Number(height);
  var root = treemapRoot(tree, focusId);
  if (!(w > 0) || !(h > 0) || !root) return [];
  var out = [];
  var contents = treemapContents(root);
  squarify(contents.length ? contents : [root], 0, 0, w, h, 0, out);
  return out;
}

function treemapHit(rects, x, y) {
  var src = Array.isArray(rects) ? rects : [];
  var px = Number(x);
  var py = Number(y);
  if (!isFinite(px) || !isFinite(py)) return null;
  var best = null;
  var i;
  var r;
  var depth;
  var bestDepth;
  for (i = 0; i < src.length; i++) {
    r = src[i];
    if (!r) continue;
    if (px < r.x || py < r.y || px > r.x + r.w || py > r.y + r.h) continue;
    depth = r.depth || 0;
    bestDepth = best ? best.depth || 0 : -1;
    if (!best || depth >= bestDepth) best = r;
  }
  return best;
}

function clampViewScale(s) {
  var n = Number(s);
  if (!(n > 0) || !isFinite(n)) return 1;
  if (n < 1) return 1;
  if (n > 24) return 24;
  return n;
}

function clampViewPan(scale, panX, panY, viewW, viewH) {
  var s = clampViewScale(scale);
  var w = Number(viewW);
  var h = Number(viewH);
  var x = Number(panX);
  var y = Number(panY);
  if (!isFinite(w) || w <= 0) w = 0;
  if (!isFinite(h) || h <= 0) h = 0;
  if (!isFinite(x)) x = 0;
  if (!isFinite(y)) y = 0;
  var sw = w * s;
  var sh = h * s;
  if (sw <= w) x = (w - sw) / 2;
  else {
    if (x > 0) x = 0;
    if (x + sw < w) x = w - sw;
  }
  if (sh <= h) y = (h - sh) / 2;
  else {
    if (y > 0) y = 0;
    if (y + sh < h) y = h - sh;
  }
  return { scale: s, x: x, y: y };
}

function zoomView(scale, panX, panY, cx, cy, factor, viewW, viewH) {
  var s0 = clampViewScale(scale);
  var k = Number(factor);
  if (!(k > 0) || !isFinite(k)) k = 1;
  var s1 = clampViewScale(s0 * k);
  var x = Number(panX);
  var y = Number(panY);
  var px = Number(cx);
  var py = Number(cy);
  if (!isFinite(x)) x = 0;
  if (!isFinite(y)) y = 0;
  if (!isFinite(px)) px = 0;
  if (!isFinite(py)) py = 0;
  var wx = (px - x) / s0;
  var wy = (py - y) / s0;
  return clampViewPan(s1, px - wx * s1, py - wy * s1, viewW, viewH);
}

function worldPoint(sx, sy, scale, panX, panY) {
  var s = clampViewScale(scale);
  var x = Number(sx);
  var y = Number(sy);
  var ox = Number(panX);
  var oy = Number(panY);
  if (!isFinite(x)) x = 0;
  if (!isFinite(y)) y = 0;
  if (!isFinite(ox)) ox = 0;
  if (!isFinite(oy)) oy = 0;
  return { x: (x - ox) / s, y: (y - oy) / s };
}

function mapViewMark(item, scale, panX, panY) {
  if (!item) return item;
  var s = clampViewScale(scale);
  var ox = Number(panX);
  var oy = Number(panY);
  if (!isFinite(ox)) ox = 0;
  if (!isFinite(oy)) oy = 0;
  var out = {};
  var key;
  for (key in item) {
    if (Object.prototype.hasOwnProperty.call(item, key)) out[key] = item[key];
  }
  if (item.x != null) out.x = item.x * s + ox;
  if (item.y != null) out.y = item.y * s + oy;
  if (item.w != null) out.w = item.w * s;
  if (item.h != null) out.h = item.h * s;
  if (item.lx != null) out.lx = item.lx * s + ox;
  if (item.ly != null) out.ly = item.ly * s + oy;
  if (item.maxW != null) out.maxW = item.maxW * s;
  return out;
}

function mapViewScene(scene, scale, panX, panY) {
  if (!scene) return scene;
  var s = clampViewScale(scale);
  if (s === 1 && !(Number(panX) || Number(panY))) return scene;
  var out = {};
  var key;
  var i;
  var list;
  var mapped;
  for (key in scene) {
    if (!Object.prototype.hasOwnProperty.call(scene, key)) continue;
    list = scene[key];
    if (!Array.isArray(list)) {
      out[key] = list;
      continue;
    }
    mapped = [];
    for (i = 0; i < list.length; i++) mapped.push(mapViewMark(list[i], s, panX, panY));
    out[key] = mapped;
  }
  return out;
}

function treemapLayout(tree, width, height, focusId) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || !tree) return { rects: [], labels: [] };
  var focused = treemapRoot(tree, focusId);
  var crumb = "";
  if (focusId != null && String(focusId).length && focused && focused.id != null) {
    crumb = focused.comm ? String(focused.comm) + " " + String(focused.id) : String(focused.id);
  }
  var band = crumb ? axisLabelBand(h) : 0;
  if (band > h * 0.28) band = Math.max(14, Math.round(h * 0.2));
  var plotH = h - band;
  if (!(plotH > 0)) {
    band = 0;
    plotH = h;
  }
  var rects = treemapRects(tree, w, plotH, focusId);
  var i;
  for (i = 0; i < rects.length; i++) {
    rects[i].y += band;
    if (rects[i].ly != null) rects[i].ly += band;
  }
  var labels = [];
  if (crumb) {
    labels.push({
      id: "__crumb",
      label: crumb,
      x: 0,
      y: 0,
      lx: 0,
      ly: 0,
      align: "left",
      baseline: "top",
      zoomOut: true,
    });
  }
  return { rects: rects, labels: labels };
}

function kdeAt(values, x, h) {
  var i;
  var s = 0;
  var z;
  for (i = 0; i < values.length; i++) {
    z = (x - values[i]) / h;
    s += Math.exp(-0.5 * z * z);
  }
  return s / (values.length * h * Math.sqrt(2 * Math.PI));
}

// Violin: per-core mirrored density of MHz.
function violinPaths(series, width, height) {
  var src = Array.isArray(series) ? series : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var i;
  var j;
  var nums;
  var n;
  var rows = [];
  var min = null;
  var max = null;
  var vals;
  var lo;
  var hi;
  var keep;
  var next;
  var ranges;
  for (i = 0; i < src.length; i++) {
    nums = [];
    vals = Array.isArray(src[i] && src[i].values) ? src[i].values : [];
    for (j = 0; j < vals.length; j++) {
      n = finiteNumber(vals[j]);
      if (n === null) continue;
      nums.push(n);
    }
    if (nums.length >= 2) {
      rows.push({ id: src[i] && src[i].id != null ? String(src[i].id) : String(i), values: nums });
    }
  }
  if (!rows.length) return [];
  // Core ids are two glyphs. Narrower slots overlap labels and collapse the KDE.
  var maxN = Math.max(1, Math.floor(w / 16));
  if (rows.length > maxN) {
    ranges = [];
    for (i = 0; i < rows.length; i++) {
      lo = rows[i].values[0];
      hi = lo;
      for (j = 1; j < rows[i].values.length; j++) {
        n = rows[i].values[j];
        if (n < lo) lo = n;
        if (n > hi) hi = n;
      }
      ranges.push(hi - lo);
    }
    keep = topIndices(ranges, maxN);
    next = [];
    for (i = 0; i < keep.length; i++) next.push(rows[keep[i]]);
    rows = next;
  }
  for (i = 0; i < rows.length; i++) {
    for (j = 0; j < rows[i].values.length; j++) {
      n = rows[i].values[j];
      if (min === null || n < min) min = n;
      if (max === null || n > max) max = n;
    }
  }
  if (min === null || max === min) return [];
  var slot = w / rows.length;
  var span = max - min;
  var bw = span / 12;
  if (!(bw > 0)) return [];
  var band = axisLabelBand(h);
  var plotH = h - band;
  if (!(plotH > 0)) {
    band = 0;
    plotH = h;
  }
  var steps = 24;
  var out = [];
  var dens;
  var peak;
  var y;
  var xMid;
  var half;
  var pts;
  var xv;
  for (i = 0; i < rows.length; i++) {
    dens = [];
    peak = 0;
    for (j = 0; j <= steps; j++) {
      xv = min + (span * j) / steps;
      n = kdeAt(rows[i].values, xv, bw);
      dens.push(n);
      if (n > peak) peak = n;
    }
    if (!(peak > 0)) continue;
    xMid = (i + 0.5) * slot;
    pts = [];
    for (j = 0; j <= steps; j++) {
      y = plotH - (j / steps) * plotH;
      half = (dens[j] / peak) * (slot * 0.42);
      pts.push([xMid - half, y]);
    }
    for (j = steps; j >= 0; j--) {
      y = plotH - (j / steps) * plotH;
      half = (dens[j] / peak) * (slot * 0.42);
      pts.push([xMid + half, y]);
    }
    out.push({
      id: rows[i].id,
      label: rows[i].id,
      points: pts,
      x: xMid,
      lx: xMid,
      ly: h,
      align: "center",
      baseline: "bottom",
      maxW: Math.max(8, slot),
    });
  }
  return out;
}

function beeswarmItemKey(item) {
  return String(item.group != null ? item.group : item.state || "other");
}

function beeswarmGroupIndex(src) {
  var groups = [];
  var index = {};
  var i;
  var key;
  for (i = 0; i < src.length; i++) {
    if (!src[i]) continue;
    key = beeswarmItemKey(src[i]);
    if (!Object.prototype.hasOwnProperty.call(index, key)) {
      index[key] = groups.length;
      groups.push(key);
    }
  }
  return { groups: groups, index: index };
}

function beeswarmPointsFromGroups(src, w, h, groups, index) {
  var slot = w / groups.length;
  var r = Math.max(2, Math.min(6, Math.min(slot * 0.08, h * 0.06)));
  var pad = r + 1;
  var band = axisLabelBand(h);
  var top = pad;
  var bot = band + pad;
  if (top + bot >= h) bot = pad;
  var jitter = Math.max(0, slot - 2 * pad) * 0.7;
  var spanY = Math.max(0, h - top - bot);
  var out = [];
  var i;
  var g;
  var key;
  var id;
  for (i = 0; i < src.length; i++) {
    if (!src[i]) continue;
    key = beeswarmItemKey(src[i]);
    g = index[key];
    id = src[i].id != null ? String(src[i].id) : String(i);
    out.push({
      x: (g + 0.5) * slot + (hash01(id + "x") - 0.5) * jitter,
      y: top + hash01(id + "y") * spanY,
      r: r,
      group: key,
      id: id,
    });
  }
  return out;
}

function beeswarmLabelsFromGroups(groups, w, h) {
  var slot = w / groups.length;
  var out = [];
  var i;
  var lx;
  for (i = 0; i < groups.length; i++) {
    lx = (i + 0.5) * slot;
    out.push({
      key: groups[i],
      label: groups[i],
      x: lx,
      y: h,
      lx: lx,
      ly: h,
      align: "center",
      baseline: "bottom",
      maxW: Math.max(8, slot - 2),
    });
  }
  return out;
}

// Beeswarm: one point per item, jittered, grouped by key.
function beeswarmPoints(items, width, height) {
  var src = Array.isArray(items) ? items : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var grouped = beeswarmGroupIndex(src);
  if (!grouped.groups.length) return [];
  return beeswarmPointsFromGroups(src, w, h, grouped.groups, grouped.index);
}

function beeswarmGroupLabels(items, width, height) {
  var src = Array.isArray(items) ? items : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var grouped = beeswarmGroupIndex(src);
  if (!grouped.groups.length) return [];
  return beeswarmLabelsFromGroups(grouped.groups, w, h);
}

function beeswarmLayout(items, width, height) {
  var src = Array.isArray(items) ? items : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return { points: [], labels: [] };
  var grouped = beeswarmGroupIndex(src);
  if (!grouped.groups.length) return { points: [], labels: [] };
  return {
    points: beeswarmPointsFromGroups(src, w, h, grouped.groups, grouped.index),
    labels: beeswarmLabelsFromGroups(grouped.groups, w, h),
  };
}

// Nightingale rose: equal-angle wedges; area (not radius) encodes value.
function nightingaleWedges(items, width, height) {
  var src = Array.isArray(items) ? items : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var rows = [];
  var i;
  var n;
  var max = 0;
  for (i = 0; i < src.length; i++) {
    n = finiteNumber(src[i] && src[i].value);
    if (n === null || n < 0) continue;
    rows.push({
      id: src[i].id != null ? String(src[i].id) : String(i),
      value: n,
    });
    if (n > max) max = n;
  }
  if (!rows.length) return [];
  var cx = w / 2;
  var cy = h / 2;
  var band = axisLabelBand(Math.min(w, h));
  var maxR = Math.min(w, h) / 2 - band;
  if (!(maxR > 0)) maxR = Math.min(w, h) / 2 - 4;
  if (!(maxR > 0)) return [];
  var theta = (Math.PI * 2) / rows.length;
  var out = [];
  var area;
  var r;
  var start;
  var end;
  var mid;
  var ox;
  var oy;
  var lx;
  var ly;
  var maxArea = 0.5 * maxR * maxR * theta;
  for (i = 0; i < rows.length; i++) {
    area = max > 0 ? (rows[i].value / max) * maxArea : 0;
    r = Math.sqrt((2 * area) / theta);
    start = i * theta - Math.PI / 2;
    end = (i + 1) * theta - Math.PI / 2;
    mid = start + theta / 2;
    ox = Math.cos(mid);
    oy = Math.sin(mid);
    lx = cx + ox * (maxR + 3);
    ly = cy + oy * (maxR + 3);
    if (lx < 0) lx = 0;
    if (ly < 0) ly = 0;
    if (lx > w) lx = w;
    if (ly > h) ly = h;
    out.push({
      id: rows[i].id,
      label: rows[i].id,
      value: rows[i].value,
      cx: cx,
      cy: cy,
      r: r,
      start: start,
      end: end,
      area: area,
      theta: theta,
      lx: lx,
      ly: ly,
      align: ox > 0.35 ? "left" : ox < -0.35 ? "right" : "center",
      baseline: oy > 0.35 ? "top" : oy < -0.35 ? "bottom" : "middle",
      maxW: Math.max(8, maxR * theta),
    });
  }
  return out;
}

function sankeyLayout(nodes, links, width, height) {
  var ns = Array.isArray(nodes) ? nodes : [];
  var ls = Array.isArray(links) ? links : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || !ns.length || !ls.length) return { nodes: [], links: [] };
  var i;
  var id;
  var map = {};
  var colCount = 1;
  for (i = 0; i < ns.length; i++) {
    if (!ns[i] || ns[i].id == null) continue;
    id = String(ns[i].id);
    map[id] = {
      id: id,
      label: ns[i].label != null ? String(ns[i].label) : id,
      col: ns[i].col != null ? Number(ns[i].col) : 0,
      value: 0,
    };
    if (map[id].col + 1 > colCount) colCount = map[id].col + 1;
  }
  var edge = [];
  var val;
  for (i = 0; i < ls.length; i++) {
    if (!ls[i]) continue;
    val = finiteNumber(ls[i].value);
    if (val === null || !(val > 0)) continue;
    if (!map[ls[i].source] || !map[ls[i].target]) continue;
    map[ls[i].source].value += val;
    map[ls[i].target].value += val;
    edge.push({ source: String(ls[i].source), target: String(ls[i].target), value: val });
  }
  if (!edge.length) return { nodes: [], links: [] };
  var byCol = [];
  for (i = 0; i < colCount; i++) byCol.push([]);
  var keys = Object.keys(map);
  for (i = 0; i < keys.length; i++) {
    if (map[keys[i]].col < 0 || map[keys[i]].col >= colCount) continue;
    byCol[map[keys[i]].col].push(map[keys[i]]);
  }
  var nodeW = Math.max(8, w * 0.06);
  var band = axisLabelBand(w);
  var innerW = w - 2 * band;
  if (!(innerW > nodeW)) {
    band = 0;
    innerW = w;
  }
  var gap = 6;
  var c;
  var y;
  var total;
  var avail;
  var nh;
  var node;
  var nx;
  var lx;
  var align;
  var maxW;
  var last = colCount - 1;
  var outNodes = [];
  var nodeGeom = {};
  for (c = 0; c < colCount; c++) {
    total = 0;
    for (i = 0; i < byCol[c].length; i++) total += byCol[c][i].value;
    avail = h - gap * Math.max(0, byCol[c].length - 1);
    y = 0;
    nx = colCount <= 1 ? band : band + (c * (innerW - nodeW)) / last;
    if (c === 0) {
      lx = nx;
      align = "right";
      maxW = Math.max(8, nx);
    } else if (c === last) {
      lx = nx + nodeW;
      align = "left";
      maxW = Math.max(8, w - (nx + nodeW));
    } else {
      lx = nx + nodeW / 2;
      align = "center";
      maxW = Math.max(8, nodeW);
    }
    for (i = 0; i < byCol[c].length; i++) {
      node = byCol[c][i];
      nh = total > 0 ? (node.value / total) * avail : 0;
      nodeGeom[node.id] = {
        id: node.id,
        label: node.label,
        x: nx,
        y: y,
        w: nodeW,
        h: nh,
        value: node.value,
        lx: lx,
        ly: y + nh / 2,
        align: align,
        baseline: "middle",
        maxW: maxW,
      };
      outNodes.push(nodeGeom[node.id]);
      y += nh + gap;
    }
  }
  var outLinks = [];
  var a;
  var b;
  var widthPx;
  var srcShare;
  var tgtShare;
  var outY = {};
  var inY = {};
  edge.sort(function (p, q) {
    var ps = nodeGeom[p.source];
    var qs = nodeGeom[q.source];
    var pt = nodeGeom[p.target];
    var qt = nodeGeom[q.target];
    var dy = ((ps && ps.y) || 0) - ((qs && qs.y) || 0);
    if (dy) return dy;
    return ((pt && pt.y) || 0) - ((qt && qt.y) || 0);
  });
  for (i = 0; i < edge.length; i++) {
    a = nodeGeom[edge[i].source];
    b = nodeGeom[edge[i].target];
    if (!a || !b) continue;
    srcShare = a.value > 0 ? (edge[i].value / a.value) * a.h : 0;
    tgtShare = b.value > 0 ? (edge[i].value / b.value) * b.h : 0;
    widthPx = Math.min(srcShare, tgtShare);
    if (!(widthPx > 0)) continue;
    if (!Object.prototype.hasOwnProperty.call(outY, a.id)) outY[a.id] = a.y;
    if (!Object.prototype.hasOwnProperty.call(inY, b.id)) inY[b.id] = b.y;
    outLinks.push({
      source: edge[i].source,
      target: edge[i].target,
      value: edge[i].value,
      width: widthPx,
      x0: a.x + a.w,
      y0: outY[a.id] + widthPx / 2,
      x1: b.x,
      y1: inY[b.id] + widthPx / 2,
    });
    outY[a.id] += widthPx;
    inY[b.id] += widthPx;
  }
  return { nodes: outNodes, links: outLinks };
}

function parallelPolylines(rows, axes, width, height) {
  var src = Array.isArray(rows) ? rows : [];
  var ax = Array.isArray(axes) ? axes : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0 || ax.length < 2) return [];
  var padTop = axisLabelBand(h);
  var plotH = h - padTop;
  if (!(plotH > 0)) {
    padTop = 0;
    plotH = h;
  }
  var mins = [];
  var maxs = [];
  var i;
  var j;
  var n;
  var key;
  for (j = 0; j < ax.length; j++) {
    mins[j] = null;
    maxs[j] = null;
  }
  for (i = 0; i < src.length; i++) {
    for (j = 0; j < ax.length; j++) {
      key = typeof ax[j] === "string" ? ax[j] : ax[j] && ax[j].key;
      n = finiteNumber(src[i] && src[i][key]);
      if (n === null) continue;
      if (mins[j] === null || n < mins[j]) mins[j] = n;
      if (maxs[j] === null || n > maxs[j]) maxs[j] = n;
    }
  }
  var complete = [];
  var vals;
  var score;
  var span;
  for (i = 0; i < src.length; i++) {
    vals = [];
    score = 0;
    for (j = 0; j < ax.length; j++) {
      key = typeof ax[j] === "string" ? ax[j] : ax[j] && ax[j].key;
      n = finiteNumber(src[i] && src[i][key]);
      if (n === null) {
        vals = null;
        break;
      }
      vals.push(n);
      span = (maxs[j] || 0) - (mins[j] || 0);
      if (span > 0) score += (n - mins[j]) / span;
    }
    if (vals && vals.length === ax.length) {
      complete.push({
        id: src[i].id != null ? String(src[i].id) : String(i),
        values: vals,
        score: score,
      });
    }
  }
  if (!complete.length) return [];
  // Overlapping polylines. Keep the most extreme rows that still read.
  var maxN = Math.max(1, Math.floor(h / 4));
  var keep = complete;
  var next;
  if (complete.length > maxN) {
    var scores = [];
    for (i = 0; i < complete.length; i++) scores.push(complete[i].score);
    next = topIndices(scores, maxN);
    keep = [];
    for (i = 0; i < next.length; i++) keep.push(complete[next[i]]);
  }
  var out = [];
  var pts;
  var x;
  var y;
  for (i = 0; i < keep.length; i++) {
    pts = [];
    for (j = 0; j < ax.length; j++) {
      n = keep[i].values[j];
      span = (maxs[j] || 0) - (mins[j] || 0);
      x = (w * j) / (ax.length - 1);
      y = span === 0 ? padTop + plotH / 2 : padTop + (1 - (n - mins[j]) / span) * plotH;
      pts.push([x, y]);
    }
    out.push({
      id: keep[i].id,
      points: pts,
    });
  }
  return out;
}

function parallelRails(axes, width, height) {
  var ax = Array.isArray(axes) ? axes : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || ax.length < 2) return [];
  var padTop = axisLabelBand(h);
  if (!(h - padTop > 0)) padTop = 0;
  var last = ax.length - 1;
  var slot = w / last;
  var out = [];
  var j;
  var key;
  var label;
  var x;
  for (j = 0; j < ax.length; j++) {
    key = typeof ax[j] === "string" ? ax[j] : ax[j] && ax[j].key;
    label = typeof ax[j] === "string" ? ax[j] : (ax[j] && ax[j].label) || key;
    x = (w * j) / last;
    out.push({
      key: key,
      label: label != null ? String(label) : "",
      x0: x,
      y0: padTop,
      x1: x,
      y1: h,
      lx: x,
      ly: 0,
      align: j === 0 ? "left" : j === last ? "right" : "center",
      baseline: "top",
      maxW: Math.max(8, slot),
    });
  }
  return out;
}

function parallelLayout(rows, axes, width, height) {
  return {
    polylines: parallelPolylines(rows, axes, width, height),
    rails: parallelRails(axes, width, height),
  };
}

function parseDay(s) {
  var m = String(s || "").match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!m) return null;
  var t = Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3]));
  if (!isFinite(t)) return null;
  return { y: Number(m[1]), mo: Number(m[2]), d: Number(m[3]), t: t };
}

// Calendar: day cells in a week × weekday grid. Color from daily-max.
function calendarCells(days, width, height) {
  var src = Array.isArray(days) ? days : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var rows = [];
  var i;
  var day;
  var n;
  var minT = null;
  var maxT = null;
  var minV = null;
  var maxV = null;
  for (i = 0; i < src.length; i++) {
    day = parseDay(src[i] && src[i].date);
    n = finiteNumber(src[i] && src[i].value);
    if (!day || n === null) continue;
    rows.push({ date: src[i].date, t: day.t, value: n });
    if (minT === null || day.t < minT) minT = day.t;
    if (maxT === null || day.t > maxT) maxT = day.t;
    if (minV === null || n < minV) minV = n;
    if (maxV === null || n > maxV) maxV = n;
  }
  if (!rows.length) return [];
  var start = new Date(minT);
  var startWeek = start.getUTCDay();
  var origin = minT - startWeek * 86400000;
  var weekCount = Math.floor((maxT - origin) / (7 * 86400000)) + 1;
  if (weekCount < 1) weekCount = 1;
  var xTile = tileMetrics(w, weekCount);
  var yTile = tileMetrics(h, 7);
  if (!(xTile.size > 0) || !(yTile.size > 0)) return [];
  var span = maxV - minV;
  var out = [];
  var week;
  var weekday;
  var fill;
  for (i = 0; i < rows.length; i++) {
    week = Math.floor((rows[i].t - origin) / (7 * 86400000));
    weekday = new Date(rows[i].t).getUTCDay();
    fill = span === 0 ? 0.5 : (rows[i].value - minV) / span;
    out.push({
      x: week * xTile.step,
      y: weekday * yTile.step,
      w: xTile.size,
      h: yTile.size,
      date: rows[i].date,
      week: week,
      weekday: weekday,
      value: rows[i].value,
      fill: fill,
    });
  }
  return out;
}

function calendarLayout(days, width, height) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0)) return { cells: [], labels: [] };
  var band = Math.max(axisLabelBand(w), Math.min(64, Math.round(w * 0.3)));
  var plotW = w - band;
  if (!(plotW > w * 0.5)) {
    band = axisLabelBand(w);
    plotW = w - band;
  }
  if (!(plotW > 0)) {
    band = 0;
    plotW = w;
  }
  var cells = calendarCells(days, plotW, h);
  if (!cells.length) return { cells: [], labels: [] };
  var i;
  if (band > 0) {
    for (i = 0; i < cells.length; i++) cells[i].x += band;
  }
  var yTile = tileMetrics(h, 7);
  var names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  var labels = [];
  var ly;
  for (i = 0; i < 7; i++) {
    ly = i * yTile.step + yTile.size / 2;
    if (ly < 0) ly = 0;
    if (h > 0 && ly > h) ly = h;
    labels.push({
      id: names[i],
      label: names[i],
      x: 0,
      y: ly,
      lx: 0,
      ly: ly,
      align: "left",
      baseline: "middle",
      maxW: Math.max(8, band),
    });
  }
  return { cells: cells, labels: labels };
}

function streamgraphLayers(series, width, height) {
  var src = Array.isArray(series) ? series : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var i;
  var j;
  var maxLen = 0;
  var rows = [];
  var nums;
  var n;
  var vals;
  for (i = 0; i < src.length; i++) {
    vals = Array.isArray(src[i] && src[i].values) ? src[i].values : [];
    nums = [];
    for (j = 0; j < vals.length; j++) {
      n = finiteNumber(vals[j]);
      nums.push(n !== null && n > 0 ? n : 0);
    }
    if (nums.length > maxLen) maxLen = nums.length;
    rows.push({ id: src[i] && src[i].id != null ? String(src[i].id) : String(i), values: nums });
  }
  if (maxLen < 2) return [];
  for (i = 0; i < rows.length; i++) {
    while (rows[i].values.length < maxLen) rows[i].values.push(0);
  }
  var totals = [];
  var maxTotal = 0;
  for (j = 0; j < maxLen; j++) {
    n = 0;
    for (i = 0; i < rows.length; i++) n += rows[i].values[j];
    totals[j] = n;
    if (n > maxTotal) maxTotal = n;
  }
  if (!(maxTotal > 0)) return [];
  var baseline = [];
  for (j = 0; j < maxLen; j++) baseline[j] = (h - (totals[j] / maxTotal) * h) / 2;
  var out = [];
  var top;
  var bot;
  var y0;
  var y1;
  var x;
  var acc;
  var thick;
  var best;
  var t;
  var layer;
  var align;
  for (i = 0; i < rows.length; i++) {
    top = [];
    bot = [];
    acc = baseline.slice();
    for (j = 0; j < i; j++) {
      for (n = 0; n < maxLen; n++) acc[n] += (rows[j].values[n] / maxTotal) * h;
    }
    for (j = 0; j < maxLen; j++) {
      x = (w * j) / (maxLen - 1);
      y0 = acc[j];
      y1 = acc[j] + (rows[i].values[j] / maxTotal) * h;
      top.push([x, y0]);
      bot.push([x, y1]);
    }
    layer = { id: rows[i].id, top: top, bottom: bot };
    thick = 0;
    best = 0;
    for (j = 0; j < maxLen; j++) {
      t = bot[j][1] - top[j][1];
      if (t >= thick) {
        thick = t;
        best = j;
      }
    }
    // Series legends stay visible; skip only an empty disk id.
    // Caption sits at the thickest sample so the disk id stays on its layer.
    if (rows[i].id) {
      x = top[best][0];
      y0 = (top[best][1] + bot[best][1]) / 2;
      if (x < 0) x = 0;
      if (x > w) x = w;
      if (y0 < 0) y0 = 0;
      if (y0 > h) y0 = h;
      align = "center";
      if (best === 0) align = "left";
      else if (best === maxLen - 1) align = "right";
      layer.label = rows[i].id;
      layer.lx = x;
      layer.ly = y0;
      layer.align = align;
      layer.baseline = "middle";
      if (align === "left") layer.maxW = Math.max(8, w - x);
      else if (align === "right") layer.maxW = Math.max(8, x);
      else layer.maxW = Math.max(8, Math.min(x, w - x) * 2);
    }
    out.push(layer);
  }
  return out;
}

function treeDepth(node) {
  var kids = Array.isArray(node && node.children) ? node.children : [];
  var d = 1;
  var i;
  var cd;
  for (i = 0; i < kids.length; i++) {
    cd = treeDepth(kids[i]);
    if (cd + 1 > d) d = cd + 1;
  }
  return d;
}

function sunburstArcs(tree, width, height) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || !tree) return [];
  var cx = w / 2;
  var cy = h / 2;
  var band = axisLabelBand(Math.min(w, h));
  var maxR = Math.min(w, h) / 2 - band;
  if (!(maxR > 0)) maxR = Math.min(w, h) / 2 - 4;
  if (!(maxR > 0)) return [];
  var maxDepth = treeDepth(tree);
  if (!(maxDepth > 0)) return [];
  var out = [];

  function labelArc(arc, kids) {
    var id = arc.id;
    var inner = arc.innerR;
    var outer = arc.outerR;
    var span = arc.end - arc.start;
    var thick = outer - inner;
    var midR = (inner + outer) / 2;
    var arcLen = Math.abs(span) * (arc.depth === 0 ? Math.max(outer, 1) : midR);
    var mid;
    var ox;
    var oy;
    var lx;
    var ly;
    var rLabel;
    // Empty ids and slivers stay unlabeled. Outer captions sit in the
    // band so long cgroup names are not clipped at the canvas edge.
    if (!id || thick < 10 || arcLen < 16) return;
    mid = arc.start + span / 2;
    ox = Math.cos(mid);
    oy = Math.sin(mid);
    if (arc.depth === 0) {
      lx = cx;
      ly = cy;
      arc.align = "center";
      arc.baseline = "middle";
      arc.maxW = Math.max(8, outer * 1.4);
    } else if (arc.depth === maxDepth - 1 || !kids.length) {
      rLabel = outer + 3;
      lx = cx + ox * rLabel;
      ly = cy + oy * rLabel;
      arc.align = ox > 0.35 ? "left" : ox < -0.35 ? "right" : "center";
      arc.baseline = oy > 0.35 ? "top" : oy < -0.35 ? "bottom" : "middle";
      arc.maxW = Math.max(8, Math.abs(span) * rLabel);
    } else {
      lx = cx + ox * midR;
      ly = cy + oy * midR;
      arc.align = "center";
      arc.baseline = "middle";
      arc.maxW = Math.max(8, arcLen);
    }
    if (lx < 0) lx = 0;
    if (ly < 0) ly = 0;
    if (lx > w) lx = w;
    if (ly > h) ly = h;
    arc.label = id;
    arc.lx = lx;
    arc.ly = ly;
  }

  function walk(node, depth, start, end) {
    if (!node) return;
    var kids = Array.isArray(node.children) ? node.children : [];
    var value = treemapNodeValue(node);
    var inner = (depth / maxDepth) * maxR;
    var outer = ((depth + 1) / maxDepth) * maxR;
    var arc = {
      id: node.id != null ? String(node.id) : "",
      value: value,
      depth: depth,
      cx: cx,
      cy: cy,
      innerR: inner,
      outerR: outer,
      start: start,
      end: end,
    };
    labelArc(arc, kids);
    out.push(arc);
    if (!kids.length || !(value > 0)) return;
    var i;
    var v;
    var a = start;
    var span = end - start;
    for (i = 0; i < kids.length; i++) {
      v = treemapNodeValue(kids[i]);
      walk(kids[i], depth + 1, a, a + span * (v / value));
      a += span * (v / value);
    }
  }

  walk(tree, 0, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2);
  return out;
}

function sunburstLayout(tree, width, height) {
  var arcs = sunburstArcs(tree, width, height);
  var labels = [];
  var i;
  var a;
  for (i = 0; i < arcs.length; i++) {
    a = arcs[i];
    if (!a || a.label == null || a.lx == null || a.ly == null) continue;
    labels.push({
      id: a.id,
      label: a.label,
      x: a.lx,
      y: a.ly,
      lx: a.lx,
      ly: a.ly,
      align: a.align,
      baseline: a.baseline,
      maxW: a.maxW,
    });
  }
  return { arcs: arcs, labels: labels };
}

function radarPolygons(series, axes, width, height) {
  var src = Array.isArray(series) ? series : [];
  var ax = Array.isArray(axes) ? axes : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0 || ax.length < 3)
    return { axes: [], polygons: [], labels: [] };
  var cx = w / 2;
  var cy = h / 2;
  var band = axisLabelBand(Math.min(w, h));
  var r = Math.min(w, h) / 2 - band;
  if (!(r > 0)) r = Math.min(w, h) / 2 - 4;
  if (!(r > 0)) return { axes: [], polygons: [], labels: [] };
  var i;
  var j;
  var key;
  var n;
  var maxs = [];
  for (j = 0; j < ax.length; j++) maxs[j] = 0;
  for (i = 0; i < src.length; i++) {
    for (j = 0; j < ax.length; j++) {
      key = typeof ax[j] === "string" ? ax[j] : ax[j] && ax[j].key;
      n = finiteNumber(src[i] && src[i][key]);
      if (n !== null && n > maxs[j]) maxs[j] = n;
    }
  }
  var spokes = [];
  var ang;
  var ox;
  var oy;
  var lx;
  var ly;
  for (j = 0; j < ax.length; j++) {
    ang = -Math.PI / 2 + (j * 2 * Math.PI) / ax.length;
    ox = Math.cos(ang);
    oy = Math.sin(ang);
    key = typeof ax[j] === "string" ? ax[j] : ax[j] && ax[j].key;
    lx = cx + ox * (r + 3);
    ly = cy + oy * (r + 3);
    if (lx < 0) lx = 0;
    if (ly < 0) ly = 0;
    if (lx > w) lx = w;
    if (ly > h) ly = h;
    spokes.push({
      key: key,
      label: typeof ax[j] === "string" ? ax[j] : (ax[j] && ax[j].label) || key,
      x: cx + ox * r,
      y: cy + oy * r,
      lx: lx,
      ly: ly,
      cx: cx,
      cy: cy,
      angle: ang,
      align: ox > 0.35 ? "left" : ox < -0.35 ? "right" : "center",
      baseline: oy > 0.35 ? "top" : oy < -0.35 ? "bottom" : "middle",
    });
  }
  var polygons = [];
  var labels = [];
  var pts;
  var rr;
  var mx;
  var my;
  var best;
  var bestD;
  var dx;
  var dy;
  var d;
  var id;
  for (i = 0; i < src.length; i++) {
    pts = [];
    for (j = 0; j < ax.length; j++) {
      key = spokes[j].key;
      n = finiteNumber(src[i] && src[i][key]);
      if (n === null || n < 0) n = 0;
      rr = maxs[j] > 0 ? (n / maxs[j]) * r : 0;
      pts.push([cx + Math.cos(spokes[j].angle) * rr, cy + Math.sin(spokes[j].angle) * rr]);
    }
    id = src[i].id != null ? String(src[i].id) : String(i);
    polygons.push({
      id: id,
      points: pts,
    });
    // Caption sits inside the fill, off the axis labels around the ring.
    mx = 0;
    my = 0;
    best = 0;
    bestD = -1;
    for (j = 0; j < pts.length; j++) {
      mx += pts[j][0];
      my += pts[j][1];
      dx = pts[j][0] - cx;
      dy = pts[j][1] - cy;
      d = dx * dx + dy * dy;
      if (d > bestD) {
        bestD = d;
        best = j;
      }
    }
    if (!pts.length) continue;
    mx /= pts.length;
    my /= pts.length;
    if (!id || !(bestD >= 64)) continue;
    lx = pts[best][0] * 0.55 + mx * 0.45;
    ly = pts[best][1] * 0.55 + my * 0.45;
    if (lx < 0) lx = 0;
    if (ly < 0) ly = 0;
    if (lx > w) lx = w;
    if (ly > h) ly = h;
    labels.push({
      id: id,
      label: id,
      x: lx,
      y: ly,
      lx: lx,
      ly: ly,
      align: "center",
      baseline: "middle",
      maxW: Math.max(8, r * 0.7),
    });
  }
  return { axes: spokes, polygons: polygons, labels: labels };
}

function waterfallBars(steps, width, height) {
  var src = Array.isArray(steps) ? steps : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var i;
  var n;
  var id;
  var rows = [];
  var minY = 0;
  var maxY = 0;
  var origin = 0;
  for (i = 0; i < src.length; i++) {
    n = finiteNumber(src[i] && src[i].value);
    if (n === null) continue;
    id = src[i].id != null ? String(src[i].id) : String(i);
    rows.push({
      id: id,
      label: src[i].label != null && String(src[i].label) !== "" ? String(src[i].label) : id,
      value: n,
      origin: origin,
    });
    origin += n;
    if (origin > maxY) maxY = origin;
    if (origin < minY) minY = origin;
    if (rows[rows.length - 1].origin > maxY) maxY = rows[rows.length - 1].origin;
    if (rows[rows.length - 1].origin < minY) minY = rows[rows.length - 1].origin;
  }
  if (!rows.length) return [];
  var span = maxY - minY;
  if (!(span > 0)) span = 1;
  var gap = 4;
  var bw = (w - gap * (rows.length - 1)) / rows.length;
  if (!(bw > 0)) return [];
  var band = axisLabelBand(h);
  var plotH = h - band;
  if (!(plotH > 0)) {
    band = 0;
    plotH = h;
  }
  var out = [];
  var y0;
  var y1;
  var top;
  var bot;
  var x;
  var lx;
  var item;
  for (i = 0; i < rows.length; i++) {
    y0 = plotH - ((rows[i].origin - minY) / span) * plotH;
    y1 = plotH - ((rows[i].origin + rows[i].value - minY) / span) * plotH;
    top = Math.min(y0, y1);
    bot = Math.max(y0, y1);
    x = i * (bw + gap);
    lx = x + bw / 2;
    item = {
      id: rows[i].id,
      value: rows[i].value,
      origin: rows[i].origin,
      x: x,
      y: top,
      w: bw,
      h: Math.max(1, bot - top),
    };
    // Series legends stay visible; skip only an empty domain name.
    if (rows[i].label) {
      item.label = rows[i].label;
      item.lx = lx;
      item.ly = h;
      item.align = "center";
      item.baseline = "bottom";
      item.maxW = Math.max(8, bw);
    }
    out.push(item);
  }
  return out;
}

function icicleRects(tree, width, height) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || !tree) return [];
  var out = [];

  function depthOf(node) {
    var kids = Array.isArray(node && node.children) ? node.children : [];
    var d = 1;
    var i;
    var cd;
    for (i = 0; i < kids.length; i++) {
      cd = depthOf(kids[i]);
      if (cd + 1 > d) d = cd + 1;
    }
    return d;
  }

  var maxDepth = depthOf(tree);
  var rowH = h / maxDepth;

  function walk(node, x, ww, depth) {
    if (!node || !(ww > 0)) return;
    var value = treemapNodeValue(node);
    var id = node.id != null ? String(node.id) : "";
    var y = depth * rowH;
    var rect = {
      x: x,
      y: y,
      w: ww,
      h: rowH,
      id: id,
      value: value,
      depth: depth,
    };
    // Every row is visible, so parents get captions too. Slivers skip.
    if (id && ww >= 16 && rowH >= 10) {
      rect.label = id;
      rect.lx = x + ww / 2;
      rect.ly = y + rowH / 2;
      rect.align = "center";
      rect.baseline = "middle";
      rect.maxW = Math.max(8, ww - 2);
    }
    out.push(rect);
    var kids = Array.isArray(node.children) ? node.children : [];
    if (!kids.length || !(value > 0)) return;
    var i;
    var v;
    var cx = x;
    var cw;
    for (i = 0; i < kids.length; i++) {
      v = treemapNodeValue(kids[i]);
      cw = ww * (v / value);
      walk(kids[i], cx, cw, depth + 1);
      cx += cw;
    }
  }

  walk(tree, 0, w, 0);
  return out;
}

var EMPTY_ROWS = [];

function irqRateMatrix(prev, next) {
  var b = next && Array.isArray(next.rows) ? next.rows : EMPTY_ROWS;
  if (!b.length || prev === next) return EMPTY_ROWS;
  var a = prev && Array.isArray(prev.rows) ? prev.rows : EMPTY_ROWS;
  var prevMap = null;
  var i;
  var j;
  var k;
  var row;
  var hit;
  var out = [];
  var cells;
  var pv;
  var n;
  var p;
  var hot;
  var len;
  var scratch;
  for (i = 0; i < b.length; i++) {
    row = b[i];
    if (!row) continue;
    cells = row.values;
    if (!Array.isArray(cells)) continue;
    len = cells.length;
    if (!len) continue;
    pv = undefined;
    if (i < a.length) {
      hit = a[i];
      if (hit && hit.id === row.id) pv = hit.values;
    }
    if (!Array.isArray(pv) && row.id != null) {
      if (!prevMap) {
        prevMap = {};
        for (k = 0; k < a.length; k++) {
          hit = a[k];
          if (hit && hit.id != null) prevMap[hit.id] = hit.values;
        }
      }
      pv = prevMap[row.id];
    }
    if (!Array.isArray(pv)) continue;
    hot = false;
    for (j = 0; j < len; j++) {
      n = finiteNumber(cells[j]);
      p = finiteNumber(pv[j]);
      if (n !== null && p !== null && n > p) {
        hot = true;
        break;
      }
    }
    if (!hot) continue;
    scratch = [];
    for (j = 0; j < len; j++) {
      n = finiteNumber(cells[j]);
      p = finiteNumber(pv[j]);
      if (n === null || p === null || n < p) {
        scratch.push(null);
        continue;
      }
      scratch.push(n - p);
    }
    out.push({ id: row.id, values: scratch });
  }
  return out.length ? out : EMPTY_ROWS;
}

function psiRidges(history) {
  var list = Array.isArray(history) ? history : [];
  var cpu = [];
  var mem = [];
  var io = [];
  var i;
  var s;
  for (i = 0; i < list.length; i++) {
    s = list[i] && list[i].psi;
    cpu.push(s ? finiteNumber(s.cpu) : null);
    mem.push(s ? finiteNumber(s.memory) : null);
    io.push(s ? finiteNumber(s.io) : null);
  }
  return [
    { id: "cpu", values: cpu },
    { id: "memory", values: mem },
    { id: "io", values: io },
  ];
}

function buddySeries(history) {
  var list = Array.isArray(history) ? history : [];
  var out = [];
  var i;
  var orders;
  var j;
  var t;
  for (i = 0; i < list.length; i++) {
    orders = list[i] && Array.isArray(list[i].buddy) ? list[i].buddy : [];
    t = 0;
    for (j = 0; j < orders.length; j++) {
      if (finiteNumber(orders[j]) !== null) t += orders[j] * Math.pow(2, j);
    }
    out.push(t > 0 ? t : null);
  }
  return out;
}

function rssTree(processes) {
  var src = Array.isArray(processes) ? processes : [];
  var byPid = {};
  var i;
  var row;
  var pid;
  var rss;
  var node;
  var parent;
  var keys;
  var roots = [];
  for (i = 0; i < src.length; i++) {
    row = src[i];
    rss = nonNeg(row && row.rssKb);
    if (rss === null || !(rss > 0)) continue;
    if (row.pid == null) continue;
    pid = String(row.pid);
    byPid[pid] = {
      id: pid,
      pid: pid,
      ppid: row.ppid != null ? String(row.ppid) : "0",
      comm: String(row.comm || "unknown"),
      value: rss,
      children: [],
    };
  }
  keys = Object.keys(byPid);
  for (i = 0; i < keys.length; i++) {
    node = byPid[keys[i]];
    parent = node.ppid && node.ppid !== node.id ? byPid[node.ppid] : null;
    if (parent) parent.children.push(node);
    else roots.push(node);
  }
  if (!roots.length) return null;
  function bakeSelf(n) {
    if (!n || !Array.isArray(n.children) || !n.children.length) return;
    var k;
    for (k = 0; k < n.children.length; k++) bakeSelf(n.children[k]);
    var own = finiteNumber(n.value);
    if (own !== null && own > 0) {
      n.children.push({
        id: String(n.id) + ":self",
        comm: n.comm || String(n.id),
        value: own,
        children: [],
      });
      n.value = 0;
    }
  }
  for (i = 0; i < roots.length; i++) bakeSelf(roots[i]);
  return { id: "rss", children: roots };
}

function freqViolins(history) {
  var list = Array.isArray(history) ? history : [];
  var byId = {};
  var i;
  var j;
  var cpus;
  var id;
  for (i = 0; i < list.length; i++) {
    cpus = list[i] && Array.isArray(list[i].cpus) ? list[i].cpus : [];
    for (j = 0; j < cpus.length; j++) {
      if (!cpus[j] || finiteNumber(cpus[j].freqMhz) === null) continue;
      id = String(cpus[j].id);
      if (!byId[id]) byId[id] = { id: id, values: [] };
      byId[id].values.push(cpus[j].freqMhz);
    }
  }
  var keys = Object.keys(byId);
  var out = [];
  for (i = 0; i < keys.length; i++) out.push(byId[keys[i]]);
  return out;
}

function tcpBees(sample) {
  var src = sample && Array.isArray(sample.tcpSockets) ? sample.tcpSockets : [];
  var out = [];
  var i;
  for (i = 0; i < src.length; i++) {
    if (!src[i] || !src[i].state) continue;
    out.push({
      id: src[i].inode != null ? String(src[i].inode) : String(i),
      group: String(src[i].state),
    });
  }
  return out;
}

function softirqWedges(prev, next) {
  var b = next && Array.isArray(next.rows) ? next.rows : EMPTY_ROWS;
  if (!b.length) return EMPTY_ROWS;
  var a = prev && Array.isArray(prev.rows) ? prev.rows : EMPTY_ROWS;
  var prevMap = null;
  var i;
  var j;
  var k;
  var row;
  var hit;
  var cells;
  var pv;
  var n;
  var p;
  var sum;
  var len;
  var out = [];
  for (i = 0; i < b.length; i++) {
    row = b[i];
    if (!row) continue;
    cells = Array.isArray(row.values) ? row.values : EMPTY_ROWS;
    len = cells.length;
    pv = undefined;
    if (i < a.length) {
      hit = a[i];
      if (hit && hit.id === row.id) pv = hit.values;
    }
    if (!Array.isArray(pv) && row.id != null) {
      if (!prevMap) {
        prevMap = {};
        for (k = 0; k < a.length; k++) {
          hit = a[k];
          if (hit && hit.id != null) prevMap[hit.id] = hit.values;
        }
      }
      pv = prevMap[row.id];
    }
    sum = 0;
    if (Array.isArray(pv) && len) {
      if (pv.length < len) len = pv.length;
      for (j = 0; j < len; j++) {
        n = finiteNumber(cells[j]);
        p = finiteNumber(pv[j]);
        if (n !== null && p !== null && n >= p) sum += n - p;
      }
    }
    out.push({ id: row.id, value: sum });
  }
  return out.length ? out : EMPTY_ROWS;
}

function meminfoSankey(sample) {
  if (!sample) return { nodes: [], links: [] };
  var total = nonNeg(sample.memTotal);
  if (total === null || !(total > 0)) return { nodes: [], links: [] };
  var parts = [
    { id: "anon", label: "Anon", kb: nonNeg(sample.memAnon) },
    { id: "file", label: "File", kb: nonNeg(sample.memFile) },
    { id: "slab", label: "Slab", kb: nonNeg(sample.memSlab) },
    { id: "pagetables", label: "PageTables", kb: nonNeg(sample.memPageTables) },
    { id: "kernelstack", label: "KernelStack", kb: nonNeg(sample.memKernelStack) },
    { id: "shmem", label: "Shmem", kb: nonNeg(sample.memShared) },
    { id: "swap", label: "Swap", kb: nonNeg(sample.swapUsed) },
  ];
  var nodes = [{ id: "total", label: "MemTotal", col: 0 }];
  var links = [];
  var i;
  for (i = 0; i < parts.length; i++) {
    if (parts[i].kb === null || !(parts[i].kb > 0)) continue;
    nodes.push({ id: parts[i].id, label: parts[i].label, col: 1 });
    links.push({ source: "total", target: parts[i].id, value: parts[i].kb });
  }
  if (!links.length) return { nodes: [], links: [] };
  return { nodes: nodes, links: links };
}

function processParallel(processes) {
  var src = Array.isArray(processes) ? processes : [];
  var out = [];
  var i;
  var row;
  for (i = 0; i < src.length; i++) {
    row = src[i];
    if (!row) continue;
    if (finiteNumber(row.cpu) === null) continue;
    if (nonNeg(row.rssKb) === null) continue;
    if (nonNeg(row.fds) === null) continue;
    if (nonNeg(row.threads) === null) continue;
    if (finiteNumber(row.nice) === null) continue;
    out.push({
      id: String(row.pid),
      cpu: row.cpu,
      rss: row.rssKb,
      fds: row.fds,
      threads: row.threads,
      nice: row.nice,
    });
  }
  return out;
}

function thermalDays(history) {
  var list = Array.isArray(history) ? history : [];
  var byDay = {};
  var i;
  var j;
  var t;
  var day;
  var sensors;
  var v;
  var d;
  for (i = 0; i < list.length; i++) {
    t = Number(list[i] && list[i].at);
    if (!isFinite(t) || t <= 0) continue;
    d = new Date(t);
    day =
      d.getUTCFullYear() +
      "-" +
      (d.getUTCMonth() + 1 < 10 ? "0" : "") +
      (d.getUTCMonth() + 1) +
      "-" +
      (d.getUTCDate() < 10 ? "0" : "") +
      d.getUTCDate();
    sensors = list[i] && Array.isArray(list[i].sensors) ? list[i].sensors : [];
    for (j = 0; j < sensors.length; j++) {
      if (!sensors[j] || sensors[j].kind !== "temp") continue;
      v = finiteNumber(sensors[j].value);
      if (v === null) continue;
      if (!Object.prototype.hasOwnProperty.call(byDay, day) || v > byDay[day]) byDay[day] = v;
    }
    v = finiteNumber(list[i].cpuTemp);
    if (v !== null) {
      if (!Object.prototype.hasOwnProperty.call(byDay, day) || v > byDay[day]) byDay[day] = v;
    }
  }
  var keys = Object.keys(byDay);
  var out = [];
  for (i = 0; i < keys.length; i++) out.push({ date: keys[i], value: byDay[keys[i]] });
  return out;
}

function diskStream(history) {
  var list = Array.isArray(history) ? history : [];
  var names = [];
  var seen = {};
  var i;
  var j;
  var disks;
  for (i = 0; i < list.length; i++) {
    disks = list[i] && Array.isArray(list[i].disks) ? list[i].disks : [];
    for (j = 0; j < disks.length; j++) {
      if (!disks[j] || !disks[j].name) continue;
      if (seen[disks[j].name]) continue;
      seen[disks[j].name] = true;
      names.push(disks[j].name);
    }
  }
  var out = [];
  var vals;
  var map;
  var n;
  for (j = 0; j < names.length; j++) {
    vals = [];
    for (i = 0; i < list.length; i++) {
      disks = list[i] && Array.isArray(list[i].disks) ? list[i].disks : [];
      n = 0;
      for (map = 0; map < disks.length; map++) {
        if (disks[map] && disks[map].name === names[j]) {
          n = (nonNeg(disks[map].readBps) || 0) + (nonNeg(disks[map].writeBps) || 0);
          break;
        }
      }
      vals.push(n);
    }
    out.push({ id: names[j], values: vals });
  }
  return out;
}

function cgroupTree(node) {
  if (!node) return null;
  return node;
}

function netdevRadar(ifaces) {
  var src = Array.isArray(ifaces) ? ifaces : [];
  var out = [];
  var i;
  var row;
  for (i = 0; i < src.length; i++) {
    row = src[i];
    if (!row || !row.name) continue;
    out.push({
      id: String(row.name),
      rx: nonNeg(row.rx) || 0,
      tx: nonNeg(row.tx) || 0,
      packets: (nonNeg(row.rxPackets) || 0) + (nonNeg(row.txPackets) || 0),
      drops: (nonNeg(row.rxDrop) || 0) + (nonNeg(row.txDrop) || 0),
      errs: (nonNeg(row.rxErr) || 0) + (nonNeg(row.txErr) || 0),
    });
  }
  return out;
}

function raplSteps(prev, next) {
  var a = prev && Array.isArray(prev.rapl) ? prev.rapl : [];
  var b = next && Array.isArray(next.rapl) ? next.rapl : [];
  if (!b.length) return [];
  var prevMap = {};
  var i;
  for (i = 0; i < a.length; i++) {
    if (a[i] && a[i].id != null) prevMap[String(a[i].id)] = a[i].uj;
  }
  var out = [];
  var n;
  var p;
  var joules;
  for (i = 0; i < b.length; i++) {
    if (!b[i] || b[i].id == null) continue;
    n = finiteNumber(b[i].uj);
    p = finiteNumber(prevMap[String(b[i].id)]);
    if (n === null || p === null || n < p) continue;
    joules = (n - p) / 1e6;
    out.push({
      id: String(b[i].id),
      label: b[i].name != null && String(b[i].name) !== "" ? String(b[i].name) : String(b[i].id),
      value: joules,
    });
  }
  return out;
}

function slabTree(slabs) {
  var src = Array.isArray(slabs) ? slabs : [];
  if (!src.length) return null;
  var groups = {};
  var i;
  var name;
  var prefix;
  var active;
  for (i = 0; i < src.length; i++) {
    name = String(src[i] && src[i].name ? src[i].name : "");
    if (!name) continue;
    active = nonNeg(src[i].active);
    if (active === null || !(active > 0)) continue;
    prefix = name.split(/[-_]/)[0] || name;
    if (!groups[prefix]) groups[prefix] = { id: prefix, children: [] };
    groups[prefix].children.push({ id: name, value: active });
  }
  var kids = [];
  var keys = Object.keys(groups);
  for (i = 0; i < keys.length; i++) kids.push(groups[keys[i]]);
  if (!kids.length) return null;
  return { id: "slab", children: kids };
}

function clampT(t) {
  var n = Number(t);
  if (!isFinite(n) || n <= 0) return 0;
  if (n >= 1) return 1;
  return n;
}

function lerpNum(a, b, t) {
  var x = Number(a);
  var y = Number(b);
  if (!isFinite(x)) x = y;
  if (!isFinite(y)) y = x;
  if (!isFinite(x)) return 0;
  return x + (y - x) * t;
}

function inViewport(x, y, w, h, viewX, viewY, viewW, viewH, pad) {
  var p = Number(pad);
  if (!isFinite(p) || p < 0) p = 0;
  var ix = Number(x);
  var iy = Number(y);
  var iw = Number(w);
  var ih = Number(h);
  var vx = Number(viewX);
  var vy = Number(viewY);
  var vw = Number(viewW);
  var vh = Number(viewH);
  if (
    !isFinite(ix) ||
    !isFinite(iy) ||
    !isFinite(iw) ||
    !isFinite(ih) ||
    !isFinite(vx) ||
    !isFinite(vy) ||
    !isFinite(vw) ||
    !isFinite(vh)
  )
    return true;
  return ix + iw > vx - p && ix < vx + vw + p && iy + ih > vy - p && iy < vy + vh + p;
}

function tweenProgress(startedAt, now, durationMs) {
  var start = Number(startedAt);
  var n = Number(now);
  var d = Number(durationMs);
  if (!(d > 0) || !isFinite(start) || !isFinite(n)) return 1;
  var elapsed = n - start;
  if (!(elapsed > 0)) return 0;
  if (elapsed >= d) return 1;
  return elapsed / d;
}

function lerpSeries(from, to, t) {
  if (from === to) return Array.isArray(to) ? to : Array.isArray(from) ? from : [];
  var k = clampT(t);
  var a = Array.isArray(from) ? from : [];
  var b = Array.isArray(to) ? to : [];
  if (k >= 1) return b;
  if (k <= 0) return a;
  if (!b.length) return a;
  if (!a.length) return b;
  var out = [];
  var n = Math.max(a.length, b.length);
  var i;
  var av;
  var bv;
  for (i = 0; i < n; i++) {
    av = i < a.length ? finiteNumber(a[i]) : null;
    bv = i < b.length ? finiteNumber(b[i]) : null;
    if (av === null && bv === null) {
      out.push(null);
      continue;
    }
    if (av === null) {
      out.push(bv);
      continue;
    }
    if (bv === null) {
      out.push(av);
      continue;
    }
    out.push(lerpNum(av, bv, k));
  }
  return out;
}

function lerpKeyed(from, to, t, numKeys) {
  if (from === to) return Array.isArray(to) ? to : Array.isArray(from) ? from : [];
  var k = clampT(t);
  var a = Array.isArray(from) ? from : [];
  var b = Array.isArray(to) ? to : [];
  if (k >= 1 || !a.length) return b;
  if (k <= 0 || !b.length) return a;
  var keys = Array.isArray(numKeys) ? numKeys : [];
  var map = {};
  var i;
  var j;
  var row;
  var prev;
  var key;
  var copy;
  for (i = 0; i < a.length; i++) {
    if (a[i] && a[i].id != null) map[String(a[i].id)] = a[i];
  }
  var out = [];
  for (i = 0; i < b.length; i++) {
    row = b[i] || {};
    copy = {};
    for (key in row) {
      if (Object.prototype.hasOwnProperty.call(row, key)) copy[key] = row[key];
    }
    prev =
      row.id != null && Object.prototype.hasOwnProperty.call(map, String(row.id))
        ? map[String(row.id)]
        : null;
    if (prev) {
      for (j = 0; j < keys.length; j++) {
        key = keys[j];
        copy[key] = lerpNum(prev[key], row[key], k);
      }
    }
    out.push(copy);
  }
  return out;
}

function lerpPair(a, b, t) {
  var p = Array.isArray(a) && a.length >= 2 ? a : null;
  var q = Array.isArray(b) && b.length >= 2 ? b : p;
  if (!q) return [0, 0];
  if (!p) p = q;
  return [lerpNum(p[0], q[0], t), lerpNum(p[1], q[1], t)];
}

function lerpPoints(a, b, t) {
  var p = Array.isArray(a) ? a : [];
  var q = Array.isArray(b) ? b : [];
  var n = q.length > p.length ? q.length : p.length;
  if (!n) return [];
  var out = [];
  var i;
  var from;
  var to;
  for (i = 0; i < n; i++) {
    from = i < p.length ? p[i] : p.length ? p[p.length - 1] : q[i];
    to = i < q.length ? q[i] : q.length ? q[q.length - 1] : from;
    out.push(lerpPair(from, to, t));
  }
  return out;
}

function itemKey(item, i) {
  if (!item || typeof item !== "object") return String(i);
  if (item.id != null && String(item.id) !== "") return "id:" + item.id;
  if (item.row != null) return "cell:" + item.row + ":" + item.col;
  if (item.source != null && item.target != null) return "link:" + item.source + ":" + item.target;
  if (item.date != null) return "day:" + item.date;
  if (item.key != null) return "key:" + item.key;
  return "i:" + i;
}

function itemsAlign(prev, next) {
  if (prev === next) return true;
  if (!prev || typeof prev !== "object") return !next || typeof next !== "object";
  if (!next || typeof next !== "object") return false;
  var hasPrevId = prev.id != null && String(prev.id) !== "";
  var hasNextId = next.id != null && String(next.id) !== "";
  if (hasPrevId || hasNextId) {
    if (!hasPrevId || !hasNextId) return false;
    if (prev.id === next.id) return true;
    return String(prev.id) === String(next.id);
  }
  if (prev.row != null || next.row != null) return prev.row === next.row && prev.col === next.col;
  if (prev.source != null && prev.target != null)
    return prev.source === next.source && prev.target === next.target;
  if (next.source != null && next.target != null) return false;
  if (prev.date != null || next.date != null) return prev.date === next.date;
  if (prev.key != null || next.key != null) return prev.key === next.key;
  return true;
}

function lerpScalarFields(from, to, t, fields) {
  if (from === to && to && typeof to === "object") return to;
  var out = {};
  var i;
  var k;
  var src = to && typeof to === "object" ? to : from;
  if (!src) return out;
  for (k in src) {
    if (!Object.prototype.hasOwnProperty.call(src, k)) continue;
    out[k] = src[k];
  }
  if (!from) return out;
  for (i = 0; i < fields.length; i++) {
    k = fields[i];
    if (k === "points") out[k] = lerpPoints(from[k], to && to[k], t);
    else if (k === "top") out[k] = lerpPoints(from[k], to && to[k], t);
    else if (k === "bottom") out[k] = lerpPoints(from[k], to && to[k], t);
    else out[k] = lerpNum(from[k], to && to[k], t);
  }
  return out;
}

function lerpList(from, to, t, fields) {
  if (from === to) return Array.isArray(to) ? to : Array.isArray(from) ? from : [];
  var a = Array.isArray(from) ? from : [];
  var b = Array.isArray(to) ? to : [];
  if (!b.length) return [];
  if (!a.length) return b;
  var map = null;
  var i;
  var j;
  var key;
  var prev;
  var aligned = true;
  var out = [];
  for (i = 0; i < b.length; i++) {
    if (aligned && itemsAlign(a[i], b[i])) {
      prev = a[i];
    } else {
      aligned = false;
      if (!map) {
        map = {};
        for (j = 0; j < a.length; j++) map[itemKey(a[j], j)] = a[j];
      }
      key = itemKey(b[i], i);
      prev = Object.prototype.hasOwnProperty.call(map, key) ? map[key] : a[i];
    }
    if (prev === b[i] && b[i]) out.push(b[i]);
    else out.push(lerpScalarFields(prev, b[i], t, fields));
  }
  return out;
}

function lerpScene(from, to, t) {
  if (from === to) {
    if (to && typeof to === "object") return to;
    if (from && typeof from === "object") return from;
    return {};
  }
  var k = clampT(t);
  var dest = to && typeof to === "object" ? to : {};
  var src = from && typeof from === "object" ? from : dest;
  if (k <= 0) return src;
  if (k >= 1) return dest;
  var out = {};
  if (dest.cells || src.cells)
    out.cells = lerpList(src.cells, dest.cells, k, ["x", "y", "w", "h", "fill", "value"]);
  if (dest.ridges || src.ridges)
    out.ridges = lerpList(src.ridges, dest.ridges, k, ["points", "baseline"]);
  if (dest.bands || src.bands) out.bands = lerpList(src.bands, dest.bands, k, ["points", "fill"]);
  if (dest.rects || src.rects)
    out.rects = lerpList(src.rects, dest.rects, k, [
      "x",
      "y",
      "w",
      "h",
      "value",
      "lx",
      "ly",
      "maxW",
    ]);
  if (dest.violins || src.violins)
    out.violins = lerpList(src.violins, dest.violins, k, ["points", "x", "lx", "ly", "maxW"]);
  if (dest.points || src.points) out.points = lerpList(src.points, dest.points, k, ["x", "y", "r"]);
  if (dest.wedges || src.wedges)
    out.wedges = lerpList(src.wedges, dest.wedges, k, [
      "cx",
      "cy",
      "r",
      "start",
      "end",
      "area",
      "value",
      "lx",
      "ly",
      "maxW",
    ]);
  if (dest.nodes || src.nodes)
    out.nodes = lerpList(src.nodes, dest.nodes, k, [
      "x",
      "y",
      "w",
      "h",
      "value",
      "lx",
      "ly",
      "maxW",
    ]);
  if (dest.links || src.links)
    out.links = lerpList(src.links, dest.links, k, ["x0", "y0", "x1", "y1", "width", "value"]);
  if (dest.polylines || src.polylines)
    out.polylines = lerpList(src.polylines, dest.polylines, k, ["points"]);
  if (dest.layers || src.layers)
    out.layers = lerpList(src.layers, dest.layers, k, ["top", "bottom", "lx", "ly", "maxW"]);
  if (dest.arcs || src.arcs)
    out.arcs = lerpList(src.arcs, dest.arcs, k, [
      "cx",
      "cy",
      "innerR",
      "outerR",
      "start",
      "end",
      "value",
      "lx",
      "ly",
      "maxW",
    ]);
  if (dest.axes || src.axes)
    out.axes = lerpList(src.axes, dest.axes, k, ["x", "y", "angle", "cx", "cy", "lx", "ly"]);
  if (dest.rails || src.rails)
    out.rails = lerpList(src.rails, dest.rails, k, ["x0", "y0", "x1", "y1", "lx", "ly", "maxW"]);
  if (dest.labels || src.labels)
    out.labels = lerpList(src.labels, dest.labels, k, ["x", "y", "lx", "ly", "maxW"]);
  if (dest.polygons || src.polygons)
    out.polygons = lerpList(src.polygons, dest.polygons, k, ["points"]);
  if (dest.bars || src.bars)
    out.bars = lerpList(src.bars, dest.bars, k, [
      "x",
      "y",
      "w",
      "h",
      "value",
      "origin",
      "lx",
      "ly",
      "maxW",
    ]);
  return out;
}

function radarAxes() {
  return [
    { key: "rx", label: "rx" },
    { key: "tx", label: "tx" },
    { key: "packets", label: "packets" },
    { key: "drops", label: "drops" },
    { key: "errs", label: "errs" },
  ];
}

function parallelAxes() {
  return ["cpu", "rss", "fds", "threads", "nice"];
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    heatmapCells: heatmapCells,
    heatmapLayout: heatmapLayout,
    ridgelinePaths: ridgelinePaths,
    ridgelineLayout: ridgelineLayout,
    horizonBands: horizonBands,
    horizonLayout: horizonLayout,
    treemapRects: treemapRects,
    treemapLayout: treemapLayout,
    treemapHit: treemapHit,
    treemapParentId: treemapParentId,
    treemapFocusId: treemapFocusId,
    treemapContents: treemapContents,
    findTreeNode: findTreeNode,
    clampViewScale: clampViewScale,
    clampViewPan: clampViewPan,
    zoomView: zoomView,
    worldPoint: worldPoint,
    mapViewScene: mapViewScene,
    violinPaths: violinPaths,
    beeswarmPoints: beeswarmPoints,
    beeswarmGroupLabels: beeswarmGroupLabels,
    beeswarmLayout: beeswarmLayout,
    nightingaleWedges: nightingaleWedges,
    sankeyLayout: sankeyLayout,
    parallelPolylines: parallelPolylines,
    parallelRails: parallelRails,
    parallelLayout: parallelLayout,
    calendarCells: calendarCells,
    calendarLayout: calendarLayout,
    streamgraphLayers: streamgraphLayers,
    sunburstArcs: sunburstArcs,
    sunburstLayout: sunburstLayout,
    radarPolygons: radarPolygons,
    waterfallBars: waterfallBars,
    icicleRects: icicleRects,
    irqRateMatrix: irqRateMatrix,
    psiRidges: psiRidges,
    buddySeries: buddySeries,
    rssTree: rssTree,
    freqViolins: freqViolins,
    tcpBees: tcpBees,
    softirqWedges: softirqWedges,
    meminfoSankey: meminfoSankey,
    processParallel: processParallel,
    thermalDays: thermalDays,
    diskStream: diskStream,
    cgroupTree: cgroupTree,
    netdevRadar: netdevRadar,
    raplSteps: raplSteps,
    slabTree: slabTree,
    treemapNodeValue: treemapNodeValue,
    lerpNum: lerpNum,
    lerpScene: lerpScene,
    inViewport: inViewport,
    tweenProgress: tweenProgress,
    lerpSeries: lerpSeries,
    lerpKeyed: lerpKeyed,
    radarAxes: radarAxes,
    parallelAxes: parallelAxes,
  };
}
