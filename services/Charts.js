// Geometry for Monitor charts. Node loads this without Quickshell.
// Empty or unreadable input yields empty geometry, never fabricated zeros.

function finiteNumber(v) {
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

function sumValues(list, key) {
  var src = Array.isArray(list) ? list : [];
  var t = 0;
  var i;
  var n;
  for (i = 0; i < src.length; i++) {
    n = key ? finiteNumber(src[i] && src[i][key]) : finiteNumber(src[i]);
    if (n !== null && n > 0) t += n;
  }
  return t;
}

function clamp01(n) {
  if (!(n > 0)) return 0;
  if (n > 1) return 1;
  return n;
}

// Heatmap: 2-D cell grid, fill 0..1 from value.
function heatmapCells(matrix, width, height) {
  var src = Array.isArray(matrix) ? matrix : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var rows = [];
  var i;
  var j;
  var vals;
  var row;
  var cols = 0;
  for (i = 0; i < src.length; i++) {
    row = src[i];
    vals = Array.isArray(row && row.values) ? row.values : Array.isArray(row) ? row : [];
    if (vals.length > cols) cols = vals.length;
    rows.push({ id: row && row.id != null ? String(row.id) : String(i), values: vals });
  }
  if (!(cols > 0)) return [];
  var min = null;
  var max = null;
  var n;
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
  var cw = w / cols;
  var rh = h / rows.length;
  var out = [];
  var fill;
  for (i = 0; i < rows.length; i++) {
    for (j = 0; j < cols; j++) {
      n = finiteNumber(rows[i].values[j]);
      if (n === null) continue;
      fill = span === 0 ? 0.5 : (n - min) / span;
      out.push({
        x: j * cw,
        y: i * rh,
        w: cw,
        h: rh,
        value: n,
        fill: fill,
        row: rows[i].id,
        col: j,
      });
    }
  }
  return out;
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
      x = (w * j) / (maxLen - 1);
      y = y0 - (rows[i].values[j] / maxV) * ridgeH * 0.9;
      pts.push([x, y]);
    }
    out.push({ id: rows[i].id, points: pts, baseline: y0, xScale: maxLen });
  }
  return out;
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
      layer: k,
      fill: (k + 1) / bands,
      points: pts,
    });
  }
  return out;
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
  var kids = Array.isArray(node && node.children) ? node.children : [];
  var value = treemapNodeValue(node);
  out.push({
    x: x,
    y: y,
    w: w,
    h: h,
    id: node && node.id != null ? String(node.id) : "",
    value: value,
    depth: depth,
  });
  if (kids.length) squarify(kids, x, y, w, h, depth + 1, out);
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

function treemapRects(tree, width, height) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || !tree) return [];
  var out = [];
  squarify(Array.isArray(tree.children) ? tree.children : [tree], 0, 0, w, h, 0, out);
  return out;
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
  for (i = 0; i < src.length; i++) {
    nums = [];
    vals = Array.isArray(src[i] && src[i].values) ? src[i].values : [];
    for (j = 0; j < vals.length; j++) {
      n = finiteNumber(vals[j]);
      if (n === null) continue;
      nums.push(n);
      if (min === null || n < min) min = n;
      if (max === null || n > max) max = n;
    }
    if (nums.length >= 2) {
      rows.push({ id: src[i] && src[i].id != null ? String(src[i].id) : String(i), values: nums });
    }
  }
  if (!rows.length || min === null || max === min) return [];
  var slot = w / rows.length;
  var span = max - min;
  var bw = span / 12;
  if (!(bw > 0)) return [];
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
      y = h - (j / steps) * h;
      half = (dens[j] / peak) * (slot * 0.42);
      pts.push([xMid - half, y]);
    }
    for (j = steps; j >= 0; j--) {
      y = h - (j / steps) * h;
      half = (dens[j] / peak) * (slot * 0.42);
      pts.push([xMid + half, y]);
    }
    out.push({ id: rows[i].id, points: pts, x: xMid });
  }
  return out;
}

