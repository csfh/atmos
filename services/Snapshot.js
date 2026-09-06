// Merge snapshot JSON and adopt patches. Missing keys in a patch keep the
// current value and the same object/array reference.

function isPlainObject(value) {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function isArray(value) {
  return Array.isArray(value) || value instanceof Array;
}

function hasOwn(obj, key) {
  return !!obj && Object.prototype.hasOwnProperty.call(obj, key);
}

function mergeSnapshot(current, patch) {
  var out = {};
  var src = isPlainObject(current) ? current : {};
  var add = isPlainObject(patch) ? patch : {};
  var key;
  for (key in src) {
    if (Object.prototype.hasOwnProperty.call(src, key)) out[key] = src[key];
  }
  for (key in add) {
    if (Object.prototype.hasOwnProperty.call(add, key)) out[key] = add[key];
  }
  return out;
}

function cloneRow(row) {
  var out = {};
  var key;
  if (!row || typeof row !== "object") return out;
  for (key in row) {
    if (Object.prototype.hasOwnProperty.call(row, key)) out[key] = row[key];
  }
  return out;
}

function patchMonitorBrightness(monitors, name, percent) {
  var list = Array.isArray(monitors) ? monitors.slice() : [];
  var i;
  for (i = 0; i < list.length; i++) {
    if (!list[i] || list[i].name !== name) continue;
    var row = cloneRow(list[i]);
    row.brightness = percent;
    list[i] = row;
    break;
  }
  return list;
}

function patchFocusedMonitorScale(monitors, scale) {
  var n = Number(scale);
  var list = Array.isArray(monitors) ? monitors.slice() : [];
  var i;
  if (!isFinite(n) || n <= 0) return list;
  for (i = 0; i < list.length; i++) {
    if (!list[i] || list[i].focused !== true) continue;
    var row = cloneRow(list[i]);
    row.scale = n;
    list[i] = row;
    break;
  }
  return list;
}

function patchPluginEnabled(plugins, id, on) {
  var list = Array.isArray(plugins) ? plugins.slice() : [];
  var want = String(id || "");
  var i;
  if (!want) return list;
  for (i = 0; i < list.length; i++) {
    if (!list[i] || String(list[i].id || "") !== want) continue;
    var row = cloneRow(list[i]);
    row.enabled = on === true;
    list[i] = row;
    break;
  }
  return list;
}

function patchRemoveMatching(list, field, value) {
  var want = String(value || "");
  var src = Array.isArray(list) ? list : [];
  var out = [];
  var i;
  if (!want) return src.slice();
  for (i = 0; i < src.length; i++) {
    var row = src[i];
    if (typeof row === "string" || typeof row === "number") {
      if (String(row) !== want) out.push(row);
      continue;
    }
    if (!row || typeof row !== "object") {
      out.push(row);
      continue;
    }
    if (String(row[field] || "") !== want) out.push(row);
  }
  return out;
}

function patchRowField(list, matchField, matchValue, patchField, patchValue) {
  var want = String(matchValue || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var i;
  if (!want) return src;
  for (i = 0; i < src.length; i++) {
    if (!src[i] || String(src[i][matchField] || "") !== want) continue;
    var row = cloneRow(src[i]);
    row[patchField] = patchValue;
    src[i] = row;
    break;
  }
  return src;
}

function patchWifiActive(list, uuid, on) {
  var want = String(uuid || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var i;
  if (!want) return src;
  for (i = 0; i < src.length; i++) {
    if (!src[i]) continue;
    var row = cloneRow(src[i]);
    if (String(row.uuid || "") === want) row.active = on === true;
    else if (on === true) row.active = false;
    src[i] = row;
  }
  return src;
}

function patchHookSample(list, path, enabled) {
  var want = String(path || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var i;
  if (!want) return src;
  for (i = 0; i < src.length; i++) {
    if (!src[i] || String(src[i].path || "") !== want) continue;
    var row = cloneRow(src[i]);
    var p = String(row.path || "");
    if (enabled) {
      if (p.length >= 7 && p.substring(p.length - 7) === ".sample")
        p = p.substring(0, p.length - 7);
      row.sample = false;
    } else {
      if (!(p.length >= 7 && p.substring(p.length - 7) === ".sample")) p = p + ".sample";
      row.sample = true;
    }
    row.path = p;
    var slash = p.lastIndexOf("/");
    row.name = slash === -1 ? p : p.substring(slash + 1);
    src[i] = row;
    break;
  }
  return src;
}

function patchKeyboardBrightness(current, direction) {
  var n = Math.round(Number(current));
  if (!isFinite(n) || n < 0) n = 0;
  if (n > 100) n = 100;
  if (direction === "off") return 0;
  if (direction === "up") {
    n += 10;
    if (n > 100) n = 100;
    return n;
  }
  if (direction === "down") {
    n -= 10;
    if (n < 0) n = 0;
    return n;
  }
  return n;
}

function patchAppendReminder(list, minutes, message) {
  var src = Array.isArray(list) ? list.slice() : [];
  var mins = Math.round(Number(minutes));
  var msg = String(message || "");
  if (!isFinite(mins) || mins < 1) return src;
  src.push({
    unit: "",
    label: msg || mins + " min",
    message: msg,
    remaining: mins + " min",
    atTime: "",
    minutes: mins,
  });
  return src;
}

function patchAppendHook(list, row) {
  if (!row || typeof row !== "object") return Array.isArray(list) ? list.slice() : [];
  var path = String(row.path || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var next = cloneRow(row);
  var i;
  if (!path) return src;
  for (i = 0; i < src.length; i++) {
    if (src[i] && String(src[i].path || "") === path) {
      src[i] = next;
      return src;
    }
  }
  src.push(next);
  return src;
}

function patchReplaceManaged(list, managed) {
  var src = Array.isArray(list) ? list : [];
  var next = Array.isArray(managed) ? managed : [];
  var out = [];
  var i;
  for (i = 0; i < src.length; i++) {
    var cur = src[i];
    if (cur && cur.managed === true) continue;
    if (cur && typeof cur === "object") out.push(cloneRow(cur));
    else if (typeof cur === "string" && cur) out.push({ command: cur, managed: false });
  }
  for (i = 0; i < next.length; i++) {
    var item = next[i];
    if (item == null) continue;
    if (typeof item === "string" || typeof item === "number") {
      var cmd = String(item);
      if (cmd) out.push({ command: cmd, managed: true });
      continue;
    }
    if (typeof item !== "object") continue;
    var row = cloneRow(item);
    row.managed = true;
    out.push(row);
  }
  return out;
}

function parseSnapshot(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return null;
  try {
    var value = JSON.parse(text);
    return isPlainObject(value) ? value : null;
  } catch (e) {
    return null;
  }
}

function adoptValue(cur, next) {
  if (next === undefined) return cur;
  if (cur === next) return cur;
  if (next !== null && typeof next === "object") {
    if (JSON.stringify(cur) === JSON.stringify(next)) return cur;
  }
  return next;
}

function adoptArray(cur, next) {
  if (next === undefined) return cur;
  if (!isArray(next)) next = [];
  return adoptValue(cur, next);
}

function sanitizeDmi(raw) {
  var source = String(raw || "");
  if (source.indexOf("\n") !== -1 || source.indexOf("\r") !== -1) return "";
  var s = source.replace(/\s+/g, " ").replace(/^\s+|\s+$/g, "");
  if (!s || s.length > 160) return "";
  if (s.indexOf("..") !== -1) return "";
  if (s.charAt(0) === "-" || s.charAt(0) === "/") return "";
  var lower = s.toLowerCase();
  if (
    lower === "none" ||
    lower === "default string" ||
    lower === "unknown" ||
    lower.indexOf("to be filled") !== -1
  )
    return "";
  return s;
}

function patchGroup(patch) {
  var g = String((patch && patch.group) || "");
  if (
    g === "look" ||
    g === "rest" ||
    g === "all" ||
    g === "network" ||
    g === "disks" ||
    g === "accounts" ||
    g === "system"
  )
    return g;
  return "";
}

var ACCOUNT_KEYS = ["hostname", "fullName", "currentUser", "avatarPath", "users", "groups"];

var INDICATOR_IDS = ["Dictation", "ScreenRecording", "Reminder", "NightLight", "Dnd", "StayAwake"];

var STRING_KEYS = {
  theme: true,
  background: true,
  font: true,
  clockFormat: true,
  clockFormatAlt: true,
  agentsSyncDir: true,
  agentsSyncFileName: true,
  agentsSyncDeviceId: true,
  browser: true,
  terminal: true,
  editor: true,
  agent: true,
  dns: true,
  wifiBand: true,
  netSsid: true,
  audioSink: true,
  audioSource: true,
  powerProfile: true,
  powerProfileAc: true,
  powerProfileBattery: true,
  weatherLocation: true,
  plymouth: true,
  omarchyVersion: true,
  updateSummary: true,
  picturesDir: true,
  videosDir: true,
  focusedClass: true,
};

var BOOL_ON_KEYS = {
  barTransparent: true,
  clockPresent: true,
  indicatorsPresent: true,
  indicatorsAlwaysShow: true,
  agentsPresent: true,
  agentsSync: true,
  spacerPresent: true,
  trayPresent: true,
  stayAwake: true,
  nightlight: true,
  screensaverBranded: true,
  aboutBranded: true,
  bluetooth: true,
  wifiConnected: true,
  wifiHw: true,
  wifiRadio: true,
  audioOutputMuted: true,
  audioInputMuted: true,
  audioTuningMatch: true,
  audioTuningOn: true,
  snapperPresent: true,
  hibernationAvailable: true,
  hibernationSupported: true,
  hibernationConfigured: true,
  powerPresent: true,
  powerShowPercentage: true,
  isLaptop: true,
  batteryPresent: true,
  internalPresent: true,
  internalEnabled: true,
  externalPresent: true,
  mirroring: true,
  touchpadPresent: true,
  touchscreenPresent: true,
  keyboardBacklightPresent: true,
  doNotDisturb: true,
  weatherPresent: true,
  reminderActive: true,
  hasAether: true,
  ntp: true,
  ntpAvailable: true,
  ntpSynchronized: true,
  hyprLookManaged: true,
  hyprInputManaged: true,
  hyprWorkspaceGesture: true,
  hyprNoGaps: true,
  hyprSquareAspect: true,
  fingerprintAvailable: true,
  fingerprintConfigured: true,
  fido2Configured: true,
  sshdEnabled: true,
  sshdActive: true,
  passwordlessSudo: true,
  sudolessDocker: true,
  updateAvailable: true,
  atmosInstalled: true,
  voxtypeInstalled: true,
  hybridGpuAvailable: true,
  hwNvidia: true,
  hwNvidiaGsp: true,
  hwNvidiaWithoutGsp: true,
  hwVulkan: true,
  hwIntel: true,
  hwIntelPtl: true,
  hwWebcam: true,
  hwFramework16: true,
  hwAsusRog: true,
  hwSurface: true,
  tailscaleInstalled: true,
  tailscaleRunning: true,
  snapperTimeline: true,
  fstrimEnabled: true,
  directBootAvailable: true,
  directBoot: true,
  recordingActive: true,
  webcamOverlay: true,
  autostartManaged: true,
  bindingsManaged: true,
  windowRulesManaged: true,
  cupsActive: true,
  printerSetup: true,
  nightlightNightOn: true,
};

var BOOL_OFF_KEYS = {
  barVisible: true,
  screensaverEnabled: true,
  suspendEnabled: true,
  touchpadEnabled: true,
  touchscreenEnabled: true,
  crashCapture: true,
  weatherAuto: true,
};

var ARRAY_KEYS = {
  themes: true,
  extraThemes: true,
  desktopApps: true,
  tuiApps: true,
  webApps: true,
  fonts: true,
  wifiConnections: true,
  bluetoothDevices: true,
  audioSinks: true,
  audioSources: true,
  disks: true,
  luksDevices: true,
  swapDevices: true,
  snapperConfigs: true,
  snapshots: true,
  powerProfiles: true,
  monitors: true,
  reminders: true,
  plymouthThemes: true,
  timezones: true,
  keyboardLayouts: true,
  locales: true,
  plugins: true,
  mimePdfOptions: true,
  mimeImageOptions: true,
  mimeVideoOptions: true,
  hooks: true,
  autostart: true,
  bindings: true,
  windowRules: true,
  keybindings: true,
  tailscalePeers: true,
  users: true,
  groups: true,
};

var OBJECT_KEYS = {
  browsers: true,
  terminals: true,
  editors: true,
  services: true,
  gaming: true,
  extras: true,
};

var DMI_KEYS = {
  dmiVendor: true,
  dmiProduct: true,
  dmiFamily: true,
  cpuIdentity: true,
  gpuIdentity: true,
  npuIdentity: true,
};

function normalizedIndicatorItems(list) {
  var next = [];
  var i;
  if (!isArray(list)) return next;
  for (i = 0; i < INDICATOR_IDS.length; i++) {
    if (list.indexOf(INDICATOR_IDS[i]) !== -1) next.push(INDICATOR_IDS[i]);
  }
  return next;
}

function normalizedStringIds(list) {
  var next = [];
  var i;
  if (!isArray(list)) return next;
  for (i = 0; i < list.length; i++) {
    var id = String(list[i] || "");
    if (id.length === 0 || next.indexOf(id) !== -1) continue;
    next.push(id);
  }
  return next;
}

function clampWeekStart(value) {
  var day = String(value || "").toLowerCase();
  if (
    day === "sunday" ||
    day === "monday" ||
    day === "tuesday" ||
    day === "wednesday" ||
    day === "thursday" ||
    day === "friday" ||
    day === "saturday"
  )
    return day;
  return "";
}

function clampVolume(value) {
  var n = Math.round(Number(value)) || 0;
  if (n < 0) n = 0;
  if (n > 100) n = 100;
  return n;
}

function clampMime(value) {
  var s = String(value || "");
  if (!/^[A-Za-z0-9._-]+\.desktop$/.test(s)) return "";
  return s;
}

function clampIface(value) {
  var s = String(value || "");
  if (!/^[a-zA-Z0-9._-]+$/.test(s)) return "";
  return s;
}

function clampDigits(value) {
  var s = String(value || "");
  if (!/^[0-9]+$/.test(s)) return "";
  return s;
}

function defaultAudioName(list) {
  var rows = isArray(list) ? list : [];
  var i;
  for (i = 0; i < rows.length; i++) {
    if (rows[i] && rows[i].default) return String(rows[i].name || "");
  }
  return "";
}

function clampPatchedValue(key, value, adapters) {
  if (STRING_KEYS[key]) return String(value || "");
  if (BOOL_ON_KEYS[key]) return value === true;
  if (BOOL_OFF_KEYS[key]) return value !== false;
  if (ARRAY_KEYS[key]) return isArray(value) ? value : [];
  if (OBJECT_KEYS[key]) return value || {};
  if (DMI_KEYS[key]) return sanitizeDmi(value);
  if (key === "textSize") return Number(value) || 12;
  if (key === "barPosition") return String(value || "top");
  if (key === "clockWeekStart") return clampWeekStart(value);
  if (key === "clockBirthYear") {
    var birth = Math.round(Number(value)) || 0;
    if (birth < 1) birth = 0;
    return birth;
  }
  if (key === "clockLifeExpectancy") {
    var life = Math.round(Number(value)) || 0;
    if (life < 1 || life > 150) life = 0;
    return life;
  }
  if (key === "indicatorsItems") return normalizedIndicatorItems(value);
  if (key === "agentsRefreshIntervalSec") {
    var refresh = Number(value) || 900;
    if (refresh < 30) refresh = 900;
    return refresh;
  }
  if (key === "spacerSize") {
    var spacer = Math.round(Number(value));
    if (!isFinite(spacer) || spacer < 0) spacer = 12;
    if (spacer > 64) spacer = 64;
    return spacer;
  }
  if (key === "trayHidden" || key === "trayPinned") return normalizedStringIds(value);
  if (key === "idleScreensaver" || key === "idleLock") return Number(value) || 0;
  if (key === "nightlightTemperature") {
    var temp = Math.round(Number(value)) || 0;
    if (temp < 0) temp = 0;
    return temp;
  }
  if (key === "wifiBandSelected") return String(value || "auto");
  if (key === "wifiBands") return isArray(value) ? value : ["auto"];
  if (key === "wifiIface" || key === "netIface") return clampIface(value);
  if (key === "netKind") {
    var kind = String(value || "disconnected");
    if (kind !== "ethernet" && kind !== "wifi") kind = "disconnected";
    return kind;
  }
  if (key === "netSignal" || key === "netSpeed") return clampDigits(value);
  if (key === "netIp") {
    var ip = String(value || "");
    if (!/^[0-9a-fA-F:.]+$/.test(ip)) return "";
    return ip;
  }
  if (key === "audioOutputVolume" || key === "audioInputVolume" || key === "keyboardBrightness")
    return clampVolume(value);
  if (key === "weatherCoords") {
    if (adapters && typeof adapters.parseWeatherCoords === "function")
      return adapters.parseWeatherCoords(value);
    return String(value || "");
  }
  if (key === "weatherUnit") {
    var unit = String(value || "auto");
    if (unit !== "metric" && unit !== "imperial") unit = "auto";
    return unit;
  }
  if (key === "weatherRefreshMinutes") {
    var mins = Number(value) || 15;
    if (mins < 1) mins = 15;
    return mins;
  }
  if (key === "reminderCount") {
    var count = Math.round(Number(value)) || 0;
    if (count < 0) count = 0;
    return count;
  }
  if (key === "timezone") {
    var timezone = String(value || "");
    if (!/^[A-Za-z0-9/_+-]+$/.test(timezone) || timezone.indexOf("..") !== -1) return "";
    return timezone;
  }
  if (key === "keyboardLayout") {
    var layout = String(value || "");
    if (layout.indexOf(",") !== -1) layout = layout.split(",")[0];
    if (!/^[a-z0-9]{1,8}$/.test(layout)) return "";
    return layout;
  }
  if (key === "locale") {
    var locale = String(value || "");
    if (locale !== "C.UTF-8" && !/^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(locale))
      return "";
    return locale;
  }
  if (key === "parallelDownloads") {
    var downloads = Math.round(Number(value)) || 5;
    if (downloads < 1) downloads = 5;
    if (downloads > 20) downloads = 20;
    return downloads;
  }
  if (key === "hyprWorkspaceLayout") {
    var ws = String(value || "dwindle");
    if (ws !== "scrolling") ws = "dwindle";
    return ws;
  }
  if (key === "omarchyChannel") {
    var channel = String(value || "");
    if (channel !== "stable" && channel !== "rc" && channel !== "edge" && channel !== "dev")
      return "";
    return channel;
  }
  if (key === "atmosRevision") {
    var rev = String(value || "");
    if (!/^[0-9a-f]{4,40}$/.test(rev)) return "";
    return rev;
  }
  if (key === "atmosChannel") {
    var atmos = "";
    if (adapters && typeof adapters.parseChannel === "function")
      atmos = adapters.parseChannel(value);
    if (!atmos) atmos = "alpha";
    return atmos;
  }
  if (key === "hybridGpuMode") {
    var gpu = String(value || "");
    if (gpu !== "Integrated" && gpu !== "Hybrid") return "";
    return gpu;
  }
  if (key === "cpuStat" || key === "memoryStat")
    return String(value || "").replace(/^\s+|\s+$/g, "");
  if (key === "snapperNumberLimit") {
    var limit = Math.round(Number(value)) || 5;
    if (limit < 1) limit = 5;
    if (limit > 50) limit = 50;
    return limit;
  }
  if (key === "mimePdf" || key === "mimeImage" || key === "mimeVideo") return clampMime(value);
  if (key === "nightlightDay") {
    var day = "";
    if (adapters && typeof adapters.parseTime === "function") day = adapters.parseTime(value);
    return day || "07:00";
  }
  if (key === "nightlightNight") {
    var night = "";
    if (adapters && typeof adapters.parseTime === "function") night = adapters.parseTime(value);
    return night || "20:00";
  }
  return value;
}

function accountKeysPatched(filtered) {
  var i;
  for (i = 0; i < ACCOUNT_KEYS.length; i++) {
    if (hasOwn(filtered, ACCOUNT_KEYS[i])) return true;
  }
  return false;
}

function accountStorePatch(parsed) {
  var src = isPlainObject(parsed) ? parsed : {};
  var out = {};
  var i;
  var found = false;
  for (i = 0; i < ACCOUNT_KEYS.length; i++) {
    var key = ACCOUNT_KEYS[i];
    if (!hasOwn(src, key)) continue;
    out[key] = src[key];
    found = true;
  }
  return found ? out : null;
}

function prepareHyprInput(src, filtered, current) {
  var input = isPlainObject(src) ? src : {};
  var prepared = {};
  var k;
  for (k in input) {
    if (Object.prototype.hasOwnProperty.call(input, k)) prepared[k] = input[k];
  }
  prepared.kbLayoutOverride = hasOwn(input, "kbLayoutOverride")
    ? input.kbLayoutOverride
    : input.kbLayout;
  prepared.kbVariantOverride = hasOwn(input, "kbVariantOverride")
    ? input.kbVariantOverride
    : input.kbVariant;
  if (hasOwn(input, "kbGroupToggle")) prepared.kbGroupToggle = input.kbGroupToggle;
  else prepared.kbGroupToggle = String(input.kbOptions || "").indexOf("grp:alts_toggle") !== -1;
  if (hasOwn(input, "workspaceGesture")) prepared.workspaceGesture = input.workspaceGesture;
  else if (hasOwn(filtered, "hyprWorkspaceGesture"))
    prepared.workspaceGesture = filtered.hyprWorkspaceGesture;
  else if (isPlainObject(current.hyprInput) && "workspaceGesture" in current.hyprInput)
    prepared.workspaceGesture = current.hyprInput.workspaceGesture;
  else prepared.workspaceGesture = current.hyprWorkspaceGesture;
  return prepared;
}

function adopt(currentRecord, patch, adapters) {
  if (
    !adapters ||
    typeof adapters.clampLook !== "function" ||
    typeof adapters.clampInput !== "function"
  )
    throw new Error("adopt requires adapters.clampLook and adapters.clampInput");

  var src = isPlainObject(patch) ? patch : {};
  var current = isPlainObject(currentRecord) ? currentRecord : {};
  var g = patchGroup(src);
  var filtered = {};
  var key;
  var i;
  var allow = null;
  if (g !== "" && g !== "all" && adapters && typeof adapters.allowedKey === "function")
    allow = adapters.allowedKey;
  for (key in src) {
    if (!Object.prototype.hasOwnProperty.call(src, key)) continue;
    if (key === "group") continue;
    if (allow && !allow(g, key)) continue;
    filtered[key] = src[key];
  }
  var merged = mergeSnapshot(current, filtered);

  for (key in filtered) {
    if (!Object.prototype.hasOwnProperty.call(filtered, key)) continue;
    merged[key] = clampPatchedValue(key, filtered[key], adapters);
    merged[key] = adoptValue(current[key], merged[key]);
  }

  if (hasOwn(filtered, "hyprLook"))
    merged.hyprLook = adoptValue(current.hyprLook, adapters.clampLook(filtered.hyprLook));

  if (hasOwn(filtered, "hyprInput")) {
    var clampedInput = adapters.clampInput(prepareHyprInput(filtered.hyprInput, filtered, current));
    merged.hyprInput = adoptValue(current.hyprInput, clampedInput);
    merged.hyprWorkspaceGesture = clampedInput.workspaceGesture === true;
  } else if (hasOwn(filtered, "hyprWorkspaceGesture")) {
    merged.hyprWorkspaceGesture = merged.hyprWorkspaceGesture === true;
  }

  if (hasOwn(filtered, "audioSinks"))
    merged.audioSink = adoptValue(current.audioSink, defaultAudioName(merged.audioSinks));
  if (hasOwn(filtered, "audioSources"))
    merged.audioSource = adoptValue(current.audioSource, defaultAudioName(merged.audioSources));

  if (accountKeysPatched(filtered) && typeof adapters.applyAccountPatch === "function") {
    var currentSlice = {
      hostname: current.hostname,
      fullName: current.fullName,
      currentUser: current.currentUser,
      avatarPath: current.avatarPath,
      users: current.users,
      groups: current.groups,
    };
    var filteredSlice = {};
    for (i = 0; i < ACCOUNT_KEYS.length; i++) {
      if (hasOwn(filtered, ACCOUNT_KEYS[i]))
        filteredSlice[ACCOUNT_KEYS[i]] = filtered[ACCOUNT_KEYS[i]];
    }
    var accounts = adapters.applyAccountPatch(currentSlice, filteredSlice);
    for (i = 0; i < ACCOUNT_KEYS.length; i++) {
      merged[ACCOUNT_KEYS[i]] = adoptValue(current[ACCOUNT_KEYS[i]], accounts[ACCOUNT_KEYS[i]]);
    }
  }

  if (hasOwn(filtered, "hardware") && typeof adapters.normalizeHardware === "function")
    merged.hardware = adoptValue(current.hardware, adapters.normalizeHardware(filtered.hardware));

  return merged;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    mergeSnapshot: mergeSnapshot,
    parseSnapshot: parseSnapshot,
    adopt: adopt,
    adoptValue: adoptValue,
    adoptArray: adoptArray,
    accountStorePatch: accountStorePatch,
    patchGroup: patchGroup,
    sanitizeDmi: sanitizeDmi,
    patchMonitorBrightness: patchMonitorBrightness,
    patchFocusedMonitorScale: patchFocusedMonitorScale,
    patchPluginEnabled: patchPluginEnabled,
    patchRemoveMatching: patchRemoveMatching,
    patchRowField: patchRowField,
    patchWifiActive: patchWifiActive,
    patchHookSample: patchHookSample,
    patchKeyboardBrightness: patchKeyboardBrightness,
    patchAppendReminder: patchAppendReminder,
    patchAppendHook: patchAppendHook,
    patchReplaceManaged: patchReplaceManaged,
    isPlainObject: isPlainObject,
  };
}
