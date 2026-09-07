// Catalog of overflow settings. Each tweak names the file or command it writes.

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
      kind: "tweak",
    },
    {
      id: "swappiness",
      group: "Kernel",
      label: "Lower swappiness",
      description: "Sets vm.swappiness=10 in a sysctl drop-in. Reset removes the drop-in.",
      modifies: "/etc/sysctl.d/99-atmos-swappiness.conf",
      key: "tweaks.swappiness",
      onValue: true,
      offValue: false,
      kind: "tweak",
      needsRoot: true,
    },
  ];
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
  return lines;
}

function sysctlConf(state) {
  var s = clampState(state);
  if (!s.swappiness) return "";
  return "vm.swappiness = 10\n";
}

function resetValue(tweak) {
  if (!tweak) return null;
  return tweak.offValue;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    catalog: catalog,
    byId: byId,
    defaultState: defaultState,
    clampState: clampState,
    gtkIni: gtkIni,
    environmentLines: environmentLines,
    sysctlConf: sysctlConf,
    resetValue: resetValue,
  };
}
