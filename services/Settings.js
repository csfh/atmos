// Settings export and import.
//
// One catalog says which snapshot keys leave the machine, which may come
// back, and what changes when they do. Everything else in this file reads
// that catalog: the Markdown writer, the Markdown reader, and the planner
// that turns a file plus a live snapshot into a reviewable list of changes.
//
// Tiers, from safe to never:
//
//   look      Cosmetic. Reversible, no root, portable anywhere.
//   behavior  Changes how the desktop answers you. Portable, but you
//             should read the consequences first.
//   identity  Machine identity. Root, and rarely right on another host.
//             Off by default even under "Everything".
//   system    Security and system state. Exported as a report, never
//             imported. There is no writer path for these on purpose:
//             a settings file that can enable sshd and passwordless sudo
//             is a privilege escalation delivered as a document.
//
// `key` is a snapshot key. A dot walks into a nested object, so
// "hyprLook.gapsIn" reads snapshot.hyprLook.gapsIn.

var SETTINGS_SCHEMA = 1;

function settingsCatalog() {
  return [
    // Appearance
    entry("theme", "appearance", "Theme", "look", {
      type: "string",
      options: "themes",
      consequence: "Every app that follows the Omarchy theme repaints, including open terminals.",
    }),
    entry("background", "appearance", "Background", "look", {
      type: "string",
      consequence:
        "The wallpaper is part of the theme. A theme you do not have installed leaves the current one.",
    }),
    entry("font", "appearance", "Font", "look", {
      type: "string",
      options: "fonts",
      consequence: "Terminals and the bar re-render. A font that is not installed is refused.",
    }),
    entry("textSize", "appearance", "Text size", "look", { type: "integer" }),
    entry("hyprLook.cursorSize", "appearance", "Cursor size", "look", { type: "integer" }),

    // Windows
    entry("hyprLook.gapsIn", "windows", "Inner gaps", "look", { type: "integer" }),
    entry("hyprLook.gapsOut", "windows", "Outer gaps", "look", { type: "integer" }),
    entry("hyprLook.rounding", "windows", "Corner rounding", "look", { type: "integer" }),
    entry("hyprLook.borderSize", "windows", "Border width", "look", { type: "integer" }),
    entry("hyprLook.activeOpacity", "windows", "Active opacity", "look", { type: "number" }),
    entry("hyprLook.inactiveOpacity", "windows", "Inactive opacity", "look", { type: "number" }),
    entry("hyprLook.layout", "windows", "Tiling layout", "behavior", {
      type: "string",
      consequence:
        "Open windows re-tile. A scrolling layout moves windows off screen instead of shrinking them.",
    }),
    entry("hyprNoGaps", "windows", "No gaps when alone", "look", { type: "boolean" }),
    entry("hyprSquareAspect", "windows", "Square single window", "look", { type: "boolean" }),

    // Bar
    entry("barPosition", "bar", "Bar position", "look", {
      type: "string",
      consequence: "Moving the bar changes which screen edge your windows stop at.",
    }),
    entry("barTransparent", "bar", "Transparent bar", "look", { type: "boolean" }),
    entry("barVisible", "bar", "Show the bar", "behavior", {
      type: "boolean",
      consequence:
        "Hiding the bar hides the clock, tray, and indicators. Super + Space still opens the menu.",
    }),
    entry("clockFormat", "bar", "Clock format", "look", { type: "string" }),
    entry("clockFormatAlt", "bar", "Alternate clock format", "look", { type: "string" }),
    entry("clockWeekStart", "bar", "Week starts on", "look", { type: "string" }),

    // Defaults
    entry("browser", "defaults", "Browser", "behavior", {
      type: "string",
      consequence: "Every link you click, and every web app, opens in this browser instead.",
    }),
    entry("terminal", "defaults", "Terminal", "behavior", {
      type: "string",
      consequence:
        "New terminals use this program. Your keybindings still point at whatever they name directly.",
    }),
    entry("editor", "defaults", "Editor", "behavior", {
      type: "string",
      consequence: "Anything that asks for an editor opens this one, including git.",
    }),
    entry("agent", "defaults", "Coding agent", "behavior", {
      type: "string",
      consequence: "The menu and keybindings that start an agent launch this one.",
    }),
    entry("mimePdf", "defaults", "PDF handler", "behavior", {
      type: "string",
      options: "mimePdfOptions",
      consequence: "Double-clicking a PDF opens this program.",
    }),
    entry("mimeImage", "defaults", "Image handler", "behavior", {
      type: "string",
      options: "mimeImageOptions",
      consequence: "Double-clicking an image opens this program.",
    }),
    entry("mimeVideo", "defaults", "Video handler", "behavior", {
      type: "string",
      options: "mimeVideoOptions",
      consequence: "Double-clicking a video opens this program.",
    }),

    // Idle and light
    entry("idleScreensaver", "idle", "Screensaver after", "behavior", {
      type: "integer",
      consequence: "The screensaver takes over after this many seconds of no input.",
    }),
    entry("idleLock", "idle", "Lock after", "behavior", {
      type: "integer",
      consequence:
        "The screen locks after this many seconds. A short value on a shared desk locks you out mid-read.",
    }),
    entry("stayAwake", "idle", "Stay awake", "behavior", {
      type: "boolean",
      consequence:
        "Staying awake stops the screen from locking or sleeping at all until you turn it off.",
    }),
    entry("screensaverEnabled", "idle", "Screensaver enabled", "behavior", {
      type: "boolean",
      consequence:
        "Turning the screensaver off leaves the desktop on screen until the lock takes over.",
    }),
    entry("nightlight", "idle", "Night light", "look", { type: "boolean" }),
    entry("nightlightTemperature", "idle", "Night light warmth", "look", { type: "integer" }),
    entry("nightlightDay", "idle", "Day starts", "look", { type: "string" }),
    entry("nightlightNight", "idle", "Night starts", "look", { type: "string" }),
    entry("nightlightNightOn", "idle", "Schedule night light", "look", { type: "boolean" }),
    entry("doNotDisturb", "idle", "Do not disturb", "behavior", {
      type: "boolean",
      consequence: "Notifications are held silently while this is on.",
    }),

    // Input
    entry("hyprInput.sensitivity", "input", "Pointer sensitivity", "behavior", {
      type: "number",
      hostBound: true,
      consequence:
        "Pointer speed is tuned per device. A value from another machine usually feels wrong.",
    }),
    entry("hyprInput.accelProfile", "input", "Acceleration", "behavior", {
      type: "string",
      hostBound: true,
    }),
    entry("hyprInput.naturalScroll", "input", "Natural scrolling", "behavior", {
      type: "boolean",
      consequence: "Scrolling reverses direction. This is the change people notice most.",
    }),
    entry("hyprInput.workspaceGesture", "input", "Workspace gesture", "behavior", {
      type: "boolean",
      hostBound: true,
      consequence: "Three-finger swipe changes workspace. Needs a touchpad.",
    }),

    // System identity
    entry("hostname", "system", "Hostname", "identity", {
      type: "string",
      hostBound: true,
      consequence:
        "The machine renames itself. Anything that reaches it by name, including SSH configs and Tailscale, sees the new one.",
    }),
    entry("timezone", "system", "Timezone", "identity", {
      type: "string",
      options: "timezones",
      consequence: "The clock jumps. Calendar reminders shift with it.",
    }),
    entry("locale", "system", "Locale", "identity", {
      type: "string",
      options: "locales",
      consequence:
        "Date, number, and sort order change. Running apps keep the old locale until they restart.",
    }),
    entry("keyboardLayout", "system", "Keyboard layout", "identity", {
      type: "string",
      options: "keyboardLayouts",
      consequence:
        "Keys remap immediately. A layout you cannot type on is hard to undo with the keyboard.",
    }),
    entry("ntp", "system", "Network time", "identity", {
      type: "boolean",
      consequence: "Turning network time off lets the clock drift until you set it by hand.",
    }),
    entry("fullName", "system", "Full name", "identity", { type: "string" }),
    entry("parallelDownloads", "system", "Parallel downloads", "identity", {
      type: "integer",
      consequence: "Edits /etc/pacman.conf. Only affects how fast updates fetch.",
    }),
    entry("dns", "network", "DNS", "identity", {
      type: "string",
      consequence:
        "All name lookups go to a different resolver. A wrong value takes the network down until you change it back.",
    }),

    // Report only. No importer, by design.
    report("sshdEnabled", "security", "SSH server"),
    report("passwordlessSudo", "security", "Passwordless sudo"),
    report("sudolessDocker", "security", "Docker without sudo"),
    report("fingerprintConfigured", "security", "Fingerprint enrolled"),
    report("fido2Configured", "security", "FIDO2 enrolled"),
    report("snapperNumberLimit", "security", "Snapshots kept"),
    report("snapperTimeline", "security", "Timeline snapshots"),
    report("fstrimEnabled", "security", "Scheduled TRIM"),
    report("directBoot", "security", "Direct boot"),
    report("omarchyChannel", "security", "Omarchy channel"),
    report("atmosChannel", "security", "Atmos channel"),
  ];
}

