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
    entry("hyprLook.blur", "windows", "Blur", "look", { type: "boolean" }),
    entry("hyprLook.shadow", "windows", "Shadow", "look", { type: "boolean" }),
    entry("hyprLook.dimInactive", "windows", "Dim others", "look", { type: "boolean" }),
    entry("hyprLook.dimStrength", "windows", "Dim strength", "look", { type: "number" }),
    entry("hyprLook.animations", "windows", "Animations", "look", { type: "boolean" }),
    entry("hyprLook.columnWidth", "windows", "Column width", "look", { type: "number" }),
    entry("hyprLook.cursorHideOnKey", "windows", "Hide cursor while typing", "look", {
      type: "boolean",
    }),
    entry("hyprLook.cursorWarp", "windows", "Warp cursor on workspace", "look", {
      type: "boolean",
    }),
    entry("hyprLook.resizeOnBorder", "windows", "Resize on border", "behavior", {
      type: "boolean",
      consequence:
        "You can drag a window edge to resize it without a modifier. Easy to catch by accident on a thin border.",
    }),
    entry("hyprLook.allowTearing", "windows", "Allow tearing", "behavior", {
      type: "boolean",
      consequence:
        "A game or other window can tear if it asks. That can cut input lag, and it can also show a torn frame.",
    }),
    entry("hyprLook.layout", "windows", "Tiling layout", "behavior", {
      type: "string",
      choices: ["dwindle", "scrolling"],
      consequence:
        "Open windows re-tile. A scrolling layout moves windows off screen instead of shrinking them.",
    }),
    entry("hyprLook.preserveSplit", "windows", "Preserve split", "behavior", {
      type: "boolean",
      consequence:
        "The dwindle split stays after the last window in a branch closes. New windows then open in that leftover split.",
    }),
    entry("hyprLook.enableSwallow", "windows", "Swallow terminals", "behavior", {
      type: "boolean",
      consequence: "A terminal that launches a GUI app is swallowed into that window.",
    }),
    entry("hyprLook.swallowRegex", "windows", "Swallow regex", "behavior", {
      type: "string",
      consequence: "Only terminal classes matching this regex are swallowed.",
    }),
    entry("hyprLook.cursorWarpOnFocus", "windows", "Cursor follows focus", "behavior", {
      type: "boolean",
      consequence: "The pointer jumps when a different window takes focus.",
    }),
    entry("hyprLook.onFocusUnderFullscreen", "windows", "Focus under fullscreen", "behavior", {
      type: "integer",
      consequence: "Focus can steal a fullscreen window or stay underneath it.",
    }),
    entry("hyprLook.focusOnActivate", "windows", "Focus on activate", "behavior", {
      type: "boolean",
      consequence:
        "A window steals focus when another client asks Hyprland to activate it. A chat or mail window can jump in front of what you are typing.",
    }),
    entry("hyprNoGaps", "windows", "No gaps when alone", "look", { type: "boolean" }),
    entry("hyprSquareAspect", "windows", "Square single window", "look", { type: "boolean" }),

    // Bar
    entry("barPosition", "bar", "Bar position", "look", {
      type: "string",
      choices: ["top", "bottom", "left", "right"],
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
    entry("clockWeekStart", "bar", "Week starts on", "look", {
      type: "string",
      // Empty is the locale default the calendar popup already uses.
      choices: ["", "sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"],
    }),

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
    entry("nightlightDay", "idle", "Day starts", "look", { type: "string", format: "time" }),
    entry("nightlightNight", "idle", "Night starts", "look", { type: "string", format: "time" }),
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
      // Empty keeps Hyprland's default profile. clampInput refuses anything else.
      choices: ["", "flat", "adaptive"],
      hostBound: true,
    }),
    entry("hyprInput.emulateDiscreteScroll", "input", "Scroll inertia", "behavior", {
      type: "integer",
      hostBound: true,
      consequence:
        "How a high-resolution or free-spin wheel is turned into scroll events. Tuned per mouse.",
    }),
    entry("hyprInput.naturalScroll", "input", "Natural scrolling", "behavior", {
      type: "boolean",
      consequence: "Scrolling reverses direction. This is the change people notice most.",
    }),
    entry("hyprInput.scrollFactor", "input", "Scroll speed", "behavior", {
      type: "number",
      hostBound: true,
    }),
    entry("hyprInput.clickfinger", "input", "Two-finger click", "behavior", {
      type: "boolean",
      hostBound: true,
    }),
    entry("hyprInput.disableWhileTyping", "input", "Ignore while typing", "behavior", {
      type: "boolean",
      hostBound: true,
    }),
    entry("hyprInput.drag3fg", "input", "Three-finger drag", "behavior", {
      type: "integer",
      hostBound: true,
    }),
    entry("hyprInput.repeatRate", "input", "Repeat rate", "behavior", {
      type: "integer",
      consequence:
        "Held keys repeat at this rate. A high value races through a line; a low one feels sticky.",
    }),
    entry("hyprInput.repeatDelay", "input", "Repeat delay", "behavior", {
      type: "integer",
      consequence:
        "How long you hold a key before it repeats. A short delay repeats while you are still deciding.",
    }),
    entry("hyprInput.numlock", "input", "Numlock on boot", "behavior", {
      type: "boolean",
      consequence: "The number pad turns on or off when Hyprland starts.",
    }),
    entry("hyprInput.followMouse", "input", "Follow mouse", "behavior", {
      type: "integer",
      consequence:
        "The pointer picks which window is focused. Click-to-focus stops a hover from stealing keys.",
    }),
    entry("hyprInput.keyPressDpms", "input", "Wake on key", "behavior", {
      type: "boolean",
      consequence: "A key press wakes the screen after it has gone dark.",
    }),
    entry("hyprInput.mouseMoveDpms", "input", "Wake on mouse", "behavior", {
      type: "boolean",
      consequence: "Moving the pointer wakes the screen after it has gone dark.",
    }),
    entry("hyprInput.kbLayoutOverride", "input", "Hyprland layouts", "behavior", {
      type: "string",
      hostBound: true,
      consequence:
        "Hyprland remaps keys to this layout list. A layout you cannot type on is hard to undo with the keyboard.",
    }),
    entry("hyprInput.kbVariantOverride", "input", "Variants", "behavior", {
      type: "string",
      hostBound: true,
    }),
    entry("hyprInput.kbGroupToggle", "input", "Alt+Alt layout switch", "behavior", {
      type: "boolean",
      hostBound: true,
      consequence:
        "Left Alt and Right Alt together cycle the Hyprland layouts. Needs those layouts set.",
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
      needsRoot: true,
    }),
    entry("timezone", "system", "Timezone", "identity", {
      type: "string",
      options: "timezones",
      consequence: "The clock jumps. Calendar reminders shift with it.",
      needsRoot: true,
    }),
    entry("locale", "system", "Locale", "identity", {
      type: "string",
      options: "locales",
      consequence:
        "Date, number, and sort order change. Running apps keep the old locale until they restart.",
      needsRoot: true,
    }),
    entry("keyboardLayout", "system", "Keyboard layout", "identity", {
      type: "string",
      options: "keyboardLayouts",
      consequence:
        "Keys remap immediately. A layout you cannot type on is hard to undo with the keyboard.",
      needsRoot: true,
    }),
    entry("ntp", "system", "Network time", "identity", {
      type: "boolean",
      consequence: "Turning network time off lets the clock drift until you set it by hand.",
      needsRoot: true,
    }),
    entry("fullName", "system", "Full name", "identity", {
      type: "string",
      needsRoot: true,
    }),
    entry("parallelDownloads", "system", "Parallel downloads", "identity", {
      type: "integer",
      consequence: "Edits /etc/pacman.conf. Only affects how fast updates fetch.",
      needsRoot: true,
    }),
    entry("dns", "network", "DNS", "identity", {
      type: "string",
      consequence:
        "All name lookups go to a different resolver. A wrong value takes the network down until you change it back.",
    }),

    // Sound
    entry("audioOutputVolume", "sound", "Output volume", "behavior", {
      type: "integer",
      consequence: "Speaker volume jumps to the exported level the moment this applies.",
    }),
    entry("audioInputVolume", "sound", "Input volume", "behavior", {
      type: "integer",
      consequence: "Microphone level changes, which callers hear before you do.",
    }),
    entry("audioOutputMuted", "sound", "Output muted", "behavior", {
      type: "boolean",
      consequence: "Muting the output silences everything until you unmute it.",
    }),
    entry("audioInputMuted", "sound", "Input muted", "behavior", {
      type: "boolean",
      consequence: "Muting the microphone is silent from your side; nobody hears you.",
    }),
    entry("audioTuningOn", "sound", "Audio tuning", "look", { type: "boolean" }),

    // Power
    entry("powerProfileAc", "power", "Profile on mains", "behavior", {
      type: "string",
      options: "powerProfiles",
      consequence: "Changes how hard the machine runs while plugged in.",
    }),
    entry("powerProfileBattery", "power", "Profile on battery", "behavior", {
      type: "string",
      options: "powerProfiles",
      consequence: "Changes the trade between speed and battery life when unplugged.",
    }),
    entry("suspendEnabled", "power", "Suspend", "behavior", {
      type: "boolean",
      consequence: "Turning suspend off means a closed lid keeps running and draining.",
    }),
    entry("crashCapture", "power", "Capture crashes", "look", { type: "boolean" }),

    // Bar widgets
    entry("clockBirthYear", "bar", "Birth year", "look", { type: "integer" }),
    entry("clockLifeExpectancy", "bar", "Life expectancy", "look", { type: "integer" }),
    entry("indicatorsAlwaysShow", "bar", "Always show indicators", "look", { type: "boolean" }),
    entry("powerShowPercentage", "bar", "Battery percentage", "look", { type: "boolean" }),
    entry("spacerSize", "bar", "Spacer width", "look", { type: "integer" }),
    entry("weatherLocation", "bar", "Weather location", "look", {
      type: "string",
      consequence:
        "The bar reports weather for the exported town, which is rarely the one you are in.",
    }),
    entry("weatherUnit", "bar", "Weather unit", "look", {
      type: "string",
      choices: ["auto", "metric", "imperial"],
    }),
    entry("weatherRefreshMinutes", "bar", "Weather refresh", "look", { type: "integer" }),
    entry("agentsRefreshIntervalSec", "bar", "Agents refresh", "look", { type: "integer" }),
    entry("agentsSync", "bar", "Sync agents", "behavior", {
      type: "boolean",
      consequence: "Turning sync on starts writing an agents file into the folder below.",
    }),
    entry("agentsSyncDir", "bar", "Agents sync folder", "behavior", {
      type: "string",
      hostBound: true,
      consequence: "A folder from another machine will not exist here.",
    }),
    entry("agentsSyncFileName", "bar", "Agents sync file", "look", { type: "string" }),
    entry("agentsSyncDeviceId", "bar", "Agents device id", "behavior", {
      type: "string",
      hostBound: true,
      consequence:
        "How this machine is named inside the synced agent-usage snapshots. Two machines with the same id overwrite each other.",
    }),

    // Appearance and boot
    entry("plymouth", "appearance", "Boot theme", "look", {
      type: "string",
      options: "plymouthThemes",
      consequence: "Changes the splash shown while the machine starts.",
      needsRoot: true,
    }),
    // screensaverBranded and aboutBranded are reported by the snapshot as
    // booleans, but the writer takes image, text, or reset. There is no
    // honest mapping from true back to one of those, so they stay out.

    // Devices. Tied to this machine's hardware, so they travel badly.
    entry("touchpadEnabled", "input", "Touchpad", "behavior", {
      type: "boolean",
      hostBound: true,
      consequence:
        "Turning the touchpad off on a laptop with no mouse attached leaves you without a pointer.",
    }),
    entry("touchscreenEnabled", "input", "Touchscreen", "behavior", {
      type: "boolean",
      hostBound: true,
    }),
    entry("bluetooth", "network", "Bluetooth radio", "behavior", {
      type: "boolean",
      hostBound: true,
      consequence:
        "Turning the radio off drops every connected device, including a mouse or headset.",
    }),
    entry("wifiRadio", "network", "Wi-Fi radio", "behavior", {
      type: "boolean",
      hostBound: true,
      consequence:
        "Turning the radio off drops Wi-Fi. A laptop with no ethernet then has no network until you turn it back on.",
    }),

    // Lists. A whole list is replaced at once, because these are the
    // settings people actually mean when they say they want to hand someone
    // their desktop.
    listEntry("bindings", "Keybindings", {
      extraConfirm: true,
      consequence:
        "Your Super-key shortcuts are replaced. Imported bindings become the block Atmos manages in bindings.lua; bindings you wrote by hand stay where they are.",
    }),
    listEntry("windowRules", "Window rules", {
      consequence:
        "Rules about which windows float, centre, or open on a given workspace are replaced. Rules naming an app you do not have simply never match.",
    }),
    listEntry("indicatorsItems", "Bar indicators", {
      consequence: "Replaces which indicators the bar shows and in what order.",
    }),
    listEntry("trayHidden", "Hidden tray icons", {
      consequence:
        "Replaces which tray icons are folded away. Icons for apps you do not have simply never appear.",
    }),
    listEntry("trayPinned", "Pinned tray icons", {
      consequence: "Replaces which tray icons stay visible.",
    }),
    entry("powerProfile", "power", "Power profile", "behavior", {
      type: "string",
      choices: ["performance", "balanced", "power-saver"],
      consequence: "How hard the machine works right now.",
    }),
    listEntry("workspaces", "Workspaces", {
      consequence: "Named persistent workspaces and monitor assignment are replaced.",
    }),
    entry("workspaceWrapSwitch", "workspaces", "Wrap workspace switching", "behavior", {
      type: "boolean",
      consequence: "The last workspace wraps to the first when you keep switching.",
    }),
    entry("workspaceWheelSwitch", "workspaces", "Mouse-wheel workspace switching", "behavior", {
      type: "boolean",
      consequence: "Super and the mouse wheel move between workspaces.",
    }),
    listEntry("monitorRules", "Displays", {
      hostBound: true,
      consequence: "Monitor modes, scale, and layout are replaced.",
    }),
    listEntry("autostart", "Startup programs", {
      extraConfirm: true,
      consequence:
        "The programs Hyprland launches at login are replaced. A program you do not have installed fails quietly at the next login.",
    }),
    listEntry("envVars", "Environment variables", {
      consequence: "User environment.d overlay variables are replaced.",
    }),
    entry("envPathPrepend", "envVars", "PATH prepend", "behavior", {
      type: "string",
      hostBound: true,
      consequence: "Directories are prepended to PATH for the next login.",
    }),
    entry("tweaks.middlePaste", "tweaks", "Disable middle-click paste", "behavior", {
      type: "boolean",
      consequence: "GTK apps stop pasting the primary selection on a middle click.",
    }),
    entry("tweaks.electronWayland", "tweaks", "Electron Wayland", "behavior", {
      type: "boolean",
      consequence: "Electron apps follow or ignore the Ozone Wayland hint on the next login.",
    }),
    entry("tweaks.forceZeroScaling", "tweaks", "XWayland zero scaling", "behavior", {
      type: "boolean",
      consequence: "XWayland apps stay at 1x and the compositor scales them.",
    }),
    entry("tweaks.swappiness", "tweaks", "Lower swappiness", "behavior", {
      type: "boolean",
      needsRoot: true,
      consequence: "Writes vm.swappiness=10, so the kernel waits longer before using swap.",
    }),
    entry("presentationMode", "power", "Presentation Mode", "behavior", {
      type: "boolean",
      consequence: "Stay awake, silence notifications, and stop the screensaver for a while.",
    }),
    entry("chargeLimit", "power", "Charge limit", "behavior", {
      type: "integer",
      hostBound: true,
      consequence: "Stops charging past this percent on hardware that exposes a charge threshold.",
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
    kind: String(o.kind || "scalar"),
    type: String(o.type || "string"),
    options: String(o.options || ""),
    // Fixed values the writer will accept. Distinct from `options`, which is
    // a snapshot key for what this machine currently has installed.
    choices: Array.isArray(o.choices) && o.choices.length > 0 ? o.choices.slice() : null,
    format: String(o.format || ""),
    hostBound: o.hostBound === true,
    // Written by a script that raises privileges (as-root.sh or sudo), so
    // applying it goes through Atmos sudo mode.
    needsRoot: o.needsRoot === true,
    extraConfirm: o.extraConfirm === true,
    consequence: String(o.consequence || ""),
    importable: tier !== "system",
    writer: String(o.writer || writerKindFor(key)),
  };
}

