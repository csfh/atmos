// Settings whose write is "ignore a no-op, otherwise send it": a string that
// must be non-empty, or a bool. The key is also the name of the Omarchy
// property that holds the current value.

var SIMPLE = {
  agent: "string",
  agentsSync: "bool",
  audioTuningOn: "bool",
  barPosition: "string",
  barTransparent: "bool",
  barVisible: "bool",
  bluetooth: "bool",
  browser: "string",
  clockFormat: "string",
  clockFormatAlt: "string",
  crashCapture: "bool",
  doNotDisturb: "bool",
  editor: "string",
  hyprNoGaps: "bool",
  hyprSquareAspect: "bool",
  indicatorsAlwaysShow: "bool",
  nightlight: "bool",
  ntp: "bool",
  powerProfileAc: "string",
  powerProfileBattery: "string",
  powerShowPercentage: "bool",
  screensaverEnabled: "bool",
  stayAwake: "bool",
  suspendEnabled: "bool",
  terminal: "string",
  touchscreenEnabled: "bool",
  wifiRadio: "bool",
};

// What Omarchy.set(key, value) should do. { known: false } for a key that is
// not in the table, { dispatch: false } for a no-op, otherwise the value to send.
function plan(key, value, current) {
  var kind = Object.prototype.hasOwnProperty.call(SIMPLE, key) ? SIMPLE[key] : "";
  if (!kind) return { known: false, dispatch: false };
  if (kind === "string") {
    if (!value || value === current) return { known: true, dispatch: false };
    return { known: true, dispatch: true, value: value };
  }
  if (value === current) return { known: true, dispatch: false };
  return { known: true, dispatch: true, value: value };
}

function keys() {
  return Object.keys(SIMPLE);
}