function entry(key, section, label, tier, opts) {
  var o = opts || {};
  return {
    key: key,
    section: section,
    label: label,
    tier: tier,
    type: String(o.type || "string"),
    options: String(o.options || ""),
    hostBound: o.hostBound === true,
    consequence: String(o.consequence || ""),
    importable: tier !== "system",
  };
}

function report(key, section, label) {
  return entry(key, section, label, "system", {});
}

function catalogByKey(catalog) {
  var list = catalog || settingsCatalog();
  var map = {};
  for (var i = 0; i < list.length; i++) map[list[i].key] = list[i];
  return map;
}

// Section order and prose for the exported file. A section the catalog
// does not use is simply never written.
function settingsSections() {
  return [
    {
      id: "appearance",
      title: "Appearance",
      note: "Theme, font, and cursor. Safe to import on any machine.",
    },
    { id: "windows", title: "Windows", note: "Gaps, borders, and how windows tile." },
    { id: "bar", title: "Bar", note: "Where the bar sits and what the clock says." },
    {
      id: "defaults",
      title: "Defaults",
      note: "The programs that open when something asks for a default.",
    },
    {
      id: "idle",
      title: "Idle and light",
      note: "Locking, the screensaver, night light, and notifications.",
    },
    {
      id: "input",
      title: "Input",
      note: "Pointer and touchpad. Tuned per device, so it travels badly.",
    },
    {
      id: "network",
      title: "Network",
      note: "Resolver choice. Wi-Fi passwords are never exported.",
    },
    { id: "system", title: "System", note: "Machine identity. Off by default when you import." },
    {
      id: "security",
      title: "Security",
      note: "Reported so you can read it. Atmos will not import anything here.",
    },
  ];
}