function report(key, section, label) {
  return entry(key, section, label, "system", {});
}

// A list setting is its own section, so the block that carries it is named
// for the setting itself: ```json atmos:bindings.
function listEntry(key, label, opts) {
  var o = opts || {};
  return entry(key, key, label, "behavior", {
    kind: "list",
    type: "list",
    consequence: o.consequence,
    hostBound: o.hostBound === true,
    extraConfirm: o.extraConfirm === true,
  });
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
      note: "Resolver choice and radios. Wi-Fi passwords are never exported.",
    },
    {
      id: "sound",
      title: "Sound",
      note: "Volumes and muting. These take effect the moment they apply.",
    },
    { id: "power", title: "Power", note: "Profiles on mains and battery, suspend, crash capture." },
    {
      id: "bindings",
      title: "Keybindings",
      note: "Every shortcut, Super key and all. The list is replaced whole.",
    },
    {
      id: "windowRules",
      title: "Window rules",
      note: "Which windows float, centre, or open somewhere specific.",
    },
    {
      id: "indicatorsItems",
      title: "Bar indicators",
      note: "Which indicators the bar shows, in order.",
    },
    {
      id: "trayHidden",
      title: "Hidden tray icons",
      note: "Tray icons folded away behind the chevron.",
    },
    { id: "trayPinned", title: "Pinned tray icons", note: "Tray icons kept visible." },
    { id: "autostart", title: "Startup programs", note: "What Hyprland launches when you log in." },
    {
      id: "workspaces",
      title: "Workspaces",
      note: "How many workspaces Hyprland keeps, and where they live.",
    },
    {
      id: "monitorRules",
      title: "Displays",
      note: "Per-output mode, scale, and disable-without-forgetting.",
    },
    {
      id: "envVars",
      title: "Environment",
      note: "A user overlay on top of the session environment.",
    },
    { id: "tweaks", title: "Tweaks", note: "Overflow settings that do not need their own page." },
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

// The importable keys in one section, for a UI that offers sections rather
// than one switch per setting.
function sectionKeys(section, catalog) {
  var list = catalog || settingsCatalog();
  var want = String(section || "");
  var out = [];
  for (var i = 0; i < list.length; i++) {
    if (list[i].section !== want) continue;
    if (!list[i].importable) continue;
    out.push(list[i].key);
  }
  return out;
}

// The sections worth offering: the ones that hold something you can carry.
// The security section is reported either way and never has a switch.
function selectableSections(catalog) {
  var list = catalog || settingsCatalog();
  var sections = settingsSections();
  var out = [];
  for (var i = 0; i < sections.length; i++) {
    var keys = sectionKeys(sections[i].id, list);
    if (keys.length === 0) continue;
    out.push({
      id: sections[i].id,
      title: sections[i].title,
      note: sections[i].note,
      count: keys.length,
      // Identity belongs to one machine, so it starts switched off.
      byDefault: sections[i].id !== "system",
    });
  }
  return out;
}

function keysForSections(ids, catalog) {
  var list = catalog || settingsCatalog();
  var want = keyLookup(ids);
  var out = [];
  for (var i = 0; i < list.length; i++) {
    if (!list[i].importable) continue;
    if (!want[list[i].section]) continue;
    out.push(list[i].key);
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
    var listBlocks = [];
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
      if (item.kind === "list") {
        if (!Array.isArray(value)) continue;
        // `managed` is where a row lives on this machine, not a setting.
        listBlocks.push({ key: item.key, value: exportList(value) });
        continue;
      }
      body.push(tomlLine(item.key, value));
    }
    if (body.length === 0 && reported.length === 0 && listBlocks.length === 0) continue;

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
    // A list is JSON rather than TOML-lite. Rows have shape, and inventing a
    // table syntax to avoid one JSON.parse would be a worse trade.
    for (var b2 = 0; b2 < listBlocks.length; b2++) {
      lines.push("```json atmos:" + listBlocks[b2].key);
      lines.push(jsonBlock(listBlocks[b2].value));
      lines.push("```");
      lines.push("");
    }
  }

  return lines.join("\n").replace(/\n+$/, "") + "\n";
}

// One row per line: long enough to read, short enough to diff.
function jsonBlock(list) {
  var rows = [];
  for (var i = 0; i < list.length; i++) rows.push("  " + JSON.stringify(list[i]));
  if (rows.length === 0) return "[]";
  return "[\n" + rows.join(",\n") + "\n]";
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
  if (Array.isArray(value)) return countLabel(value.length, "entry", "entries");
  return String(value);
}

