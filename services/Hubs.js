// Hub identity for nav, search, launcher, and snapshot groups.
// QML and Node both eval this file.

var LAUNCHER_ALIASES = ["theme", "gpu", "cpu", "npu", "machine"];

function uniqueWords(list) {
  var out = [];
  var seen = {};
  var i, w;
  for (i = 0; i < list.length; i++) {
    w = String(list[i] || "");
    if (!w || seen[w]) continue;
    seen[w] = true;
    out.push(w);
  }
  return out;
}

function keywordList(text, extra) {
  var parts = String(text || "").split(" ");
  var more = extra || [];
  var i;
  for (i = 0; i < more.length; i++) parts.push(more[i]);
  return uniqueWords(parts);
}

function child(id, title, file) {
  return { id: id, title: title, file: file };
}

function hubs() {
  return [
    // First, in its own section. Everything below assumes you already know
    // which page you want; this is the one for when you do not. Placement is
    // a judgement call -- moving it is this block plus one line in
    // hubs.test.js.
    {
      id: "ask",
      title: "Ask",
      description: "Describe what you want in plain words and Atmos will find the setting.",
      navGroup: "ask",
      snapshotGroup: "",
      icon: "question-line",
      file: "AskPage.qml",
      keywords: keywordList("ask question help find search natural language plain words assistant"),
      children: [],
    },
    {
      id: "favorites",
      title: "Favorites",
      description: "Settings you starred, in one place.",
      navGroup: "look",
      snapshotGroup: "look",
      icon: "star-line",
      file: "FavoritesPage.qml",
      keywords: keywordList("favorite favourite star pin bookmark saved", ["favorite", "star"]),
      children: [],
    },
    {
      id: "appearance",
      title: "Appearance",
      description: "Theme, wallpaper, fonts, and how the desktop looks.",
      navGroup: "look",
      snapshotGroup: "look",
      icon: "palette-line",
      file: "AppearancePage.qml",
      keywords: keywordList(
        "theme background wallpaper font text size reset default scale aether palette night light nightlight warmth temperature kelvin schedule hyprsunset plymouth boot screen unlock sddm login refresh reapply templates switcher preview thumbnail picker install git extra clone update pull remove uninstall delete custom folder file image path directory cache logo png render",
        ["theme", "background", "wallpaper", "font", "text", "size", "palette", "nightlight"],
      ),
      children: [
        child("appearance/background", "Background", "appearance/BackgroundPage.qml"),
        child("appearance/boot", "Boot screen", "appearance/BootPage.qml"),
      ],
    },
    {
      id: "display",
      title: "Displays",
      description: "Monitors, scale, and brightness.",
      navGroup: "look",
      snapshotGroup: "look",
      icon: "computer-line",
      file: "DisplaysPage.qml",
      keywords: keywordList(
        "monitor scale hidpi brightness backlight ddc keyboard laptop lid internal mirror clone edp hdmi dp resolution refresh touchpad trackpad pointer mouse input touch touchscreen tablet digitizer",
        ["monitor", "scale", "brightness"],
      ),
      children: [],
    },
    {
      id: "windows",
      title: "Windows",
      description: "Gaps, borders, bindings, and window rules.",
      navGroup: "look",
      snapshotGroup: "look",
      icon: "layout-grid-line",
      file: "WindowsPage.qml",
      keywords: keywordList(
        "gaps border rounding blur shadow tiling dwindle scrolling niri column opacity transparency fullscreen tight square aspect dim animations cursor tearing looknfeel preserve split focus activate swallow window rule float tile class",
        ["gaps", "window"],
      ),
      children: [child("windows/rules", "Window rules", "windows/RulesPage.qml")],
    },
    {
      id: "workspaces",
      title: "Workspaces",
      description: "Count, names, monitors, and how you switch.",
      navGroup: "look",
      snapshotGroup: "look",
      icon: "layout-masonry-line",
      file: "WorkspacesPage.qml",
      keywords: keywordList(
        "workspace named persistent monitor scratch special wrap wheel switch default login bar plugin clone",
        ["workspace"],
      ),
      children: [],
    },
    {
      id: "bar",
      title: "Bar",
      description: "Position, clock, and widgets.",
      navGroup: "look",
      snapshotGroup: "look",
      icon: "layout-top-2-line",
      file: "BarPage.qml",
      keywords: keywordList(
        "position transparent menu bar top bottom left right show hide visible clock format time alternate date week start calendar sunday monday birth year age life expectancy indicators always show status icons dictation recording reminder night light dnd stay awake agents usage refresh sync snapshot file hostname device id spacer gap padding tray hidden icons unhide pin unpin plugin widget",
        ["bar", "clock", "tray"],
      ),
      children: [],
    },
    {
      id: "notifications",
      title: "Notifications",
      description: "Do not disturb and reminders.",
      navGroup: "look",
      snapshotGroup: "look",
      icon: "notification-3-line",
      file: "NotificationsPage.qml",
      keywords: keywordList(
        "do not disturb dnd silent mute quiet toast reminder timer notify later test time battery weather",
        ["dnd", "reminder"],
      ),
      children: [],
    },
    {
      id: "profiles",
      title: "Profiles",
      description: "Apply a bundle of settings at once.",
      navGroup: "look",
      snapshotGroup: "all",
      icon: "profile-line",
      file: "ProfilesPage.qml",
      keywords: keywordList(
        "profile coding gaming battery bundle apply import stay awake performance",
        ["profile"],
      ),
      children: [],
    },
    {
      id: "input",
      title: "Input",
      description: "Pointer, keyboard, and gestures.",
      navGroup: "input",
      snapshotGroup: "all",
      icon: "mouse-line",
      file: "InputPage.qml",
      keywords: keywordList(
        "mouse pointer sensitivity acceleration natural scroll inertia wheel high-res discrete smooth touchpad clickfinger repeat delay numlock follow dpms gesture swipe layout xkb",
        ["mouse", "keyboard", "scroll", "inertia"],
      ),
      children: [],
    },
    {
      id: "keybindings",
      title: "Keybindings",
      description: "Search, add, and override Hyprland chords.",
      navGroup: "input",
      snapshotGroup: "all",
      icon: "keyboard-box-line",
      file: "windows/BindingsPage.qml",
      keywords: keywordList(
        "keybinding hotkey shortcut bind unbind chord record conflict reset window workspace launcher media system",
        ["bind", "shortcut"],
      ),
      children: [],
    },
    {
      id: "accessibility",
      title: "Accessibility",
      description: "Motion, text size, and assistive tools.",
      navGroup: "input",
      snapshotGroup: "all",
      icon: "accessibility-line",
      file: "AccessibilityPage.qml",
      keywords: keywordList(
        "a11y motion animations reduce text size font cursor pointer hide typing touchscreen herdr screen reader",
        ["a11y", "motion"],
      ),
      children: [],
    },
    {
      id: "sound",
      title: "Sound",
      description: "Volume, sinks, and sources.",
      navGroup: "input",
      snapshotGroup: "all",
      icon: "volume-up-line",
      file: "SoundPage.qml",
      keywords: keywordList(
        "audio volume mute speaker headphone sink source microphone mic input output pipewire pactl wpctl tuning dsp restart voxtype dictation speech",
        ["audio", "volume"],
      ),
      children: [],
    },
    {
      id: "capture",
      title: "Capture",
      description: "Screenshots, recordings, and OCR.",
      navGroup: "input",
      snapshotGroup: "all",
      icon: "screenshot-2-line",
      file: "CapturePage.qml",
      keywords: keywordList(
        "screenshot record recording screen video ocr qr webcam grim slurp pictures videos",
        ["screenshot", "record"],
      ),
      children: [],
    },
    {
      id: "hardware",
      title: "Hardware",
      description: "CPU, memory, firmware identity, and the rest of the machine.",
      navGroup: "device",
      snapshotGroup: "all",
      icon: "cpu-line",
      file: "HardwarePage.qml",
      keywords: keywordList(
        "cpu processor intel amd memory ram dimm ddr chipset motherboard bios uefi firmware tpm npu xdna neural tpu ai accelerator dmi chassis laptop desktop framework usb nic thermal battery",
        ["cpu", "memory"],
      ),
      children: [],
    },
    {
      id: "drivers",
      title: "Drivers",
      description: "GPU stack, hybrid switching, and firmware updates.",
      navGroup: "device",
      snapshotGroup: "all",
      icon: "ram-2-line",
      file: "DriversPage.qml",
      keywords: keywordList(
        "gpu graphics nvidia vulkan hybrid supergfx igpu cuda radeon drm amdgpu i915 xe driver firmware fwupd bios gsp",
        ["gpu", "driver", "firmware"],
      ),
      children: [],
    },
    {
      id: "disks",
      title: "Disks",
      description: "Drives, snapshots, and swap.",
      navGroup: "device",
      snapshotGroup: "disks",
      icon: "hard-drive-2-line",
      file: "DisksPage.qml",
      keywords: keywordList(
        "drive nvme ssd lsblk luks encryption snapper snapshot btrfs hibernation zram swap speedtest df usage trim fstrim timeline retention",
        ["drive", "snapper", "swap"],
      ),
      children: [],
    },
    {
      id: "network",
      title: "Network",
      description: "Wi-Fi, DNS, VPN, and speed test.",
      navGroup: "device",
      snapshotGroup: "network",
      icon: "wifi-line",
      file: "NetworkPage.qml",
      keywords: keywordList(
        "dns cloudflare google dhcp custom nameserver resolver radio wifi band ghz wlan qr share ssid password psk copy ethernet status ip speedtest join scan forget restart rfkill connectivity tailscale vpn tailnet localsend taildrop clipboard file folder",
        ["wifi", "dns"],
      ),
      children: [
        child("network/wifi", "Wi-Fi", "network/WifiPage.qml"),
        child("network/speedtest", "Speed test", "network/SpeedtestPage.qml"),
      ],
    },
    {
      id: "bluetooth",
      title: "Bluetooth",
      description: "Adapter, pairing, and trusted devices.",
      navGroup: "device",
      snapshotGroup: "network",
      icon: "bluetooth-line",
      file: "network/BluetoothPage.qml",
      keywords: keywordList(
        "bluetooth radio pair connect disconnect forget trust battery discoverable headset mouse",
        ["bluetooth"],
      ),
      children: [],
    },
    {
      id: "power",
      title: "Power",
      description: "Profiles and battery.",
      navGroup: "device",
      snapshotGroup: "all",
      icon: "battery-2-charge-line",
      file: "PowerPage.qml",
      keywords: keywordList(
        "power profile performance balanced battery saver percentage charge laptop ac plugged adapter charger unplugged discharging status notify draw",
        ["battery", "profile"],
      ),
      children: [],
    },
    {
      id: "idle",
      title: "Idle",
      description: "Screensaver, lock, and lid.",
      navGroup: "device",
      snapshotGroup: "look",
      icon: "lock-2-line",
      file: "IdlePage.qml",
      keywords: keywordList(
        "screensaver lock timeout idle security stay awake caffeine allow disable suspend sleep branding logo ascii lid clamshell",
        ["lock", "screensaver"],
      ),
      children: [],
    },
    {
      id: "defaults",
      title: "Defaults",
      description: "Browser, terminal, editor, and MIME.",
      navGroup: "apps",
      snapshotGroup: "all",
      icon: "links-line",
      file: "DefaultsPage.qml",
      keywords: keywordList(
        "browser terminal editor agent chrome firefox nvim pdf mime image video",
        ["browser", "terminal"],
      ),
      children: [],
    },
    {
      id: "applications",
      title: "Applications",
      description: "Desktop, TUI, and web launchers.",
      navGroup: "apps",
      snapshotGroup: "all",
      icon: "apps-2-line",
      file: "ApplicationsPage.qml",
      keywords: keywordList(
        "desktop app tui webapp web app add install create remove uninstall launcher shortcut delete autostart startup launch",
        ["app", "launcher"],
      ),
      children: [child("applications/startup", "Startup", "applications/StartupPage.qml")],
    },
    {
      id: "software",
      title: "Software",
      description: "Packages and extras.",
      navGroup: "apps",
      snapshotGroup: "all",
      icon: "box-3-line",
      file: "SoftwarePage.qml",
      keywords: keywordList(
        "install remove browser terminal editor service gaming steam heroic lutris docker chatgpt 1password dropbox tailscale nordvpn signal spotify sunshine",
        ["install", "package"],
      ),
      children: [],
    },
    {
      id: "hooks",
      title: "Hooks",
      description: "Theme-set and other scripts.",
      navGroup: "apps",
      snapshotGroup: "all",
      icon: "code-s-slash-line",
      file: "HooksPage.qml",
      keywords: keywordList(
        "hook script theme-set font-set post-boot post-update pacman battery-low",
        ["hook", "script"],
      ),
      children: [],
    },
    {
      id: "system",
      title: "System",
      description: "Host, locale, updates, and about.",
      navGroup: "general",
      snapshotGroup: "system",
      icon: "settings-3-line",
      file: "SystemPage.qml",
      keywords: keywordList(
        "crash capture diagnostics coredump weather location city forecast coordinates latitude longitude gps units celsius fahrenheit metric imperial refresh interval about logo branding fastfetch timezone tz utc region city date time zoneinfo timedatectl hostname computer machine device name hostnamectl keyboard layout keymap xkb qwerty language input localectl ntp timesync synchronize automatic clock network time locale lang utf-8 i18n translation pacman parallel downloads packages mirrors aur update channel orphan prune version printer cups print restore hyprland shell restart atmos git pull reset sentinel overrides report journal systemd portal pipewire kernel environment path editor shell",
        ["hostname", "locale", "update", "diagnostics", "kernel", "uki"],
      ),
      children: [
        child("system/environment", "Environment", "system/EnvironmentPage.qml"),
        child("system/kernel", "Kernel", "system/KernelPage.qml"),
        child("system/diagnostics", "Diagnostics", "system/DiagnosticsPage.qml"),
      ],
    },
    {
      id: "tweaks",
      title: "Tweaks",
      description: "Overflow settings with a one-click reset.",
      navGroup: "general",
      snapshotGroup: "all",
      icon: "equalizer-line",
      file: "TweaksPage.qml",
      keywords: keywordList(
        "tweak acceleration natural scroll paste electron wayland cursor sysctl swappiness nvidia amd laptop",
        ["tweak"],
      ),
      children: [],
    },
    {
      id: "export",
      title: "Omafile",
      description: "Share this Omarchy system as a Markdown file, or apply one.",
      navGroup: "general",
      snapshotGroup: "all",
      icon: "file-transfer-line",
      file: "ExportPage.qml",
      keywords: keywordList(
        "omafile import export backup restore settings file markdown md share declare machine profile transfer move migrate copy clone another laptop dotfiles plan dry run review undo revert apply keybindings bindings rules autostart omarchy",
        ["omafile", "import", "export", "transfer", "backup", "restore", "markdown"],
      ),
      children: [],
    },
    {
      id: "accounts",
      title: "Accounts",
      description: "Face, password, users, and groups.",
      navGroup: "admin",
      snapshotGroup: "accounts",
      icon: "user-3-line",
      file: "AccountsPage.qml",
      keywords: keywordList(
        "account avatar face icon picture sddm user group password passwd login wheel admin docker useradd userdel groupadd usermod chfn gecos full name real name display name chpasswd",
        ["avatar", "user", "group", "password"],
      ),
      children: [],
    },
    {
      id: "security",
      title: "Security",
      description: "Fingerprint, SSH, and sudo.",
      navGroup: "admin",
      snapshotGroup: "all",
      icon: "shield-keyhole-line",
      file: "SecurityPage.qml",
      keywords: keywordList("fingerprint fido2 yubikey ssh sshd sudo passwordless docker pam u2f", [
        "ssh",
        "fingerprint",
      ]),
      children: [],
    },
    {
      id: "services",
      title: "Services",
      description: "Search, filter, and control user and system units.",
      navGroup: "admin",
      snapshotGroup: "all",
      icon: "server-line",
      file: "ServicesPage.qml",
      keywords: keywordList(
        "systemd unit service enable disable start stop restart failed logs pipewire portal bluetooth search filter",
        ["systemd", "service"],
      ),
      children: [],
    },
  ];
}