// Beeswarm: one point per item, jittered, grouped by key.
function beeswarmPoints(items, width, height) {
  var src = Array.isArray(items) ? items : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var groups = [];
  var index = {};
  var i;
  var g;
  var key;
  for (i = 0; i < src.length; i++) {
    if (!src[i]) continue;
    key = String(src[i].group != null ? src[i].group : src[i].state || "other");
    if (!Object.prototype.hasOwnProperty.call(index, key)) {
      index[key] = groups.length;
      groups.push(key);
    }
  }
  if (!groups.length) return [];
  var slot = w / groups.length;
  var out = [];
  var id;
  for (i = 0; i < src.length; i++) {
    if (!src[i]) continue;
    key = String(src[i].group != null ? src[i].group : src[i].state || "other");
    g = index[key];
    id = src[i].id != null ? String(src[i].id) : String(i);
    out.push({
      x: (g + 0.5) * slot + (hash01(id + "x") - 0.5) * slot * 0.55,
      y: 8 + hash01(id + "y") * (h - 16),
      group: key,
      id: id,
    });
  }
  return out;
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
  var maxR = Math.min(w, h) / 2;
  var theta = (Math.PI * 2) / rows.length;
  var out = [];
  var area;
  var r;
  var maxArea = 0.5 * maxR * maxR * theta;
  for (i = 0; i < rows.length; i++) {
    area = max > 0 ? (rows[i].value / max) * maxArea : 0;
    r = Math.sqrt((2 * area) / theta);
    out.push({
      id: rows[i].id,
      value: rows[i].value,
      cx: cx,
      cy: cy,
      r: r,
      start: i * theta - Math.PI / 2,
      end: (i + 1) * theta - Math.PI / 2,
      area: area,
      theta: theta,
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
  var gap = 6;
  var c;
  var y;
  var total;
  var avail;
  var nh;
  var node;
  var outNodes = [];
  var nodeGeom = {};
  for (c = 0; c < colCount; c++) {
    total = 0;
    for (i = 0; i < byCol[c].length; i++) total += byCol[c][i].value;
    avail = h - gap * Math.max(0, byCol[c].length - 1);
    y = 0;
    for (i = 0; i < byCol[c].length; i++) {
      node = byCol[c][i];
      nh = total > 0 ? (node.value / total) * avail : 0;
      nodeGeom[node.id] = {
        id: node.id,
        label: node.label,
        x: colCount <= 1 ? 0 : (c * (w - nodeW)) / (colCount - 1),
        y: y,
        w: nodeW,
        h: nh,
        value: node.value,
      };
      outNodes.push(nodeGeom[node.id]);
      y += nh + gap;
    }
  }
  var outLinks = [];
  var a;
  var b;
  var widthPx;
  var maxLink = 0;
  for (i = 0; i < edge.length; i++) if (edge[i].value > maxLink) maxLink = edge[i].value;
  var maxWidth = h * 0.5;
  for (i = 0; i < edge.length; i++) {
    a = nodeGeom[edge[i].source];
    b = nodeGeom[edge[i].target];
    if (!a || !b) continue;
    widthPx = maxLink > 0 ? (edge[i].value / maxLink) * maxWidth : 0;
    outLinks.push({
      source: edge[i].source,
      target: edge[i].target,
      value: edge[i].value,
      width: widthPx,
      x0: a.x + a.w,
      y0: a.y + a.h / 2,
      x1: b.x,
      y1: b.y + b.h / 2,
    });
  }
  return { nodes: outNodes, links: outLinks };
}

function parallelPolylines(rows, axes, width, height) {
  var src = Array.isArray(rows) ? rows : [];
  var ax = Array.isArray(axes) ? axes : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0 || ax.length < 2) return [];
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
  var out = [];
  var pts;
  var x;
  var y;
  var span;
  for (i = 0; i < src.length; i++) {
    pts = [];
    for (j = 0; j < ax.length; j++) {
      key = typeof ax[j] === "string" ? ax[j] : ax[j] && ax[j].key;
      n = finiteNumber(src[i] && src[i][key]);
      if (n === null) {
        pts = [];
        break;
      }
      span = (maxs[j] || 0) - (mins[j] || 0);
      x = (w * j) / (ax.length - 1);
      y = span === 0 ? h / 2 : h - ((n - mins[j]) / span) * h;
      pts.push([x, y]);
    }
    if (pts.length === ax.length) {
      out.push({
        id: src[i].id != null ? String(src[i].id) : String(i),
        points: pts,
      });
    }
  }
  return out;
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
  var cw = w / weekCount;
  var rh = h / 7;
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
      x: week * cw,
      y: weekday * rh,
      w: cw,
      h: rh,
      date: rows[i].date,
      week: week,
      weekday: weekday,
      value: rows[i].value,
      fill: fill,
    });
  }
  return out;
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
    out.push({ id: rows[i].id, top: top, bottom: bot });
  }
  return out;
}