// Read every ```toml atmos:<name> block. Returns
// { meta, sections: { name: { key: value } }, errors: [string] }.
function parseSettingsMarkdown(text) {
  var lines = String(text || "").split("\n");
  var sections = {};
  var errors = [];
  var open = "";
  var openKind = "toml";
  var buffer = [];

  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    var trimmed = line.replace(/^\s+|\s+$/g, "");
    if (!open) {
      var start = trimmed.match(/^```\s*(toml|json)\s+atmos:([A-Za-z][A-Za-z0-9-]*)\s*$/);
      if (start) {
        openKind = start[1];
        open = start[2];
        buffer = [];
      }
      continue;
    }
    if (trimmed === "```") {
      var parsed =
        openKind === "json"
          ? parseJsonBlock(buffer, open, errors)
          : parseTomlLite(buffer, open, errors);
      if (!sections[open]) sections[open] = {};
      for (var key in parsed) {
        if (Object.prototype.hasOwnProperty.call(parsed, key)) sections[open][key] = parsed[key];
      }
      open = "";
      openKind = "toml";
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

function parseJsonBlock(lines, name, errors) {
  var out = {};
  try {
    var value = JSON.parse((lines || []).join("\n"));
    if (!Array.isArray(value)) {
      errors.push("atmos:" + name + " is not a list");
      return out;
    }
    out[name] = value;
  } catch (e) {
    errors.push("atmos:" + name + " is not readable JSON");
  }
  return out;
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
  if (text.charAt(0) === '"') return parseTomlString(text);
  if (text.charAt(0) === "[") return parseTomlArray(text);
  // Same trailing-comment rule parseTomlString already applies after a quote.
  var hash = text.indexOf("#");
  if (hash !== -1) {
    text = text.slice(0, hash).replace(/\s+$/, "");
    if (!text) return undefined;
  }
  if (text === "true") return true;
  if (text === "false") return false;
  if (/^-?[0-9]+$/.test(text)) return parseInt(text, 10);
  if (/^-?[0-9]*\.[0-9]+$/.test(text)) return parseFloat(text);
  return undefined;
}

// Split on commas that are not inside quotes, so a clock-format list or a
// hex colour is not cut in half, and a comment after ] is not part of the
// last item.
function parseTomlArray(text) {
  if (text.charAt(0) !== "[") return undefined;
  var items = [];
  var buf = "";
  var inString = false;
  var escape = false;
  var i = 1;
  while (i < text.length) {
    var c = text.charAt(i);
    if (inString) {
      buf += c;
      if (escape) escape = false;
      else if (c === "\\") escape = true;
      else if (c === '"') inString = false;
      i++;
      continue;
    }
    if (c === '"') {
      inString = true;
      buf += c;
      i++;
      continue;
    }
    if (c === ",") {
      var piece = buf.replace(/^\s+|\s+$/g, "");
      if (!piece) return undefined;
      var item = parseTomlValue(piece);
      if (item === undefined) return undefined;
      items.push(item);
      buf = "";
      i++;
      continue;
    }
    if (c === "]") {
      var last = buf.replace(/^\s+|\s+$/g, "");
      if (last) {
        var tail = parseTomlValue(last);
        if (tail === undefined) return undefined;
        items.push(tail);
      }
      var rest = text.slice(i + 1).replace(/^\s+/, "");
      if (rest && rest.charAt(0) !== "#") return undefined;
      return items;
    }
    buf += c;
    i++;
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
  return '"' + escapeTomlString(value) + '"';
}

// Vertical clock formats (and anything else with a tab or CR) have to stay
// one TOML line or parseTomlLite splits them and the value is lost.
function escapeTomlString(value) {
  return String(value === null || value === undefined ? "" : value)
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t");
}

function parseTomlString(text) {
  var out = "";
  var i = 1;
  while (i < text.length) {
    var c = text.charAt(i);
    if (c === '"') {
      var rest = text.slice(i + 1).replace(/^\s+/, "");
      if (rest && rest.charAt(0) !== "#") return undefined;
      return out;
    }
    if (c === "\\") {
      if (i + 1 >= text.length) return undefined;
      var n = text.charAt(i + 1);
      i += 2;
      if (n === "n") out += "\n";
      else if (n === "t") out += "\t";
      else if (n === "r") out += "\r";
      else if (n === '"') out += '"';
      else if (n === "\\") out += "\\";
      else return undefined;
      continue;
    }
    if (c === "\n" || c === "\r") return undefined;
    out += c;
    i++;
  }
  return undefined;
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
      var from = readValue(snap, key);

      // Settled before anything is validated. A value the machine already
      // holds needs no permission to stay: plymouth reports "default" as
      // its theme while the installable themes list does not contain it,
      // so validating first would refuse the machine its own setting.
      if (sameValue(from, to)) {
        unchanged.push(key);
        continue;
      }

      if (!typeMatches(item.type, to)) {
        blocked.push({
          key: key,
          reason: item.label + " expects " + item.type + " but the file has " + describe(to) + ".",
        });
        continue;
      }
      // Sentinel writers drop a row they cannot write. One bad binding in a
      // list would empty the Atmos block while the review still showed it.
      if (item.kind === "list") {
        var canonTo = canonicalizeList(item, to);
        if (canonTo.error) {
          blocked.push({ key: key, reason: canonTo.error });
          continue;
        }
        to = canonTo.value;
        var fromCmp = from;
        if (Array.isArray(from)) {
          var canonFrom = canonicalizeList(item, from);
          if (!canonFrom.error) fromCmp = canonFrom.value;
        }
        if (sameValue(fromCmp, to)) {
          unchanged.push(key);
          continue;
        }
      }
      if (item.options) {
        var allowed = readValue(snap, item.options);
        if (Array.isArray(allowed) && allowed.length > 0) {
          if (!allowedContains(allowed, to)) {
            blocked.push({
              key: key,
              reason: displayValue(to) + " is not available on this machine.",
            });
            continue;
          }
        } else {
          // Refusing everything on a machine that cannot list its own options
          // would be worse than letting it through, but saying nothing would
          // leave the check looking exact when it was never made.
          warnings.push({
            key: key,
            message:
              "This machine did not report which " +
              item.label.toLowerCase() +
              " values it has, so " +
              displayValue(to) +
              " could not be checked. It may fail when applied.",
          });
        }
      }
      // Writers clamp or drop anything outside this list, so the plan has
      // to refuse it rather than show a value that will not land.
      if (item.choices && item.choices.length > 0 && !allowedContains(item.choices, to)) {
        blocked.push({
          key: key,
          reason: choiceReason(item, to),
        });
        continue;
      }
      if (item.format === "time" && !isClockTime(to)) {
        blocked.push({
          key: key,
          reason: item.label + " expects a 24-hour time such as 07:00.",
        });
        continue;
      }
      if (item.hostBound && differentHardware) {
        warnings.push({
          key: key,
          message: item.label + " is tied to this machine's hardware. Held back.",
        });
        continue;
      }
      // The sentinel writers add a block of their own and leave whatever you
      // wrote by hand where it is. Importing a list Atmos does not already
      // own therefore leaves two copies of those rows in the file, with the
      // Atmos block winning. That is worth saying before it happens.
      var handWritten = unmanagedCount(from);
      if (handWritten > 0) {
        warnings.push({
          key: key,
          message:
            countLabel(handWritten, "row", "rows") +
            " of your current " +
            item.label.toLowerCase() +
            " are written by hand in the config file. Atmos writes its own block and leaves those lines alone, " +
            "so they stay in the file with the imported ones taking effect.",
        });
      }
      // Undo is this change with from and value swapped. The writer replaces
      // the Atmos block, so from has to be that block, not the hand-written
      // rows that stay in the file. Sending those too would copy them into
      // the sentinel and leave the originals, two of each.
      var planFrom = from;
      if (item.kind === "list" && Array.isArray(from)) planFrom = managedRows(from);
      changes.push({
        key: key,
        section: item.section,
        label: item.label,
        tier: item.tier,
        from: planFrom === undefined ? null : planFrom,
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

// How many of these raise privileges. enqueueIo asks for sudo mode once,
// then the rest of the import runs without asking again.
function passwordCount(plan) {
  var list = (plan && plan.changes) || [];
  var byKey = catalogByKey();
  var n = 0;
  for (var i = 0; i < list.length; i++) {
    var item = byKey[list[i].key];
    if (item && item.needsRoot) n++;
  }
  return n;
}

// What to expect before pressing the button: how much, how long, and
// whether it will interrupt you.
function applyForecast(plan) {
  var changes = ((plan && plan.changes) || []).length;
  if (changes === 0) return "";
  var asks = passwordCount(plan);
  var lines = [countLabel(changes, "change", "changes") + ", a few seconds."];
  if (asks > 0) {
    lines.push(
      countLabel(asks, "change needs", "changes need") +
        " root. Atmos asks for sudo mode once and the rest run without asking again.",
    );
  }
  if (hasCommandImport(plan)) {
    lines.push(
      "This file replaces keybindings or startup programs. Read every command above before you apply.",
    );
  }
  return lines.join("\n");
}

function hasCommandImport(plan) {
  return commandImportLines(plan).length > 0;
}

function commandImportLines(plan) {
  var list = (plan && plan.changes) || [];
  var byKey = catalogByKey();
  var out = [];
  var seen = {};
  var i;
  var j;
  for (i = 0; i < list.length; i++) {
    var change = list[i];
    var item = byKey[change.key];
    if (!item || !item.extraConfirm) continue;
    var rows = Array.isArray(change.to) ? change.to : [];
    for (j = 0; j < rows.length; j++) {
      var row = rows[j];
      var cmd = "";
      if (typeof row === "string") cmd = row;
      else if (row && typeof row === "object") cmd = String(row.command || "");
      cmd = cmd.replace(/^\s+|\s+$/g, "");
      if (!cmd || seen[cmd]) continue;
      seen[cmd] = true;
      out.push(cmd);
    }
  }
  return out;
}

function applyConfirmMessage(plan) {
  var forecast = applyForecast(plan);
  if (!forecast) return "";
  return (
    forecast + "\n\nAtmos writes a way back first, and Put it back restores every one of them."
  );
}

function commandConfirmMessage(plan) {
  var commands = commandImportLines(plan);
  var lines = ["These commands will be installed as shortcuts or startup programs:"];
  var i;
  for (i = 0; i < commands.length; i++) lines.push("• " + commands[i]);
  lines.push("");
  lines.push(
    "A shared file can put a download on Super+Return or run it at login. Only apply if you trust every line.",
  );
  return lines.join("\n");
}

function parseApplyResult(text) {
  var lines = String(text || "").split("\n");
  var i;
  for (i = lines.length - 1; i >= 0; i--) {
    var line = lines[i].replace(/^\s+|\s+$/g, "");
    if (!line || line.charAt(0) !== "{") continue;
    try {
      var obj = JSON.parse(line);
      if (obj && typeof obj === "object" && !Array.isArray(obj)) return obj;
    } catch (e) {}
  }
  return { backup: "", results: [] };
}

function appliedCountFromResult(result) {
  var rows = (result && result.results) || [];
  var n = 0;
  var i;
  for (i = 0; i < rows.length; i++) {
    if (rows[i] && rows[i].status === "applied") n++;
  }
  return n;
}

function backupDirFromResult(result) {
  return result && result.backup ? String(result.backup) : "";
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
  if (type === "list") return Array.isArray(value);
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

function choiceReason(item, value) {
  var shown = shownValue(value);
  return shown + " is not a valid " + String(item.label || "value").toLowerCase() + ".";
}

// Same 24-hour clock HyprSunset.parseTime accepts, so a file that would
// fall back to 07:00 is blocked instead of silently rewriting the schedule.
function isClockTime(value) {
  return typeof value === "string" && /^([01]?\d|2[0-3]):([0-5]\d)$/.test(value);
}

// Rows the snapshot says live outside the block Atmos manages.
function unmanagedCount(value) {
  if (!Array.isArray(value)) return 0;
  var n = 0;
  for (var i = 0; i < value.length; i++) {
    var row = value[i];
    if (row && typeof row === "object" && row.managed === false) n++;
  }
  return n;
}

// The Atmos-managed rows of a live list. String lists have no managed flag
// and pass through. A missing managed is treated as managed: the file we
// export never carries the flag, and a row without it is still the block.
function managedRows(value) {
  if (!Array.isArray(value)) return [];
  var out = [];
  for (var i = 0; i < value.length; i++) {
    var row = value[i];
    if (row && typeof row === "object" && row.managed === false) continue;
    out.push(row);
  }
  return out;
}

// The shape the writer will actually apply. Extra fields, key order, and
// indicator order are not settings; a row the writer would drop is blocked
// so the review cannot show a list that will not land.
function canonicalizeList(item, value) {
  if (!Array.isArray(value)) return { error: item.label + " expects a list." };
  var out = [];
  var i;
  for (i = 0; i < value.length; i++) {
    var row = canonicalizeListRow(item, value[i], i);
    if (row.error) return { error: row.error };
    out.push(row.value);
  }
  if (item.key === "indicatorsItems") {
    var all = indicatorIds();
    var next = [];
    for (i = 0; i < all.length; i++) {
      if (out.indexOf(all[i]) !== -1) next.push(all[i]);
    }
    out = next.length === all.length ? [] : next;
  }
  return { value: out };
}

function workspaceCountFromItems(list) {
  var rows = Array.isArray(list) ? list : [];
  var max = 0;
  var i, n;
  for (i = 0; i < rows.length; i++) {
    n = Number(rows[i] && (rows[i].id || rows[i].workspace));
    if (isFinite(n) && n >= 1 && n <= 10 && n > max) max = n;
  }
  return max < 1 ? 10 : max;
}

function canonicalizeListRow(item, row, index) {
  var n = index + 1;
  var key = item.key;
  if (key === "bindings") return canonicalizeBindingRow(row, n, item.label);
  if (key === "autostart") return canonicalizeAutostartRow(row, n, item.label);
  if (key === "windowRules") return canonicalizeWindowRuleRow(row, n, item.label);
  if (key === "workspaces") return canonicalizeWorkspaceRow(row, n, item.label);
  if (key === "monitorRules") return canonicalizeMonitorRow(row, n, item.label);
  if (key === "envVars") return canonicalizeEnvVarRow(row, n, item.label);
  if (key === "indicatorsItems") return canonicalizeIndicatorRow(row, n, item.label);
  if (key === "trayHidden" || key === "trayPinned")
    return canonicalizeStringIdRow(row, n, item.label);
  return { value: normalizeListRow(row) };
}

function listRowPrefix(label, n) {
  return label + " row " + n + " ";
}

function sanitizeBindingKeys(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "").replace(/\s+/g, " ");
  if (!text || text.length > 64) return "";
  if (!/^[A-Za-z0-9_ +.:,-]+$/.test(text)) return "";
  return text;
}

function sanitizeBindingLabel(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 64) return "";
  return text;
}

function sanitizeListCommand(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 256) return "";
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  return text;
}

function sanitizeWindowMatch(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 128) return "";
  if (text.indexOf("]]") !== -1) return "";
  return text;
}

function sanitizeWindowWorkspace(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  if (!/^[A-Za-z0-9:_-]{1,32}$/.test(text)) return "";
  return text;
}

function clampWindowSize(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 100 || n > 4000) return 0;
  return n;
}

function indicatorIds() {
  return ["Dictation", "ScreenRecording", "Reminder", "NightLight", "Dnd", "StayAwake"];
}

function canonicalizeBindingRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (!row || typeof row !== "object" || Array.isArray(row))
    return { error: prefix + "is not a binding." };
  var keys = sanitizeBindingKeys(row.keys);
  if (!keys) return { error: prefix + "is not a valid chord." };
  if (row.unbind != null && row.unbind !== true && row.unbind !== false)
    return { error: prefix + "has a bad unbind flag." };
  var unbind = row.unbind === true;
  var command = "";
  if (row.command != null && row.command !== "") {
    if (typeof row.command !== "string") return { error: prefix + "has a bad command." };
    command = sanitizeListCommand(row.command);
    if (!command) return { error: prefix + "has a command Atmos will not write." };
  }
  if (!command && !unbind) return { error: prefix + "needs a command or an unbind." };
  var rowLabel = "";
  if (row.label != null && row.label !== "") {
    if (typeof row.label !== "string") return { error: prefix + "has a bad label." };
    rowLabel = sanitizeBindingLabel(row.label);
    if (String(row.label).replace(/^\s+|\s+$/g, "") && !rowLabel)
      return { error: prefix + "has a label Atmos will not write." };
  }
  return {
    value: normalizeListRow({ keys: keys, label: rowLabel, command: command, unbind: unbind }),
  };
}

function canonicalizeAutostartRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (!row || typeof row !== "object" || Array.isArray(row))
    return { error: prefix + "is not a startup program." };
  if (typeof row.command !== "string") return { error: prefix + "has no command." };
  var command = sanitizeListCommand(row.command);
  if (!command) return { error: prefix + "has a command Atmos will not write." };
  var delay = 0;
  if (row.delay != null && row.delay !== "") {
    delay = Math.round(Number(row.delay));
    if (!isFinite(delay) || delay < 0 || delay > 600)
      return { error: prefix + "has a delay Atmos will not write." };
  }
  if (row.enabled != null && row.enabled !== true && row.enabled !== false)
    return { error: prefix + "has a bad enabled flag." };
  return { value: { command: command, delay: delay, enabled: row.enabled !== false } };
}

function canonicalizeWindowRuleRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (!row || typeof row !== "object" || Array.isArray(row))
    return { error: prefix + "is not a window rule." };
  var match = sanitizeWindowMatch(row.match);
  if (!match) return { error: prefix + "has no class to match." };
  var placement = row.placement == null ? "" : String(row.placement);
  if (row.float === true) placement = "float";
  if (row.tile === true && placement !== "float") placement = "tile";
  if (placement !== "" && placement !== "float" && placement !== "tile")
    return { error: prefix + "is not a valid placement." };
  if (row.center != null && row.center !== true && row.center !== false)
    return { error: prefix + "has a bad center flag." };
  var center = row.center === true;
  var width = clampWindowSize(row.width);
  var height = clampWindowSize(row.height);
  if (Array.isArray(row.size) && row.size.length >= 2) {
    width = clampWindowSize(row.size[0]);
    height = clampWindowSize(row.size[1]);
  }
  var sentSize =
    row.width != null || row.height != null || (Array.isArray(row.size) && row.size.length >= 2);
  if (!(width && height)) {
    width = 0;
    height = 0;
  }
  if (sentSize && !(width && height)) return { error: prefix + "has a size Atmos will not write." };
  var workspace = "";
  if (row.workspace != null && row.workspace !== "") {
    workspace = sanitizeWindowWorkspace(row.workspace);
    if (!workspace) return { error: prefix + "is not a valid workspace." };
  }
  var title = "";
  if (row.title != null && row.title !== "") {
    title = sanitizeWindowMatch(row.title);
    if (!title) return { error: prefix + "has a title Atmos will not write." };
  }
  if (row.pin != null && row.pin !== true && row.pin !== false)
    return { error: prefix + "has a bad pin flag." };
  if (row.fullscreen != null && row.fullscreen !== true && row.fullscreen !== false)
    return { error: prefix + "has a bad fullscreen flag." };
  var opacity = "";
  if (row.opacity != null && row.opacity !== "") {
    opacity = String(row.opacity).replace(/^\s+|\s+$/g, "");
    if (!/^[0-9.]+( [0-9.]+)?$/.test(opacity) || opacity.length > 16)
      return { error: prefix + "has an opacity Atmos will not write." };
  }
  if (
    !placement &&
    !center &&
    !width &&
    !workspace &&
    !title &&
    row.pin !== true &&
    row.fullscreen !== true &&
    !opacity
  )
    return { error: prefix + "does nothing." };
  return {
    value: normalizeListRow({
      match: match,
      title: title,
      placement: placement,
      center: center,
      width: width,
      height: height,
      workspace: workspace,
      pin: row.pin === true,
      fullscreen: row.fullscreen === true,
      opacity: opacity,
    }),
  };
}

function canonicalizeWorkspaceRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (!row || typeof row !== "object" || Array.isArray(row))
    return { error: prefix + "is not a workspace." };
  var id = String(row.id || row.workspace || "").replace(/^\s+|\s+$/g, "");
  if (!/^special:[A-Za-z0-9_-]{1,24}$/.test(id) && !/^[1-9]$|^10$/.test(id))
    return { error: prefix + "is not a valid workspace id." };
  var name = "";
  if (row.name != null && row.name !== "") {
    name = String(row.name);
    if (name.indexOf("\n") !== -1 || name.length > 32 || !/^[A-Za-z0-9 _.-]+$/.test(name))
      return { error: prefix + "has a name Atmos will not write." };
  }
  var monitor = "";
  if (row.monitor != null && row.monitor !== "") {
    monitor = String(row.monitor).replace(/^\s+|\s+$/g, "");
    if (!/^[A-Za-z0-9._-]+$/.test(monitor))
      return { error: prefix + "has a monitor Atmos will not write." };
  }
  return {
    value: normalizeListRow({
      id: id,
      name: name,
      persistent: row.persistent !== false,
      monitor: monitor,
      isDefault: row.isDefault === true || row.default === true,
      special: id.indexOf("special:") === 0,
      onCreatedEmpty: sanitizeListCommand(row.onCreatedEmpty || row.on_created_empty || ""),
    }),
  };
}

function canonicalizeMonitorRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (!row || typeof row !== "object" || Array.isArray(row))
    return { error: prefix + "is not a monitor rule." };
  var output = String(row.output || "").replace(/^\s+|\s+$/g, "");
  if (!/^[A-Za-z0-9._-]+$/.test(output) || output.length > 64)
    return { error: prefix + "has no output name." };
  var mode = String(row.mode || "preferred").replace(/^\s+|\s+$/g, "");
  if (mode !== "preferred" && mode !== "highres" && mode !== "highrr") {
    if (!/^[0-9]{3,5}x[0-9]{3,5}(@[0-9]+(\.[0-9]+)?)?$/.test(mode))
      return { error: prefix + "has a mode Atmos will not write." };
  }
  var scale = Number(row.scale);
  if (!isFinite(scale) || scale <= 0) scale = 1;
  if (scale > 4) scale = 4;
  scale = Math.round(scale * 1000) / 1000;
  var transform = Math.round(Number(row.transform || 0));
  if (!isFinite(transform) || transform < 0 || transform > 7) transform = 0;
  var vrr = Math.round(Number(row.vrr || 0));
  if (!isFinite(vrr) || vrr < 0 || vrr > 3) vrr = 0;
  var bitdepth = Math.round(Number(row.bitdepth || 8)) === 10 ? 10 : 8;
  var cm = String(row.cm || "");
  if (["", "auto", "srgb", "dcip3", "wide", "hdr", "hdredid"].indexOf(cm) === -1) cm = "";
  var mirror = String(row.mirror || "").replace(/^\s+|\s+$/g, "");
  if (mirror && !/^[A-Za-z0-9._-]+$/.test(mirror))
    return { error: prefix + "has a mirror output Atmos will not write." };
  var position = String(row.position || "auto").replace(/^\s+|\s+$/g, "");
  if (position !== "auto" && position !== "0x0" && !/^-?[0-9]+x-?[0-9]+$/.test(position))
    position = "auto";
  return {
    value: normalizeListRow({
      output: output,
      mode: mode,
      position: position,
      scale: scale,
      transform: transform,
      disabled: row.disabled === true,
      mirror: mirror,
      vrr: vrr,
      bitdepth: bitdepth,
      cm: cm,
    }),
  };
}

function canonicalizeEnvVarRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (!row || typeof row !== "object" || Array.isArray(row))
    return { error: prefix + "is not an environment variable." };
  var key = String(row.key || "").replace(/^\s+|\s+$/g, "");
  if (!/^[A-Za-z_][A-Za-z0-9_]{0,63}$/.test(key) || key === "PATH")
    return { error: prefix + "has a name Atmos will not write." };
  var value = String(row.value == null ? "" : row.value);
  if (value.indexOf("\n") !== -1 || value.indexOf("\r") !== -1 || value.length > 512)
    return { error: prefix + "has a value Atmos will not write." };
  return { value: { key: key, value: value } };
}

function canonicalizeIndicatorRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (typeof row !== "string" || !row) return { error: prefix + "is not a known indicator." };
  if (indicatorIds().indexOf(row) === -1) return { error: prefix + "is not a known indicator." };
  return { value: row };
}

function canonicalizeStringIdRow(row, n, label) {
  var prefix = listRowPrefix(label, n);
  if (typeof row !== "string" || !row) return { error: prefix + "is not a tray id." };
  return { value: row };
}

function sameValue(a, b) {
  if (typeof a === "number" && typeof b === "number") return Math.abs(a - b) < 1e-9;
  if (Array.isArray(a) && Array.isArray(b))
    return JSON.stringify(exportList(a)) === JSON.stringify(exportList(b));
  return a === b;
}

// A name that says what the file is, whose machine it came from, and when
// it was taken, because these files pile up in a downloads folder and
// "atmos-settings.md" tells you nothing about which one you want.
//
//   atmos-export-vic-2026-09-03-1930.md
//
// Local time rather than UTC: the person reading the folder listing is the
// person who made it. The date leads the time so the names sort by age.
function exportFileName(hostname, when) {
  var host = fileSafe(hostname);
  // Duck-typed rather than instanceof: a Date made in another QML or JS
  // context is not an instance of this context's Date.
  var usable = !!when && typeof when.getTime === "function" && !isNaN(when.getTime());
  var date = usable ? when : new Date();
  var stamp =
    date.getFullYear() +
    "-" +
    pad2(date.getMonth() + 1) +
    "-" +
    pad2(date.getDate()) +
    "-" +
    pad2(date.getHours()) +
    pad2(date.getMinutes());
  return "atmos-export-" + (host.length > 0 ? host + "-" : "") + stamp + ".md";
}

// Hostnames are usually tame, but a name is only useful if it is also a
// filename, so anything that is not a letter, digit, or dash goes.
function fileSafe(value) {
  return String(value === null || value === undefined ? "" : value)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 40);
}

function pad2(n) {
  return n < 10 ? "0" + n : String(n);
}

