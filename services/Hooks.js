// ~/.config/omarchy/hooks/<name>.d/ listing helpers. Snapshot emits the file list.

var HOOK_TYPES = [
  {
    id: "theme-set",
    label: "Theme set",
    when: "After you switch themes.",
    arg: "the snake-cased theme name that was just set",
    runArg: "theme",
  },
  {
    id: "font-set",
    label: "Font set",
    when: "After you change the monospace font.",
    arg: "the snake-cased font name that was just set",
    runArg: "font",
  },
  {
    id: "post-boot",
    label: "After boot",
    when: "After the desktop starts.",
    arg: "",
    runArg: "",
  },
  {
    id: "post-update",
    label: "After update",
    when: "During omarchy update, after packages and migrations.",
    arg: "",
    runArg: "",
  },
  {
    id: "pre-refresh-pacman",
    label: "Before pacman refresh",
    when: "Before omarchy refresh pacman re-syncs packages.",
    arg: "",
    runArg: "",
  },
  {
    id: "battery-low",
    label: "Low battery",
    when: "When Omarchy sends the low-battery notification.",
    arg: "the current battery percentage",
    runArg: "",
  },
];

function types() {
  return HOOK_TYPES;
}

function typeIds() {
  var out = [];
  for (var i = 0; i < HOOK_TYPES.length; i++) out.push(HOOK_TYPES[i].id);
  return out;
}

function isType(id) {
  var name = String(id || "");
  for (var i = 0; i < HOOK_TYPES.length; i++) {
    if (HOOK_TYPES[i].id === name) return true;
  }
  return false;
}

function isHookId(id) {
  var name = String(id || "");
  if (isType(name)) return true;
  return /^[a-z0-9][a-z0-9-]*$/.test(name);
}

function typeInfo(id) {
  var name = String(id || "");
  for (var i = 0; i < HOOK_TYPES.length; i++) {
    if (HOOK_TYPES[i].id === name) return HOOK_TYPES[i];
  }
  if (!isHookId(name)) return null;
  return { id: name, label: name, when: "A hook folder on this machine.", arg: "", runArg: "" };
}

function labelFor(id) {
  var info = typeInfo(id);
  return info ? info.label : "";
}

function whenFor(id) {
  var info = typeInfo(id);
  return info ? info.when : "";
}

function argFor(id) {
  var info = typeInfo(id);
  return info ? info.arg : "";
}

function runArgFor(id) {
  var info = typeInfo(id);
  return info ? info.runArg : "";
}

function options() {
  var out = [];
  for (var i = 0; i < HOOK_TYPES.length; i++) {
    out.push({ value: HOOK_TYPES[i].id, label: HOOK_TYPES[i].label });
  }
  return out;
}

function eventBlurb(id) {
  var info = typeInfo(id);
  if (!info) return "";
  if (info.arg) return info.when + " $1 is " + info.arg + ".";
  return info.when + " This event has no extra argument.";
}

function sanitizeName(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 64) return "";
  if (text.indexOf("/") !== -1 || text.indexOf("..") !== -1) return "";
  if (text.length >= 7 && text.substring(text.length - 7) === ".sample") return "";
  if (!/^[A-Za-z0-9][A-Za-z0-9._+-]*$/.test(text)) return "";
  if (!/\.[A-Za-z0-9]+$/.test(text)) text += ".sh";
  return text;
}

function sanitizeLine(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 512) return "";
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  return text;
}

function scriptBody(type, command) {
  var line = sanitizeLine(command);
  if (!line) return "";
  var arg = argFor(type);
  var comment = arg ? "# $1 is " + arg + "." : "# This event has no extra argument.";
  return "#!/bin/bash\n" + comment + "\n" + line + "\n";
}

function destHint(type, name) {
  var hook = String(type || "");
  var file = sanitizeName(name);
  if (!isHookId(hook) || !file) return "";
  return "~/.config/omarchy/hooks/" + hook + ".d/" + file;
}

function sanitizeItem(raw) {
  if (!raw || typeof raw !== "object") return null;
  var type = String(raw.type || "");
  if (!isHookId(type)) return null;
  var name = String(raw.name || "");
  if (!name || name.indexOf("/") !== -1 || name.indexOf("..") !== -1) return null;
  var path = String(raw.path || "");
  if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return null;
  return {
    type: type,
    name: name,
    path: path,
    sample: raw.sample === true,
    flat: raw.flat === true,
  };
}

function parseListing(raw) {
  var list = Array.isArray(raw) ? raw : [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var item = sanitizeItem(list[i]);
    if (item) out.push(item);
  }
  return out;
}

function itemsFor(raw, type) {
  var name = String(type || "");
  var list = parseListing(raw);
  var out = [];
  for (var i = 0; i < list.length; i++) {
    if (list[i].type === name) out.push(list[i]);
  }
  return out;
}

function displayTypes(raw) {
  var seen = {};
  var out = [];
  var i;
  for (i = 0; i < HOOK_TYPES.length; i++) {
    seen[HOOK_TYPES[i].id] = true;
    out.push(HOOK_TYPES[i]);
  }
  var list = parseListing(raw);
  for (i = 0; i < list.length; i++) {
    var id = list[i].type;
    if (seen[id]) continue;
    var extra = typeInfo(id);
    if (!extra) continue;
    seen[id] = true;
    out.push(extra);
  }
  return out;
}