// Presets are the honest answer to an "everything" button: three named
// intents rather than sixty checkboxes.
//
//   look      cosmetic only
//   portable  everything that is true on any machine
//   full      adds machine identity, still never security
function presetKeys(preset, catalog) {
  var list = catalog || settingsCatalog();
  var want = String(preset || "portable");
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var item = list[i];
    if (!item.importable) continue;
    if (want === "look" && item.tier !== "look") continue;
    if (want === "portable" && item.tier === "identity") continue;
    if (want === "portable" && item.hostBound) continue;
    out.push(item.key);
  }
  return out;
}

function readValue(source, key) {
  var parts = String(key || "").split(".");
  var node = source;
  for (var i = 0; i < parts.length; i++) {
    if (node === null || node === undefined || typeof node !== "object") return undefined;
    node = node[parts[i]];
  }
  return node;
}

// ---------------------------------------------------------------------------
// Markdown
// ---------------------------------------------------------------------------
//
// The file is Markdown so a person can read it before running it, and the
// payload lives in fenced blocks so nothing ever has to parse prose. Edit
// the words freely; only ```toml atmos:<section> blocks are read back.

function exportMarkdown(snapshot, keys, meta) {
  var snap = snapshot || {};
  var info = meta || {};
  var selected = keyLookup(keys);
  var catalog = settingsCatalog();
  var byKey = catalogByKey(catalog);
  var sections = settingsSections();
  var lines = [];

  lines.push("# Atmos settings");
  lines.push("");
  lines.push(
    "Settings exported from " +
      quotedOr(info.hostname || snap.hostname, "an Omarchy machine") +
      ".",
  );
  lines.push("Read it before you import it. Atmos shows you every change first.");
  lines.push("");

  var metaBody = [
    tomlLine("schema", SETTINGS_SCHEMA),
    tomlLine("exported", String(info.exported || "")),
    tomlLine("hostname", String(info.hostname || snap.hostname || "")),
    tomlLine("atmos", String(info.atmosRevision || snap.atmosRevision || "")),
    tomlLine("omarchy", String(info.omarchyVersion || snap.omarchyVersion || "")),
    tomlLine("hardware", String(info.hardware || "")),
  ];
  lines.push("```toml atmos:meta");
  lines = lines.concat(metaBody);
  lines.push("```");
  lines.push("");

  for (var s = 0; s < sections.length; s++) {
    var section = sections[s];
    var body = [];
    var reported = [];
    for (var i = 0; i < catalog.length; i++) {
      var item = catalog[i];
      if (item.section !== section.id) continue;
      var value = readValue(snap, item.key);
      if (value === undefined || value === null) continue;
      if (!item.importable) {
        reported.push("- " + item.label + ": " + displayValue(value));
        continue;
      }
      if (!selected[item.key]) continue;
      body.push(tomlLine(item.key, value));
    }
    if (body.length === 0 && reported.length === 0) continue;

    lines.push("## " + section.title);
    lines.push("");
    lines.push(section.note);
    lines.push("");
    if (reported.length > 0) {
      lines = lines.concat(reported);
      lines.push("");
    }
    if (body.length > 0) {
      lines.push("```toml atmos:" + section.id);
      lines = lines.concat(body);
      lines.push("```");
      lines.push("");
    }
  }

  return lines.join("\n").replace(/\n+$/, "") + "\n";
}

