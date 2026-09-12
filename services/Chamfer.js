// Geometry for the 45° cut-corner path. QML paints it; Node tests the math.

function devicePixelRatio(dpr) {
  var n = Number(dpr);
  return isFinite(n) && n > 0 ? n : 1;
}

function snap(value, dpr) {
  dpr = devicePixelRatio(dpr);
  return Math.round(Number(value) * dpr) / dpr;
}

function strokeMetrics(strokeWidth, dpr) {
  dpr = devicePixelRatio(dpr);
  var s = Number(strokeWidth);
  if (!isFinite(s) || s <= 0) return { width: 0, inset: 0 };
  var physical = Math.max(1, Math.round(s * dpr));
  var width = physical / dpr;
  return { width: width, inset: width / 2 };
}

function cutSize(cut, width, height, inset) {
  var c = Number(cut);
  var w = Number(width);
  var h = Number(height);
  var o = Number(inset);
  if (!isFinite(c) || c < 0) c = 0;
  if (!isFinite(w) || !isFinite(h) || w <= 0 || h <= 0) return 0;
  if (!isFinite(o) || o < 0) o = 0;
  var max = Math.min(w, h) / 2 - o;
  if (!isFinite(max) || max < 0) return 0;
  return Math.max(0, Math.min(c, max));
}

function colorAlpha(color) {
  if (color == null) return 0;
  if (typeof color === "object" && color.a !== undefined) {
    var objectAlpha = Number(color.a);
    return isFinite(objectAlpha) ? objectAlpha : 0;
  }
  var s = String(color)
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (!s || s === "transparent") return 0;
  if (s.charAt(0) === "#") {
    var hex = s.slice(1);
    if (hex.length === 8) return parseInt(hex.slice(0, 2), 16) / 255;
    if (hex.length === 4) return parseInt(hex.charAt(0) + hex.charAt(0), 16) / 255;
    if (hex.length === 6 || hex.length === 3) return 1;
    return 0;
  }
  var rgba = s.match(/^rgba?\(\s*[\d.]+\s*,\s*[\d.]+\s*,\s*[\d.]+\s*(?:,\s*([\d.]+))?\s*\)$/);
  if (rgba) return rgba[1] === undefined ? 1 : Number(rgba[1]);
  return 1;
}

function shouldPaint(color) {
  return colorAlpha(color) > 0;
}

function clampByte(n) {
  n = Math.round(Number(n));
  if (!isFinite(n) || n < 0) return 0;
  if (n > 255) return 255;
  return n;
}

function cssColor(color) {
  if (color && typeof color === "object" && color.r !== undefined) {
    var a = Number(color.a);
    if (!isFinite(a)) a = 1;
    if (a < 0) a = 0;
    if (a > 1) a = 1;
    return (
      "rgba(" +
      clampByte(color.r * 255) +
      ", " +
      clampByte(color.g * 255) +
      ", " +
      clampByte(color.b * 255) +
      ", " +
      a +
      ")"
    );
  }
  var s = String(color || "transparent");
  if (s.charAt(0) === "#" && s.length === 9) {
    var hex = s.slice(1);
    return (
      "rgba(" +
      parseInt(hex.slice(2, 4), 16) +
      ", " +
      parseInt(hex.slice(4, 6), 16) +
      ", " +
      parseInt(hex.slice(6, 8), 16) +
      ", " +
      parseInt(hex.slice(0, 2), 16) / 255 +
      ")"
    );
  }
  return s;
}

function defaultCorners() {
  return {
    cutTopLeft: true,
    cutTopRight: false,
    cutBottomRight: true,
    cutBottomLeft: false,
  };
}

function pathPoints(width, height, cut, corners, strokeWidth, dpr) {
  var w = Number(width);
  var h = Number(height);
  if (!isFinite(w) || !isFinite(h) || w <= 0 || h <= 0) return [];
  var metrics = strokeMetrics(strokeWidth, dpr);
  var o = metrics.inset;
  var c = snap(cutSize(cut, w, h, o), dpr);
  var flags = corners || defaultCorners();
  var tl = !!flags.cutTopLeft;
  var tr = !!flags.cutTopRight;
  var br = !!flags.cutBottomRight;
  var bl = !!flags.cutBottomLeft;
  // Inset is already on the stroke centre. Rounding it to a whole item
  // pixel would land a 1px hairline between two pixels and smear it.
  var l = o;
  var t = o;
  var r = w - o;
  var b = h - o;
  var pts = [];
  pts.push([l + (tl ? c : 0), t]);
  pts.push([r - (tr ? c : 0), t]);
  if (tr) pts.push([r, t + c]);
  pts.push([r, b - (br ? c : 0)]);
  if (br) pts.push([r - c, b]);
  pts.push([l + (bl ? c : 0), b]);
  if (bl) pts.push([l, b - c]);
  pts.push([l, t + (tl ? c : 0)]);
  return pts;
}
