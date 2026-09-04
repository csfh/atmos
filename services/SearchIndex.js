"use strict";

const fs = require("fs");
const os = require("os");
const path = require("path");
const vm = require("vm");
const { DatabaseSync } = require("node:sqlite");

function shellConfig() {
  if (!shellConfig._loaded) {
    const src = fs.readFileSync(path.join(__dirname, "ShellConfig.js"), "utf8");
    const ctx = {};
    vm.runInNewContext(src, ctx, { filename: "ShellConfig.js" });
    shellConfig._loaded = ctx;
  }
  return shellConfig._loaded;
}

const HUBS = [
  {
    id: "appearance",
    title: "Appearance",
    description: "Theme, wallpaper, fonts, and how the desktop looks.",
    keywords: ["theme", "background", "wallpaper", "font", "text", "size", "palette", "nightlight"],
  },
  {
    id: "display",
    title: "Displays",
    description: "Monitors, scale, and brightness.",
    keywords: ["monitor", "scale", "brightness"],
  },
  {
    id: "windows",
    title: "Windows",
    description: "Gaps, borders, bindings, and window rules.",
    keywords: ["gaps", "bind", "window"],
  },
  {
    id: "bar",
    title: "Bar",
    description: "Position, clock, and widgets.",
    keywords: ["bar", "clock", "tray"],
  },
  {
    id: "notifications",
    title: "Notifications",
    description: "Do not disturb and reminders.",
    keywords: ["dnd", "reminder"],
  },
  {
    id: "input",
    title: "Input",
    description: "Pointer, keyboard, and gestures.",
    keywords: ["mouse", "keyboard", "scroll", "inertia"],
  },
  {
    id: "accessibility",
    title: "Accessibility",
    description: "Motion, text size, and assistive tools.",
    keywords: ["a11y", "motion"],
  },
  {
    id: "sound",
    title: "Sound",
    description: "Volume, sinks, and sources.",
    keywords: ["audio", "volume"],
  },
  {
    id: "capture",
    title: "Capture",
    description: "Screenshots, recordings, and OCR.",
    keywords: ["screenshot", "record"],
  },
  {
    id: "hardware",
    title: "Hardware",
    description: "CPU, GPU, memory, and firmware.",
    keywords: ["cpu", "gpu", "memory"],
  },
  {
    id: "disks",
    title: "Disks",
    description: "Drives, snapshots, and swap.",
    keywords: ["drive", "snapper", "swap"],
  },
  {
    id: "network",
    title: "Network",
    description: "Wi-Fi, Bluetooth, DNS, and speed test.",
    keywords: ["wifi", "bluetooth", "dns"],
  },
  {
    id: "power",
    title: "Power",
    description: "Profiles and battery.",
    keywords: ["battery", "profile"],
  },
  {
    id: "idle",
    title: "Idle and lock",
    description: "Screensaver, lock, and lid.",
    keywords: ["lock", "screensaver"],
  },
  {
    id: "defaults",
    title: "Defaults",
    description: "Browser, terminal, editor, and MIME.",
    keywords: ["browser", "terminal"],
  },
  {
    id: "applications",
    title: "Applications",
    description: "Desktop, TUI, and web launchers.",
    keywords: ["app", "launcher"],
  },
  {
    id: "software",
    title: "Software",
    description: "Packages and extras.",
    keywords: ["install", "package"],
  },
  {
    id: "hooks",
    title: "Hooks",
    description: "Theme-set and other scripts.",
    keywords: ["hook", "script"],
  },
  {
    id: "security",
    title: "Security",
    description: "Fingerprint, SSH, and sudo.",
    keywords: ["ssh", "fingerprint"],
  },
  {
    id: "accounts",
    title: "Accounts",
    description: "Face, password, users, and groups.",
    keywords: ["avatar", "user", "group", "password"],
  },
  {
    id: "system",
    title: "System",
    description: "Host, locale, updates, and about.",
    keywords: ["hostname", "locale", "update"],
  },
];

