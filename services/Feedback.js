// Write feedback: one global phase plus the row that most likely caused it.
// A control touches its row just before it writes; the next write result is
// attributed to that row if the touch was recent. Pure so Node can test it.

var TOUCH_MS = 4000;
var SAVED_MS = 2500;

function empty() {
  return { phase: "idle", rowKey: "", message: "", touchedKey: "", touchedAt: 0, settledAt: 0 };
}

function touch(state, key, now) {
  var next = Object.assign({}, state || empty());
  next.touchedKey = String(key || "");
  next.touchedAt = Number(now) || 0;
  return next;
}

function begin(state, now) {
  var next = Object.assign({}, state || empty());
  var fresh = next.touchedKey && Number(now) - next.touchedAt <= TOUCH_MS;
  next.phase = "pending";
  next.rowKey = fresh ? next.touchedKey : "";
  next.message = "";
  next.settledAt = 0;
  return next;
}

function finish(state, ok, message, now) {
  var next = Object.assign({}, state || empty());
  if (next.phase !== "pending") next = begin(next, now);
  next.phase = ok ? "saved" : "failed";
  next.message = ok ? "" : String(message || "Command failed");
  next.settledAt = Number(now) || 0;
  return next;
}

// A saved mark fades; a failure stays until the next write or a dismiss.
function expire(state, now) {
  if (!state || state.phase !== "saved") return state;
  if (Number(now) - state.settledAt < SAVED_MS) return state;
  return Object.assign({}, state, { phase: "idle", rowKey: "", message: "" });
}

function dismiss(state) {
  return Object.assign({}, state || empty(), { phase: "idle", rowKey: "", message: "" });
}

function rowKey(hub, label) {
  return String(hub || "") + "|" + String(label || "");
}

function rowStatus(state, key) {
  if (!state || !key || state.rowKey !== key || state.phase === "idle") {
    return { phase: "idle", message: "" };
  }
  return { phase: state.phase, message: state.message };
}

// Short text for the header chip.
function chipLabel(state) {
  if (!state) return "";
  if (state.phase === "pending") return "Applying…";
  if (state.phase === "saved") return "Saved";
  if (state.phase === "failed") return "Failed";
  return "";
}