function hubById(id) {
  var key = String(id || "");
  if (!key) return null;
  var list = hubs();
  var i, j, kids;
  for (i = 0; i < list.length; i++) {
    if (list[i].id === key) return list[i];
    kids = list[i].children || [];
    for (j = 0; j < kids.length; j++) {
      if (kids[j].id === key) return kids[j];
    }
  }
  return null;
}

function rootHub(id) {
  var key = String(id || "");
  var slash = key.indexOf("/");
  return slash === -1 ? key : key.substring(0, slash);
}

function hubTitle(id) {
  var found = hubById(id);
  if (found && found.title) return found.title;
  found = hubById(rootHub(id));
  if (found && found.title) return found.title;
  return String(id || "");
}

function hubIds() {
  var list = hubs();
  var out = [];
  var i;
  for (i = 0; i < list.length; i++) out.push(list[i].id);
  return out;
}

function fileHub(rel) {
  var base = String(rel || "").replace(/\\/g, "/");
  var list = hubs();
  var i, j, kids;
  for (i = 0; i < list.length; i++) {
    if (list[i].file === base) return list[i].id;
    kids = list[i].children || [];
    for (j = 0; j < kids.length; j++) {
      if (kids[j].file === base) return kids[j].id;
    }
  }
  if (base.indexOf("appearance/") === 0) return "appearance";
  if (base.indexOf("network/") === 0) return "network";
  if (base.indexOf("windows/") === 0) return "windows";
  if (base.indexOf("applications/") === 0) return "applications";
  if (base.indexOf("system/") === 0) return "system";
  return "";
}

