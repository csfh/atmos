// ~/.config/hypr/hyprsunset.conf day/night profiles. Prefs writes this file, then restarts hyprsunset.

function padTimePart(n) {
  var s = String(n);
  return s.length < 2 ? "0" + s : s;
}

function parseTime(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  var m = text.match(/^([01]?\d|2[0-3]):([0-5]\d)$/);
  if (!m) return "";
  return padTimePart(Number(m[1])) + ":" + padTimePart(Number(m[2]));
}

function clampTemp(raw, fallback) {
  var n = Math.round(Number(raw));
  if (!isFinite(n)) n = fallback;
  if (n < 3000) n = 3000;
  if (n > 6500) n = 6500;
  return n;
}

function defaultSchedule() {
  return {
    day: "07:00",
    night: "20:00",
    nightOn: false,
    temperature: 4000,
  };
}

function clampSchedule(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var base = defaultSchedule();
  var day = parseTime(src.day) || base.day;
  var night = parseTime(src.night) || base.night;
  return {
    day: day,
    night: night,
    nightOn: src.nightOn === true,
    temperature: clampTemp(src.temperature, base.temperature),
  };
}

function stripConfComments(text) {
  var lines = String(text || "").split("\n");
  var out = [];
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    var hash = line.indexOf("#");
    if (hash !== -1) line = line.substring(0, hash);
    out.push(line);
  }
  return out.join("\n");
}

function parseConf(text) {
  // Omarchy's stock hyprsunset.conf comments out the night profile.
  // Splitting on profile { would treat that example as live.
  var src = stripConfComments(text);
  var base = defaultSchedule();
  var blocks = src.split(/profile\s*\{/);
  var day = "";
  var night = "";
  var nightOn = false;
  var temperature = 0;
  for (var i = 1; i < blocks.length; i++) {
    var body = blocks[i].split("}")[0] || "";
    var timeM = body.match(/time\s*=\s*"?([0-2]?\d:[0-5]\d)"?/);
    var time = timeM ? parseTime(timeM[1]) : "";
    if (!time) continue;
    if (/\bidentity\s*=\s*true\b/.test(body)) {
      day = time;
      continue;
    }
    var tempM = body.match(/temperature\s*=\s*([0-9]+)/);
    if (tempM) {
      night = time;
      nightOn = true;
      temperature = clampTemp(tempM[1], base.temperature);
    }
  }
  return clampSchedule({
    day: day || base.day,
    night: night || base.night,
    nightOn: nightOn,
    temperature: temperature || base.temperature,
  });
}

function serializeConf(raw) {
  var s = clampSchedule(raw);
  var lines = [
    "# Written by atmos. Day leaves the screen untinted.",
    "profile {",
    "    time = " + s.day,
    "    identity = true",
    "}",
  ];
  if (s.nightOn) {
    lines.push("");
    lines.push("profile {");
    lines.push("    time = " + s.night);
    lines.push("    temperature = " + s.temperature);
    lines.push("}");
  }
  return lines.join("\n") + "\n";
}
