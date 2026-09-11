// Resolve a plain-English request against the hub catalogue.
//
// The design constraint that makes this shippable: the agent never writes.
// It can navigate you somewhere, or it can hand you a plan you confirm, and
// that is the whole of its authority. An LLM that can only ever produce a
// diff is one people will actually leave switched on, and Atmos already has
// the piece that makes it honest -- an import planner that says what a
// change will cost before it costs it.
//
// Resolution is deliberately local first. Most of "take me there" is just
// fuzzy matching over a catalogue that already carries titles, descriptions
// and keywords, and doing that in-process means the common case is instant,
// free, private, and works with no agent installed at all. The agent is the
// fallback for the requests local matching cannot honestly answer.

function norm(text) {
  return (
    String(text || "")
      .toLowerCase()
      // Join across an internal hyphen or apostrophe before splitting on
      // anything else. Splitting them turns "wi-fi" into "wi" and "fi", two
      // fragments that match nothing, so searching the single most obvious
      // word a person could type found everything except Network.
      .replace(/([a-z0-9])[-'’]([a-z0-9])/g, "$1$2")
      .replace(/[^a-z0-9 ]+/g, " ")
      .replace(/\s+/g, " ")
      .replace(/^ | $/g, "")
  );
}

// Filler that appears in half of all plain-English requests and belongs to
// none of them. Left in, these match page descriptions at random and drown
// the one word that actually carried the intent -- "battery" lost to
// Notifications because "my" and "is" scored everywhere.
var STOP_WORDS = [
  "a",
  "an",
  "and",
  "are",
  "at",
  "be",
  "can",
  "do",
  "for",
  "from",
  "how",
  "i",
  "in",
  "is",
  "it",
  "me",
  "my",
  "of",
  "on",
  "or",
  "please",
  "so",
  "the",
  "then",
  "this",
  "to",
  "want",
  "was",
  "way",
  "what",
  "when",
  "where",
  "why",
  "with",
  "you",
  "your",
];

// Keys are prefixed before they go into a lookup object. A bare object is
// not a set: STOP["constructor"] and STOP["toString"] are truthy through the
// prototype, so "constructor" would silently become a stop word and any
// keyword named after an Object member would match everything. The prefix
// puts every key outside that namespace.
function mark(map, word) {
  if (word) map["w_" + word] = true;
}

function marked(map, word) {
  return map["w_" + word] === true;
}

var STOP = {};
for (var si = 0; si < STOP_WORDS.length; si++) mark(STOP, STOP_WORDS[si]);

function words(text) {
  var n = norm(text);
  if (!n) return [];
  var raw = n.split(" ");
  var out = [];
  for (var i = 0; i < raw.length; i++) {
    if (raw[i] && !marked(STOP, raw[i])) out.push(raw[i]);
  }
  // A request made entirely of filler carries no intent, so it gets no
  // answer. Falling back to the raw tokens here undid the whole point of the
  // stop list: "on" came back as Notifications and Applications, which is
  // precisely the substring confusion the list exists to prevent.
  return out;
}

// Intent words that mean "change it", not "show me". Used only to phrase the
// answer honestly -- a request to change still resolves to a destination,
// because navigating is the safe half we can always deliver.
var CHANGE_WORDS = [
  "set",
  "make",
  "change",
  "turn",
  "enable",
  "disable",
  "increase",
  "decrease",
  "raise",
  "lower",
  "stop",
  "start",
];

function looksLikeChange(query) {
  var w = words(query);
  for (var i = 0; i < w.length; i++) {
    for (var j = 0; j < CHANGE_WORDS.length; j++) {
      if (w[i] === CHANGE_WORDS[j]) return true;
    }
  }
  return false;
}

// Score one hub against the query. Title hits beat keyword hits beat
// description hits, because a user who types "bluetooth" means the hub
// called Bluetooth and not the six pages that mention it in passing.
function scoreHub(hub, query) {
  if (!hub) return 0;
  var qs = words(query);
  if (qs.length === 0) return 0;

  var title = norm(hub.title);
  var desc = norm(hub.description);
  var keys = [];
  if (Array.isArray(hub.keywords)) {
    for (var k = 0; k < hub.keywords.length; k++) keys.push(norm(hub.keywords[k]));
  }
  var keyBlob = keys.join(" ");
  var keyWords = {};
  var kwParts = keyBlob.split(" ");
  for (var kw = 0; kw < kwParts.length; kw++) mark(keyWords, kwParts[kw]);

  var score = 0;
  var matched = 0;
  for (var i = 0; i < qs.length; i++) {
    var q = qs[i];
    if (q.length < 2) continue;
    var hit = false;
    if (title === q) {
      score += 100;
      hit = true;
    } else if (title.indexOf(q) !== -1) {
      score += 40;
      hit = true;
    }
    // Whole word scores full. A substring still counts, at roughly half,
    // and only for words of four characters or more -- enough for "blueto"
    // to find Bluetooth without letting "on" match "notifications". The
    // short-fragment case is what makes a resolver confidently wrong.
    if (marked(keyWords, q)) {
      score += 18;
      hit = true;
    } else if (q.length >= 4 && keyBlob.indexOf(q) !== -1) {
      score += 9;
      hit = true;
    }
    // Description matches are the weakest signal and the noisiest, so only
    // a substantial word may claim one. Short fragments match everything.
    if (q.length >= 4 && desc.indexOf(q) !== -1) {
      score += 6;
      hit = true;
    }
    if (hit) matched += 1;
  }

  // Every meaningful word landing somewhere is a much stronger signal than
  // one word landing hard, so reward coverage rather than raw hits.
  var meaningful = 0;
  for (var m = 0; m < qs.length; m++) {
    if (qs[m].length >= 2) meaningful += 1;
  }
  if (meaningful > 0 && matched === meaningful) score += 25;
  return score;
}

function rank(hubs, query, limit) {
  var list = Array.isArray(hubs) ? hubs : [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var s = scoreHub(list[i], query);
    if (s > 0) out.push({ hub: list[i], score: s });
  }
  out.sort(function (a, b) {
    return b.score - a.score;
  });
  var max = limit || 5;
  return out.slice(0, max);
}

// A confident local answer is one clear winner. Anything ambiguous is
// offered as a list instead of guessed at, because silently landing someone
// on the wrong page is worse than asking which they meant.
function resolve(hubs, query) {
  var q = norm(query);
  if (!q) return { kind: "empty", matches: [] };

  var ranked = rank(hubs, query, 5);
  if (ranked.length === 0) {
    return { kind: "none", matches: [], change: looksLikeChange(query) };
  }

  var best = ranked[0];
  var runnerUp = ranked.length > 1 ? ranked[1].score : 0;
  // One candidate and nothing else in the running is a confident answer even
  // when the score is modest -- there is nothing to be ambiguous between.
  // Otherwise a winner has to be clearly ahead, not merely ahead, because a
  // near-tie means the request genuinely was ambiguous.
  var confident =
    ranked.length === 1 ? best.score >= 18 : best.score >= 40 && best.score >= runnerUp * 1.5;

  return {
    kind: confident ? "go" : "choose",
    target: confident ? best.hub : null,
    matches: ranked,
    change: looksLikeChange(query),
  };
}

// Pull the page name back out. An agent will sometimes wrap it in prose no
// matter how firmly the prompt said not to, so match the label when it is
// there and otherwise scan for any known page name.
function readAnswer(text, hubs) {
  var raw = String(text || "");
  var list = Array.isArray(hubs) ? hubs : [];
  var page = "";
  var m = raw.match(/PAGE:\s*([^\n\r]+)/i);
  if (m) page = m[1].replace(/^\s+|\s+$/g, "").replace(/[.*_`"']/g, "");
  var target = null;
  var i;
  if (page) {
    for (i = 0; i < list.length; i++) {
      if (list[i] && norm(list[i].title) === norm(page)) {
        target = list[i];
        break;
      }
    }
  }
  if (!target) {
    var lower = norm(raw);
    for (i = 0; i < list.length; i++) {
      var t = list[i] && list[i].title ? norm(list[i].title) : "";
      if (t && lower.indexOf(t) !== -1) {
        target = list[i];
        break;
      }
    }
  }
  var what = "";
  var w = raw.match(/WHAT:\s*([^\n\r]+)/i);
  if (w) what = w[1].replace(/^\s+|\s+$/g, "");
  else what = raw.replace(/PAGE:[^\n\r]*/i, "").replace(/^\s+|\s+$/g, "");
  return { target: target, what: what };
}

// Asked of the machine's configured agent. The shape is deliberately rigid
// and matches what readAnswer parses: a page name from a fixed list and one
// sentence. An agent given room to write prose writes prose, and then the
// only useful part -- which page -- has to be guessed at.
function agentPrompt(query, hubTitles) {
  var q = String(query || "").replace(/^\s+|\s+$/g, "");
  if (!q) return "";
  var titles = Array.isArray(hubTitles) ? hubTitles.join(", ") : "";
  return (
    "These are the pages of Atmos, the Omarchy settings app:\n" +
    titles +
    "\n\nThe user wants: " +
    q +
    "\n\nDo not change anything on this machine. Answer in exactly two short " +
    "lines and nothing else.\n" +
    "Line 1: PAGE: <one page name from the list above>\n" +
    "Line 2: WHAT: <one sentence on what to change there>\n"
  );
}

// The page reads the catalogue through here so the QML side does not have to
// import Hubs.js separately.
function hubsFrom(list) {
  return Array.isArray(list) ? list : [];
}
