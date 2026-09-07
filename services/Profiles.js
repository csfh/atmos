// Named bundles of Settings catalog keys. Apply goes through commandFor.

function builtins() {
  return [
    {
      id: "coding",
      title: "Coding",
      description: "Performance power, stay awake, fewer notifications.",
      values: {
        powerProfile: "performance",
        stayAwake: true,
        doNotDisturb: false,
        "hyprLook.animations": true,
      },
    },
    {
      id: "gaming",
      title: "Gaming",
      description: "Performance, tearing, animations off, notifications silenced, stay awake.",
      values: {
        powerProfile: "performance",
        stayAwake: true,
        doNotDisturb: true,
        screensaverEnabled: false,
        "hyprLook.animations": false,
        "hyprLook.allowTearing": true,
      },
    },
    {
      id: "battery",
      title: "Battery",
      description: "Power saver, shorter idle, animations off.",
      values: {
        powerProfile: "power-saver",
        stayAwake: false,
        "hyprLook.animations": false,
        idleScreensaver: 120,
        idleLock: 300,
      },
    },
  ];
}

function byId(id) {
  var list = builtins();
  var i;
  for (i = 0; i < list.length; i++) {
    if (list[i].id === id) return list[i];
  }
  return null;
}

function changesFor(profile) {
  var src = profile && typeof profile === "object" ? profile.values || profile : {};
  var out = [];
  var key;
  for (key in src) {
    if (!Object.prototype.hasOwnProperty.call(src, key)) continue;
    out.push({ key: key, value: src[key] });
  }
  return out;
}

function sanitizeUserName(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]{0,31}$/.test(text)) return "";
  return text;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    builtins: builtins,
    byId: byId,
    changesFor: changesFor,
    sanitizeUserName: sanitizeUserName,
  };
}