const FILE_HUB = {
  "AppearancePage.qml": "appearance",
  "DisplaysPage.qml": "display",
  "HardwarePage.qml": "hardware",
  "WindowsPage.qml": "windows",
  "InputPage.qml": "input",
  "AccessibilityPage.qml": "accessibility",
  "SoundPage.qml": "sound",
  "CapturePage.qml": "capture",
  "DisksPage.qml": "disks",
  "BarPage.qml": "bar",
  "NotificationsPage.qml": "notifications",
  "DefaultsPage.qml": "defaults",
  "ApplicationsPage.qml": "applications",
  "SoftwarePage.qml": "software",
  "NetworkPage.qml": "network",
  "PowerPage.qml": "power",
  "IdlePage.qml": "idle",
  "SecurityPage.qml": "security",
  "AccountsPage.qml": "accounts",
  "HooksPage.qml": "hooks",
  "SystemPage.qml": "system",
};

function hubTitle(hub) {
  for (let i = 0; i < HUBS.length; i++) {
    if (HUBS[i].id === hub) return HUBS[i].title;
  }
  return hub;
}

function hubRows() {
  return HUBS.map(function (hub) {
    return {
      id: hub.id,
      hub: hub.id,
      hubTitle: hub.title,
      label: hub.title,
      description: hub.description,
      hint: "",
      detail: "",
      keywords: hub.keywords,
    };
  });
}

function indexPath(env) {
  const e = env || process.env;
  if (e.ATMOS_SEARCH_INDEX) return e.ATMOS_SEARCH_INDEX;
  const cacheHome = e.XDG_CACHE_HOME || path.join(e.HOME || os.homedir(), ".cache");
  return path.join(cacheHome, "atmos", "search.sqlite");
}

function openIndex(file) {
  const dest = file || ":memory:";
  if (dest !== ":memory:") fs.mkdirSync(path.dirname(dest), { recursive: true });
  const db = new DatabaseSync(dest);
  db.exec(`
    CREATE TABLE IF NOT EXISTS rows (
      id TEXT PRIMARY KEY,
      hub TEXT NOT NULL,
      hub_title TEXT NOT NULL DEFAULT '',
      label TEXT NOT NULL,
      description TEXT NOT NULL DEFAULT '',
      hint TEXT NOT NULL DEFAULT '',
      detail TEXT NOT NULL DEFAULT '',
      keywords TEXT NOT NULL DEFAULT '[]',
      haystack TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS state (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );
  `);
  return db;
}

function rowHaystack(row) {
  const parts = [row.label, row.description, row.hint, row.detail];
  const list = row.keywords || [];
  for (let i = 0; i < list.length; i++) parts.push(list[i]);
  return shellConfig().joinSearchHaystack(parts);
}

function ingestRows(db, rows) {
  const insert = db.prepare(`
    INSERT INTO rows (id, hub, hub_title, label, description, hint, detail, keywords, haystack)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT(id) DO UPDATE SET
      hub = excluded.hub,
      hub_title = excluded.hub_title,
      label = excluded.label,
      description = excluded.description,
      hint = excluded.hint,
      detail = excluded.detail,
      keywords = excluded.keywords,
      haystack = excluded.haystack
  `);
  const list = rows || [];
  for (let i = 0; i < list.length; i++) {
    const row = list[i];
    const hub = String(row.hub || "");
    const keywords = Array.isArray(row.keywords) ? row.keywords : [];
    insert.run(
      String(row.id || hub + "/" + i),
      hub,
      String(row.hubTitle || hubTitle(hub)),
      String(row.label || ""),
      String(row.description || ""),
      String(row.hint || ""),
      String(row.detail || ""),
      JSON.stringify(keywords),
      rowHaystack({
        label: row.label,
        description: row.description,
        hint: row.hint,
        detail: row.detail,
        keywords: keywords,
      }),
    );
  }
}