function keyLookup(keys) {
  var map = {};
  var list = Array.isArray(keys) ? keys : [];
  for (var i = 0; i < list.length; i++) map[String(list[i])] = true;
  return map;
}

function quotedOr(value, fallback) {
  var text = String(value || "");
  return text.length > 0 ? text : fallback;
}

function displayValue(value) {
  if (value === true) return "on";
  if (value === false) return "off";
  return String(value);
}

// Read every ```toml atmos:<name> block. Returns
// { meta, sections: { name: { key: value } }, errors: [string] }.
function parseSettingsMarkdown(text) {
  var lines = String(text || "").split("\n");
  var sections = {};
  var errors = [];
  var open = "";
  var buffer = [];

  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    var trimmed = line.replace(/^\s+|\s+$/g, "");
    if (!open) {
      var start = trimmed.match(/^```\s*toml\s+atmos:([a-z][a-z0-9-]*)\s*$/);
      if (start) {
        open = start[1];
        buffer = [];
      }
      continue;
    }
    if (trimmed === "```") {
      var parsed = parseTomlLite(buffer, open, errors);
      if (!sections[open]) sections[open] = {};
      for (var key in parsed) {
        if (Object.prototype.hasOwnProperty.call(parsed, key)) sections[open][key] = parsed[key];
      }
      open = "";
      buffer = [];
      continue;
    }
    buffer.push(line);
  }

  if (open) errors.push("The atmos:" + open + " block is never closed");

  var meta = sections.meta || {};
  delete sections.meta;
  return { meta: meta, sections: sections, errors: errors };
}

function parseTomlLite(lines, section, errors) {
  var out = {};
  var list = lines || [];
  for (var i = 0; i < list.length; i++) {
    var raw = String(list[i]).replace(/^\s+|\s+$/g, "");
    if (!raw || raw.charAt(0) === "#") continue;
    var eq = raw.indexOf("=");
    if (eq < 1) {
      errors.push("atmos:" + section + " line " + (i + 1) + " is not a setting: " + raw);
      continue;
    }
    var key = raw.slice(0, eq).replace(/\s+$/, "");
    var rest = raw.slice(eq + 1).replace(/^\s+/, "");
    if (!/^[A-Za-z][A-Za-z0-9]*(\.[A-Za-z][A-Za-z0-9]*)*$/.test(key)) {
      errors.push("atmos:" + section + " has a bad setting name: " + key);
      continue;
    }
    var value = parseTomlValue(rest);
    if (value === undefined) {
      errors.push("atmos:" + section + " could not read a value for " + key + ": " + rest);
      continue;
    }
    out[key] = value;
  }
  return out;
}

