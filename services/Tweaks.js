// Catalog of overflow settings. Each tweak names the file or command it writes,
// and its default: what Omarchy sets, which Reset puts back.

function catalog() {
  return [
    {
      id: "accelFlat",
      group: "Pointer",
      label: "Disable mouse acceleration",
      description: "Flat pointer acceleration. Same writer as Input → Acceleration.",
      modifies: "~/.config/hypr/input.lua · input.accel_profile",
      key: "hyprInput.accelProfile",
      onValue: "flat",
      offValue: "",
      // Omarchy sets no profile (default/hypr/input.lua), so Hyprland's adaptive one applies.
      defaultValue: "",
      kind: "hypr-input",
    },
    {
      id: "naturalScroll",
      group: "Pointer",
      label: "Natural scrolling",
      description: "Content moves with the fingers. Same writer as Input.",
      modifies: "~/.config/hypr/input.lua · input.touchpad.natural_scroll",
      key: "hyprInput.naturalScroll",
      onValue: true,
      offValue: false,
      // default/hypr/input.lua: touchpad.natural_scroll = false.
      defaultValue: false,
      kind: "hypr-input",
    },
    {
      id: "middlePaste",
      group: "Pointer",
      label: "Disable middle-click paste",
      description: "Stops the middle button from pasting the primary selection in GTK apps.",
      modifies: "~/.config/gtk-4.0/settings.ini · gtk-enable-primary-paste",
      key: "tweaks.middlePaste",
      onValue: true,
      offValue: false,
      // Omarchy turns primary paste on (install/user/first-run/gtk-primary-paste.sh).
      defaultValue: false,
      kind: "tweak",
    },
    {
      id: "electronWayland",
      group: "GTK/Electron",
      label: "Electron Wayland",
      description:
        "Hint Electron apps to use Ozone Wayland. Omarchy already sets this; turn off to opt out.",
      modifies: "~/.config/environment.d/10-atmos.conf · ELECTRON_OZONE_PLATFORM_HINT",
      key: "tweaks.electronWayland",
      onValue: true,
      offValue: false,
      // default/hypr/envs.lua: ELECTRON_OZONE_PLATFORM_HINT=wayland.
      defaultValue: true,
      kind: "tweak",
    },
    {
      id: "forceZeroScaling",
      group: "Compatibility",
      label: "XWayland zero scaling",
      description:
        "XWayland apps stay at 1x and the compositor scales them. Omarchy default is on.",
      modifies: "default/hypr/envs.lua · xwayland.force_zero_scaling",
      key: "tweaks.forceZeroScaling",
      onValue: true,
      offValue: false,
      // default/hypr/envs.lua: xwayland.force_zero_scaling = true.
      defaultValue: true,
      kind: "tweak",
    },
  ];
}

function groupedCatalog() {
  var list = catalog();
  var groups = [];
  var index = {};
  var i, item, name, bucket;
  for (i = 0; i < list.length; i++) {
    item = list[i];
    if (!item) continue;
    name = String(item.group || "Tweak");
    bucket = index[name];
    if (!bucket) {
      bucket = { group: name, items: [] };
      index[name] = bucket;
      groups.push(bucket);
    }
    bucket.items.push(item);
  }
  return groups;
}

function byId(id) {
  var list = catalog();
  var i;
  for (i = 0; i < list.length; i++) {
    if (list[i].id === id) return list[i];
  }
  return null;
}

function defaultState() {
  return {
    middlePaste: false,
    electronWayland: true,
    forceZeroScaling: true,
    swappiness: false,
  };
}

function asBool(raw, fallback) {
  if (raw === true || raw === false) return raw;
  return fallback === true;
}

function clampState(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var base = defaultState();
  return {
    middlePaste: asBool(src.middlePaste, base.middlePaste),
    electronWayland: asBool(src.electronWayland, base.electronWayland),
    forceZeroScaling: asBool(src.forceZeroScaling, base.forceZeroScaling),
    swappiness: asBool(src.swappiness, base.swappiness),
  };
}

function gtkIni(state) {
  var s = clampState(state);
  if (!s.middlePaste) return "";
  return "[Settings]\ngtk-enable-primary-paste=false\n";
}

function environmentLines(state) {
  var s = clampState(state);
  var lines = [];
  if (s.electronWayland === false) lines.push("ELECTRON_OZONE_PLATFORM_HINT=auto");
  if (s.forceZeroScaling === false) lines.push("ATMOS_XWAYLAND_ZERO_SCALING=0");
  return lines;
}

function sysctlConf(state) {
  var s = clampState(state);
  if (!s.swappiness) return "";
  return "vm.swappiness = 10\n";
}

// What Reset writes: the tweak's default, as Omarchy sets it.
function resetValue(tweak) {
  if (!tweak) return null;
  return tweak.defaultValue;
}

function fieldOf(tweak) {
  var key = String(tweak.key);
  return key.slice(key.indexOf(".") + 1);
}

// Whether a tweak is on, from the snapshot's hyprInput and tweaks. A pointer
// tweak is on only when Hyprland reports its on value. A tweak file value that
// is not a boolean reads as the default, so the two that Omarchy turns on
// (electronWayland, forceZeroScaling) are on until the snapshot says false.
function currentOn(tweak, hyprInput, tweaks) {
  if (!tweak || !tweak.key) return false;
  var field = fieldOf(tweak);
  if (tweak.kind === "hypr-input") {
    var input = hyprInput && typeof hyprInput === "object" ? hyprInput : {};
    return input[field] === tweak.onValue;
  }
  var state = tweaks && typeof tweaks === "object" ? tweaks : {};
  return asBool(state[field], tweak.defaultValue === tweak.onValue);
}

// The setting a toggle writes: its domain, the field of the snapshot key it
// sets, and the value.
function writeFor(tweak, on) {
  if (!tweak || !tweak.key) return null;
  return {
    domain: String(tweak.key),
    field: fieldOf(tweak),
    value: on === true ? tweak.onValue : tweak.offValue,
  };
}

// The setting Reset writes: the same, with the default.
function resetWrite(tweak) {
  if (!tweak || !tweak.key) return null;
  return { domain: String(tweak.key), field: fieldOf(tweak), value: resetValue(tweak) };
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    catalog: catalog,
    groupedCatalog: groupedCatalog,
    byId: byId,
    defaultState: defaultState,
    clampState: clampState,
    gtkIni: gtkIni,
    environmentLines: environmentLines,
    sysctlConf: sysctlConf,
    resetValue: resetValue,
    currentOn: currentOn,
    writeFor: writeFor,
    resetWrite: resetWrite,
  };
}
