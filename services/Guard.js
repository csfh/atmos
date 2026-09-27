// Keep-or-revert for a write that can blank a display or strand the keyboard.
// The new value applies immediately. This module only remembers the values
// from when the bar opened and decides when they should be written back.
// A later guarded edit resets the deadline and keeps that original baseline.

var WINDOW_MS = 12000;

function clone(value) {
  if (value === null || typeof value === "undefined") return value;
  try {
    return JSON.parse(JSON.stringify(value));
  } catch (e) {
    return value;
  }
}

function empty() {
  return { open: false, deadline: 0, reverts: [] };
}

function normalize(state) {
  if (!state || state.open !== true) return empty();
  return {
    open: true,
    deadline: Number(state.deadline) || 0,
    reverts: Array.isArray(state.reverts) ? state.reverts : [],
  };
}

function open(state, change, now) {
  var base = normalize(state);
  now = Number(now);
  if (!isFinite(now)) now = 0;
  if (!change || !change.id || !change.revert || typeof change.revert !== "object")
    return { state: base, armed: false };
  var id = String(change.id);
  if (!id) return { state: base, armed: false };
  var reverts = base.open ? base.reverts.slice() : [];
  var found = false;
  var i;
  for (i = 0; i < reverts.length; i++) {
    if (reverts[i] && reverts[i].id === id) {
      found = true;
      break;
    }
  }
  if (!found) reverts.push({ id: id, revert: clone(change.revert) });
  return {
    state: { open: true, deadline: now + WINDOW_MS, reverts: reverts },
    armed: true,
  };
}

function keep() {
  return empty();
}

function fireList(state) {
  var reverts = state.reverts || [];
  var out = [];
  var i;
  for (i = reverts.length - 1; i >= 0; i--) {
    if (reverts[i] && reverts[i].revert) out.push(clone(reverts[i].revert));
  }
  return out;
}

function revert(state) {
  var base = normalize(state);
  if (!base.open) return { state: empty(), fire: [] };
  return { state: empty(), fire: fireList(base) };
}

function tick(state, now) {
  var base = normalize(state);
  if (!base.open) return { state: base, fire: [] };
  if (!(Number(now) >= base.deadline)) return { state: base, fire: [] };
  return { state: empty(), fire: fireList(base) };
}

function secondsLeft(state, now) {
  var base = normalize(state);
  if (!base.open) return 0;
  var ms = base.deadline - Number(now);
  if (!(ms > 0)) return 0;
  return Math.ceil(ms / 1000);
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    WINDOW_MS: WINDOW_MS,
    empty: empty,
    open: open,
    keep: keep,
    revert: revert,
    tick: tick,
    secondsLeft: secondsLeft,
  };
}