function parseTomlValue(raw) {
  var text = String(raw || "").replace(/\s+$/, "");
  if (!text) return undefined;
  if (text === "true") return true;
  if (text === "false") return false;
  if (/^-?[0-9]+$/.test(text)) return parseInt(text, 10);
  if (/^-?[0-9]*\.[0-9]+$/.test(text)) return parseFloat(text);
  if (text.charAt(0) === '"') {
    var closing = text.lastIndexOf('"');
    if (closing <= 0) return undefined;
    var body = text.slice(1, closing);
    if (/[^\\]"/.test(body) || body.indexOf('"') === 0) return undefined;
    return body.replace(/\\"/g, '"').replace(/\\\\/g, "\\");
  }
  if (text.charAt(0) === "[") {
    if (text.charAt(text.length - 1) !== "]") return undefined;
    var inner = text.slice(1, -1).replace(/^\s+|\s+$/g, "");
    if (!inner) return [];
    var parts = inner.split(",");
    var out = [];
    for (var i = 0; i < parts.length; i++) {
      var item = parseTomlValue(parts[i].replace(/^\s+|\s+$/g, ""));
      if (item === undefined) return undefined;
      out.push(item);
    }
    return out;
  }
  return undefined;
}

function tomlLine(key, value) {
  return key + " = " + tomlValue(value);
}

function tomlValue(value) {
  if (value === true) return "true";
  if (value === false) return "false";
  if (typeof value === "number") return isFinite(value) ? String(value) : "0";
  if (Array.isArray(value)) {
    var parts = [];
    for (var i = 0; i < value.length; i++) parts.push(tomlValue(value[i]));
    return "[" + parts.join(", ") + "]";
  }
  return (
    '"' +
    String(value === null || value === undefined ? "" : value)
      .replace(/\\/g, "\\\\")
      .replace(/"/g, '\\"') +
    '"'
  );
}

// ---------------------------------------------------------------------------
// Planning
// ---------------------------------------------------------------------------
//
// One computation serves both the review screen and the apply. The plan you
// read is the plan that runs; there is no second code path that could drift
// away from what you were shown.
//
// Returns:
//   changes   [{ key, section, label, tier, from, to, consequence }]
//   unchanged [key]
//   warnings  [{ key, message }]
//   blocked   [{ key, reason }]

function planImport(doc, snapshot, keys, options) {
  var parsed = doc || { meta: {}, sections: {}, errors: [] };
  var snap = snapshot || {};
  var opts = options || {};
  var catalog = settingsCatalog();
  var byKey = catalogByKey(catalog);
  var selected = Array.isArray(keys) ? keyLookup(keys) : null;

  var changes = [];
  var unchanged = [];
  var warnings = [];
  var blocked = [];

  var errors = Array.isArray(parsed.errors) ? parsed.errors : [];
  for (var e = 0; e < errors.length; e++) blocked.push({ key: "", reason: errors[e] });

  var schema = Number(parsed.meta && parsed.meta.schema);
  if (!isFinite(schema) || schema <= 0) {
    warnings.push({
      key: "",
      message: "No schema version in this file. Reading it as version " + SETTINGS_SCHEMA + ".",
    });
  } else if (schema > SETTINGS_SCHEMA) {
    blocked.push({
      key: "",
      reason:
        "This file is schema " +
        schema +
        " and this Atmos reads " +
        SETTINGS_SCHEMA +
        ". Update Atmos first.",
    });
    return result(changes, unchanged, warnings, blocked);
  }

  var fileHardware = String((parsed.meta && parsed.meta.hardware) || "");
  var liveHardware = String(opts.hardware || "");
  var differentHardware =
    fileHardware.length > 0 && liveHardware.length > 0 && fileHardware !== liveHardware;
  if (differentHardware) {
    warnings.push({
      key: "",
      message: "This file came from different hardware. Device-specific settings are held back.",
    });
  }

  var sections = parsed.sections || {};
  for (var name in sections) {
    if (!Object.prototype.hasOwnProperty.call(sections, name)) continue;
    var values = sections[name];
    for (var key in values) {
      if (!Object.prototype.hasOwnProperty.call(values, key)) continue;
      var item = byKey[key];
      if (!item) {
        warnings.push({ key: key, message: "Atmos does not know this setting. Skipped." });
        continue;
      }
      if (item.section !== name) {
        warnings.push({
          key: key,
          message: "Found under " + name + " but it belongs to " + item.section + ". Skipped.",
        });
        continue;
      }
      if (!item.importable) {
        blocked.push({
          key: key,
          reason: item.label + " is reported only. Atmos never imports security settings.",
        });
        continue;
      }
      if (selected && !selected[key]) continue;

      var to = values[key];
      if (!typeMatches(item.type, to)) {
        blocked.push({
          key: key,
          reason: item.label + " expects " + item.type + " but the file has " + describe(to) + ".",
        });
        continue;
      }
      if (item.options) {
        var allowed = readValue(snap, item.options);
        if (Array.isArray(allowed) && allowed.length > 0 && !allowedContains(allowed, to)) {
          blocked.push({
            key: key,
            reason: displayValue(to) + " is not available on this machine.",
          });
          continue;
        }
      }
      if (item.hostBound && differentHardware) {
        warnings.push({
          key: key,
          message: item.label + " is tied to this machine's hardware. Held back.",
        });
        continue;
      }

      var from = readValue(snap, key);
      if (sameValue(from, to)) {
        unchanged.push(key);
        continue;
      }
      changes.push({
        key: key,
        section: item.section,
        label: item.label,
        tier: item.tier,
        from: from === undefined ? null : from,
        to: to,
        consequence: item.consequence,
      });
    }
  }

  changes.sort(function (a, b) {
    return a.key < b.key ? -1 : a.key > b.key ? 1 : 0;
  });
  unchanged.sort();
  return result(changes, unchanged, warnings, blocked);
}

function result(changes, unchanged, warnings, blocked) {
  return {
    changes: changes,
    unchanged: unchanged,
    warnings: warnings,
    blocked: blocked,
    summary: planSummary(changes, unchanged, warnings, blocked),
  };
}

function planSummary(changes, unchanged, warnings, blocked) {
  var parts = [];
  parts.push(countLabel(changes.length, "change", "changes"));
  if (unchanged.length > 0) parts.push(unchanged.length + " already match");
  if (warnings.length > 0) parts.push(countLabel(warnings.length, "warning", "warnings"));
  if (blocked.length > 0) parts.push(blocked.length + " blocked");
  return parts.join(", ");
}

function countLabel(n, one, many) {
  return n + " " + (n === 1 ? one : many);
}

function typeMatches(type, value) {
  if (type === "boolean") return value === true || value === false;
  if (type === "integer")
    return typeof value === "number" && isFinite(value) && Math.floor(value) === value;
  if (type === "number") return typeof value === "number" && isFinite(value);
  if (type === "string") return typeof value === "string";
  return false;
}

function describe(value) {
  if (value === true || value === false) return "a boolean";
  if (typeof value === "number") return "a number";
  if (Array.isArray(value)) return "a list";
  return "text";
}

function allowedContains(list, value) {
  for (var i = 0; i < list.length; i++) {
    var option = list[i];
    if (option === value) return true;
    if (option && typeof option === "object" && option.value === value) return true;
  }
  return false;
}

function sameValue(a, b) {
  if (typeof a === "number" && typeof b === "number") return Math.abs(a - b) < 1e-9;
  return a === b;
}

// Every change the plan carries, as the argument list its writer takes.
// The executor is `scripts/apply-settings.sh`; this keeps the shape in one
// place so the review screen and the executor cannot disagree.
function planToJson(plan) {
  var list = (plan && plan.changes) || [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    out.push({ key: list[i].key, value: list[i].to });
  }
  return JSON.stringify({ schema: SETTINGS_SCHEMA, changes: out });
}
