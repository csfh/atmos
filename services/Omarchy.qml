pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "AtmosUpdate.js" as AtmosUpdate
import "Hardware.js" as HardwareJs
import "Snapshot.js" as SnapshotJs
import "WorkQueue.js" as WorkQueue

QtObject {
  id: root

  readonly property string shellDir: Quickshell.shellDir
  readonly property string snapshotScript: shellDir + "/scripts/snapshot.sh"
  readonly property string setIdleScript: shellDir + "/scripts/set-idle.sh"
  readonly property string setBarWidgetScript: shellDir + "/scripts/set-bar-widget.sh"
  readonly property string setWifiConnectionScript: shellDir + "/scripts/set-wifi-connection.sh"
  readonly property string setAudioScript: shellDir + "/scripts/set-audio.sh"
  readonly property string setDnsCustomScript: shellDir + "/scripts/set-dns-custom.sh"
  readonly property string luksChangeKeyScript: shellDir + "/scripts/luks-change-key.sh"
  readonly property string rollbackSnapshotScript: shellDir + "/scripts/rollback-snapshot.sh"
  readonly property string enterpriseWifiScript: shellDir + "/scripts/enterprise-wifi-connect.sh"
  readonly property string listThemeImagesScript: shellDir + "/scripts/list-theme-images.sh"
  readonly property string setTimezoneScript: shellDir + "/scripts/set-timezone.sh"
  readonly property string setNtpScript: shellDir + "/scripts/set-ntp.sh"
  readonly property string setHostnameScript: shellDir + "/scripts/set-hostname.sh"
  readonly property string setFullNameScript: shellDir + "/scripts/set-full-name.sh"
  readonly property string setKeyboardLayoutScript: shellDir + "/scripts/set-keyboard-layout.sh"
  readonly property string setLocaleScript: shellDir + "/scripts/set-locale.sh"
  readonly property string setParallelDownloadsScript: shellDir + "/scripts/set-parallel-downloads.sh"
  readonly property string addDesktopLauncherScript: shellDir + "/scripts/add-desktop-launcher.sh"
  readonly property string setHyprLookScript: shellDir + "/scripts/set-hypr-look.sh"
  readonly property string setHyprInputScript: shellDir + "/scripts/set-hypr-input.sh"
  readonly property string setHyprAutostartScript: shellDir + "/scripts/set-hypr-autostart.sh"
  readonly property string setHyprBindingsScript: shellDir + "/scripts/set-hypr-bindings.sh"
  readonly property string setHyprWindowsScript: shellDir + "/scripts/set-hypr-windows.sh"
  readonly property string refreshHyprlandScript: shellDir + "/scripts/refresh-hyprland.sh"
  readonly property string setHyprsunsetScript: shellDir + "/scripts/set-hyprsunset.sh"
  readonly property string setNightlightTempScript: shellDir + "/scripts/set-nightlight-temp.sh"
  readonly property string updateAtmosScript: shellDir + "/scripts/update-atmos.sh"
  readonly property string setAtmosChannelScript: shellDir + "/scripts/set-atmos-channel.sh"
  readonly property string setSnapperPolicyScript: shellDir + "/scripts/set-snapper-policy.sh"
  readonly property string setFstrimScript: shellDir + "/scripts/set-fstrim.sh"
  readonly property string setMimeDefaultScript: shellDir + "/scripts/set-mime-default.sh"
  readonly property string setSshdScript: shellDir + "/scripts/set-sshd.sh"
  readonly property string createHookScript: shellDir + "/scripts/create-hook.sh"
  readonly property string setHookSampleScript: shellDir + "/scripts/set-hook-sample.sh"
  readonly property string looknfeelLuaFile: Quickshell.env("HOME") + "/.config/hypr/looknfeel.lua"
  readonly property string inputLuaFile: Quickshell.env("HOME") + "/.config/hypr/input.lua"
  readonly property string pacmanConfFile: "/etc/pacman.conf"
  readonly property string localtimeFile: "/etc/localtime"
  readonly property string hostnameFile: "/etc/hostname"
  readonly property string vconsoleFile: "/etc/vconsole.conf"
  readonly property string localeConfFile: "/etc/locale.conf"
  readonly property string gumStubDir: shellDir + "/scripts/stubs"
  readonly property string userShellJson: Quickshell.env("HOME") + "/.config/omarchy/shell.json"
  readonly property string defaultShellJson: "/usr/share/omarchy/config/omarchy/shell.json"
  readonly property string userShellToml: Quickshell.env("HOME") + "/.config/omarchy/shell.toml"
  readonly property string weatherJson: Quickshell.env("HOME") + "/.local/state/omarchy/settings/weather.json"
  readonly property string notificationsJson: Quickshell.env("HOME") + "/.local/state/omarchy/notifications.json"
  readonly property string currentThemeNameFile: Quickshell.env("HOME") + "/.local/state/omarchy/current/theme.name"
  readonly property string currentBackgroundFile: Quickshell.env("HOME") + "/.local/state/omarchy/current/background"
  readonly property string screensaverBrandFile: Quickshell.env("HOME") + "/.config/omarchy/branding/screensaver.txt"
  readonly property string defaultScreensaverBrandFile: "/usr/share/omarchy/logo.txt"
  readonly property string aboutBrandFile: Quickshell.env("HOME") + "/.config/omarchy/branding/about.txt"
  readonly property string defaultAboutBrandFile: "/usr/share/omarchy/icon.txt"
  readonly property string powerProfileAcFile: Quickshell.env("HOME") + "/.local/state/omarchy/powerprofiles/ac"
  readonly property string powerProfileBatteryFile: Quickshell.env("HOME") + "/.local/state/omarchy/powerprofiles/battery"
  readonly property string togglesDir: Quickshell.env("HOME") + "/.local/state/omarchy/toggles"
  readonly property string hyprTogglesDir: Quickshell.env("HOME") + "/.local/state/omarchy/toggles/hypr"
  readonly property string touchpadDisabledFile: hyprTogglesDir + "/touchpad-disabled-name"
  readonly property string touchscreenDisabledFile: hyprTogglesDir + "/touchscreen-disabled-name"
  readonly property string indicatorsDir: Quickshell.env("HOME") + "/.local/state/omarchy/indicators"
  readonly property string extraThemesDir: Quickshell.env("HOME") + "/.config/omarchy/themes"
  readonly property string applicationsDir: Quickshell.env("HOME") + "/.local/share/applications"
  readonly property string packagedThemesDir: "/usr/share/omarchy/themes"
  readonly property string defaultEditorFile: Quickshell.env("HOME") + "/.local/state/omarchy/defaults/editor"
  readonly property string defaultAgentFile: Quickshell.env("HOME") + "/.config/omarchy/defaults/agent"
  readonly property string defaultTerminalFile: Quickshell.env("HOME") + "/.config/xdg-terminals.list"
  readonly property string defaultBrowserFile: Quickshell.env("HOME") + "/.config/mimeapps.list"
  readonly property string fontconfigFile: Quickshell.env("HOME") + "/.config/fontconfig/fonts.conf"
  readonly property string reminderDir: (Quickshell.env("XDG_RUNTIME_DIR") || "/tmp") + "/omarchy-reminders"
  readonly property string dnsConfFile: "/etc/NetworkManager/conf.d/20-omarchy-dns.conf"
  readonly property string bluetoothRfkillDir: "/var/lib/systemd/rfkill"
  readonly property string plymouthLogoFile: "/usr/share/plymouth/themes/omarchy/logo.png"
  readonly property string defaultPlymouthLogoFile: "/usr/share/omarchy/default/plymouth/logo.png"
  readonly property string powerProfilesStateFile: "/var/lib/power-profiles-daemon/state.ini"
  readonly property string networkManagerDevicesDir: "/run/NetworkManager/devices"
  readonly property string monitorsLuaFile: Quickshell.env("HOME") + "/.config/hypr/monitors.lua"

  property string lastError: ""
  property string theme: ""
  property string background: ""
  property string font: ""
  property int textSize: 12
  property var themes: []
  property var extraThemes: []
  property var desktopApps: []
  property var tuiApps: []
  property var webApps: []
  property var fonts: []
  property string barPosition: "top"
  property bool barTransparent: false
  property bool barVisible: true
  property string clockFormat: ""
  property string clockFormatAlt: ""
  property string clockWeekStart: ""
  property bool clockPresent: false
  property int clockBirthYear: 0
  property int clockLifeExpectancy: 0
  property bool indicatorsPresent: false
  property bool indicatorsAlwaysShow: false
  property var indicatorsItems: []
  property bool agentsPresent: false
  property int agentsRefreshIntervalSec: 900
  property bool agentsSync: false
  property string agentsSyncDir: ""
  property string agentsSyncFileName: ""
  property string agentsSyncDeviceId: ""
  property bool spacerPresent: false
  property int spacerSize: 12
  property bool trayPresent: false
  property var trayHidden: []
  property var trayPinned: []
  property string browser: ""
  property string terminal: ""
  property string editor: ""
  property string agent: ""
  property string dns: ""
  property int idleScreensaver: 150
  property int idleLock: 300
  property bool stayAwake: false
  property bool nightlight: false
  property int nightlightTemperature: 0
  property bool screensaverEnabled: true
  property bool screensaverBranded: false
  property bool aboutBranded: false
  property bool bluetooth: false
  property bool wifiConnected: false
  property string wifiBand: ""
  property string wifiBandSelected: "auto"
  property var wifiBands: ["auto"]
  property string wifiIface: ""
  property string netKind: "disconnected"
  property string netIface: ""
  property string netSsid: ""
  property string netSignal: ""
  property string netIp: ""
  property string netSpeed: ""
  property bool wifiHw: false
  property bool wifiRadio: false
  property var wifiConnections: []
  property var bluetoothDevices: []
  property var audioSinks: []
  property var audioSources: []
  property int audioOutputVolume: 0
  property bool audioOutputMuted: false
  property int audioInputVolume: 0
  property bool audioInputMuted: false
  property bool audioTuningMatch: false
  property bool audioTuningOn: false
  property string audioSink: ""
  property string audioSource: ""
  property var disks: []
  property var hardware: ({})
  property var luksDevices: []
  property var swapDevices: []
  property bool snapperPresent: false
  property var snapperConfigs: []
  property var snapshots: []
  property bool hibernationAvailable: false
  property bool hibernationSupported: false
  property bool hibernationConfigured: false
  property bool suspendEnabled: true
  property string powerProfile: ""
  property string powerProfileAc: ""
  property string powerProfileBattery: ""
  property var powerProfiles: []
  property bool powerPresent: false
  property bool powerShowPercentage: false
  property bool isLaptop: false
  property bool batteryPresent: false
  property var monitors: []
  property bool internalPresent: false
  property bool internalEnabled: false
  property bool externalPresent: false
  property bool mirroring: false
  property bool touchpadPresent: false
  property bool touchpadEnabled: true
  property bool touchscreenPresent: false
  property bool touchscreenEnabled: true
  property bool keyboardBacklightPresent: false
  property int keyboardBrightness: 0
  property bool crashCapture: true
  property bool doNotDisturb: false
  property string weatherLocation: ""
  property string weatherCoords: ""
  property bool weatherAuto: true
  property bool weatherPresent: false
  property string weatherUnit: "auto"
  property int weatherRefreshMinutes: 15
  property int reminderCount: 0
  property bool reminderActive: false
  property var reminders: []
  property bool jobBusy: false
  property string jobKind: ""
  property string jobLog: ""
  property string jobStdin: ""
  property string plymouth: ""
  property var plymouthThemes: []
  property bool hasAether: false
  property var browsers: ({})
  property var terminals: ({})
  property var editors: ({})
  property string timezone: ""
  property var timezones: []
  property bool ntp: false
  property bool ntpAvailable: false
  property bool ntpSynchronized: false
  property string hostname: ""
  property string fullName: ""
  property string keyboardLayout: ""
  property var keyboardLayouts: []
  property string locale: ""
  property var locales: []
  property int parallelDownloads: 5
  property int hyprGapsIn: 5
  property int hyprGapsOut: 10
  property int hyprBorderSize: 2
  property int hyprRounding: 0
  property bool hyprBlur: false
  property bool hyprShadow: false
  property string hyprLayout: "dwindle"
  property real hyprColumnWidth: 0.49
  property bool hyprDimInactive: false
  property real hyprDimStrength: 0.15
  property bool hyprAnimations: true
  property bool hyprCursorHideOnKey: true
  property bool hyprCursorWarp: true
  property int hyprCursorSize: 24
  property bool hyprAllowTearing: false
  property bool hyprResizeOnBorder: false
  property real hyprActiveOpacity: 1
  property bool hyprPreserveSplit: false
  property bool hyprFocusOnActivate: false
  property bool hyprLookManaged: false
  property real hyprSensitivity: 0
  property string hyprAccelProfile: ""
  property bool hyprNaturalScroll: false
  property real hyprScrollFactor: 0.4
  property bool hyprClickfinger: true
  property bool hyprDisableWhileTyping: true
  property int hyprDrag3fg: 0
  property int hyprRepeatRate: 40
  property int hyprRepeatDelay: 250
  property bool hyprNumlock: true
  property int hyprFollowMouse: 1
  property bool hyprKeyPressDpms: true
  property bool hyprMouseMoveDpms: true
  property string hyprKbLayout: ""
  property string hyprKbVariant: ""
  property string hyprKbOptions: ""
  property bool hyprKbGroupToggle: false
  property bool hyprWorkspaceGesture: false
  property bool hyprInputManaged: false
  property bool hyprNoGaps: false
  property bool hyprSquareAspect: false
  property string hyprWorkspaceLayout: "dwindle"
  property bool fingerprintAvailable: false
  property bool fingerprintConfigured: false
  property bool fido2Configured: false
  property bool sshdEnabled: false
  property bool sshdActive: false
  property bool passwordlessSudo: false
  property bool sudolessDocker: false
  property string omarchyVersion: ""
  property string omarchyChannel: ""
  property bool updateAvailable: false
  property string updateSummary: ""
  property string atmosRevision: ""
  property string atmosChannel: "alpha"
  property bool atmosInstalled: false
  property bool atmosUpdateAvailable: false
  property string atmosUpdateSummary: ""
  property bool voxtypeInstalled: false
  property bool hybridGpuAvailable: false
  property string hybridGpuMode: ""
  property bool hwNvidia: false
  property bool hwNvidiaGsp: false
  property bool hwNvidiaWithoutGsp: false
  property bool hwVulkan: false
  property bool hwIntel: false
  property bool hwIntelPtl: false
  property bool hwWebcam: false
  property bool hwFramework16: false
  property bool hwAsusRog: false
  property bool hwSurface: false
  property string dmiVendor: ""
  property string dmiProduct: ""
  property string dmiFamily: ""
  property string cpuStat: ""
  property string memoryStat: ""
  property string cpuIdentity: ""
  property string gpuIdentity: ""
  property string npuIdentity: ""
  property bool tailscaleInstalled: false
  property bool tailscaleRunning: false
  property var plugins: []
  property int snapperNumberLimit: 5
  property bool snapperTimeline: false
  property bool fstrimEnabled: false
  property bool directBootAvailable: false
  property bool directBoot: false
  property string mimePdf: ""
  property string mimeImage: ""
  property string mimeVideo: ""
  property var mimePdfOptions: []
  property var mimeImageOptions: []
  property var mimeVideoOptions: []
  property string picturesDir: ""
  property string videosDir: ""
  property bool recordingActive: false
  property bool webcamOverlay: false
  property var services: ({})
  property var gaming: ({})
  property var extras: ({})
  property var hooks: []
  property var autostart: []
  property bool autostartManaged: false
  property var bindings: []
  property bool bindingsManaged: false
  property var windowRules: []
  property bool windowRulesManaged: false
  property var keybindings: []
  property string focusedClass: ""
  property bool cupsActive: false
  property bool printerSetup: false
  property string nightlightDay: "07:00"
  property string nightlightNight: "20:00"
  property bool nightlightNightOn: false
  property var tailscalePeers: []

  property var ioQueue: WorkQueue.createWorkQueue()
  property var snapshotData: ({})
  property var ioJob: null
  property bool snapshotReady: false

  function sanitizeDmi(raw) {
    var source = String(raw || "")
    if (source.indexOf("\n") !== -1 || source.indexOf("\r") !== -1) return ""
    var s = source.replace(/\s+/g, " ").replace(/^\s+|\s+$/g, "")
    if (!s || s.length > 160) return ""
    if (s.indexOf("..") !== -1) return ""
    if (s.charAt(0) === "-" || s.charAt(0) === "/") return ""
    var lower = s.toLowerCase()
    if (lower === "none" || lower === "default string" || lower === "unknown" || lower.indexOf("to be filled") !== -1)
      return ""
    return s
  }

  function adoptArray(cur, next) {
    if (!(next instanceof Array)) next = []
    if (JSON.stringify(cur) === JSON.stringify(next)) return cur
    return next
  }

  function applySnapshot(raw) {
    var parsed = SnapshotJs.parseSnapshot(raw)
    if (!parsed) {
      lastError = "Could not parse Omarchy snapshot"
      return
    }
    var data = SnapshotJs.mergeSnapshot(snapshotData, parsed)
    snapshotData = data
    if (!("barPosition" in parsed) && !("hardware" in parsed) && !("disks" in parsed)) {
      root.applyLookPatch(parsed)
      return
    }
    theme = String(data.theme || "")
    background = String(data.background || "")
    font = String(data.font || "")
    textSize = Number(data.textSize) || 12
    themes = adoptArray(themes, data.themes)
    extraThemes = adoptArray(extraThemes, data.extraThemes)
    desktopApps = adoptArray(desktopApps, data.desktopApps)
    tuiApps = adoptArray(tuiApps, data.tuiApps)
    webApps = adoptArray(webApps, data.webApps)
    fonts = adoptArray(fonts, data.fonts)
    barPosition = String(data.barPosition || "top")
    barTransparent = data.barTransparent === true
    barVisible = data.barVisible !== false
    clockFormat = String(data.clockFormat || "")
    clockFormatAlt = String(data.clockFormatAlt || "")
    clockWeekStart = String(data.clockWeekStart || "").toLowerCase()
    if (clockWeekStart !== "sunday" && clockWeekStart !== "monday" && clockWeekStart !== "tuesday" && clockWeekStart !== "wednesday" && clockWeekStart !== "thursday" && clockWeekStart !== "friday" && clockWeekStart !== "saturday")
      clockWeekStart = ""
    clockPresent = data.clockPresent === true
    clockBirthYear = Math.round(Number(data.clockBirthYear)) || 0
    if (clockBirthYear < 1) clockBirthYear = 0
    clockLifeExpectancy = Math.round(Number(data.clockLifeExpectancy)) || 0
    if (clockLifeExpectancy < 1 || clockLifeExpectancy > 150) clockLifeExpectancy = 0
    indicatorsPresent = data.indicatorsPresent === true
    indicatorsAlwaysShow = data.indicatorsAlwaysShow === true
    indicatorsItems = adoptArray(indicatorsItems, root.normalizedIndicatorItems(data.indicatorsItems))
    agentsPresent = data.agentsPresent === true
    agentsRefreshIntervalSec = Number(data.agentsRefreshIntervalSec) || 900
    if (agentsRefreshIntervalSec < 30) agentsRefreshIntervalSec = 900
    agentsSync = data.agentsSync === true
    agentsSyncDir = String(data.agentsSyncDir || "")
    agentsSyncFileName = String(data.agentsSyncFileName || "")
    agentsSyncDeviceId = String(data.agentsSyncDeviceId || "")
    spacerPresent = data.spacerPresent === true
    spacerSize = Math.round(Number(data.spacerSize))
    if (!isFinite(spacerSize) || spacerSize < 0) spacerSize = 12
    if (spacerSize > 64) spacerSize = 64
    trayPresent = data.trayPresent === true
    trayHidden = adoptArray(trayHidden, root.normalizedStringIds(data.trayHidden))
    trayPinned = adoptArray(trayPinned, root.normalizedStringIds(data.trayPinned))
    browser = String(data.browser || "")
    terminal = String(data.terminal || "")
    editor = String(data.editor || "")
    agent = String(data.agent || "")
    dns = String(data.dns || "")
    idleScreensaver = Number(data.idleScreensaver) || 0
    idleLock = Number(data.idleLock) || 0
    stayAwake = data.stayAwake === true
    nightlight = data.nightlight === true
    nightlightTemperature = Math.round(Number(data.nightlightTemperature)) || 0
    if (nightlightTemperature < 0) nightlightTemperature = 0
    screensaverEnabled = data.screensaverEnabled !== false
    screensaverBranded = data.screensaverBranded === true
    aboutBranded = data.aboutBranded === true
    bluetooth = data.bluetooth === true
    wifiConnected = data.wifiConnected === true
    wifiBand = String(data.wifiBand || "")
    wifiBandSelected = String(data.wifiBandSelected || "auto")
    wifiBands = adoptArray(wifiBands, data.wifiBands instanceof Array ? data.wifiBands : ["auto"])
    wifiIface = String(data.wifiIface || "")
    if (!/^[a-zA-Z0-9._-]+$/.test(wifiIface)) wifiIface = ""
    netKind = String(data.netKind || "disconnected")
    if (netKind !== "ethernet" && netKind !== "wifi") netKind = "disconnected"
    netIface = String(data.netIface || "")
    if (!/^[a-zA-Z0-9._-]+$/.test(netIface)) netIface = ""
    netSsid = String(data.netSsid || "")
    netSignal = String(data.netSignal || "")
    if (!/^[0-9]+$/.test(netSignal)) netSignal = ""
    netIp = String(data.netIp || "")
    if (!/^[0-9a-fA-F:.]+$/.test(netIp)) netIp = ""
    netSpeed = String(data.netSpeed || "")
    if (!/^[0-9]+$/.test(netSpeed)) netSpeed = ""
    wifiHw = data.wifiHw === true
    wifiRadio = data.wifiRadio === true
    wifiConnections = adoptArray(wifiConnections, data.wifiConnections)
    bluetoothDevices = adoptArray(bluetoothDevices, data.bluetoothDevices)
    audioSinks = adoptArray(audioSinks, data.audioSinks)
    audioSources = adoptArray(audioSources, data.audioSources)
    audioOutputVolume = Math.round(Number(data.audioOutputVolume)) || 0
    if (audioOutputVolume < 0) audioOutputVolume = 0
    if (audioOutputVolume > 100) audioOutputVolume = 100
    audioOutputMuted = data.audioOutputMuted === true
    audioInputVolume = Math.round(Number(data.audioInputVolume)) || 0
    if (audioInputVolume < 0) audioInputVolume = 0
    if (audioInputVolume > 100) audioInputVolume = 100
    audioInputMuted = data.audioInputMuted === true
    audioTuningMatch = data.audioTuningMatch === true
    audioTuningOn = data.audioTuningOn === true
    disks = adoptArray(disks, data.disks)
    hardware = HardwareJs.normalize(data.hardware)
    luksDevices = adoptArray(luksDevices, data.luksDevices)
    swapDevices = adoptArray(swapDevices, data.swapDevices)
    snapperPresent = data.snapperPresent === true
    snapperConfigs = adoptArray(snapperConfigs, data.snapperConfigs)
    snapshots = adoptArray(snapshots, data.snapshots)
    hibernationAvailable = data.hibernationAvailable === true
    hibernationSupported = data.hibernationSupported === true
    hibernationConfigured = data.hibernationConfigured === true
    audioSink = ""
    audioSource = ""
    var i
    for (i = 0; i < audioSinks.length; i++) {
      if (audioSinks[i] && audioSinks[i].default) {
        audioSink = String(audioSinks[i].name || "")
        break
      }
    }
    for (i = 0; i < audioSources.length; i++) {
      if (audioSources[i] && audioSources[i].default) {
        audioSource = String(audioSources[i].name || "")
        break
      }
    }
    suspendEnabled = data.suspendEnabled !== false
    powerProfile = String(data.powerProfile || "")
    powerProfileAc = String(data.powerProfileAc || "")
    powerProfileBattery = String(data.powerProfileBattery || "")
    powerProfiles = adoptArray(powerProfiles, data.powerProfiles)
    powerPresent = data.powerPresent === true
    powerShowPercentage = data.powerShowPercentage === true
    isLaptop = data.isLaptop === true
    batteryPresent = data.batteryPresent === true
    monitors = adoptArray(monitors, data.monitors)
    internalPresent = data.internalPresent === true
    internalEnabled = data.internalEnabled === true
    externalPresent = data.externalPresent === true
    mirroring = data.mirroring === true
    touchpadPresent = data.touchpadPresent === true
    touchpadEnabled = data.touchpadEnabled !== false
    touchscreenPresent = data.touchscreenPresent === true
    touchscreenEnabled = data.touchscreenEnabled !== false
    keyboardBacklightPresent = data.keyboardBacklightPresent === true
    keyboardBrightness = Math.round(Number(data.keyboardBrightness)) || 0
    if (keyboardBrightness < 0) keyboardBrightness = 0
    if (keyboardBrightness > 100) keyboardBrightness = 100
    crashCapture = data.crashCapture !== false
    doNotDisturb = data.doNotDisturb === true
    weatherLocation = String(data.weatherLocation || "")
    weatherCoords = String(data.weatherCoords || "")
    if (!/^-?[0-9]+(\.[0-9]+)?,-?[0-9]+(\.[0-9]+)?$/.test(weatherCoords)) weatherCoords = ""
    weatherAuto = data.weatherAuto !== false
    weatherPresent = data.weatherPresent === true
    weatherUnit = String(data.weatherUnit || "auto")
    if (weatherUnit !== "metric" && weatherUnit !== "imperial") weatherUnit = "auto"
    weatherRefreshMinutes = Number(data.weatherRefreshMinutes) || 15
    if (weatherRefreshMinutes < 1) weatherRefreshMinutes = 15
    reminderCount = Math.round(Number(data.reminderCount)) || 0
    if (reminderCount < 0) reminderCount = 0
    reminderActive = data.reminderActive === true
    reminders = adoptArray(reminders, data.reminders)
    plymouth = String(data.plymouth || "")
    plymouthThemes = adoptArray(plymouthThemes, data.plymouthThemes)
    hasAether = data.hasAether === true
    browsers = data.browsers || ({})
    terminals = data.terminals || ({})
    editors = data.editors || ({})
    timezone = String(data.timezone || "")
    if (!/^[A-Za-z0-9/_+-]+$/.test(timezone) || timezone.indexOf("..") !== -1) timezone = ""
    timezones = adoptArray(timezones, data.timezones)
    ntp = data.ntp === true
    ntpAvailable = data.ntpAvailable === true
    ntpSynchronized = data.ntpSynchronized === true
    hostname = String(data.hostname || "")
    if (!/^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)*$/.test(hostname) || hostname.length > 253)
      hostname = ""
    fullName = String(data.fullName || "")
    if (fullName.length > 256 || fullName.charAt(0) === "-" || /[:\n\r,]/.test(fullName))
      fullName = ""
    keyboardLayout = String(data.keyboardLayout || "")
    if (keyboardLayout.indexOf(",") !== -1) keyboardLayout = keyboardLayout.split(",")[0]
    if (!/^[a-z0-9]{1,8}$/.test(keyboardLayout)) keyboardLayout = ""
    keyboardLayouts = adoptArray(keyboardLayouts, data.keyboardLayouts)
    locale = String(data.locale || "")
    if (locale !== "C.UTF-8" && !/^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(locale))
      locale = ""
    locales = adoptArray(locales, data.locales)
    parallelDownloads = Math.round(Number(data.parallelDownloads)) || 5
    if (parallelDownloads < 1) parallelDownloads = 5
    if (parallelDownloads > 20) parallelDownloads = 20
    root.applyHyprLook(data.hyprLook)
    root.applyHyprInput(data.hyprInput)
    hyprLookManaged = data.hyprLookManaged === true
    hyprInputManaged = data.hyprInputManaged === true
    hyprWorkspaceGesture = data.hyprWorkspaceGesture === true
    hyprNoGaps = data.hyprNoGaps === true
    hyprSquareAspect = data.hyprSquareAspect === true
    hyprWorkspaceLayout = String(data.hyprWorkspaceLayout || "dwindle")
    if (hyprWorkspaceLayout !== "scrolling") hyprWorkspaceLayout = "dwindle"
    fingerprintAvailable = data.fingerprintAvailable === true
    fingerprintConfigured = data.fingerprintConfigured === true
    fido2Configured = data.fido2Configured === true
    sshdEnabled = data.sshdEnabled === true
    sshdActive = data.sshdActive === true
    passwordlessSudo = data.passwordlessSudo === true
    sudolessDocker = data.sudolessDocker === true
    omarchyVersion = String(data.omarchyVersion || "")
    omarchyChannel = String(data.omarchyChannel || "")
    if (omarchyChannel !== "stable" && omarchyChannel !== "rc" && omarchyChannel !== "edge" && omarchyChannel !== "dev")
      omarchyChannel = ""
    updateAvailable = data.updateAvailable === true
    updateSummary = String(data.updateSummary || "")
    atmosRevision = String(data.atmosRevision || "")
    if (!/^[0-9a-f]{4,40}$/.test(atmosRevision)) atmosRevision = ""
    atmosChannel = AtmosUpdate.parseChannel(data.atmosChannel)
    if (!atmosChannel) atmosChannel = "alpha"
    atmosInstalled = data.atmosInstalled === true
    voxtypeInstalled = data.voxtypeInstalled === true
    hybridGpuAvailable = data.hybridGpuAvailable === true
    hybridGpuMode = String(data.hybridGpuMode || "")
    if (hybridGpuMode !== "Integrated" && hybridGpuMode !== "Hybrid") hybridGpuMode = ""
    hwNvidia = data.hwNvidia === true
    hwNvidiaGsp = data.hwNvidiaGsp === true
    hwNvidiaWithoutGsp = data.hwNvidiaWithoutGsp === true
    hwVulkan = data.hwVulkan === true
    hwIntel = data.hwIntel === true
    hwIntelPtl = data.hwIntelPtl === true
    hwWebcam = data.hwWebcam === true
    hwFramework16 = data.hwFramework16 === true
    hwAsusRog = data.hwAsusRog === true
    hwSurface = data.hwSurface === true
    dmiVendor = root.sanitizeDmi(data.dmiVendor)
    dmiProduct = root.sanitizeDmi(data.dmiProduct)
    dmiFamily = root.sanitizeDmi(data.dmiFamily)
    cpuStat = String(data.cpuStat || "").replace(/^\s+|\s+$/g, "")
    memoryStat = String(data.memoryStat || "").replace(/^\s+|\s+$/g, "")
    cpuIdentity = root.sanitizeDmi(data.cpuIdentity)
    gpuIdentity = root.sanitizeDmi(data.gpuIdentity)
    npuIdentity = root.sanitizeDmi(data.npuIdentity)
    tailscaleInstalled = data.tailscaleInstalled === true
    tailscaleRunning = data.tailscaleRunning === true
    plugins = adoptArray(plugins, data.plugins)
    snapperNumberLimit = Math.round(Number(data.snapperNumberLimit)) || 5
    if (snapperNumberLimit < 1) snapperNumberLimit = 5
    if (snapperNumberLimit > 50) snapperNumberLimit = 50
    snapperTimeline = data.snapperTimeline === true
    fstrimEnabled = data.fstrimEnabled === true
    directBootAvailable = data.directBootAvailable === true
    directBoot = data.directBoot === true
    mimePdf = String(data.mimePdf || "")
    mimeImage = String(data.mimeImage || "")
    mimeVideo = String(data.mimeVideo || "")
    if (!/^[A-Za-z0-9._-]+\.desktop$/.test(mimePdf)) mimePdf = ""
    if (!/^[A-Za-z0-9._-]+\.desktop$/.test(mimeImage)) mimeImage = ""
    if (!/^[A-Za-z0-9._-]+\.desktop$/.test(mimeVideo)) mimeVideo = ""
    mimePdfOptions = adoptArray(mimePdfOptions, data.mimePdfOptions)
    mimeImageOptions = adoptArray(mimeImageOptions, data.mimeImageOptions)
    mimeVideoOptions = adoptArray(mimeVideoOptions, data.mimeVideoOptions)
    picturesDir = String(data.picturesDir || "")
    videosDir = String(data.videosDir || "")
    recordingActive = data.recordingActive === true
    webcamOverlay = data.webcamOverlay === true
    services = data.services || ({})
    gaming = data.gaming || ({})
    extras = data.extras || ({})
    hooks = adoptArray(hooks, data.hooks)
    autostart = adoptArray(autostart, data.autostart)
    autostartManaged = data.autostartManaged === true
    bindings = adoptArray(bindings, data.bindings)
    bindingsManaged = data.bindingsManaged === true
    windowRules = adoptArray(windowRules, data.windowRules)
    windowRulesManaged = data.windowRulesManaged === true
    keybindings = adoptArray(keybindings, data.keybindings)
    focusedClass = String(data.focusedClass || "")
    cupsActive = data.cupsActive === true
    printerSetup = data.printerSetup === true
    nightlightDay = String(data.nightlightDay || "07:00")
    if (!/^[0-2]?\d:[0-5]\d$/.test(nightlightDay)) nightlightDay = "07:00"
    nightlightNight = String(data.nightlightNight || "20:00")
    if (!/^[0-2]?\d:[0-5]\d$/.test(nightlightNight)) nightlightNight = "20:00"
    nightlightNightOn = data.nightlightNightOn === true
    tailscalePeers = adoptArray(tailscalePeers, data.tailscalePeers)
    var opts = String(data.hyprInput && data.hyprInput.kbOptions || "")
    hyprKbGroupToggle = opts.indexOf("grp:alts_toggle") !== -1
  }

  function applyLookPatch(parsed) {
    if ("theme" in parsed) theme = String(parsed.theme || "")
    if ("background" in parsed) background = String(parsed.background || "")
    if ("font" in parsed) font = String(parsed.font || "")
    if ("textSize" in parsed) {
      textSize = Number(parsed.textSize) || 12
    }
    if ("themes" in parsed) themes = adoptArray(themes, parsed.themes)
    if ("extraThemes" in parsed) extraThemes = adoptArray(extraThemes, parsed.extraThemes)
    if ("fonts" in parsed) fonts = adoptArray(fonts, parsed.fonts)
    if ("stayAwake" in parsed) stayAwake = parsed.stayAwake === true
    if ("nightlight" in parsed) nightlight = parsed.nightlight === true
    if ("nightlightTemperature" in parsed) {
      nightlightTemperature = Math.round(Number(parsed.nightlightTemperature)) || 0
      if (nightlightTemperature < 0) nightlightTemperature = 0
    }
    if ("screensaverBranded" in parsed) screensaverBranded = parsed.screensaverBranded === true
    if ("aboutBranded" in parsed) aboutBranded = parsed.aboutBranded === true
    if ("plymouth" in parsed) plymouth = String(parsed.plymouth || "")
    if ("plymouthThemes" in parsed) plymouthThemes = adoptArray(plymouthThemes, parsed.plymouthThemes)
    if ("nightlightDay" in parsed) {
      nightlightDay = String(parsed.nightlightDay || "07:00")
      if (!/^[0-2]?\d:[0-5]\d$/.test(nightlightDay)) nightlightDay = "07:00"
    }
    if ("nightlightNight" in parsed) {
      nightlightNight = String(parsed.nightlightNight || "20:00")
      if (!/^[0-2]?\d:[0-5]\d$/.test(nightlightNight)) nightlightNight = "20:00"
    }
    if ("nightlightNightOn" in parsed) nightlightNightOn = parsed.nightlightNightOn === true
  }

  function refresh() {
    scheduleRefresh()
  }

  function scheduleRefresh() {
    refreshTimer.restart()
  }

  function enqueueRead(group) {
    WorkQueue.enqueueRead(ioQueue, group || "all")
    kickIo()
  }

  function startSession(hub) {
    var first = WorkQueue.snapshotGroupForHub(hub)
    WorkQueue.enqueueRead(ioQueue, first)
    if (first === "look") WorkQueue.enqueueRead(ioQueue, "all")
    kickIo()
  }

  function kickIo() {
    if (ioQueue.running) return
    var job = WorkQueue.takeNext(ioQueue)
    if (!job) return
    ioJob = job
    startIoJob(job)
  }

  function ioFinished() {
    ioJob = null
    WorkQueue.release(ioQueue)
    kickIo()
  }

  function startIoJob(job) {
    if (job.kind === "read") {
      snapshotProc.command = ["bash", root.snapshotScript, job.group || "all"]
      snapshotProc.running = true
      return
    }
    if (job.kind === "job") {
      lastError = ""
      jobLog = ""
      jobKind = String(job.jobKind || "")
      jobStdin = String(job.stdin || "")
      jobBusy = true
      jobProc.stdinEnabled = jobStdin.length > 0
      jobProc.command = job.argv
      jobProc.running = true
      return
    }
    lastError = ""
    mutProc.command = job.argv
    mutProc.running = true
  }

  function startSnapshot() {
    enqueueRead("all")
  }

  function runCommand(argv) {
    if (!(argv instanceof Array) || argv.length === 0) return
    WorkQueue.enqueueWrite(ioQueue, { kind: "mut", argv: argv })
    kickIo()
  }

  function applyHyprLook(raw) {
    var look = raw && typeof raw === "object" ? raw : {}
    hyprGapsIn = Math.round(Number(look.gapsIn))
    if (!isFinite(hyprGapsIn) || hyprGapsIn < 0) hyprGapsIn = 5
    if (hyprGapsIn > 64) hyprGapsIn = 64
    hyprGapsOut = Math.round(Number(look.gapsOut))
    if (!isFinite(hyprGapsOut) || hyprGapsOut < 0) hyprGapsOut = 10
    if (hyprGapsOut > 64) hyprGapsOut = 64
    hyprBorderSize = Math.round(Number(look.borderSize))
    if (!isFinite(hyprBorderSize) || hyprBorderSize < 0) hyprBorderSize = 2
    if (hyprBorderSize > 16) hyprBorderSize = 16
    hyprRounding = Math.round(Number(look.rounding))
    if (!isFinite(hyprRounding) || hyprRounding < 0) hyprRounding = 0
    if (hyprRounding > 32) hyprRounding = 32
    hyprBlur = look.blur === true
    hyprShadow = look.shadow === true
    hyprLayout = String(look.layout || "dwindle")
    if (hyprLayout !== "scrolling") hyprLayout = "dwindle"
    hyprColumnWidth = Number(look.columnWidth)
    if (!isFinite(hyprColumnWidth)) hyprColumnWidth = 0.49
    if (hyprColumnWidth < 0.2) hyprColumnWidth = 0.2
    if (hyprColumnWidth > 1) hyprColumnWidth = 1
    hyprDimInactive = look.dimInactive === true
    hyprDimStrength = Number(look.dimStrength)
    if (!isFinite(hyprDimStrength)) hyprDimStrength = 0.15
    if (hyprDimStrength < 0) hyprDimStrength = 0
    if (hyprDimStrength > 1) hyprDimStrength = 1
    hyprAnimations = look.animations !== false
    hyprCursorHideOnKey = look.cursorHideOnKey !== false
    hyprCursorWarp = look.cursorWarp !== false
    hyprCursorSize = Math.round(Number(look.cursorSize)) || 24
    if (hyprCursorSize < 8) hyprCursorSize = 8
    if (hyprCursorSize > 64) hyprCursorSize = 64
    hyprAllowTearing = look.allowTearing === true
    hyprResizeOnBorder = look.resizeOnBorder === true
    hyprActiveOpacity = Number(look.activeOpacity)
    if (!isFinite(hyprActiveOpacity)) hyprActiveOpacity = 1
    if (hyprActiveOpacity < 0.2) hyprActiveOpacity = 0.2
    if (hyprActiveOpacity > 1) hyprActiveOpacity = 1
    hyprPreserveSplit = look.preserveSplit === true
    hyprFocusOnActivate = look.focusOnActivate === true
  }

  function applyHyprInput(raw) {
    var input = raw && typeof raw === "object" ? raw : {}
    hyprSensitivity = Number(input.sensitivity)
    if (!isFinite(hyprSensitivity)) hyprSensitivity = 0
    if (hyprSensitivity < -1) hyprSensitivity = -1
    if (hyprSensitivity > 1) hyprSensitivity = 1
    hyprAccelProfile = String(input.accelProfile || "")
    if (hyprAccelProfile !== "flat" && hyprAccelProfile !== "adaptive") hyprAccelProfile = ""
    hyprNaturalScroll = input.naturalScroll === true
    hyprScrollFactor = Number(input.scrollFactor)
    if (!isFinite(hyprScrollFactor)) hyprScrollFactor = 0.4
    if (hyprScrollFactor < 0.1) hyprScrollFactor = 0.1
    if (hyprScrollFactor > 3) hyprScrollFactor = 3
    hyprClickfinger = input.clickfinger !== false
    hyprDisableWhileTyping = input.disableWhileTyping !== false
    hyprDrag3fg = Math.round(Number(input.drag3fg)) || 0
    if (hyprDrag3fg !== 1) hyprDrag3fg = 0
    hyprRepeatRate = Math.round(Number(input.repeatRate)) || 40
    if (hyprRepeatRate < 10) hyprRepeatRate = 10
    if (hyprRepeatRate > 100) hyprRepeatRate = 100
    hyprRepeatDelay = Math.round(Number(input.repeatDelay)) || 250
    if (hyprRepeatDelay < 100) hyprRepeatDelay = 100
    if (hyprRepeatDelay > 1000) hyprRepeatDelay = 1000
    hyprNumlock = input.numlock !== false
    hyprFollowMouse = Math.round(Number(input.followMouse))
    if (!isFinite(hyprFollowMouse) || hyprFollowMouse < 0 || hyprFollowMouse > 3) hyprFollowMouse = 1
    hyprKeyPressDpms = input.keyPressDpms !== false
    hyprMouseMoveDpms = input.mouseMoveDpms !== false
    hyprKbLayout = String(input.kbLayout || "")
    hyprKbVariant = String(input.kbVariant || "")
    hyprKbOptions = String(input.kbOptions || "")
  }

  function lookPayload() {
    return JSON.stringify({
      gapsIn: hyprGapsIn,
      gapsOut: hyprGapsOut,
      borderSize: hyprBorderSize,
      rounding: hyprRounding,
      blur: hyprBlur,
      shadow: hyprShadow,
      layout: hyprLayout,
      columnWidth: hyprColumnWidth,
      dimInactive: hyprDimInactive,
      dimStrength: hyprDimStrength,
      animations: hyprAnimations,
      cursorHideOnKey: hyprCursorHideOnKey,
      cursorWarp: hyprCursorWarp,
      cursorSize: hyprCursorSize,
      allowTearing: hyprAllowTearing,
      resizeOnBorder: hyprResizeOnBorder,
      activeOpacity: hyprActiveOpacity,
      preserveSplit: hyprPreserveSplit,
      focusOnActivate: hyprFocusOnActivate
    })
  }

  function inputPayload() {
    return JSON.stringify({
      sensitivity: hyprSensitivity,
      accelProfile: hyprAccelProfile,
      naturalScroll: hyprNaturalScroll,
      scrollFactor: hyprScrollFactor,
      clickfinger: hyprClickfinger,
      disableWhileTyping: hyprDisableWhileTyping,
      drag3fg: hyprDrag3fg,
      repeatRate: hyprRepeatRate,
      repeatDelay: hyprRepeatDelay,
      numlock: hyprNumlock,
      followMouse: hyprFollowMouse,
      keyPressDpms: hyprKeyPressDpms,
      mouseMoveDpms: hyprMouseMoveDpms,
      kbLayoutOverride: hyprKbLayout,
      kbVariantOverride: hyprKbLayout ? hyprKbVariant : "",
      kbGroupToggle: hyprKbGroupToggle,
      workspaceGesture: hyprWorkspaceGesture
    })
  }

  function writeHyprLook() {
    hyprLookManaged = true
    runCommand(["bash", setHyprLookScript, lookPayload()])
  }

  function writeHyprInput() {
    hyprInputManaged = true
    runCommand(["bash", setHyprInputScript, inputPayload()])
  }

  function runGumJob(argv, kind) {
    if (!(argv instanceof Array) || argv.length === 0) return
    var cmd = ["bash", "-c", "PATH=\"$1:$PATH\" exec \"$@\"", "prefs-job", gumStubDir]
    for (var i = 0; i < argv.length; i++) cmd.push(argv[i])
    runJob(cmd, "", kind)
  }

  function runJob(argv, stdinText, kind) {
    if (!(argv instanceof Array) || argv.length === 0) return
    WorkQueue.enqueueWrite(ioQueue, {
      kind: "job",
      argv: argv,
      stdin: String(stdinText || ""),
      jobKind: String(kind || "")
    })
    kickIo()
  }

  function cancelJob() {
    if (!jobProc.running) return
    jobProc.running = false
  }

  function stderrLooksLikeFailure(text) {
    var raw = String(text || "")
    var lines = raw.split("\n")
    for (var i = 0; i < lines.length; i++) {
      var line = lines[i].replace(/^\s+|\s+$/g, "")
      if (!line) continue
      if (line.indexOf("warn: wayland.") === 0) continue
      if (line.indexOf("warn: terminal.") === 0) continue
      if (line.indexOf("xdg-toplevel-icon") !== -1) continue
      if (line.indexOf("slave exited with signal") !== -1) continue
      return true
    }
    return false
  }

  function pump() {
    kickIo()
  }

  function setTheme(name) {
    name = String(name || "")
    if (!name || name === theme) return
    theme = name
    runCommand(["omarchy", "theme", "set", name])
  }
  function openThemeSwitcher() {
    runCommand(["bash", "-c", "theme=$(omarchy theme switcher || true); [[ -n $theme ]] && omarchy theme set \"$theme\""])
  }
  function refreshTheme() { runCommand(["omarchy", "theme", "refresh"]) }
  function openThemeFolder() {
    if (!theme) return
    runCommand(["bash", "-c", "dir=$(omarchy theme dir \"$1\") && [[ -d \"$dir\" ]] && xdg-open \"$dir\" >/dev/null 2>&1 &", "theme-dir", theme])
  }
  function installTheme(url) {
    url = String(url || "").replace(/^\s+|\s+$/g, "")
    if (!url) return
    runJob(["omarchy", "theme", "install", url], "", "theme-install")
  }
  function updateThemes() {
    runJob(["omarchy", "theme", "update"], "", "theme-update")
  }
  function removeTheme(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name.indexOf("/") !== -1 || name.indexOf(".") === 0) return
    runCommand(["omarchy", "theme", "remove", name])
  }
  function setBackgroundPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/") return
    background = path
    runCommand(["omarchy", "theme", "bg", "set", path])
  }
  function nextBackground() { runCommand(["omarchy", "theme", "bg", "next"]) }
  function openBackgroundSwitcher() { runCommand(["omarchy", "theme", "bg-switcher"]) }
  function setBackgroundFromFile() {
    runCommand(["bash", "-c", "path=$(omarchy file select --title \"Set background\" --extensions \"jpg jpeg png gif webp bmp\" || true); [[ -n $path ]] && omarchy theme bg set \"$path\""])
  }
  function openBackgroundFolder() { runCommand(["omarchy", "theme", "bg", "install"]) }
  function cacheBackgrounds() { runCommand(["omarchy", "theme", "bg", "cache"]) }
  function setFont(name) {
    name = String(name || "")
    if (!name || name === font) return
    font = name
    runCommand(["omarchy", "font", "set", name])
  }
  function setTextSize(size) {
    size = Math.round(Number(size))
    if (!isFinite(size) || size === textSize) return
    textSize = size
    runCommand(["omarchy", "display", "text", "size", String(size)])
  }
  function resetTextSize() {
    textSize = 12
    runCommand(["omarchy", "display", "text", "size", "reset"])
  }
  function setMonitorScale(scale) {
    scale = String(scale || "")
    if (!scale) return
    runCommand(["omarchy", "hyprland", "monitor", "scaling", scale])
  }
  function setDisplayBrightness(name, percent) {
    name = String(name || "")
    percent = Math.round(Number(percent))
    if (!/^[A-Za-z0-9._-]+$/.test(name)) return
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    var list = monitors instanceof Array ? monitors.slice() : []
    for (var i = 0; i < list.length; i++) {
      if (!list[i] || list[i].name !== name) continue
      var row = {}
      for (var k in list[i]) row[k] = list[i][k]
      row.brightness = percent
      list[i] = row
      monitors = list
      break
    }
    runCommand(["omarchy", "brightness", "display", "--no-osd", "--monitor", name, percent + "%"])
  }
  function setInternalDisplay(on) {
    if (on === internalEnabled) return
    internalEnabled = on
    runCommand(["omarchy", "hyprland", "monitor", "internal", on ? "on" : "off"])
  }
  function setInternalMirror(on) {
    if (on === mirroring) return
    mirroring = on
    runCommand(["omarchy", "hyprland", "monitor", "internal", "mirror", on ? "on" : "off"])
  }
  function setTouchpad(on) {
    if (on === touchpadEnabled) return
    touchpadEnabled = on
    runCommand(["omarchy", "toggle", "touchpad", on ? "on" : "off"])
  }
  function setTouchscreen(on) {
    if (on === touchscreenEnabled) return
    touchscreenEnabled = on
    runCommand(["omarchy", "toggle", "touchscreen", on ? "on" : "off"])
  }
  function adjustKeyboardBacklight(direction) {
    if (direction !== "up" && direction !== "down" && direction !== "off" && direction !== "restore") return
    runCommand(["omarchy", "brightness", "keyboard", "--no-osd", direction])
  }
  function setBarPosition(position) {
    if (!position || position === barPosition) return
    barPosition = position
    runCommand(["omarchy", "bar", "position", position])
  }
  function setBarTransparent(on) {
    if (on === barTransparent) return
    barTransparent = on
    runCommand(["omarchy", "bar", "transparent", on ? "true" : "false"])
  }
  // `omarchy toggle bar on` sets the bar-off flag and hides the bar.
  function setBarVisible(on) {
    if (on === barVisible) return
    barVisible = on
    runCommand(["omarchy", "toggle", "bar", on ? "off" : "on"])
  }
  function setClockFormat(fmt) {
    if (!fmt || fmt === clockFormat) return
    clockFormat = fmt
    var key = (barPosition === "left" || barPosition === "right") ? "verticalFormat" : "format"
    runCommand(["omarchy", "bar", "set", "omarchy.clock", key, fmt])
  }
  function setClockFormatAlt(fmt) {
    if (!fmt || fmt === clockFormatAlt) return
    clockFormatAlt = fmt
    var key = (barPosition === "left" || barPosition === "right") ? "verticalFormatAlt" : "formatAlt"
    runCommand(["omarchy", "bar", "set", "omarchy.clock", key, fmt])
  }
  function setClockWeekStart(day) {
    day = String(day || "").toLowerCase()
    if (day !== "sunday" && day !== "monday" && day !== "tuesday" && day !== "wednesday" && day !== "thursday" && day !== "friday" && day !== "saturday") return
    if (day === clockWeekStart) return
    clockWeekStart = day
    runCommand(["omarchy", "bar", "set", "omarchy.clock", "weekStartDay", day])
  }
  function setClockBirthYear(year) {
    if (typeof year === "number") year = String(Math.round(year))
    year = String(year || "").replace(/^\s+|\s+$/g, "")
    if (year.length === 0 || year === "0") {
      if (clockBirthYear === 0) return
      clockBirthYear = 0
      runCommand(["omarchy", "bar", "set", "omarchy.clock", "birthYear", "0", "--json"])
      return
    }
    if (!/^\d{4}$/.test(year)) return
    var born = parseInt(year, 10)
    var now = new Date().getFullYear()
    if (!(born >= now - 120 && born <= now)) return
    if (born === clockBirthYear) return
    clockBirthYear = born
    runCommand(["omarchy", "bar", "set", "omarchy.clock", "birthYear", String(born), "--json"])
  }
  function setClockLifeExpectancy(years) {
    if (typeof years === "number") years = String(Math.round(years))
    years = String(years || "").replace(/^\s+|\s+$/g, "")
    if (years.length === 0 || years === "0") {
      if (clockLifeExpectancy === 0) return
      clockLifeExpectancy = 0
      runCommand(["omarchy", "bar", "set", "omarchy.clock", "lifeExpectancy", "0", "--json"])
      return
    }
    if (!/^\d+$/.test(years)) return
    var span = parseInt(years, 10)
    if (!(span >= 1 && span <= 150)) return
    if (span === clockLifeExpectancy) return
    clockLifeExpectancy = span
    runCommand(["omarchy", "bar", "set", "omarchy.clock", "lifeExpectancy", String(span), "--json"])
  }
  function setIndicatorsAlwaysShow(on) {
    if (on === indicatorsAlwaysShow) return
    indicatorsAlwaysShow = on
    runCommand(["omarchy", "bar", "set", "omarchy.indicators", "alwaysShow", on ? "true" : "false", "--json"])
  }
  function indicatorIds() {
    return ["Dictation", "ScreenRecording", "Reminder", "NightLight", "Dnd", "StayAwake"]
  }
  function normalizedIndicatorItems(list) {
    var all = indicatorIds()
    var next = []
    if (list instanceof Array) {
      for (var i = 0; i < all.length; i++) {
        if (list.indexOf(all[i]) !== -1) next.push(all[i])
      }
    }
    return next
  }
  function setIndicatorsItems(list) {
    var next = normalizedIndicatorItems(list)
    if (next.length === indicatorIds().length) next = []
    var current = indicatorsItems instanceof Array ? indicatorsItems : []
    if (JSON.stringify(next) === JSON.stringify(current)) return
    indicatorsItems = next
    runCommand(["bash", setBarWidgetScript, "omarchy.indicators", "items", JSON.stringify(next)])
  }
  function setAgentsRefreshIntervalSec(seconds) {
    seconds = Math.round(Number(seconds))
    if (!(seconds >= 30) || seconds === agentsRefreshIntervalSec) return
    agentsRefreshIntervalSec = seconds
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "refreshIntervalSec", String(seconds), "--json"])
  }
  function setAgentsSync(on) {
    if (on === agentsSync) return
    agentsSync = on
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncMode", on ? "On" : "Off"])
  }
  function setAgentsSyncDir(path) {
    path = String(path || "").replace(/^\s+|\s+$/g, "")
    if (path === agentsSyncDir) return
    agentsSyncDir = path
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncDir", path])
  }
  function setAgentsSyncFileName(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "").split("/").pop()
    if (name === agentsSyncFileName) return
    agentsSyncFileName = name
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncFileName", name])
  }
  function setAgentsSyncDeviceId(id) {
    id = String(id || "").replace(/^\s+|\s+$/g, "")
    if (id === agentsSyncDeviceId) return
    agentsSyncDeviceId = id
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncDeviceId", id])
  }
  function setSpacerSize(size) {
    size = Math.round(Number(size))
    if (!isFinite(size) || size < 0 || size > 64 || size === spacerSize) return
    spacerSize = size
    runCommand(["omarchy", "bar", "set", "omarchy.spacer", "size", String(size), "--json"])
  }
  function addSpacer() {
    if (spacerPresent) return
    spacerPresent = true
    runCommand(["omarchy", "bar", "put", "omarchy.spacer"])
  }
  function removeSpacer() {
    if (!spacerPresent) return
    spacerPresent = false
    runCommand(["omarchy", "plugin", "disable", "omarchy.spacer"])
  }
  function installDesktopApp(name, command, icon) {
    name = String(name || "")
    command = String(command || "")
    icon = String(icon || "application-x-executable")
    if (!name || !command) return
    if (name.indexOf("/") !== -1 || name.charAt(0) === "-") return
    runJob(["bash", addDesktopLauncherScript, name, command, icon], "", "desktop-install")
  }
  function installTui(name, command, style, icon) {
    name = String(name || "")
    command = String(command || "")
    style = String(style || "tile")
    icon = String(icon || "utilities-terminal")
    if (!name || !command || !icon) return
    if (style !== "float" && style !== "tile") return
    if (name.indexOf("/") !== -1 || name.charAt(0) === "-") return
    runJob(["omarchy", "tui", "install", name, command, style, icon], "", "tui-install")
  }
  function installWebApp(name, url, icon) {
    name = String(name || "")
    url = String(url || "")
    icon = String(icon || "")
    if (!name || !url) return
    if (name.indexOf("/") !== -1 || name.charAt(0) === "-") return
    runJob(["omarchy", "webapp", "install", name, url, icon], "", "webapp-install")
  }
  function removeDesktopApp(id, name) {
    id = String(id || "")
    name = String(name || id)
    if (!id) return
    runCommand(["omarchy", "remove", "launcher", "entry", id, name])
  }
  function removeTui(name) {
    name = String(name || "")
    if (!name) return
    runCommand(["omarchy", "tui", "remove", name])
  }
  function removeWebApp(name) {
    name = String(name || "")
    if (!name) return
    runCommand(["omarchy", "webapp", "remove", name])
  }
  function normalizedStringIds(list) {
    var next = []
    if (list instanceof Array) {
      for (var i = 0; i < list.length; i++) {
        var id = String(list[i] || "")
        if (id.length === 0 || next.indexOf(id) !== -1) continue
        next.push(id)
      }
    }
    return next
  }
  function setTrayHidden(list) {
    var next = normalizedStringIds(list)
    var current = trayHidden instanceof Array ? trayHidden : []
    if (JSON.stringify(next) === JSON.stringify(current)) return
    trayHidden = next
    runCommand(["bash", setBarWidgetScript, "omarchy.tray", "hidden", JSON.stringify(next)])
  }
  function clearTrayHidden() {
    setTrayHidden([])
  }
  function setTrayPinned(list) {
    var next = normalizedStringIds(list)
    var current = trayPinned instanceof Array ? trayPinned : []
    if (JSON.stringify(next) === JSON.stringify(current)) return
    trayPinned = next
    runCommand(["bash", setBarWidgetScript, "omarchy.tray", "pinned", JSON.stringify(next)])
  }
  function clearTrayPinned() {
    setTrayPinned([])
  }
  function setBrowser(name) {
    if (!name || name === browser) return
    browser = name
    runCommand(["omarchy", "default", "browser", name])
  }
  function setTerminal(name) {
    if (!name || name === terminal) return
    terminal = name
    runCommand(["omarchy", "default", "terminal", name])
  }
  function setEditor(name) {
    if (!name || name === editor) return
    editor = name
    runCommand(["omarchy", "default", "editor", name])
  }
  function setAgent(name) {
    if (!name || name === agent) return
    agent = name
    runCommand(["omarchy", "default", "agent", name])
  }
  function setDns(name) {
    if (name !== "Cloudflare" && name !== "Google" && name !== "DHCP") return
    if (name === dns) return
    dns = name
    runCommand(["omarchy", "dns", name])
  }
  function setCustomDns(servers) {
    servers = String(servers || "").replace(/^\s+|\s+$/g, "")
    if (!servers) return
    dns = "Custom"
    runJob(["bash", setDnsCustomScript, servers], "", "dns-custom")
  }
  function openAether() { runCommand(["aether"]) }

  function setIdle(screensaver, lock) {
    idleScreensaver = Math.round(Number(screensaver)) || 0
    idleLock = Math.round(Number(lock)) || 0
    runCommand(["bash", setIdleScript, String(screensaver), String(lock)])
  }

  function setStayAwake(on) {
    if (on === stayAwake) return
    stayAwake = on
    runCommand(["omarchy", "toggle", "idle", on ? "stay-awake" : "allow-idle"])
  }

  function setNightlight(on) {
    if (on === nightlight) return
    nightlight = on
    runCommand(["omarchy", "toggle", "nightlight"])
  }

  function setScreensaverEnabled(on) {
    if (on === screensaverEnabled) return
    screensaverEnabled = on
    runCommand(["omarchy", "toggle", "screensaver-off", on ? "off" : "on"])
  }

  function setScreensaverBranding(action) {
    if (action !== "image" && action !== "text" && action !== "reset") return
    runCommand(["omarchy", "branding", "screensaver", action])
  }

  function setAboutBranding(action) {
    if (action !== "image" && action !== "text" && action !== "reset") return
    runCommand(["omarchy", "branding", "about", action])
  }

  function setTimezone(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === timezone) return
    if (!/^[A-Za-z0-9/_+-]+$/.test(name) || name.indexOf("..") !== -1) return
    timezone = name
    runCommand(["bash", setTimezoneScript, name])
  }

  function setNtp(on) {
    if (on === ntp) return
    ntp = on
    if (!on) ntpSynchronized = false
    runCommand(["bash", setNtpScript, on ? "true" : "false"])
  }

  function setHostname(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === hostname) return
    if (name.length > 253) return
    if (!/^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)*$/.test(name)) return
    hostname = name
    runCommand(["bash", setHostnameScript, name])
  }

  function setFullName(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (name === fullName) return
    if (name.length > 256) return
    if (name.charAt(0) === "-") return
    if (/[:\n\r,]/.test(name)) return
    fullName = name
    runCommand(["bash", setFullNameScript, name])
  }

  function setKeyboardLayout(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (name.indexOf(",") !== -1) name = name.split(",")[0]
    if (!name || name === keyboardLayout) return
    if (!/^[a-z0-9]{1,8}$/.test(name)) return
    keyboardLayout = name
    runCommand(["bash", setKeyboardLayoutScript, name])
  }

  function setLocale(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === locale) return
    if (name !== "C.UTF-8" && !/^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(name)) return
    locale = name
    runCommand(["bash", setLocaleScript, name])
  }

  function setParallelDownloads(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 1 || n > 20 || n === parallelDownloads) return
    parallelDownloads = n
    runCommand(["bash", setParallelDownloadsScript, String(n)])
  }

  function setHyprGapsIn(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 64 || n === hyprGapsIn) return
    hyprGapsIn = n
    writeHyprLook()
  }
  function setHyprGapsOut(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 64 || n === hyprGapsOut) return
    hyprGapsOut = n
    writeHyprLook()
  }
  function setHyprBorderSize(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 16 || n === hyprBorderSize) return
    hyprBorderSize = n
    writeHyprLook()
  }
  function setHyprRounding(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 32 || n === hyprRounding) return
    hyprRounding = n
    writeHyprLook()
  }
  function setHyprBlur(on) {
    if (on === hyprBlur) return
    hyprBlur = on
    writeHyprLook()
  }
  function setHyprShadow(on) {
    if (on === hyprShadow) return
    hyprShadow = on
    writeHyprLook()
  }
  function setHyprLayout(name) {
    if (name !== "dwindle" && name !== "scrolling") return
    if (name === hyprLayout) return
    hyprLayout = name
    writeHyprLook()
  }
  function setHyprColumnWidth(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0.2 || n > 1 || n === hyprColumnWidth) return
    hyprColumnWidth = n
    writeHyprLook()
  }
  function setHyprDimInactive(on) {
    if (on === hyprDimInactive) return
    hyprDimInactive = on
    writeHyprLook()
  }
  function setHyprDimStrength(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0 || n > 1 || n === hyprDimStrength) return
    hyprDimStrength = n
    writeHyprLook()
  }
  function setHyprAnimations(on) {
    if (on === hyprAnimations) return
    hyprAnimations = on
    writeHyprLook()
  }
  function setHyprCursorHideOnKey(on) {
    if (on === hyprCursorHideOnKey) return
    hyprCursorHideOnKey = on
    writeHyprLook()
  }
  function setHyprCursorWarp(on) {
    if (on === hyprCursorWarp) return
    hyprCursorWarp = on
    writeHyprLook()
  }
  function setHyprAllowTearing(on) {
    if (on === hyprAllowTearing) return
    hyprAllowTearing = on
    writeHyprLook()
  }
  function setHyprResizeOnBorder(on) {
    if (on === hyprResizeOnBorder) return
    hyprResizeOnBorder = on
    writeHyprLook()
  }
  function setHyprCursorSize(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 8 || n > 64 || n === hyprCursorSize) return
    hyprCursorSize = n
    writeHyprLook()
  }
  function setHyprActiveOpacity(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0.2 || n > 1 || n === hyprActiveOpacity) return
    hyprActiveOpacity = n
    writeHyprLook()
  }
  function setHyprPreserveSplit(on) {
    if (on === hyprPreserveSplit) return
    hyprPreserveSplit = on
    writeHyprLook()
  }
  function setHyprFocusOnActivate(on) {
    if (on === hyprFocusOnActivate) return
    hyprFocusOnActivate = on
    writeHyprLook()
  }
  function resetHyprLook() {
    if (!hyprLookManaged) return
    hyprLookManaged = false
    runCommand(["bash", setHyprLookScript, "--reset"])
  }
  function setHyprNoGaps(on) {
    if (on === hyprNoGaps) return
    hyprNoGaps = on
    runCommand(["omarchy", "hyprland", "toggle", "window-no-gaps", on ? "on" : "off"])
  }
  function setHyprSquareAspect(on) {
    if (on === hyprSquareAspect) return
    hyprSquareAspect = on
    runCommand(["omarchy", "hyprland", "toggle", "single-window-aspect-ratio", on ? "on" : "off"])
  }
  function toggleWorkspaceLayout() {
    runCommand(["omarchy", "hyprland", "workspace", "layout", "toggle"])
  }
  function toggleWindowTransparency() {
    runCommand(["omarchy", "hyprland", "window", "transparency", "toggle"])
  }
  function toggleTiledFullscreen() {
    runCommand(["omarchy", "hyprland", "window", "tiled", "fullscreen", "toggle"])
  }

  function setHyprSensitivity(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < -1 || n > 1 || n === hyprSensitivity) return
    hyprSensitivity = n
    writeHyprInput()
  }
  function setHyprAccelProfile(name) {
    if (name !== "flat" && name !== "adaptive" && name !== "") return
    if (name === hyprAccelProfile) return
    hyprAccelProfile = name
    writeHyprInput()
  }
  function setHyprNaturalScroll(on) {
    if (on === hyprNaturalScroll) return
    hyprNaturalScroll = on
    writeHyprInput()
  }
  function setHyprScrollFactor(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0.1 || n > 3 || n === hyprScrollFactor) return
    hyprScrollFactor = n
    writeHyprInput()
  }
  function setHyprClickfinger(on) {
    if (on === hyprClickfinger) return
    hyprClickfinger = on
    writeHyprInput()
  }
  function setHyprDisableWhileTyping(on) {
    if (on === hyprDisableWhileTyping) return
    hyprDisableWhileTyping = on
    writeHyprInput()
  }
  function setHyprDrag3fg(on) {
    var n = on ? 1 : 0
    if (n === hyprDrag3fg) return
    hyprDrag3fg = n
    writeHyprInput()
  }
  function setHyprRepeatRate(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 10 || n > 100 || n === hyprRepeatRate) return
    hyprRepeatRate = n
    writeHyprInput()
  }
  function setHyprRepeatDelay(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 100 || n > 1000 || n === hyprRepeatDelay) return
    hyprRepeatDelay = n
    writeHyprInput()
  }
  function setHyprNumlock(on) {
    if (on === hyprNumlock) return
    hyprNumlock = on
    writeHyprInput()
  }
  function setHyprFollowMouse(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 3 || n === hyprFollowMouse) return
    hyprFollowMouse = n
    writeHyprInput()
  }
  function setHyprKeyPressDpms(on) {
    if (on === hyprKeyPressDpms) return
    hyprKeyPressDpms = on
    writeHyprInput()
  }
  function setHyprMouseMoveDpms(on) {
    if (on === hyprMouseMoveDpms) return
    hyprMouseMoveDpms = on
    writeHyprInput()
  }
  function setHyprKbOverride(layouts, variants, groupToggle) {
    layouts = String(layouts || "").replace(/^\s+|\s+$/g, "").toLowerCase()
    variants = String(variants || "").replace(/^\s+|\s+$/g, "")
    if (layouts && !/^[a-z0-9]{1,8}(,[a-z0-9]{1,8})*$/.test(layouts)) return
    hyprKbLayout = layouts
    hyprKbVariant = variants
    hyprKbGroupToggle = groupToggle === true
    writeHyprInput()
  }
  function setHyprWorkspaceGesture(on) {
    if (on === hyprWorkspaceGesture) return
    hyprWorkspaceGesture = on
    writeHyprInput()
  }
  function resetHyprInput() {
    if (!hyprInputManaged) return
    hyprInputManaged = false
    runCommand(["bash", setHyprInputScript, "--reset"])
  }

  function setNightlightTemperature(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 3000 || n > 6500) return
    if (n === nightlightTemperature) return
    nightlightTemperature = n
    nightlight = n < 6000
    runCommand(["bash", setNightlightTempScript, String(n)])
  }

  function setupFingerprint() {
    runGumJob(["omarchy", "setup", "security", "fingerprint"], "security-fingerprint")
  }
  function removeFingerprint() {
    if (!fingerprintConfigured) return
    runGumJob(["omarchy", "remove", "security", "fingerprint"], "security-fingerprint-remove")
  }
  function setupFido2() {
    runGumJob(["omarchy", "setup", "security", "fido2"], "security-fido2")
  }
  function removeFido2() {
    if (!fido2Configured) return
    runGumJob(["omarchy", "remove", "security", "fido2"], "security-fido2-remove")
  }
  function setupSshd(key) {
    key = String(key || "").replace(/^\s+|\s+$/g, "")
    if (!key || key.length > 8192) return
    if (key.indexOf("\n") !== -1) return
    runGumJob(["omarchy", "setup", "security", "sshd", "--key=" + key], "security-sshd")
  }
  function disableSshd() {
    if (!sshdEnabled && !sshdActive) return
    runJob(["bash", setSshdScript, "disable"], "", "security-sshd-disable")
  }
  function enablePasswordlessSudo(minutes) {
    minutes = Math.round(Number(minutes))
    if (!isFinite(minutes) || minutes < 1 || minutes > 240) minutes = 15
    runGumJob(["omarchy", "sudo", "passwordless", String(minutes)], "passwordless-sudo")
  }
  function disablePasswordlessSudo() {
    if (!passwordlessSudo) return
    runGumJob(["omarchy", "sudo", "passwordless"], "passwordless-sudo-off")
  }
  function setupSudolessDocker() {
    runGumJob(["omarchy", "setup", "security", "sudoless", "docker"], "security-docker")
  }
  function removeSudolessDocker() {
    if (!sudolessDocker) return
    runGumJob(["omarchy", "remove", "security", "sudoless", "docker"], "security-docker-remove")
  }

  function setOmarchyChannel(name) {
    if (name !== "stable" && name !== "rc" && name !== "edge" && name !== "dev") return
    if (name === omarchyChannel) return
    runGumJob(["omarchy", "channel", "set", name], "channel-set")
  }
  function runOmarchyUpdate() {
    runGumJob(["omarchy", "update"], "omarchy-update")
  }
  function checkOmarchyUpdate() {
    runJob(["omarchy", "update", "available"], "", "update-check")
  }
  function setAtmosChannel(name) {
    if (AtmosUpdate.parseChannel(name) !== "alpha") return
    if (name === atmosChannel) return
    atmosChannel = "alpha"
    runCommand(["bash", setAtmosChannelScript, "alpha"])
  }
  function checkAtmosUpdate() {
    runJob(["bash", updateAtmosScript, "check"], "", "atmos-update-check")
  }
  function runAtmosUpdate() {
    runJob(["bash", updateAtmosScript, "apply"], "", "atmos-update")
  }
  function updateFirmware() {
    runGumJob(["omarchy", "update", "firmware"], "update-firmware")
  }
  function updateOrphanPkgs() {
    runGumJob(["omarchy", "update", "orphan", "pkgs"], "update-orphans")
  }
  function prunePkgCache() {
    runGumJob(["omarchy", "update", "pkg", "prune"], "update-prune")
  }

  function installVoxtype() {
    runGumJob(["omarchy", "voxtype", "install"], "voxtype-install")
  }
  function removeVoxtype() {
    if (!voxtypeInstalled) return
    runGumJob(["omarchy", "voxtype", "remove"], "voxtype-remove")
  }
  function toggleHybridGpu() {
    if (!hybridGpuAvailable) return
    runGumJob(["omarchy", "toggle", "hybrid", "gpu"], "hybrid-gpu")
  }
  function installTailscale() {
    runGumJob(["omarchy", "install", "service", "tailscale"], "tailscale-install")
  }
  function removeTailscale() {
    if (!tailscaleInstalled) return
    runGumJob(["omarchy", "remove", "service", "tailscale"], "tailscale-remove")
  }
  function setPluginEnabled(id, on) {
    id = String(id || "")
    if (!/^[A-Za-z0-9._-]+$/.test(id)) return
    if (on) runCommand(["omarchy", "plugin", "enable", id])
    else runCommand(["omarchy", "plugin", "disable", id])
  }
  function setSnapperNumberLimit(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 1 || n > 50 || n === snapperNumberLimit) return
    snapperNumberLimit = n
    runCommand(["bash", setSnapperPolicyScript, "number-limit", String(n)])
  }
  function setSnapperTimeline(on) {
    if (on === snapperTimeline) return
    snapperTimeline = on
    runCommand(["bash", setSnapperPolicyScript, "timeline", on ? "on" : "off"])
  }
  function setFstrim(on) {
    if (on === fstrimEnabled) return
    fstrimEnabled = on
    runCommand(["bash", setFstrimScript, on ? "on" : "off"])
  }
  function setupDirectBoot() {
    if (!directBootAvailable) return
    runGumJob(["omarchy", "setup", "direct", "boot"], "direct-boot")
  }
  function setMimeDefault(kind, desktop) {
    if (kind !== "pdf" && kind !== "image" && kind !== "video") return
    desktop = String(desktop || "")
    if (!/^[A-Za-z0-9._-]+\.desktop$/.test(desktop)) return
    if (kind === "pdf") mimePdf = desktop
    else if (kind === "image") mimeImage = desktop
    else mimeVideo = desktop
    runCommand(["bash", setMimeDefaultScript, kind, desktop])
  }

  function setBluetooth(on) {
    if (on === bluetooth) return
    bluetooth = on
    runCommand(["omarchy", "bluetooth", "power", on ? "on" : "off"])
  }

  function setWifiBand(band) {
    if (band !== "auto" && band !== "2.4" && band !== "5" && band !== "6") return
    if (band === wifiBandSelected) return
    wifiBandSelected = band
    runCommand(["omarchy", "network", "band", band])
  }

  function copyWifiPassword() {
    if (!wifiIface || !/^[a-zA-Z0-9._-]+$/.test(wifiIface)) return
    runCommand(["bash", "-c", "omarchy network password \"$1\" | wl-copy -n", "wifi-password", wifiIface])
  }
  function copyText(text) {
    text = String(text || "")
    if (!text || text.length > 1024) return
    if (/[\r\n\0]/.test(text)) return
    runCommand(["bash", "-c", "printf '%s' \"$1\" | wl-copy -n", "copy-text", text])
  }
  function setWifiRadio(on) {
    if (on === wifiRadio) return
    wifiRadio = on
    runCommand(["bash", setWifiConnectionScript, "radio", on ? "on" : "off"])
  }
  function activateWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    runCommand(["bash", setWifiConnectionScript, "up", uuid])
  }
  function deactivateWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    runCommand(["bash", setWifiConnectionScript, "down", uuid])
  }
  function forgetWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    runCommand(["bash", setWifiConnectionScript, "delete", uuid])
  }
  function restartWifi() {
    runCommand(["omarchy", "restart", "wifi"])
  }
  function restartBluetooth() {
    runCommand(["omarchy", "restart", "bluetooth"])
  }
  function pairBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(["omarchy", "bluetooth", "device", "pair", address])
  }
  function connectBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(["omarchy", "bluetooth", "device", "connect", address])
  }
  function disconnectBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(["omarchy", "bluetooth", "device", "disconnect", address])
  }
  function forgetBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(["omarchy", "bluetooth", "device", "forget", address])
  }
  function setAudioOutputVolume(percent) {
    percent = Math.round(Number(percent))
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    if (percent === audioOutputVolume && !audioOutputMuted) return
    audioOutputVolume = percent
    audioOutputMuted = false
    runCommand(["bash", setAudioScript, "output-volume", String(percent)])
  }
  function toggleAudioOutputMute() {
    audioOutputMuted = !audioOutputMuted
    runCommand(["omarchy", "audio", "output", "volume", "mute-toggle"])
  }
  function setAudioInputVolume(percent) {
    percent = Math.round(Number(percent))
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    if (percent === audioInputVolume && !audioInputMuted) return
    audioInputVolume = percent
    audioInputMuted = false
    runCommand(["bash", setAudioScript, "input-volume", String(percent)])
  }
  function toggleAudioInputMute() {
    audioInputMuted = !audioInputMuted
    runCommand(["omarchy", "audio", "input", "mute"])
  }
  function setAudioSink(name) {
    name = String(name || "")
    if (!name || name === audioSink) return
    var list = audioSinks
    for (var i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].name) === name) {
        audioSink = name
        runCommand(["omarchy", "audio", "output", "set", "default", String(list[i].id), name])
        return
      }
    }
  }
  function setAudioSource(name) {
    name = String(name || "")
    if (!name || name === audioSource) return
    var list = audioSources
    for (var j = 0; j < list.length; j++) {
      if (list[j] && String(list[j].name) === name) {
        audioSource = name
        runCommand(["omarchy", "audio", "input", "set", "default", String(list[j].id), name])
        return
      }
    }
  }
  function switchAudioOutput() {
    runCommand(["omarchy", "audio", "output", "switch"])
  }
  function setAudioTuning(on) {
    if (on === audioTuningOn) return
    audioTuningOn = on
    runCommand(["omarchy", "audio", "tuning", on ? "on" : "off"])
  }
  function restartAudio() {
    runCommand(["omarchy", "restart", "audio"])
  }
  function validMountPath(dir) {
    dir = String(dir || "")
    return dir.charAt(0) === "/" && dir.indexOf("..") === -1 && /^\/[A-Za-z0-9._/-]*$/.test(dir)
  }
  function openUserDir(dir) {
    dir = String(dir || "")
    if (!dir || dir.length > 1024) return
    if (dir.charAt(0) !== "/" || dir.indexOf("..") !== -1) return
    if (/[\r\n\0]/.test(dir)) return
    launchDetached(["xdg-open", dir])
  }
  function changeDrivePassword(device, currentPass, newPass) {
    device = String(device || "")
    currentPass = String(currentPass || "")
    newPass = String(newPass || "")
    if (!device || device.charAt(0) !== "/" || device.indexOf("..") !== -1) return
    if (!currentPass || !newPass) return
    runJob(["bash", luksChangeKeyScript], device + "\n" + currentPass + "\n" + newPass + "\n", "luks")
  }
  function createSnapshot() {
    runJob(["omarchy", "snapshot", "create"], "", "snapshot-create")
  }
  function restoreSnapshot(config, id) {
    config = String(config || "")
    id = String(id || "")
    if (!/^[A-Za-z0-9_-]+$/.test(config)) return
    if (!/^[0-9]+$/.test(id)) return
    runJob(["bash", rollbackSnapshotScript, config, id], "", "snapshot-rollback")
  }
  function setupHibernation() {
    runJob(["omarchy", "hibernation", "setup", "--force"], "", "hibernation-setup")
  }
  function removeHibernation() {
    runJob(["bash", "-c", "PATH=\"$1:$PATH\" exec omarchy hibernation remove", "hibernation-remove", gumStubDir], "", "hibernation-remove")
  }

  function setSuspendEnabled(on) {
    if (on === suspendEnabled) return
    suspendEnabled = on
    runCommand(["omarchy", "toggle", "suspend-off", on ? "off" : "on"])
  }

  function setPowerProfile(name) {
    if (!name || name === powerProfile) return
    powerProfile = name
    runCommand(["omarchy", "powerprofiles", "set", "autodetect", name])
  }

  function setPowerProfileAc(name) {
    if (!name || name === powerProfileAc) return
    powerProfileAc = name
    runCommand(["omarchy", "powerprofiles", "set", "ac", name])
  }

  function setPowerProfileBattery(name) {
    if (!name || name === powerProfileBattery) return
    powerProfileBattery = name
    runCommand(["omarchy", "powerprofiles", "set", "battery", name])
  }

  function setPowerShowPercentage(on) {
    if (on === powerShowPercentage) return
    powerShowPercentage = on
    runCommand(["omarchy", "bar", "set", "omarchy.power", "showPercentage", on ? "true" : "false", "--json"])
  }

  function showBatteryNotification() {
    runCommand(["omarchy", "notification", "battery"])
  }

  function setCrashCapture(on) {
    if (on === crashCapture) return
    crashCapture = on
    runCommand(["omarchy", "toggle", "crash", "capture"])
  }

  function setDoNotDisturb(on) {
    if (on === doNotDisturb) return
    doNotDisturb = on
    runCommand(["omarchy", "toggle", "notification", "silencing"])
  }

  function setWeatherLocation(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name) return
    if (!weatherAuto && name === weatherLocation) return
    weatherLocation = name
    weatherAuto = false
    runCommand(["omarchy", "weather", "location", "--set", name])
  }

  function clearWeatherLocation() {
    if (weatherAuto) return
    weatherAuto = true
    weatherLocation = ""
    runCommand(["omarchy", "weather", "location", "--clear"])
  }

  function setWeatherCoordinates(coords) {
    coords = String(coords || "").replace(/^\s+|\s+$/g, "")
    if (!/^-?[0-9]+(\.[0-9]+)?,-?[0-9]+(\.[0-9]+)?$/.test(coords)) return
    var name = String(weatherLocation || "").replace(/^\s+|\s+$/g, "")
    if (!name || weatherAuto) return
    if (coords === weatherCoords) return
    weatherCoords = coords
    runCommand(["omarchy", "weather", "location", "--set", name, coords])
  }

  function setWeatherUnit(unit) {
    if (unit !== "auto" && unit !== "metric" && unit !== "imperial") return
    if (unit === weatherUnit) return
    weatherUnit = unit
    runCommand(["omarchy", "bar", "set", "omarchy.weather", "unit", unit])
  }

  function setWeatherRefreshMinutes(minutes) {
    minutes = Math.round(Number(minutes))
    if (!(minutes >= 1) || minutes === weatherRefreshMinutes) return
    weatherRefreshMinutes = minutes
    runCommand(["omarchy", "bar", "set", "omarchy.weather", "refreshMinutes", String(minutes), "--json"])
  }

  function setReminder(minutes, message) {
    minutes = String(minutes || "").replace(/^\s+|\s+$/g, "")
    if (!/^[1-9][0-9]*$/.test(minutes)) return
    message = String(message || "").replace(/^\s+|\s+$/g, "")
    if (message.length > 0)
      runCommand(["omarchy", "reminder", minutes, message])
    else
      runCommand(["omarchy", "reminder", minutes])
  }

  function clearReminders() {
    if (!reminderActive) return
    runCommand(["omarchy", "reminder", "clear"])
  }

  function showReminders() {
    runCommand(["omarchy", "reminder", "show"])
  }

  function sendTestNotification() {
    runCommand(["omarchy", "notification", "send", "Atmos", "This is a test toast."])
  }
  function sendTimeNotification() {
    runCommand(["omarchy", "notification", "time"])
  }
  function sendWeatherNotification() {
    runCommand(["omarchy", "notification", "weather"])
  }

  function captureScreenshot(mode, dest) {
    mode = String(mode || "smart")
    dest = String(dest || "slurp")
    if (mode !== "smart" && mode !== "region" && mode !== "windows" && mode !== "fullscreen") return
    if (dest !== "slurp" && dest !== "copy" && dest !== "save") return
    runCommand(["omarchy", "capture", "screenshot", mode, dest])
  }
  function startScreenrecording(desktopAudio, microphone, webcam, webcamSize, fullscreen) {
    var argv = ["omarchy", "capture", "screenrecording"]
    if (fullscreen) argv.push("--fullscreen")
    if (desktopAudio) argv.push("--with-desktop-audio")
    if (microphone) argv.push("--with-microphone-audio")
    if (webcam) argv.push("--with-webcam")
    var size = String(webcamSize || "medium")
    if (size !== "small" && size !== "medium" && size !== "large") size = "medium"
    if (webcam) argv.push("--webcam-size=" + size)
    runCommand(argv)
  }
  function stopScreenrecording() {
    runCommand(["omarchy", "capture", "screenrecording", "--stop-recording"])
  }
  function captureText() {
    runCommand(["omarchy", "capture", "text"])
  }
  function captureQr() {
    runCommand(["omarchy", "capture", "qr"])
  }
  function resizeWebcam(action) {
    action = String(action || "")
    if (action !== "smaller" && action !== "larger" && action !== "reset" && action !== "small" && action !== "medium" && action !== "large") return
    runCommand(["omarchy", "capture", "webcam", "resize", action])
  }

  function shareClipboard() {
    runCommand(["omarchy", "share", "clipboard"])
  }
  function shareFile(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runCommand(["omarchy", "share", "file", path])
  }
  function shareFolder(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runCommand(["omarchy", "share", "folder", path])
  }
  function tailscaleSend(machine, path) {
    machine = String(machine || "").replace(/^\s+|\s+$/g, "")
    if (!machine || !/^[A-Za-z0-9._-]+$/.test(machine)) return
    path = String(path || "")
    if (path.length > 0) {
      if (path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
      runCommand(["omarchy", "tailscale", "send", machine, path])
      return
    }
    runCommand(["omarchy", "tailscale", "send", machine])
  }
  function tailscaleReceive() {
    runCommand(["omarchy", "tailscale", "receive", "--once"])
  }

  function runSoftware(argv, kind) {
    if (!(argv instanceof Array) || argv.length < 2) return
    if (argv[0] !== "omarchy") return
    runGumJob(argv, kind || "software")
  }
  function installDevEnv(lang) {
    lang = String(lang || "")
    if (!/^[a-z]+$/.test(lang)) return
    runGumJob(["omarchy", "install", "dev", "env", lang], "dev-env-install")
  }
  function removeDevEnv(lang) {
    lang = String(lang || "")
    if (!/^[a-z]+$/.test(lang)) return
    runGumJob(["omarchy", "remove", "dev", "env", lang], "dev-env-remove")
  }
  function installDockerDb(name) {
    name = String(name || "")
    if (!/^[A-Za-z]+$/.test(name)) return
    runGumJob(["omarchy", "install", "docker", "dbs", name], "docker-db-install")
  }

  function isHookId(name) {
    name = String(name || "")
    if (name === "theme-set" || name === "font-set" || name === "post-boot" || name === "post-update" || name === "pre-refresh-pacman" || name === "battery-low")
      return true
    return /^[a-z0-9][a-z0-9-]*$/.test(name)
  }

  function installHook(type, path) {
    type = String(type || "")
    path = String(path || "")
    if (!isHookId(type) || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runCommand(["omarchy", "hook", "install", type, path])
  }

  function createHook(type, name, command) {
    type = String(type || "")
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    command = String(command || "").replace(/^\s+|\s+$/g, "")
    if (!isHookId(type) || !name || !command) return
    if (name.indexOf("/") !== -1 || name.indexOf("..") !== -1) return
    if (command.indexOf("\n") !== -1 || command.length > 512) return
    runCommand(["bash", createHookScript, type, name, command])
  }

  function removeHook(path) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0) return
    if (path.indexOf("..") !== -1) return
    if (path.length >= 7 && path.substring(path.length - 7) === ".sample") return
    runCommand(["rm", "-f", path])
  }

  function setHookSample(path, enabled) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0 || path.indexOf("..") !== -1) return
    runCommand(["bash", setHookSampleScript, enabled ? "enable" : "disable", path])
  }

  function runHook(name, arg) {
    name = String(name || "")
    if (!isHookId(name)) return
    arg = String(arg || "").replace(/^\s+|\s+$/g, "")
    if (arg)
      runCommand(["omarchy", "hook", name, arg])
    else
      runCommand(["omarchy", "hook", name])
  }

  function openHookFolder(type) {
    type = String(type || "")
    if (!isHookId(type)) return
    var dir = Quickshell.env("HOME") + "/.config/omarchy/hooks/" + type + ".d"
    runCommand(["bash", "-c", "mkdir -p \"$1\"; xdg-open \"$1\" >/dev/null 2>&1 &", "prefs-hooks", dir])
  }

  function editHook(path) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0 || path.indexOf("..") !== -1) return
    launchDetached(["xdg-open", path])
  }

  function editMonitorsLua() {
    var path = String(monitorsLuaFile || "")
    var root = Quickshell.env("HOME") + "/.config/hypr/"
    if (path.indexOf(root) !== 0 || path.indexOf("..") !== -1) return
    launchDetached(["xdg-open", path])
  }

  function launchHerdr() {
    if (!(extras && extras.herdr === true)) return
    runCommand(["omarchy", "launch", "terminal", "herdr"])
  }

  function setNightlightSchedule(day, night, nightOn) {
    day = String(day || "").replace(/^\s+|\s+$/g, "")
    night = String(night || "").replace(/^\s+|\s+$/g, "")
    if (!/^[0-2]?\d:[0-5]\d$/.test(day) || !/^[0-2]?\d:[0-5]\d$/.test(night)) return
    nightlightDay = day
    nightlightNight = night
    nightlightNightOn = nightOn === true
    var temp = nightlightTemperature > 0 ? nightlightTemperature : 4000
    runCommand(["bash", setHyprsunsetScript, JSON.stringify({
      day: nightlightDay,
      night: nightlightNight,
      nightOn: nightlightNightOn,
      temperature: temp
    })])
  }

  function autostartCommands() {
    var list = Array.isArray(autostart) ? autostart : []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].managed === true && list[i].command)
        out.push(String(list[i].command))
    }
    return out
  }
  function writeAutostart(commands) {
    autostartManaged = true
    runCommand(["bash", setHyprAutostartScript, JSON.stringify({ commands: commands })])
  }
  function addAutostart(command) {
    command = String(command || "").replace(/^\s+|\s+$/g, "")
    if (!command || command.length > 256 || command.indexOf("\n") !== -1) return
    var next = autostartCommands()
    for (var i = 0; i < next.length; i++) {
      if (next[i] === command) return
    }
    next.push(command)
    writeAutostart(next)
  }
  function removeAutostart(command) {
    command = String(command || "")
    var cur = autostartCommands()
    var next = []
    for (var i = 0; i < cur.length; i++) {
      if (cur[i] !== command) next.push(cur[i])
    }
    if (next.length === cur.length) return
    writeAutostart(next)
  }

  function catalogHas(keys) {
    var list = Array.isArray(keybindings) ? keybindings : []
    var chord = String(keys || "")
    for (var i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].keys || "") === chord) return true
    }
    return false
  }

  function managedBindings() {
    var list = Array.isArray(bindings) ? bindings : []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].managed === true && list[i].keys)
        out.push({
          keys: String(list[i].keys),
          label: String(list[i].label || ""),
          command: String(list[i].command || ""),
          unbind: list[i].unbind === true
        })
    }
    return out
  }

  function writeBindings(items) {
    bindingsManaged = true
    runCommand(["bash", setHyprBindingsScript, JSON.stringify({ items: items })])
  }

  function addBinding(keys, label, command, unbind) {
    keys = String(keys || "").replace(/^\s+|\s+$/g, "").replace(/\s+/g, " ")
    label = String(label || "").replace(/^\s+|\s+$/g, "")
    command = String(command || "").replace(/^\s+|\s+$/g, "")
    unbind = unbind === true
    if (!keys || keys.length > 64 || keys.indexOf("\n") !== -1) return
    if (command && (command.length > 256 || command.indexOf("\n") !== -1)) return
    if (!command && !unbind) return
    if (command && catalogHas(keys)) unbind = true
    var cur = managedBindings()
    var next = []
    for (var i = 0; i < cur.length; i++) {
      if (cur[i].keys !== keys) next.push(cur[i])
    }
    next.push({ keys: keys, label: label, command: command, unbind: unbind })
    writeBindings(next)
  }

  function removeBinding(keys) {
    keys = String(keys || "")
    var cur = managedBindings()
    var next = []
    for (var i = 0; i < cur.length; i++) {
      if (cur[i].keys !== keys) next.push(cur[i])
    }
    if (next.length === cur.length) return
    writeBindings(next)
  }

  function managedWindowRules() {
    var list = Array.isArray(windowRules) ? windowRules : []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].managed === true && list[i].match)
        out.push({
          match: String(list[i].match),
          placement: String(list[i].placement || ""),
          center: list[i].center === true,
          width: Math.round(Number(list[i].width)) || 0,
          height: Math.round(Number(list[i].height)) || 0,
          workspace: String(list[i].workspace || "")
        })
    }
    return out
  }

  function writeWindowRules(items) {
    windowRulesManaged = true
    runCommand(["bash", setHyprWindowsScript, JSON.stringify({ items: items })])
  }

  function addWindowRule(match, placement, center, width, height, workspace) {
    match = String(match || "").replace(/^\s+|\s+$/g, "")
    placement = String(placement || "")
    if (placement !== "float" && placement !== "tile") placement = ""
    workspace = String(workspace || "").replace(/^\s+|\s+$/g, "")
    width = Math.round(Number(width)) || 0
    height = Math.round(Number(height)) || 0
    if (!match || match.length > 128 || match.indexOf("\n") !== -1 || match.indexOf("]]") !== -1) return
    if (!(width >= 100 && height >= 100)) {
      width = 0
      height = 0
    }
    if (!placement && center !== true && !width && !workspace) return
    var cur = managedWindowRules()
    var next = []
    for (var i = 0; i < cur.length; i++) {
      if (cur[i].match !== match) next.push(cur[i])
    }
    next.push({
      match: match,
      placement: placement,
      center: center === true,
      width: width,
      height: height,
      workspace: workspace
    })
    writeWindowRules(next)
  }

  function removeWindowRule(match) {
    match = String(match || "")
    var cur = managedWindowRules()
    var next = []
    for (var i = 0; i < cur.length; i++) {
      if (cur[i].match !== match) next.push(cur[i])
    }
    if (next.length === cur.length) return
    writeWindowRules(next)
  }

  function launchDetached(argv) {
    if (!(argv instanceof Array) || argv.length === 0) return
    var cmd = ["bash", "-c", "exec \"$@\" >/dev/null 2>&1 &", "prefs-open"]
    for (var i = 0; i < argv.length; i++) cmd.push(argv[i])
    runCommand(cmd)
  }

  function openPrinters() {
    if (printerSetup)
      launchDetached(["system-config-printer"])
    else
      openCupsAdmin()
  }

  function openCupsAdmin() {
    launchDetached(["xdg-open", "http://127.0.0.1:631"])
  }

  function restartShell() {
    runCommand(["omarchy", "restart", "shell"])
  }

  function refreshHyprland() {
    runJob(["bash", refreshHyprlandScript], "", "refresh-hyprland")
  }

  function refreshShell() {
    runJob(["omarchy", "refresh", "shell"], "", "refresh-shell")
  }

  function setPlymouth(name) {
    if (!name || name === "default") return
    if (name === plymouth) return
    plymouth = name
    runCommand(["omarchy", "plymouth", "set", "by", "theme", name])
  }

  function resetPlymouth() {
    if (plymouth === "default") return
    plymouth = "default"
    runJob(["omarchy", "plymouth", "reset"], "", "plymouth-reset")
  }
  function setPlymouthFromPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runJob(["bash", "-c", "bg=$(omarchy theme color background); text=$(omarchy theme color foreground); omarchy plymouth set \"$bg\" \"$text\" \"$1\"", "plymouth-set", path], "", "plymouth-set")
  }
  function previewPlymouthFromPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runJob(["bash", "-c", "bg=$(omarchy theme color background); text=$(omarchy theme color foreground); out=$(mktemp --suffix=.png); omarchy plymouth preview \"$bg\" \"$text\" \"$1\" \"$out\"; echo \"$out\"", "plymouth-preview", path], "", "plymouth-preview")
  }

  function installedOptions(all, available) {
    var out = []
    for (var i = 0; i < all.length; i++) {
      var item = all[i]
      var key = typeof item === "object" ? item.value : item
      if (available && available[key] === false) continue
      out.push(item)
    }
    return out
  }

  Component.onCompleted: startSession(Quickshell.env("ATMOS_PAGE") || "appearance")

  readonly property var watchPaths: [
    userShellJson, defaultShellJson, weatherJson, notificationsJson,
    currentThemeNameFile, currentBackgroundFile, screensaverBrandFile,
    defaultScreensaverBrandFile, aboutBrandFile, defaultAboutBrandFile,
    powerProfileAcFile, powerProfileBatteryFile, togglesDir, hyprTogglesDir,
    touchpadDisabledFile, touchscreenDisabledFile, indicatorsDir,
    applicationsDir, extraThemesDir, packagedThemesDir, defaultEditorFile,
    defaultAgentFile, defaultTerminalFile, defaultBrowserFile, fontconfigFile,
    userShellToml, reminderDir, dnsConfFile, bluetoothRfkillDir,
    plymouthLogoFile, defaultPlymouthLogoFile, powerProfilesStateFile,
    networkManagerDevicesDir, looknfeelLuaFile, inputLuaFile, monitorsLuaFile,
    localtimeFile, hostnameFile, vconsoleFile, localeConfFile, pacmanConfFile
  ]

  property Instantiator fileWatchers: Instantiator {
    model: root.watchPaths
    delegate: FileView {
      path: modelData
      watchChanges: true
      printErrors: false
      onFileChanged: {
        reload()
        root.scheduleRefresh()
      }
    }
  }

  property Timer refreshTimer: Timer {
    interval: 180
    repeat: false
    onTriggered: root.enqueueRead("all")
  }

  property Process snapshotProc: Process {
    command: ["bash", root.snapshotScript]
    stdout: StdioCollector {
      id: snapOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: snapErr
      waitForEnd: true
    }
    onExited: function(exitCode) {
      if (exitCode === 0) {
        root.lastError = ""
        root.applySnapshot(snapOut.text)
      } else {
        root.lastError = String(snapErr.text || "omarchy snapshot failed").replace(/^\s+|\s+$/g, "")
      }
      root.snapshotReady = true
      root.ioFinished()
    }
  }

  property Process mutProc: Process {
    command: ["true"]
    stdout: StdioCollector { waitForEnd: true }
    stderr: StdioCollector {
      id: mutErr
      waitForEnd: true
    }
    onExited: function(exitCode) {
      if (exitCode !== 0) {
        var err = String(mutErr.text || "").replace(/^\s+|\s+$/g, "")
        if (root.stderrLooksLikeFailure(err))
          root.lastError = err || "Command failed"
        else
          root.lastError = ""
      }
      WorkQueue.enqueueRead(root.ioQueue, "all")
      root.ioFinished()
    }
  }

  property Process jobProc: Process {
    command: ["true"]
    stdinEnabled: false
    stdout: StdioCollector {
      id: jobOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: jobErr
      waitForEnd: true
    }
    onStarted: {
      if (root.jobStdin.length > 0) {
        write(root.jobStdin)
        root.jobStdin = ""
      }
    }
    onExited: function(exitCode) {
      root.jobBusy = false
      var out = String(jobOut.text || "").replace(/^\s+|\s+$/g, "")
      var err = String(jobErr.text || "").replace(/^\s+|\s+$/g, "")
      root.jobLog = out
      if (root.jobKind === "update-check") {
        root.lastError = ""
        root.jobKind = ""
        WorkQueue.enqueueRead(root.ioQueue, "all")
        root.ioFinished()
        return
      }
      if (root.jobKind === "atmos-update-check" || root.jobKind === "atmos-update") {
        var parsed = AtmosUpdate.parseCheckOutput(out + "\n" + err)
        var applied = root.jobKind === "atmos-update" && exitCode === 0
        root.atmosUpdateAvailable = parsed.status === "behind"
        root.atmosUpdateSummary = parsed.summary
        if (parsed.short) root.atmosRevision = parsed.short
        if (parsed.channel) root.atmosChannel = parsed.channel
        root.lastError = (exitCode !== 0 && parsed.status !== "behind") ? (parsed.summary || err || "Atmos update failed") : ""
        root.jobKind = ""
        if (applied) WorkQueue.enqueueRead(root.ioQueue, "all")
        root.ioFinished()
        return
      }
      if (exitCode !== 0) {
        if (root.stderrLooksLikeFailure(err))
          root.lastError = err || "Command failed"
        else
          root.lastError = ""
      } else {
        root.lastError = ""
        WorkQueue.enqueueRead(root.ioQueue, "all")
      }
      root.jobKind = ""
      root.ioFinished()
    }
  }
}
