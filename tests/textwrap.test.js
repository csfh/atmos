const { load, assertEqual } = require("./harness");

const wrap = load("services/TextWrap.js");
function ch(s) {
  return s.length;
}

assertEqual(
  wrap.prettyWrap("The quick brown fox jumps", ch, 20, 1),
  "The quick brown\nfox jumps",
  "prettyWrap balances a leftover last word",
);
assertEqual(
  wrap.balanceWrap("aaa bbb ccc ddd", ch, 7, 1),
  "aaa bbb\nccc ddd",
  "balanceWrap splits even groups",
);
assertEqual(
  wrap.balanceWrap("Word another last", ch, 12, 1),
  "Word another\nlast",
  "balanceWrap keeps the greedy line count",
);
assertEqual(
  wrap.prettyWrap("Word another last", ch, 12, 1),
  "Word\nanother last",
  "prettyWrap pulls a one-word last line up",
);
assertEqual(
  wrap.prettyWrap("Hello world", ch, 20, 1),
  "Hello world",
  "prettyWrap leaves a single line alone",
);
assertEqual(
  wrap.prettyWrap("/home/foo/very-long-path", ch, 10, 1),
  "/home/foo/very-long-path",
  "prettyWrap leaves a path without spaces alone",
);
assertEqual(wrap.prettyWrap("", ch, 20, 1), "", "prettyWrap empty");
assertEqual(wrap.shouldSkip("oneword"), true, "shouldSkip a token without spaces");
assertEqual(wrap.shouldSkip("two words"), false, "shouldSkip false when there is a space");
assertEqual(
  wrap.wrapAll("aaa bbb\nccc ddd", ch, 7, 1, false),
  "aaa bbb\nccc ddd",
  "wrapAll wraps each line separately",
);
assertEqual(
  wrap.wrapAll("aaa bbb", null, 7, 1, false),
  "aaa bbb",
  "wrapAll leaves text when measure is missing",
);
assertEqual(
  wrap.wrapAll("aaa bbb", ch, 0, 1, false),
  "aaa bbb",
  "wrapAll leaves text when maxWidth is zero",
);
assertEqual(
  wrap.splitWords("  alpha   beta ").join(","),
  "alpha,beta",
  "splitWords collapses whitespace",
);
assertEqual(
  wrap.wrapAll("aaa bbb\n\nccc ddd", ch, 7, 1, false),
  "aaa bbb\n\nccc ddd",
  "wrapAll preserves a blank line between paragraphs",
);
assertEqual(
  wrap.wrapAll("aaa bbb", ch, 6, -1, false),
  "aaa bbb",
  "wrapAll treats a negative space as zero",
);