// What the file says about itself. A plan tells you what would change; this
// tells you what you are about to trust, which is the other half of reading
// a settings file before running it.
function fileSummary(doc, plan) {
  var meta = (doc && doc.meta) || {};
  var sections = (doc && doc.sections) || {};
  var lines = [];

  var host = String(meta.hostname || "");
  var when = String(meta.exported || "");
  if (host || when) {
    lines.push(
      "From " +
        (host || "an unnamed machine") +
        (when
          ? ", exported " + when.replace("T", " ").replace(/\..*$/, "").replace("Z", " UTC")
          : ""),
    );
  }

  var hardware = String(meta.hardware || "");
  if (hardware) lines.push("Hardware: " + hardware);

  var names = [];
  for (var id in sections) {
    if (Object.prototype.hasOwnProperty.call(sections, id)) names.push(id);
  }
  names.sort();
  if (names.length > 0)
    lines.push(countLabel(names.length, "section", "sections") + ": " + names.join(", "));

  if (plan) {
    var settled = plan.unchanged.length;
    if (settled > 0) lines.push(settled + " of these already match this machine.");
  }

  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// Rendering a plan
// ---------------------------------------------------------------------------
//
// The page shows these as text. Building them here rather than in QML keeps
// the wording under test.

function changeLines(plan) {
  var list = (plan && plan.changes) || [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var change = list[i];
    var line = "• " + change.label + ": " + shownValue(change.from) + " → " + shownValue(change.to);
    if (Array.isArray(change.from) || Array.isArray(change.to)) {
      var diffs = listDiffLines(change.from, change.to, change.key);
      if (diffs.length > 0) line += "\n" + diffs.join("\n");
    }
    if (change.consequence) line += "\n    " + change.consequence;
    out.push(line);
  }
  return out.join("\n");
}

function listRowId(row, key) {
  if (row === null || row === undefined) return "";
  if (typeof row !== "object") return "\0" + String(row);
  if (key === "bindings")
    return "k:" + String(row.keys || "") + "\0u:" + (row.unbind === true ? "1" : "0");
  if (key === "autostart") return "c:" + String(row.command || "");
  if (key === "windowRules")
    return (
      "m:" +
      String(row.match || "") +
      "\0p:" +
      String(row.placement || "") +
      "\0w:" +
      String(row.workspace || "")
    );
  if (row.command) return "c:" + String(row.command);
  if (row.keys) return "k:" + String(row.keys);
  return JSON.stringify(row);
}

function shownListRow(row, key) {
  if (row === null || row === undefined) return "(empty)";
  if (typeof row !== "object") return String(row);
  if (key === "bindings" || row.keys) {
    var keys = String(row.keys || "?");
    if (row.unbind) return keys + " unbound";
    var cmd = String(row.command || "(no command)");
    var label = String(row.label || "");
    return label ? keys + " → " + cmd + "  (" + label + ")" : keys + " → " + cmd;
  }
  if (key === "autostart" || row.command) return String(row.command || "(no command)");
  if (key === "windowRules" || row.match) {
    var bits = [String(row.match || "?")];
    if (row.placement) bits.push(String(row.placement));
    if (row.workspace) bits.push("workspace " + row.workspace);
    if (row.width && row.height) bits.push(row.width + "×" + row.height);
    return bits.join(", ");
  }
  return JSON.stringify(row);
}

function normalizeListRow(row) {
  if (row === null || row === undefined || typeof row !== "object" || Array.isArray(row))
    return row;
  var keys = Object.keys(row).sort();
  var copy = {};
  for (var i = 0; i < keys.length; i++) {
    if (keys[i] === "managed") continue;
    copy[keys[i]] = row[keys[i]];
  }
  return copy;
}

// Drop `managed` and sort keys so a file you typed matches the live list.
function exportList(list) {
  var src = Array.isArray(list) ? list : [];
  var out = [];
  for (var i = 0; i < src.length; i++) out.push(normalizeListRow(src[i]));
  return out;
}

function listDiffLines(from, to, key) {
  var before = Array.isArray(from) ? from : [];
  var after = Array.isArray(to) ? to : [];
  var oldMap = {};
  var newMap = {};
  var i;
  for (i = 0; i < before.length; i++) oldMap[listRowId(before[i], key)] = before[i];
  for (i = 0; i < after.length; i++) newMap[listRowId(after[i], key)] = after[i];
  var lines = [];
  for (i = 0; i < after.length; i++) {
    var id = listRowId(after[i], key);
    if (!Object.prototype.hasOwnProperty.call(oldMap, id)) {
      lines.push("    + " + shownListRow(after[i], key));
    } else if (
      JSON.stringify(normalizeListRow(oldMap[id])) !== JSON.stringify(normalizeListRow(after[i]))
    ) {
      lines.push("    ~ " + shownListRow(after[i], key));
    }
  }
  for (i = 0; i < before.length; i++) {
    if (!Object.prototype.hasOwnProperty.call(newMap, listRowId(before[i], key)))
      lines.push("    - " + shownListRow(before[i], key));
  }
  return lines;
}

function warningLines(plan) {
  var list = (plan && plan.warnings) || [];
  var out = [];
  for (var i = 0; i < list.length; i++) out.push("• " + list[i].message);
  return out.join("\n");
}

function blockedLines(plan) {
  var list = (plan && plan.blocked) || [];
  var out = [];
  for (var i = 0; i < list.length; i++) out.push("• " + list[i].reason);
  return out.join("\n");
}

// A value as a person would read it, including the ones that are absent.
function shownValue(value) {
  if (value === null || value === undefined || value === "") return "not set";
  return displayValue(value);
}

// The plan as the executor reads it. `scripts/apply-settings.sh` takes this
// on stdin, so the review screen and the executor cannot disagree about what
// is being applied. Each change carries `from` as well as `value`, which is
// what lets the executor write an undo plan before it touches anything.
function planToJson(plan) {
  var list = (plan && plan.changes) || [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    out.push({ key: list[i].key, value: list[i].to, from: list[i].from });
  }
  return JSON.stringify({ schema: SETTINGS_SCHEMA, changes: out });
}

// ---------------------------------------------------------------------------
// Write dispatch
// ---------------------------------------------------------------------------
//
// Catalog rows own the writer. QML setters and apply-settings.sh both call
// commandFor / planCommands so argv cannot drift between live UI and import.

var SCRIPT_FILES = {
  look: "set-hypr-look.sh",
  input: "set-hypr-input.sh",
  bindings: "set-hypr-bindings.sh",
  windows: "set-hypr-windows.sh",
  autostart: "set-hypr-autostart.sh",
  workspaces: "set-hypr-workspaces.sh",
  monitors: "set-hypr-monitors.sh",
  env: "set-env.sh",
  tweaks: "set-tweaks.sh",
  idle: "set-idle.sh",
  presentation: "set-presentation.sh",
  chargeLimit: "set-charge-limit.sh",
  hyprsunset: "set-hyprsunset.sh",
  nightlightTemp: "set-nightlight-temp.sh",
  mime: "set-mime-default.sh",
  audio: "set-audio.sh",
  barWidget: "set-bar-widget.sh",
  hostname: "set-hostname.sh",
  timezone: "set-timezone.sh",
  locale: "set-locale.sh",
  keyboard: "set-keyboard-layout.sh",
  ntp: "set-ntp.sh",
  fullName: "set-full-name.sh",
  parallelDownloads: "set-parallel-downloads.sh",
  wifiRadio: "set-wifi-connection.sh",
};

var APPLY_GROUP = {
  theme: "look",
  background: "look",
  font: "look",
  textSize: "look",
  hyprLook: "look",
  hyprLookManaged: "look",
  hyprNoGaps: "look",
  hyprSquareAspect: "look",
  barPosition: "look",
  barTransparent: "look",
  barVisible: "look",
  clockFormat: "look",
  clockFormatAlt: "look",
  clockWeekStart: "look",
  clockBirthYear: "look",
  clockLifeExpectancy: "look",
  idleScreensaver: "look",
  idleLock: "look",
  stayAwake: "look",
  screensaverEnabled: "look",
  nightlight: "look",
  nightlightTemperature: "look",
  nightlightDay: "look",
  nightlightNight: "look",
  nightlightNightOn: "look",
  doNotDisturb: "look",
  indicatorsAlwaysShow: "look",
  indicatorsItems: "look",
  agentsRefreshIntervalSec: "look",
  agentsSync: "look",
  agentsSyncDir: "look",
  agentsSyncFileName: "look",
  agentsSyncDeviceId: "look",
  spacerSize: "look",
  trayHidden: "look",
  trayPinned: "look",
  plymouth: "look",
  touchpadEnabled: "look",
  touchscreenEnabled: "look",
  suspendEnabled: "look",
  dns: "network",
  bluetooth: "network",
  wifiRadio: "network",
  hostname: "system",
  timezone: "system",
  locale: "system",
  keyboardLayout: "system",
  ntp: "system",
  ntpSynchronized: "system",
  parallelDownloads: "system",
  crashCapture: "system",
  fullName: "accounts",
  workspaces: "look",
  workspaceWrapSwitch: "look",
  workspaceWheelSwitch: "look",
  monitorRules: "look",
  envVars: "system",
  envPathPrepend: "system",
  presentationMode: "look",
};

var WRITERS;

function writers() {
  if (WRITERS) return WRITERS;
  WRITERS = {
    theme: omarchyArgv(["omarchy", "theme", "set"], "look"),
    background: omarchyArgv(["omarchy", "theme", "bg", "set"], "look"),
    font: omarchyArgv(["omarchy", "font", "set"], "look"),
    textSize: omarchyArgv(["omarchy", "display", "text", "size"], "look"),
    hyprNoGaps: {
      kind: "hypr-toggle",
      prefix: ["omarchy", "hyprland", "toggle", "window-no-gaps"],
      snapshotGroup: "look",
    },
    hyprSquareAspect: {
      kind: "hypr-toggle",
      prefix: ["omarchy", "hyprland", "toggle", "single-window-aspect-ratio"],
      snapshotGroup: "look",
    },
    barPosition: omarchyArgv(["omarchy", "bar", "position"], "look"),
    barTransparent: omarchyArgv(["omarchy", "bar", "transparent"], "look", "true-false"),
    barVisible: {
      kind: "toggle-inverted",
      prefix: ["omarchy", "toggle", "bar"],
      invert: true,
      snapshotGroup: "look",
    },
    clockFormat: { kind: "clock-format", base: "format", snapshotGroup: "look", backup: "clock" },
    clockFormatAlt: {
      kind: "clock-format",
      base: "formatAlt",
      snapshotGroup: "look",
      backup: "clock",
    },
    browser: omarchyArgv(["omarchy", "default", "browser"], ""),
    terminal: omarchyArgv(["omarchy", "default", "terminal"], ""),
    editor: omarchyArgv(["omarchy", "default", "editor"], ""),
    agent: omarchyArgv(["omarchy", "default", "agent"], ""),
    mimePdf: { kind: "script", script: "mime", args: ["pdf"] },
    mimeImage: { kind: "script", script: "mime", args: ["image"] },
    mimeVideo: { kind: "script", script: "mime", args: ["video"] },
    idleScreensaver: { kind: "idle-pair", snapshotGroup: "look" },
    idleLock: { kind: "idle-pair", snapshotGroup: "look" },
    stayAwake: { kind: "toggle-named", snapshotGroup: "look" },
    screensaverEnabled: {
      kind: "toggle-inverted",
      prefix: ["omarchy", "toggle", "screensaver-off"],
      invert: true,
      snapshotGroup: "look",
    },
    doNotDisturb: {
      kind: "toggle-flip",
      prefix: ["omarchy", "toggle", "notification", "silencing"],
      snapshotGroup: "look",
    },
    nightlight: {
      kind: "toggle-flip",
      prefix: ["omarchy", "toggle", "nightlight"],
      snapshotGroup: "look",
    },
    nightlightTemperature: {
      kind: "nightlight-temp",
      snapshotGroup: "look",
      backup: "nightlightTemp",
    },
    nightlightDay: {
      kind: "nightlight-schedule",
      snapshotGroup: "look",
      backup: "nightlightSchedule",
    },
    nightlightNight: {
      kind: "nightlight-schedule",
      snapshotGroup: "look",
      backup: "nightlightSchedule",
    },
    nightlightNightOn: {
      kind: "nightlight-schedule",
      snapshotGroup: "look",
      backup: "nightlightSchedule",
    },
    hostname: { kind: "script", script: "hostname", snapshotGroup: "system" },
    timezone: { kind: "script", script: "timezone", snapshotGroup: "system" },
    locale: { kind: "script", script: "locale", snapshotGroup: "system" },
    keyboardLayout: { kind: "script", script: "keyboard", snapshotGroup: "system" },
    ntp: { kind: "script", script: "ntp", bool: "true-false", snapshotGroup: "system" },
    fullName: { kind: "script", script: "fullName", snapshotGroup: "accounts" },
    parallelDownloads: { kind: "script", script: "parallelDownloads", snapshotGroup: "system" },
    dns: omarchyArgv(["omarchy", "dns"], "network"),
    audioOutputVolume: { kind: "script", script: "audio", args: ["output-volume"] },
    audioInputVolume: { kind: "script", script: "audio", args: ["input-volume"] },
    audioOutputMuted: {
      kind: "mute-deferred",
      prefix: ["omarchy", "audio", "output", "volume", "mute-toggle"],
    },
    audioInputMuted: { kind: "mute-deferred", prefix: ["omarchy", "audio", "input", "mute"] },
    audioTuningOn: omarchyArgv(["omarchy", "audio", "tuning"], "", "on-off"),
    powerProfile: omarchyArgv(["omarchy", "powerprofiles", "set", "autodetect"], ""),
    powerProfileAc: omarchyArgv(["omarchy", "powerprofiles", "set", "ac"], ""),
    powerProfileBattery: omarchyArgv(["omarchy", "powerprofiles", "set", "battery"], ""),
    suspendEnabled: {
      kind: "toggle-inverted",
      prefix: ["omarchy", "toggle", "suspend-off"],
      invert: true,
      snapshotGroup: "look",
    },
    crashCapture: {
      kind: "toggle-flip",
      prefix: ["omarchy", "toggle", "crash", "capture"],
      snapshotGroup: "system",
    },
    clockWeekStart: barSet(["omarchy", "bar", "set", "omarchy.clock", "weekStartDay"], "look"),
    clockBirthYear: barSet(["omarchy", "bar", "set", "omarchy.clock", "birthYear"], "look", true),
    clockLifeExpectancy: barSet(
      ["omarchy", "bar", "set", "omarchy.clock", "lifeExpectancy"],
      "look",
      true,
    ),
    indicatorsAlwaysShow: barSet(
      ["omarchy", "bar", "set", "omarchy.indicators", "alwaysShow"],
      "look",
      true,
      "true-false",
    ),
    powerShowPercentage: barSet(
      ["omarchy", "bar", "set", "omarchy.power", "showPercentage"],
      "look",
      true,
      "true-false",
    ),
    spacerSize: barSet(["omarchy", "bar", "set", "omarchy.spacer", "size"], "look", true),
    weatherLocation: { kind: "weather-location" },
    weatherUnit: barSet(["omarchy", "bar", "set", "omarchy.weather", "unit"], ""),
    weatherRefreshMinutes: barSet(
      ["omarchy", "bar", "set", "omarchy.weather", "refreshMinutes"],
      "",
      true,
    ),
    agentsRefreshIntervalSec: barSet(
      ["omarchy", "bar", "set", "omarchy.agents", "refreshIntervalSec"],
      "look",
      true,
    ),
    agentsSync: {
      kind: "bar-set",
      argv: ["omarchy", "bar", "set", "omarchy.agents", "syncMode"],
      syncMode: true,
      backup: "clock",
      snapshotGroup: "look",
    },
    agentsSyncDir: barSet(["omarchy", "bar", "set", "omarchy.agents", "syncDir"], "look"),
    agentsSyncFileName: barSet(["omarchy", "bar", "set", "omarchy.agents", "syncFileName"], "look"),
    agentsSyncDeviceId: barSet(["omarchy", "bar", "set", "omarchy.agents", "syncDeviceId"], "look"),
    indicatorsItems: {
      kind: "bar-widget",
      id: "omarchy.indicators",
      field: "items",
      backup: "clock",
      snapshotGroup: "look",
    },
    trayHidden: {
      kind: "bar-widget",
      id: "omarchy.tray",
      field: "hidden",
      backup: "clock",
      snapshotGroup: "look",
    },
    trayPinned: {
      kind: "bar-widget",
      id: "omarchy.tray",
      field: "pinned",
      backup: "clock",
      snapshotGroup: "look",
    },
    plymouth: omarchyArgv(["omarchy", "plymouth", "set", "by", "theme"], "look"),
    touchpadEnabled: {
      kind: "toggle-inverted",
      prefix: ["omarchy", "toggle", "touchpad"],
      snapshotGroup: "look",
    },
    touchscreenEnabled: {
      kind: "toggle-inverted",
      prefix: ["omarchy", "toggle", "touchscreen"],
      snapshotGroup: "look",
    },
    bluetooth: {
      kind: "toggle-inverted",
      prefix: ["omarchy", "bluetooth", "power"],
      snapshotGroup: "network",
    },
    wifiRadio: {
      kind: "toggle-inverted",
      script: "wifiRadio",
      args: ["radio"],
      snapshotGroup: "network",
    },
    bindings: { kind: "list-stdin", script: "bindings", backup: "bindings" },
    windowRules: { kind: "list-stdin", script: "windows", backup: "windowRules" },
    autostart: { kind: "list-stdin", script: "autostart", backup: "autostart" },
    workspaces: { kind: "list-stdin", script: "workspaces", backup: "workspaces" },
    workspaceWrapSwitch: {
      kind: "workspaces-meta",
      field: "wrapSwitch",
      script: "workspaces",
      snapshotGroup: "look",
      backup: "workspaces",
    },
    workspaceWheelSwitch: {
      kind: "workspaces-meta",
      field: "wheelSwitch",
      script: "workspaces",
      snapshotGroup: "look",
      backup: "workspaces",
    },
    monitorRules: { kind: "list-stdin", script: "monitors", backup: "monitorRules" },
    envVars: {
      kind: "env-group",
      field: "vars",
      script: "env",
      snapshotGroup: "system",
      backup: "envVars",
    },
    envPathPrepend: {
      kind: "env-group",
      field: "pathPrepend",
      script: "env",
      snapshotGroup: "system",
      backup: "envVars",
    },
    presentationMode: {
      kind: "script",
      script: "presentation",
      bool: "on-off",
      snapshotGroup: "look",
    },
    chargeLimit: { kind: "script", script: "chargeLimit", snapshotGroup: "" },
  };
  return WRITERS;
}

function omarchyArgv(prefix, snapshotGroup, boolStyle) {
  return {
    kind: "omarchy-argv",
    prefix: prefix,
    snapshotGroup: snapshotGroup || "",
    bool: boolStyle || "",
  };
}

function barSet(argv, snapshotGroup, jsonFlag, boolStyle) {
  return {
    kind: "bar-set",
    argv: argv,
    json: jsonFlag === true,
    bool: boolStyle || "",
    backup: "clock",
    snapshotGroup: snapshotGroup || "",
  };
}

function writerSpec(key) {
  if (String(key || "").indexOf("hyprLook.") === 0) {
    return {
      kind: "hypr-group",
      group: "hyprLook",
      script: "look",
      snapshotGroup: "look",
      backup: "hyprLook",
    };
  }
  if (String(key || "").indexOf("hyprInput.") === 0) {
    return {
      kind: "hypr-group",
      group: "hyprInput",
      script: "input",
      snapshotGroup: "rest",
      backup: "hyprInput",
    };
  }
  if (String(key || "").indexOf("tweaks.") === 0) {
    return {
      kind: "tweak",
      id: String(key).slice("tweaks.".length),
      script: "tweaks",
      snapshotGroup: "rest",
      backup: "tweaks",
    };
  }
  return writers()[key] || null;
}

function writerKindFor(key) {
  var spec = writerSpec(key);
  return spec ? spec.kind : "";
}

function scriptPath(opts, name) {
  var scripts = (opts && opts.scripts) || {};
  if (scripts[name]) return String(scripts[name]);
  var file = SCRIPT_FILES[name] || name;
  var root = opts && opts.root ? String(opts.root).replace(/\/$/, "") : "";
  if (root) return root + "/scripts/" + file;
  return file;
}

function bashScript(opts, name) {
  return ["bash", scriptPath(opts, name)];
}

function scriptsFromRoot(scriptsRoot) {
  var root = String(scriptsRoot || "").replace(/\/$/, "");
  var out = {};
  var k;
  for (k in SCRIPT_FILES) {
    if (Object.prototype.hasOwnProperty.call(SCRIPT_FILES, k))
      out[k] = root + "/" + SCRIPT_FILES[k];
  }
  return out;
}

function copyObject(src) {
  var out = {};
  var k;
  if (!src || typeof src !== "object" || Array.isArray(src)) return out;
  for (k in src) {
    if (Object.prototype.hasOwnProperty.call(src, k)) out[k] = src[k];
  }
  return out;
}

function formatBool(value, style) {
  if (style === "true-false") return value === true ? "true" : "false";
  if (style === "on-off") return value === true ? "on" : "off";
  return String(value);
}

function onOff(value, invert) {
  var on = value === true;
  if (invert) on = !on;
  return on ? "on" : "off";
}

function clockBarKey(base, snapshot) {
  var position = String(readValue(snapshot, "barPosition") || "");
  if (position === "left" || position === "right") {
    return "vertical" + base.charAt(0).toUpperCase() + base.slice(1);
  }
  return base;
}

function skipRecord(key) {
  return { skip: true, key: key };
}

function tagApply(apply) {
  if (!apply || typeof apply !== "object") return apply || {};
  var group = "";
  var k;
  for (k in apply) {
    if (!Object.prototype.hasOwnProperty.call(apply, k) || k === "group") continue;
    var g = APPLY_GROUP[k] || "";
    if (!g) {
      group = "";
      break;
    }
    if (!group) group = g;
    else if (group !== g) {
      group = "";
      break;
    }
  }
  if (group) apply.group = group;
  return apply;
}

function keyApply(key, value, extra) {
  var apply = extra ? copyObject(extra) : {};
  apply[key] = value;
  return tagApply(apply);
}

function commandRecord(key, argv, spec, extra) {
  extra = extra || {};
  spec = spec || {};
  var apply = extra.apply;
  if (!apply) apply = keyApply(key, extra.value);
  else apply = tagApply(apply);
  return {
    key: key,
    argv: argv || [],
    stdin: extra.stdin || "",
    mergeGroup: extra.mergeGroup || "",
    sudo: extra.sudo === true,
    skip: false,
    coalesceKey: extra.coalesceKey || "",
    apply: apply,
    backup: extra.backup || spec.backup || "",
    snapshotGroup: extra.snapshotGroup || spec.snapshotGroup || "",
  };
}

function listPayload(key, value) {
  var list = Array.isArray(value) ? value : [];
  var i;
  if (key === "bindings") {
    var items = [];
    for (i = 0; i < list.length; i++) {
      var row = list[i] && typeof list[i] === "object" ? list[i] : {};
      items.push({
        keys: String(row.keys || ""),
        label: String(row.label || ""),
        command: String(row.command || ""),
        unbind: row.unbind === true,
      });
    }
    return JSON.stringify({ items: items });
  }
  if (key === "windowRules") {
    var rules = [];
    for (i = 0; i < list.length; i++) rules.push(stripManagedRow(list[i]));
    return JSON.stringify({ items: rules });
  }
  if (key === "autostart") {
    var items = [];
    for (i = 0; i < list.length; i++) {
      var item = list[i];
      if (typeof item === "string") items.push({ command: item, delay: 0, enabled: true });
      else if (item && item.command != null) {
        items.push({
          command: String(item.command),
          delay: Math.round(Number(item.delay || 0)) || 0,
          enabled: item.enabled !== false,
        });
      }
    }
    return JSON.stringify({ items: items, commands: items });
  }
  if (key === "workspaces") {
    return JSON.stringify({ items: list.map(stripManagedRow) });
  }
  if (key === "monitorRules") {
    return JSON.stringify({ items: list.map(stripManagedRow) });
  }
  if (key === "envVars") {
    return JSON.stringify({ vars: list.map(stripManagedRow) });
  }
  return JSON.stringify(list);
}

function stripManagedRow(row) {
  if (!row || typeof row !== "object" || Array.isArray(row)) return row;
  var out = {};
  var k;
  for (k in row) {
    if (Object.prototype.hasOwnProperty.call(row, k) && k !== "managed") out[k] = row[k];
  }
  return out;
}

function mergeUnmanaged(current, next, key) {
  var src = Array.isArray(current) ? current : [];
  var add = Array.isArray(next) ? next : [];
  var out = [];
  var i;
  for (i = 0; i < src.length; i++) {
    if (src[i] && src[i].managed === false) out.push(src[i]);
  }
  for (i = 0; i < add.length; i++) {
    var item = add[i];
    if (item == null) continue;
    if (typeof item === "string") {
      if (key === "autostart") out.push({ command: item, managed: true });
      else out.push(item);
      continue;
    }
    if (typeof item !== "object") continue;
    var row = copyObject(item);
    row.managed = true;
    out.push(row);
  }
  return out;
}

function overlayChanges(snapshot, changes) {
  var snap = copyObject(snapshot);
  var list = Array.isArray(changes) ? changes : [];
  var i;
  for (i = 0; i < list.length; i++) {
    var key = String(list[i].key || "");
    if (!key) continue;
    var value = list[i].value;
    var dot = key.indexOf(".");
    if (dot === -1) {
      snap[key] = value;
      continue;
    }
    var head = key.slice(0, dot);
    var tail = key.slice(dot + 1);
    var node = copyObject(snap[head]);
    node[tail] = value;
    snap[head] = node;
  }
  return snap;
}

function mergeId(spec) {
  if (!spec) return "";
  if (spec.kind === "hypr-group") return spec.group;
  if (spec.kind === "idle-pair") return "idle";
  if (spec.kind === "nightlight-schedule") return "nightlightSchedule";
  return "";
}

function argvValue(value, spec) {
  if (spec && spec.bool) return formatBool(value, spec.bool);
  if (spec && spec.syncMode) return value === true ? "On" : "Off";
  if (typeof value === "boolean") return value ? "true" : "false";
  return String(value);
}

function commandFor(key, value, snapshot, opts) {
  var item = catalogByKey()[key];
  if (!item || !item.importable) return null;
  var spec = writerSpec(key);
  if (!spec) return null;
  opts = opts || {};
  snapshot = snapshot || {};
  var sudo = item.needsRoot === true;
  if (
    spec.kind === "toggle-inverted" ||
    spec.kind === "toggle-named" ||
    spec.kind === "toggle-flip" ||
    spec.kind === "mute-deferred"
  ) {
    if (sameValue(readValue(snapshot, key), value)) return skipRecord(key);
  }

  if (spec.kind === "omarchy-argv") {
    return commandRecord(key, spec.prefix.concat([argvValue(value, spec)]), spec, {
      sudo: sudo,
      value: value,
    });
  }

  if (spec.kind === "toggle-inverted") {
    var flag = onOff(value, spec.invert === true);
    var invertedArgv;
    if (spec.script) {
      invertedArgv = bashScript(opts, spec.script)
        .concat(spec.args || [])
        .concat([flag]);
    } else {
      invertedArgv = spec.prefix.concat([flag]);
    }
    return commandRecord(key, invertedArgv, spec, { sudo: sudo, value: value === true });
  }

  if (spec.kind === "toggle-named") {
    var named = value === true ? "stay-awake" : "allow-idle";
    return commandRecord(key, ["omarchy", "toggle", "idle", named], spec, {
      sudo: sudo,
      value: value === true,
    });
  }

  if (spec.kind === "toggle-flip") {
    return commandRecord(key, spec.prefix.slice(), spec, { sudo: sudo, value: value === true });
  }

  if (spec.kind === "hypr-toggle") {
    return commandRecord(key, spec.prefix.concat([value === true ? "on" : "off"]), spec, {
      sudo: sudo,
      value: value === true,
    });
  }

  if (spec.kind === "hypr-group") {
    var field = String(key).slice(spec.group.length + 1);
    var merged = copyObject(readValue(snapshot, spec.group));
    merged[field] = value;
    var payload = JSON.stringify(merged);
    var applyHypr = {};
    applyHypr[spec.group] = merged;
    applyHypr[spec.group === "hyprLook" ? "hyprLookManaged" : "hyprInputManaged"] = true;
    return commandRecord(key, bashScript(opts, spec.script).concat([payload]), spec, {
      stdin: payload,
      mergeGroup: spec.group,
      coalesceKey: spec.group,
      backup: spec.backup,
      snapshotGroup: spec.snapshotGroup,
      apply: applyHypr,
      sudo: sudo,
    });
  }

  if (spec.kind === "idle-pair") {
    var saver = Number(readValue(snapshot, "idleScreensaver"));
    var lock = Number(readValue(snapshot, "idleLock"));
    if (key === "idleScreensaver") saver = Number(value);
    if (key === "idleLock") lock = Number(value);
    if (!isFinite(saver)) saver = 0;
    if (!isFinite(lock)) lock = 0;
    saver = Math.round(saver);
    lock = Math.round(lock);
    return commandRecord(
      key,
      bashScript(opts, "idle").concat([String(saver), String(lock)]),
      spec,
      {
        mergeGroup: "idle",
        coalesceKey: "idle",
        apply: { idleScreensaver: saver, idleLock: lock },
        sudo: sudo,
      },
    );
  }

  if (spec.kind === "nightlight-temp") {
    var temp = Math.round(Number(value));
    return commandRecord(key, bashScript(opts, "nightlightTemp").concat([String(temp)]), spec, {
      backup: "nightlightTemp",
      apply: { nightlightTemperature: temp, nightlight: temp < 6000 },
      sudo: sudo,
    });
  }

  if (spec.kind === "nightlight-schedule") {
    var day = String(readValue(snapshot, "nightlightDay") || "");
    var night = String(readValue(snapshot, "nightlightNight") || "");
    var nightOn = readValue(snapshot, "nightlightNightOn") === true;
    if (key === "nightlightDay") day = String(value || "");
    if (key === "nightlightNight") night = String(value || "");
    if (key === "nightlightNightOn") nightOn = value === true;
    var warmth = Number(readValue(snapshot, "nightlightTemperature"));
    if (!isFinite(warmth) || warmth <= 0) warmth = 4000;
    warmth = Math.round(warmth);
    var schedule = JSON.stringify({
      day: day,
      night: night,
      nightOn: nightOn,
      temperature: warmth,
    });
    return commandRecord(key, bashScript(opts, "hyprsunset").concat([schedule]), spec, {
      mergeGroup: "nightlightSchedule",
      coalesceKey: "nightlightSchedule",
      backup: "nightlightSchedule",
      apply: { nightlightDay: day, nightlightNight: night, nightlightNightOn: nightOn },
      sudo: sudo,
    });
  }

  if (spec.kind === "clock-format") {
    var clockArgv = [
      "omarchy",
      "bar",
      "set",
      "omarchy.clock",
      clockBarKey(spec.base, snapshot),
      String(value),
    ];
    return commandRecord(key, clockArgv, spec, {
      backup: "clock",
      sudo: sudo,
      value: value,
    });
  }

  if (spec.kind === "bar-set") {
    var barArgv = spec.argv.concat([argvValue(value, spec)]);
    if (spec.json) barArgv.push("--json");
    return commandRecord(key, barArgv, spec, { backup: "clock", sudo: sudo, value: value });
  }

  if (spec.kind === "bar-widget") {
    var widgetArgv = bashScript(opts, "barWidget").concat([
      spec.id,
      spec.field,
      JSON.stringify(Array.isArray(value) ? value : []),
    ]);
    return commandRecord(key, widgetArgv, spec, { backup: "clock", sudo: sudo, value: value });
  }

  if (spec.kind === "list-stdin") {
    var listJson = listPayload(key, value);
    if (key === "workspaces") {
      var wrap = readValue(snapshot, "workspaceWrapSwitch");
      var wheel = readValue(snapshot, "workspaceWheelSwitch");
      var parsedWs = {};
      try {
        parsedWs = JSON.parse(listJson);
      } catch (e) {
        parsedWs = { items: [] };
      }
      parsedWs.wrapSwitch = wrap !== false;
      parsedWs.wheelSwitch = wheel !== false;
      parsedWs.count = workspaceCountFromItems(parsedWs.items);
      listJson = JSON.stringify(parsedWs);
    }
    var listApply = {};
    listApply[key] = mergeUnmanaged(snapshot[key], value, key);
    listApply[key + "Managed"] = true;
    return commandRecord(key, bashScript(opts, spec.script).concat([listJson]), spec, {
      stdin: listJson,
      coalesceKey: key,
      backup: spec.backup,
      apply: listApply,
      sudo: sudo,
    });
  }

  if (spec.kind === "workspaces-meta") {
    var items = readValue(snapshot, "workspaces") || [];
    var wrapSwitch = readValue(snapshot, "workspaceWrapSwitch") !== false;
    var wheelSwitch = readValue(snapshot, "workspaceWheelSwitch") !== false;
    if (spec.field === "wrapSwitch") wrapSwitch = value !== false;
    if (spec.field === "wheelSwitch") wheelSwitch = value !== false;
    var wsItems = Array.isArray(items) ? items.map(stripManagedRow) : [];
    var wsPayload = JSON.stringify({
      items: wsItems,
      count: workspaceCountFromItems(wsItems),
      wrapSwitch: wrapSwitch,
      wheelSwitch: wheelSwitch,
    });
    var wsApply = { workspaceWrapSwitch: wrapSwitch, workspaceWheelSwitch: wheelSwitch };
    return commandRecord(key, bashScript(opts, spec.script).concat([wsPayload]), spec, {
      stdin: wsPayload,
      mergeGroup: "workspaces",
      coalesceKey: "workspaces",
      apply: wsApply,
      sudo: sudo,
    });
  }

  if (spec.kind === "env-group") {
    var vars = readValue(snapshot, "envVars") || [];
    var pathPrepend = String(readValue(snapshot, "envPathPrepend") || "");
    if (spec.field === "vars") vars = Array.isArray(value) ? value : [];
    if (spec.field === "pathPrepend") pathPrepend = String(value || "");
    var envPayload = JSON.stringify({
      vars: Array.isArray(vars) ? vars.map(stripManagedRow) : [],
      pathPrepend: pathPrepend,
    });
    return commandRecord(key, bashScript(opts, spec.script).concat([envPayload]), spec, {
      stdin: envPayload,
      mergeGroup: "env",
      coalesceKey: "env",
      apply: { envVars: vars, envPathPrepend: pathPrepend },
      sudo: sudo,
    });
  }

  if (spec.kind === "tweak") {
    var tweakId = String(spec.id || "");
    var action = "";
    if (tweakId === "middlePaste") action = "gtk-middle-paste";
    else if (tweakId === "electronWayland") action = "electron-wayland";
    else if (tweakId === "forceZeroScaling") action = "force-zero-scaling";
    else if (tweakId === "swappiness") action = "swappiness";
    else return null;
    var on = value === true ? "on" : "off";
    var tweakApply = {};
    tweakApply[key] = value === true;
    return commandRecord(key, bashScript(opts, spec.script).concat([action, on]), spec, {
      apply: tweakApply,
      sudo: sudo || tweakId === "swappiness",
    });
  }

  if (spec.kind === "script") {
    var scriptArgv = bashScript(opts, spec.script)
      .concat(spec.args || [])
      .concat([argvValue(value, spec)]);
    var scriptApply = {};
    scriptApply[key] = value;
    if (key === "audioOutputVolume") scriptApply.audioOutputMuted = false;
    if (key === "audioInputVolume") scriptApply.audioInputMuted = false;
    if (key === "ntp")
      scriptApply.ntpSynchronized = value === true ? snapshot.ntpSynchronized === true : false;
    return commandRecord(key, scriptArgv, spec, { apply: scriptApply, sudo: sudo });
  }

  if (spec.kind === "weather-location") {
    var name = String(value || "");
    if (!name) {
      return commandRecord(key, ["omarchy", "weather", "location", "--clear"], spec, {
        apply: { weatherLocation: "", weatherAuto: true, weatherCoords: "" },
        sudo: sudo,
      });
    }
    return commandRecord(key, ["omarchy", "weather", "location", "--set", name], spec, {
      apply: { weatherLocation: name, weatherAuto: false, weatherCoords: "" },
      sudo: sudo,
    });
  }

  if (spec.kind === "mute-deferred") {
    return commandRecord(key, spec.prefix.slice(), spec, { sudo: sudo, value: value === true });
  }

  return null;
}

function planCommands(changes, snapshot, opts) {
  var list = Array.isArray(changes) ? changes : [];
  var snap = overlayChanges(snapshot || {}, list);
  var seen = {};
  var out = [];
  var mutes = { audioOutputMuted: null, audioInputMuted: null };
  var i;
  for (i = 0; i < list.length; i++) {
    var key = String(list[i].key || "");
    var spec = writerSpec(key);
    if (spec && spec.kind === "mute-deferred") {
      mutes[key] = list[i].value;
      continue;
    }
    var merged = mergeId(spec);
    if (merged && seen[merged]) continue;
    // Overlay is for paired writers. onlyIfChanged must see the live value,
    // or every planned toggle looks already applied and is skipped.
    var cmdSnap = snap;
    if (
      spec &&
      (spec.kind === "toggle-inverted" ||
        spec.kind === "toggle-named" ||
        spec.kind === "toggle-flip")
    )
      cmdSnap = snapshot || {};
    var cmd = commandFor(key, list[i].value, cmdSnap, opts);
    if (!cmd || cmd.skip) continue;
    if (merged) seen[merged] = true;
    out.push(cmd);
  }
  var muteSnap = copyObject(snapshot || {});
  // Volume writes unmute, so compare mute against that leftover state.
  if (volumeIn(list, "audioOutputVolume")) muteSnap.audioOutputMuted = false;
  if (volumeIn(list, "audioInputVolume")) muteSnap.audioInputMuted = false;
  var muteKeys = ["audioOutputMuted", "audioInputMuted"];
  for (i = 0; i < muteKeys.length; i++) {
    if (mutes[muteKeys[i]] === null || mutes[muteKeys[i]] === undefined) continue;
    var muteCmd = commandFor(muteKeys[i], mutes[muteKeys[i]], muteSnap, opts);
    if (muteCmd && !muteCmd.skip) out.push(muteCmd);
  }
  return out;
}

function volumeIn(changes, key) {
  var i;
  for (i = 0; i < changes.length; i++) {
    if (String(changes[i].key || "") === key) return true;
  }
  return false;
}

function runCommandsCli(argv) {
  var planPath = "";
  var snapshotPath = "";
  var scriptsRoot = "";
  var i;
  for (i = 0; i < argv.length; i++) {
    var arg = argv[i];
    if (arg === "commands") continue;
    if (arg === "--plan") {
      planPath = String(argv[++i] || "");
      continue;
    }
    if (arg === "--snapshot") {
      snapshotPath = String(argv[++i] || "");
      continue;
    }
    if (arg === "--scripts-root") {
      scriptsRoot = String(argv[++i] || "");
      continue;
    }
    console.error("Settings.js: unknown argument " + arg);
    process.exit(2);
  }
  if (!planPath || !snapshotPath || !scriptsRoot) {
    console.error(
      "Usage: Settings.js commands --plan <file> --snapshot <file> --scripts-root <scripts dir>",
    );
    process.exit(2);
  }
  var fs = require("fs");
  var path = require("path");
  var plan;
  var snapshot;
  try {
    plan = JSON.parse(fs.readFileSync(planPath, "utf8"));
    snapshot = JSON.parse(fs.readFileSync(snapshotPath, "utf8"));
  } catch (e) {
    console.error("apply-settings.sh: could not read plan or snapshot");
    process.exit(1);
  }
  if (!snapshot || typeof snapshot !== "object" || Array.isArray(snapshot)) snapshot = {};
  var changes = plan && Array.isArray(plan.changes) ? plan.changes : [];
  var opts = { root: path.dirname(scriptsRoot), scripts: scriptsFromRoot(scriptsRoot) };
  for (i = 0; i < changes.length; i++) {
    var key = String(changes[i].key || "");
    if (!commandFor(key, changes[i].value, snapshot, opts)) {
      console.error("apply-settings.sh: no writer for " + key);
      process.exit(1);
    }
  }
  process.stdout.write(JSON.stringify(planCommands(changes, snapshot, opts)) + "\n");
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    SETTINGS_SCHEMA: SETTINGS_SCHEMA,
    settingsCatalog: settingsCatalog,
    catalogByKey: catalogByKey,
    settingsSections: settingsSections,
    presetKeys: presetKeys,
    sectionKeys: sectionKeys,
    selectableSections: selectableSections,
    keysForSections: keysForSections,
    exportMarkdown: exportMarkdown,
    parseSettingsMarkdown: parseSettingsMarkdown,
    planImport: planImport,
    planToJson: planToJson,
    commandFor: commandFor,
    planCommands: planCommands,
    typeMatches: typeMatches,
    passwordCount: passwordCount,
    applyForecast: applyForecast,
    applyConfirmMessage: applyConfirmMessage,
    commandConfirmMessage: commandConfirmMessage,
    hasCommandImport: hasCommandImport,
    parseApplyResult: parseApplyResult,
    appliedCountFromResult: appliedCountFromResult,
    backupDirFromResult: backupDirFromResult,
    changeLines: changeLines,
    warningLines: warningLines,
    blockedLines: blockedLines,
    fileSummary: fileSummary,
    exportFileName: exportFileName,
  };
}

if (typeof require !== "undefined" && typeof module !== "undefined" && require.main === module) {
  runCommandsCli(process.argv.slice(2));
}
