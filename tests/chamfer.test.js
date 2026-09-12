const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const chamfer = load("services/Chamfer.js");
const chamferQml = fs.readFileSync(path.join(__dirname, "..", "components", "Chamfer.qml"), "utf8");
const buttonSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsButton.qml"),
  "utf8",
);
const dialogSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsDialog.qml"),
  "utf8",
);
const confirmSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsConfirm.qml"),
  "utf8",
);
const groupSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsGroup.qml"),
  "utf8",
);
const shellSrc = fs.readFileSync(path.join(__dirname, "..", "shell.qml"), "utf8");

assert(chamferQml.indexOf("Canvas") !== -1, "Chamfer.qml paints on a Canvas");
assert(chamferQml.indexOf("Chamfer.js") !== -1, "Chamfer.qml uses Chamfer.js for path math");
assert(chamferQml.indexOf("Qt.colorEqual") === -1, "Chamfer.qml does not use Qt.colorEqual");
assert(chamferQml.indexOf("cutTopLeft: true") !== -1, "Chamfer defaults to a top-left cut");
assert(chamferQml.indexOf("cutBottomRight: true") !== -1, "Chamfer defaults to a bottom-right cut");
assert(
  chamferQml.indexOf("cutTopRight: false") !== -1,
  "Chamfer does not cut the top-right by default",
);
assert(
  chamferQml.indexOf("cutBottomLeft: false") !== -1,
  "Chamfer does not cut the bottom-left by default",
);
assert(chamferQml.indexOf("Window.window") !== -1, "Chamfer reads the window devicePixelRatio");

assertEqual(chamfer.devicePixelRatio(0), 1, "devicePixelRatio treats 0 as 1");
assertEqual(chamfer.devicePixelRatio(-2), 1, "devicePixelRatio treats a negative as 1");
assertEqual(chamfer.devicePixelRatio("nope"), 1, "devicePixelRatio treats NaN as 1");
assertEqual(chamfer.devicePixelRatio(1.5), 1.5, "devicePixelRatio keeps a fractional ratio");

assertEqual(chamfer.snap(0.49, 1), 0, "snap at 1x rounds 0.49 down");
assertEqual(chamfer.snap(0.5, 1), 1, "snap at 1x rounds 0.5 up");
assertEqual(chamfer.snap(0.5, 2), 0.5, "snap at 2x keeps a half item pixel");

const hairline = chamfer.strokeMetrics(1, 1);
assertEqual(hairline.width, 1, "1x hairline stays one item pixel");
assertEqual(hairline.inset, 0.5, "1x hairline insets by half a stroke");
const retina = chamfer.strokeMetrics(1, 2);
assertEqual(retina.width, 1, "2x hairline stays one item pixel");
assertEqual(retina.inset, 0.5, "2x hairline still insets by half a stroke");
const fractional = chamfer.strokeMetrics(1, 1.5);
assertEqual(fractional.width, 2 / 1.5, "1.5x hairline snaps to two physical pixels");
assertEqual(fractional.inset, 1 / 1.5, "1.5x inset is half the snapped stroke");
assertEqual(chamfer.strokeMetrics(0, 1).width, 0, "zero stroke has no width");
assertEqual(chamfer.strokeMetrics(-1, 1).inset, 0, "negative stroke has no inset");

assertEqual(chamfer.cutSize(4, 80, 28, 0.5), 4, "cutSize keeps a cut that fits");
assertEqual(
  chamfer.cutSize(40, 20, 28, 0.5),
  9.5,
  "cutSize clamps to half the short side minus inset",
);
assertEqual(chamfer.cutSize(-3, 80, 28, 0.5), 0, "cutSize rejects a negative cut");
assertEqual(chamfer.cutSize(4, 0, 28, 0.5), 0, "cutSize is zero when width is zero");
assertEqual(chamfer.cutSize(4, 4, 4, 3), 0, "cutSize is zero when inset eats the rect");