function flattenSnapshot(value, prefix, out) {
  if (value === null || value === undefined) {
    out[prefix] = "";
    return;
  }
  if (typeof value !== "object") {
    out[prefix] = String(value);
    return;
  }
  if (Array.isArray(value)) {
    out[prefix] = JSON.stringify(value);
    return;
  }
  const keys = Object.keys(value);
  if (keys.length === 0 && prefix) {
    out[prefix] = "{}";
    return;
  }
  for (let i = 0; i < keys.length; i++) {
    const key = prefix ? prefix + "." + keys[i] : keys[i];
    flattenSnapshot(value[keys[i]], key, out);
  }
}

function ingestSnapshot(db, snapshot) {
  const flat = {};
  flattenSnapshot(snapshot && typeof snapshot === "object" ? snapshot : {}, "", flat);
  const insert = db.prepare(`
    INSERT INTO state (key, value) VALUES (?, ?)
    ON CONFLICT(key) DO UPDATE SET value = excluded.value
  `);
  const keys = Object.keys(flat);
  for (let i = 0; i < keys.length; i++) insert.run(keys[i], flat[keys[i]]);
}

function getState(db, key) {
  const row = db.prepare("SELECT value FROM state WHERE key = ?").get(String(key || ""));
  return row ? row.value : null;
}

function likeNeedle(query) {
  return (
    "%" +
    String(query || "")
      .replace(/\\/g, "\\\\")
      .replace(/%/g, "\\%")
      .replace(/_/g, "\\_") +
    "%"
  );
}

function mapRow(row) {
  let keywords = [];
  try {
    keywords = JSON.parse(row.keywords);
  } catch (e) {
    keywords = [];
  }
  if (!Array.isArray(keywords)) keywords = [];
  return {
    id: row.id,
    hub: row.hub,
    hubTitle: row.hub_title,
    label: row.label,
    description: row.description,
    hint: row.hint,
    detail: row.detail,
    keywords: keywords,
  };
}

function queryRows(db, query) {
  const q = shellConfig().normalizeQuery(query);
  if (!q) {
    return db.prepare("SELECT * FROM rows ORDER BY hub, label").all().map(mapRow);
  }
  return db
    .prepare("SELECT * FROM rows WHERE haystack LIKE ? ESCAPE '\\' ORDER BY hub, label")
    .all(likeNeedle(q))
    .map(mapRow);
}

function stringProp(block, name) {
  const re = new RegExp(name + ':\\s*"((?:\\\\.|[^"\\\\])*)"');
  const m = String(block || "").match(re);
  if (!m) return "";
  try {
    return JSON.parse('"' + m[1] + '"');
  } catch (e) {
    return m[1];
  }
}

function keywordsProp(block) {
  const m = String(block || "").match(/keywords:\s*\[([^\]]*)\]/);
  if (!m) return [];
  const out = [];
  const re = /"((?:\\.|[^"\\])*)"/g;
  let hit;
  while ((hit = re.exec(m[1]))) {
    try {
      out.push(JSON.parse('"' + hit[1] + '"'));
    } catch (e) {
      out.push(hit[1]);
    }
  }
  return out;
}

function namedBlocks(src, name) {
  const needle = name + " {";
  const out = [];
  let from = 0;
  while (true) {
    const start = src.indexOf(needle, from);
    if (start < 0) break;
    const brace = src.indexOf("{", start);
    if (brace < 0) break;
    let depth = 0;
    let end = brace;
    for (; end < src.length; end++) {
      const ch = src[end];
      if (ch === "{") depth++;
      else if (ch === "}") {
        depth--;
        if (depth === 0) {
          end++;
          break;
        }
      }
    }
    out.push(src.slice(brace, end));
    from = start + needle.length;
  }
  return out;
}

function hubForPage(rel) {
  const base = String(rel || "").replace(/\\/g, "/");
  if (base.indexOf("appearance/") === 0) return "appearance";
  if (base.indexOf("network/") === 0) return "network";
  if (base.indexOf("windows/") === 0) return "windows";
  return FILE_HUB[base] || "";
}