function snapshotGroupForHub(hub) {
  var id = rootHub(hub);
  if (!id) return "look";
  var found = hubById(id);
  if (found && found.snapshotGroup) return found.snapshotGroup;
  return "all";
}

function navPages() {
  var list = hubs();
  var out = [];
  var i;
  for (i = 0; i < list.length; i++) {
    out.push({
      id: list[i].id,
      title: list[i].title,
      group: list[i].navGroup,
      icon: list[i].icon,
      keywords: list[i].keywords.join(" "),
    });
  }
  return out;
}

function searchHubs() {
  var list = hubs();
  var out = [];
  var i;
  for (i = 0; i < list.length; i++) {
    out.push({
      id: list[i].id,
      title: list[i].title,
      description: list[i].description,
      keywords: list[i].keywords,
    });
  }
  return out;
}

function childIds() {
  var list = hubs();
  var out = [];
  var i, j, kids;
  for (i = 0; i < list.length; i++) {
    kids = list[i].children || [];
    for (j = 0; j < kids.length; j++) out.push(kids[j].id);
  }
  return out;
}

function launcherSuffixes() {
  var kids = childIds();
  var out = [];
  var seen = {};
  var i, tail;
  for (i = 0; i < kids.length; i++) {
    tail = kids[i].split("/").pop();
    if (!tail || seen[tail]) continue;
    seen[tail] = true;
    out.push(tail);
  }
  for (i = 0; i < LAUNCHER_ALIASES.length; i++) {
    tail = LAUNCHER_ALIASES[i];
    if (seen[tail]) continue;
    seen[tail] = true;
    out.push(tail);
  }
  return out;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    hubs: hubs,
    hubById: hubById,
    rootHub: rootHub,
    hubTitle: hubTitle,
    hubIds: hubIds,
    fileHub: fileHub,
    snapshotGroupForHub: snapshotGroupForHub,
    navPages: navPages,
    searchHubs: searchHubs,
    childIds: childIds,
    launcherSuffixes: launcherSuffixes,
  };
}
