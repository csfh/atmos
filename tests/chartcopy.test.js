const { load, assert, assertEqual } = require("./harness");

const copy = load("services/ChartCopy.js");

const chartKinds = [
  "heatmap",
  "ridgeline",
  "violin",
  "rose",
  "horizon",
  "treemap",
  "sankey",
  "sunburst",
  "icicle",
  "streamgraph",
  "beeswarm",
  "radar",
  "calendar",
  "waterfall",
  "parallel",
];

chartKinds.forEach(function (kind) {
  const text = copy.explainer(kind);
  const key = copy.legend(kind);
  assert(text.length > 20, kind + " has a human explainer");
  assert(text.indexOf("—") === -1, kind + " explainer has no em dash");
  assert(text.indexOf("It's not") === -1, kind + " explainer is affirmative");
  assert(key.length > 20, kind + " has a legend explainer");
  assert(key.indexOf("—") === -1, kind + " legend has no em dash");
  assert(copy.blurb(kind).indexOf(text) !== -1, kind + " blurb includes the explainer");
  assert(copy.blurb(kind).indexOf(key) !== -1, kind + " blurb includes the legend");
});

assertEqual(copy.explainer(""), "", "unknown kind is empty");
assertEqual(copy.explainer("nope"), "", "missing kind is empty");
assert(copy.explainer("histogram").length > 20, "histogram has a human explainer");
assert(copy.legend("histogram").length > 20, "histogram has a legend explainer");
assert(copy.explainer("corebars").length > 20, "corebars has a human explainer");
assert(copy.legend("corebars").length > 20, "corebars has a legend explainer");
assert(copy.explainer("stackedbar").length > 20, "stackedbar has a human explainer");
assert(copy.legend("stackedbar").length > 20, "stackedbar has a legend explainer");
assert(copy.explainer("sparkline").length > 20, "sparkline has a human explainer");
assert(copy.legend("sparkline").length > 20, "sparkline has a legend explainer");
assert(copy.kinds().indexOf("heatmap") !== -1, "kinds lists heatmap");