function sunburstArcs(tree, width, height) {
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || !tree) return [];
  var cx = w / 2;
  var cy = h / 2;
  var maxR = Math.min(w, h) / 2;
  var out = [];

  function walk(node, depth, start, end) {
    if (!node) return;
    var kids = Array.isArray(node.children) ? node.children : [];
    var value = treemapNodeValue(node);
    var maxDepth = 6;
    var inner = (depth / maxDepth) * maxR;
    var outer = ((depth + 1) / maxDepth) * maxR;
    out.push({
      id: node.id != null ? String(node.id) : "",
      value: value,
      depth: depth,
      cx: cx,
      cy: cy,
      innerR: inner,
      outerR: outer,
      start: start,
      end: end,
    });
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

function radarPolygons(series, axes, width, height) {
  var src = Array.isArray(series) ? series : [];
  var ax = Array.isArray(axes) ? axes : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0 || ax.length < 3) return { axes: [], polygons: [] };
  var cx = w / 2;
  var cy = h / 2;
  var r = Math.min(w, h) / 2 - 4;
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
  for (j = 0; j < ax.length; j++) {
    ang = -Math.PI / 2 + (j * 2 * Math.PI) / ax.length;
    key = typeof ax[j] === "string" ? ax[j] : ax[j] && ax[j].key;
    spokes.push({
      key: key,
      label: typeof ax[j] === "string" ? ax[j] : (ax[j] && ax[j].label) || key,
      x: cx + Math.cos(ang) * r,
      y: cy + Math.sin(ang) * r,
      angle: ang,
    });
  }
  var polygons = [];
  var pts;
  var rr;
  for (i = 0; i < src.length; i++) {
    pts = [];
    for (j = 0; j < ax.length; j++) {
      key = spokes[j].key;
      n = finiteNumber(src[i] && src[i][key]);
      if (n === null || n < 0) n = 0;
      rr = maxs[j] > 0 ? (n / maxs[j]) * r : 0;
      pts.push([cx + Math.cos(spokes[j].angle) * rr, cy + Math.sin(spokes[j].angle) * rr]);
    }
    polygons.push({
      id: src[i].id != null ? String(src[i].id) : String(i),
      points: pts,
    });
  }
  return { axes: spokes, polygons: polygons };
}

