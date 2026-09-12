// Simple / Everything fold math.
//
// A row or section can opt in with advanced:true. Simple folds those
// without removing them: a non-empty query skips the fold, and a search
// hit pins the landing row (and its section) until the user leaves the hub.
// Mode is session state. Nothing here is persisted.

function normalizeHub(hub) {
  var raw = String(hub || "");
  var slash = raw.indexOf("/");
  return slash === -1 ? raw : raw.substring(0, slash);
}

function emptyState() {
  return {
    simple: false,
    revealedHub: "",
    revealedLabel: "",
    pending: false,
  };
}

function copyState(state) {
  var s = state || emptyState();
  return {
    simple: !!s.simple,
    revealedHub: String(s.revealedHub || ""),
    revealedLabel: String(s.revealedLabel || ""),
    pending: !!s.pending,
  };
}

function revealFromSearch(state, hub, label) {
  var next = copyState(state);
  next.revealedHub = normalizeHub(hub);
  next.revealedLabel = String(label || "");
  next.pending = next.revealedLabel.length > 0;
  if (!next.pending) {
    next.revealedHub = "";
    next.revealedLabel = "";
  }
  return next;
}

// Sidebar / loadHub. A search click clears the find field first, which
// reloads the previous hub before openPage lands on the hit. Pending
// keeps that intermediate load from dropping the pin.
function leaveHub(state, hub) {
  var next = copyState(state);
  if (next.pending) return next;
  var id = normalizeHub(hub);
  if (next.revealedHub && id !== next.revealedHub) {
    next.revealedHub = "";
    next.revealedLabel = "";
  }
  return next;
}

function finishReveal(state, hub) {
  var next = copyState(state);
  var id = normalizeHub(hub);
  var keep = next.revealedHub && next.revealedLabel && id === next.revealedHub;
  next.pending = false;
  if (!keep) {
    next.revealedHub = "";
    next.revealedLabel = "";
  }
  return next;
}

function isRevealed(row, state) {
  var s = state || emptyState();
  if (!s.revealedLabel) return false;
  if (String((row && row.label) || "") !== s.revealedLabel) return false;
  var rowHub = normalizeHub(row && row.hub);
  if (rowHub && s.revealedHub && rowHub !== s.revealedHub) return false;
  return true;
}

function groupRevealed(group, state) {
  var s = state || emptyState();
  if (!s.revealedLabel) return false;
  var hub = normalizeHub(group && group.hub);
  if (hub && s.revealedHub && hub !== s.revealedHub) return false;
  var labels = (group && group.labels) || [];
  var i;
  for (i = 0; i < labels.length; i++) {
    if (String(labels[i] || "") === s.revealedLabel) return true;
  }
  return false;
}

function rowFolded(row, state) {
  if (!row || !row.advanced) return false;
  var s = state || emptyState();
  if (!s.simple) return false;
  if (String(row.query || "").length > 0) return false;
  if (isRevealed(row, s)) return false;
  return true;
}

function groupFolded(group, state) {
  if (!group || !group.advanced) return false;
  var s = state || emptyState();
  if (!s.simple) return false;
  if (String(group.query || "").length > 0) return false;
  if (groupRevealed(group, s)) return false;
  return true;
}

function showModeToggle(hasAdvanced, opts) {
  var o = opts || {};
  if (!hasAdvanced) return false;
  if (o.embed) return false;
  if (String(o.query || "").length > 0) return false;
  return true;
}

// Section "?" walks rows even when Simple folded them. Other hides
// (no match, unavailable) stay omitted.
function helpCollectsRow(row, opts) {
  var o = opts || {};
  if (!row) return false;
  if (row.available === false) return false;
  if (row.matches === false) return false;
  if (row.sectionHelp === false) return false;
  if (row.visible === false && !(o.includeFolded && row.folded === true)) return false;
  return true;
}