function slug(text) {
  return (
    String(text || "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "") || "row"
  );
}

function rowsFromQml(src, hub) {
  const title = hubTitle(hub);
  const out = [];
  const kinds = ["PrefsRow", "PrefsLink", "PrefsGroup"];
  for (let k = 0; k < kinds.length; k++) {
    const blocks = namedBlocks(src, kinds[k]);
    for (let i = 0; i < blocks.length; i++) {
      const block = blocks[i];
      const label =
        kinds[k] === "PrefsGroup" ? stringProp(block, "title") : stringProp(block, "label");
      if (!label) continue;
      out.push({
        id: hub + "/" + slug(label) + "-" + kinds[k] + "-" + i,
        hub: hub,
        hubTitle: title,
        label: label,
        description: stringProp(block, "description"),
        hint: stringProp(block, "hint"),
        detail: stringProp(block, "detail"),
        keywords: keywordsProp(block),
      });
    }
  }
  return out;
}

function walkQmlFiles(dir, prefix, files) {
  const names = fs.readdirSync(dir);
  for (let i = 0; i < names.length; i++) {
    const name = names[i];
    const full = path.join(dir, name);
    const rel = prefix ? prefix + "/" + name : name;
    if (fs.statSync(full).isDirectory()) walkQmlFiles(full, rel, files);
    else if (name.endsWith(".qml")) files.push({ rel: rel, full: full });
  }
}

function catalogFromQmlDir(pagesDir) {
  if (!pagesDir || !fs.existsSync(pagesDir)) return [];
  const files = [];
  walkQmlFiles(pagesDir, "", files);
  const out = [];
  for (let i = 0; i < files.length; i++) {
    const hub = hubForPage(files[i].rel);
    if (!hub) continue;
    out.push.apply(out, rowsFromQml(fs.readFileSync(files[i].full, "utf8"), hub));
  }
  return out;
}

function defaultCatalog(rootDir) {
  const root = rootDir || path.join(__dirname, "..");
  return hubRows().concat(catalogFromQmlDir(path.join(root, "pages")));
}

function readJsonFile(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function parseArgs(argv) {
  const out = {
    cmd: "",
    query: "",
    key: "",
    rows: "",
    snapshot: "",
    root: path.join(__dirname, ".."),
  };
  const rest = argv.slice();
  const cmd = rest.shift();
  if (cmd === "query" || cmd === "state") out.cmd = cmd;
  else if (cmd) rest.unshift(cmd);
  if (out.cmd === "query") out.query = rest.shift() || "";
  if (out.cmd === "state") out.key = rest.shift() || "";
  while (rest.length) {
    const flag = rest.shift();
    if (flag === "--rows") out.rows = rest.shift() || "";
    else if (flag === "--snapshot") out.snapshot = rest.shift() || "";
    else if (flag === "--root") out.root = rest.shift() || out.root;
  }
  return out;
}

function main(argv) {
  const args = parseArgs(argv);
  if (args.cmd !== "query" && args.cmd !== "state") {
    process.stderr.write(
      "usage: SearchIndex.js query <text> [--rows file] [--snapshot file] [--root dir]\n",
    );
    process.stderr.write("       SearchIndex.js state <key> --snapshot file\n");
    return 2;
  }
  const db = openIndex(indexPath());
  try {
    if (args.rows) ingestRows(db, readJsonFile(args.rows));
    else ingestRows(db, defaultCatalog(args.root));
    if (args.snapshot) ingestSnapshot(db, readJsonFile(args.snapshot));
    if (args.cmd === "query") {
      process.stdout.write(JSON.stringify(queryRows(db, args.query)) + "\n");
      return 0;
    }
    process.stdout.write(JSON.stringify(getState(db, args.key)) + "\n");
    return 0;
  } finally {
    db.close();
  }
}

module.exports = {
  HUBS,
  indexPath,
  openIndex,
  ingestRows,
  ingestSnapshot,
  getState,
  queryRows,
  defaultCatalog,
  catalogFromQmlDir,
  hubRows,
  rowHaystack,
  parseArgs,
  main,
};

if (require.main === module) process.exit(main(process.argv.slice(2)) || 0);