function waterfallBars(steps, width, height) {
  var src = Array.isArray(steps) ? steps : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || src.length === 0) return [];
  var i;
  var n;
  var rows = [];
  var minY = 0;
  var maxY = 0;
  var origin = 0;
  for (i = 0; i < src.length; i++) {
    n = finiteNumber(src[i] && src[i].value);
    if (n === null) continue;
    rows.push({
      id: src[i].id != null ? String(src[i].id) : String(i),
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
  var out = [];
  var y0;
  var y1;
  var top;
  var bot;
  for (i = 0; i < rows.length; i++) {
    y0 = h - ((rows[i].origin - minY) / span) * h;
    y1 = h - ((rows[i].origin + rows[i].value - minY) / span) * h;
    top = Math.min(y0, y1);
    bot = Math.max(y0, y1);
    out.push({
      id: rows[i].id,
      value: rows[i].value,
      origin: rows[i].origin,
      x: i * (bw + gap),
      y: top,
      w: bw,
      h: Math.max(1, bot - top),
    });
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
    out.push({
      x: x,
      y: depth * rowH,
      w: ww,
      h: rowH,
      id: node.id != null ? String(node.id) : "",
      value: value,
      depth: depth,
    });
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

function irqRateMatrix(prev, next) {
  var a = prev && Array.isArray(prev.rows) ? prev.rows : [];
  var b = next && Array.isArray(next.rows) ? next.rows : [];
  if (!b.length) return [];
  var prevMap = {};
  var i;
  var j;
  for (i = 0; i < a.length; i++) {
    if (a[i] && a[i].id != null) prevMap[String(a[i].id)] = a[i].values || [];
  }
  var out = [];
  var vals;
  var pv;
  var n;
  var p;
  for (i = 0; i < b.length; i++) {
    if (!b[i]) continue;
    vals = [];
    pv = prevMap[String(b[i].id)] || [];
    for (j = 0; j < (b[i].values || []).length; j++) {
      n = finiteNumber(b[i].values[j]);
      p = finiteNumber(pv[j]);
      if (n === null || p === null || n < p) vals.push(null);
      else vals.push(n - p);
    }
    out.push({ id: b[i].id, values: vals });
  }
  return out;
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
  var users = {};
  var i;
  var row;
  var uid;
  var comm;
  var rss;
  for (i = 0; i < src.length; i++) {
    row = src[i];
    rss = nonNeg(row && row.rssKb);
    if (rss === null || !(rss > 0)) continue;
    uid = String(row.uid != null ? row.uid : "?");
    comm = String(row.comm || "unknown");
    if (!users[uid]) users[uid] = { id: "uid:" + uid, children: {} };
    if (!users[uid].children[comm]) users[uid].children[comm] = { id: comm, value: 0 };
    users[uid].children[comm].value += rss;
  }
  var kids = [];
  var comms;
  var names;
  var j;
  names = Object.keys(users);
  for (i = 0; i < names.length; i++) {
    comms = [];
    var ck = Object.keys(users[names[i]].children);
    for (j = 0; j < ck.length; j++) comms.push(users[names[i]].children[ck[j]]);
    kids.push({ id: users[names[i]].id, children: comms });
  }
  if (!kids.length) return null;
  return { id: "rss", children: kids };
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
  var a = prev && Array.isArray(prev.rows) ? prev.rows : [];
  var b = next && Array.isArray(next.rows) ? next.rows : [];
  var prevMap = {};
  var i;
  var j;
  var sum;
  var n;
  var p;
  for (i = 0; i < a.length; i++) {
    if (a[i] && a[i].id != null) prevMap[String(a[i].id)] = a[i].values || [];
  }
  var out = [];
  for (i = 0; i < b.length; i++) {
    if (!b[i]) continue;
    sum = 0;
    for (j = 0; j < (b[i].values || []).length; j++) {
      n = finiteNumber(b[i].values[j]);
      p = finiteNumber((prevMap[String(b[i].id)] || [])[j]);
      if (n !== null && p !== null && n >= p) sum += n - p;
    }
    out.push({ id: b[i].id, value: sum });
  }
  return out;
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
    out.push({ id: String(b[i].id), value: joules });
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

function lerpScalarFields(from, to, t, fields) {
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
  var a = Array.isArray(from) ? from : [];
  var b = Array.isArray(to) ? to : [];
  if (!b.length && !a.length) return [];
  var map = {};
  var i;
  for (i = 0; i < a.length; i++) map[itemKey(a[i], i)] = a[i];
  var out = [];
  var key;
  var prev;
  for (i = 0; i < b.length; i++) {
    key = itemKey(b[i], i);
    prev = Object.prototype.hasOwnProperty.call(map, key) ? map[key] : a[i];
    out.push(lerpScalarFields(prev, b[i], t, fields));
  }
  return out;
}

function lerpScene(from, to, t) {
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
    out.rects = lerpList(src.rects, dest.rects, k, ["x", "y", "w", "h", "value"]);
  if (dest.violins || src.violins)
    out.violins = lerpList(src.violins, dest.violins, k, ["points", "x"]);
  if (dest.points || src.points) out.points = lerpList(src.points, dest.points, k, ["x", "y"]);
  if (dest.wedges || src.wedges)
    out.wedges = lerpList(src.wedges, dest.wedges, k, [
      "cx",
      "cy",
      "r",
      "start",
      "end",
      "area",
      "value",
    ]);
  if (dest.nodes || src.nodes)
    out.nodes = lerpList(src.nodes, dest.nodes, k, ["x", "y", "w", "h", "value"]);
  if (dest.links || src.links)
    out.links = lerpList(src.links, dest.links, k, ["x0", "y0", "x1", "y1", "width", "value"]);
  if (dest.polylines || src.polylines)
    out.polylines = lerpList(src.polylines, dest.polylines, k, ["points"]);
  if (dest.layers || src.layers)
    out.layers = lerpList(src.layers, dest.layers, k, ["top", "bottom"]);
  if (dest.arcs || src.arcs)
    out.arcs = lerpList(src.arcs, dest.arcs, k, [
      "cx",
      "cy",
      "innerR",
      "outerR",
      "start",
      "end",
      "value",
    ]);
  if (dest.axes || src.axes) out.axes = lerpList(src.axes, dest.axes, k, ["x", "y", "angle"]);
  if (dest.polygons || src.polygons)
    out.polygons = lerpList(src.polygons, dest.polygons, k, ["points"]);
  if (dest.bars || src.bars)
    out.bars = lerpList(src.bars, dest.bars, k, ["x", "y", "w", "h", "value", "origin"]);
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
    ridgelinePaths: ridgelinePaths,
    horizonBands: horizonBands,
    treemapRects: treemapRects,
    violinPaths: violinPaths,
    beeswarmPoints: beeswarmPoints,
    nightingaleWedges: nightingaleWedges,
    sankeyLayout: sankeyLayout,
    parallelPolylines: parallelPolylines,
    calendarCells: calendarCells,
    streamgraphLayers: streamgraphLayers,
    sunburstArcs: sunburstArcs,
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
    radarAxes: radarAxes,
    parallelAxes: parallelAxes,
  };
}
