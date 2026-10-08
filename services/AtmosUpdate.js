function parseSha(raw) {
  var s = String(raw || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (!/^[0-9a-f]{4,40}$/.test(s)) return "";
  return s;
}

function parseChannel(raw) {
  var s = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (s === "stable" || s === "alpha") return "stable";
  return "";
}

function parseCheckOutput(text) {
  var out = {
    status: "unknown",
    channel: "",
    local: "",
    remote: "",
    short: "",
    summary: "",
  };
  var lines = String(text || "").split("\n");
  for (var i = 0; i < lines.length; i++) {
    var line = String(lines[i] || "").replace(/^\s+|\s+$/g, "");
    if (!line) continue;
    var space = line.indexOf(" ");
    var key = space === -1 ? line : line.slice(0, space);
    var value = space === -1 ? "" : line.slice(space + 1).replace(/^\s+/, "");
    if (key === "status") {
      if (
        value === "current" ||
        value === "behind" ||
        value === "fetch-failed" ||
        value === "packaged"
      )
        out.status = value;
    } else if (key === "channel") out.channel = parseChannel(value);
    else if (key === "local") out.local = parseSha(value);
    else if (key === "remote") out.remote = parseSha(value);
    else if (key === "short") out.short = parseSha(value);
    else if (key === "summary") {
      if (
        value.indexOf("\n") !== -1 ||
        value.indexOf("/") !== -1 ||
        value.indexOf("..") !== -1 ||
        value.length > 200
      )
        continue;
      if (value.charAt(0) === "-") continue;
      out.summary = value;
    }
  }
  if (!out.summary) {
    if (out.status === "behind") out.summary = "A newer Atmos is on stable.";
    else if (out.status === "current") out.summary = "Atmos is up to date.";
    else if (out.status === "fetch-failed") out.summary = "Could not fetch the stable branch.";
    else if (out.status === "packaged")
      out.summary = "Atmos is managed by pacman. Update it with omarchy update.";
  }
  return out;
}