assertEqual(chamfer.colorAlpha("transparent"), 0, "transparent has alpha 0");
assertEqual(chamfer.colorAlpha("#00000000"), 0, "Qt transparent black has alpha 0");
assertEqual(chamfer.colorAlpha("#00ffffff"), 0, "Qt transparent white has alpha 0");
assertEqual(
  chamfer.colorAlpha({ r: 1, g: 1, b: 1, a: 0.04 }),
  0.04,
  "colorAlpha reads an object alpha",
);
assertEqual(chamfer.colorAlpha("#cacccc"), 1, "opaque hex has alpha 1");
assert(chamfer.shouldPaint({ r: 0.8, g: 0.8, b: 0.8, a: 0.04 }), "a 0.04 fill still paints");
assert(!chamfer.shouldPaint("transparent"), "transparent does not paint");
assert(!chamfer.shouldPaint("#00ffffff"), "transparent white does not paint");
assert(!chamfer.shouldPaint(null), "a missing color does not paint");

assertEqual(
  chamfer.cssColor({ r: 1, g: 0, b: 0, a: 0.22 }),
  "rgba(255, 0, 0, 0.22)",
  "cssColor keeps alpha from a QML color",
);
assertEqual(
  chamfer.cssColor("#80cacccc"),
  "rgba(202, 204, 204, 0.5019607843137255)",
  "cssColor converts Qt #AARRGGBB so Canvas does not drop alpha",
);

const defaults = chamfer.defaultCorners();
assert(defaults.cutTopLeft && defaults.cutBottomRight, "default corners are the TL+BR diagonal");
assert(!defaults.cutTopRight && !defaults.cutBottomLeft, "default corners leave TR and BL square");

assertEqual(
  chamfer.pathPoints(0, 28, 4, defaults, 1, 1).length,
  0,
  "pathPoints is empty when width is 0",
);
assertEqual(
  chamfer.pathPoints(80, 0, 4, defaults, 1, 1).length,
  0,
  "pathPoints is empty when height is 0",
);

const box = chamfer.pathPoints(80, 28, 4, defaults, 1, 1);
assertEqual(box.length, 6, "TL+BR path has six vertices");
assertEqual(box[0][0], 4.5, "TL cut starts inset plus cut along the top");
assertEqual(box[0][1], 0.5, "top edge sits on the 1x hairline");
assertEqual(box[1][0], 79.5, "uncut top-right is the inset right edge");
assertEqual(box[2][0], 79.5, "right edge is vertical after the top");
assertEqual(box[2][1], 23.5, "BR cut starts inset plus cut up from the bottom");
assertEqual(box[3][0], 75.5, "BR cut lands on the bottom edge");
assertEqual(box[3][1], 27.5, "bottom edge sits on the 1x hairline");
assertEqual(box[5][0], 0.5, "left edge is the inset");
assertEqual(box[5][1], 4.5, "TL cut ends inset plus cut down from the top");

const octagon = chamfer.pathPoints(
  80,
  28,
  4,
  { cutTopLeft: true, cutTopRight: true, cutBottomRight: true, cutBottomLeft: true },
  1,
  1,
);
assertEqual(octagon.length, 8, "all four cuts make eight vertices");

assert(buttonSrc.indexOf("Chamfer") !== -1, "PrefsButton uses Chamfer");
assert(buttonSrc.indexOf("Theme.chamferSm") !== -1, "PrefsButton uses the small chamfer token");
assert(dialogSrc.indexOf("Chamfer") === -1, "PrefsDialog stays a square card");
assert(confirmSrc.indexOf("Chamfer {") === -1, "PrefsConfirm stays a square card");
assert(groupSrc.indexOf("Chamfer") === -1, "PrefsGroup stays a square card");
assert(shellSrc.indexOf("Chamfer") === -1, "sidebar chrome does not use Chamfer");

function walk(dir, acc) {
  acc = acc || [];
  fs.readdirSync(dir, { withFileTypes: true }).forEach(function (ent) {
    if (ent.name === "node_modules") return;
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(full, acc);
    else if (ent.name.endsWith(".qml")) acc.push(full);
  });
  return acc;
}

const chamferUsers = walk(path.join(__dirname, ".."))
  .map(function (file) {
    return path.relative(path.join(__dirname, ".."), file);
  })
  .filter(function (rel) {
    if (rel === "components/Chamfer.qml") return false;
    const src = fs.readFileSync(path.join(__dirname, "..", rel), "utf8");
    return src.indexOf("Chamfer") !== -1;
  });
assertEqual(
  chamferUsers.join(","),
  "components/PrefsButton.qml",
  "only PrefsButton instantiates Chamfer",
);
