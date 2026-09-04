pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "Accounts.js" as AccountsJs
import "AtmosUpdate.js" as AtmosUpdate
import "Hardware.js" as HardwareJs
import "Hooks.js" as HooksJs
import "RichUi.js" as RichUi
import "Snapshot.js" as SnapshotJs
import "Theme.js" as ThemeJs
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
  readonly property string setAvatarScript: shellDir + "/scripts/set-avatar.sh"
  readonly property string manageAccountScript: shellDir + "/scripts/manage-account.sh"
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
  readonly property string resetAtmosScript: shellDir + "/scripts/reset-atmos.sh"
  readonly property string setHyprsunsetScript: shellDir + "/scripts/set-hyprsunset.sh"
  readonly property string setNightlightTempScript: shellDir + "/scripts/set-nightlight-temp.sh"
  readonly property string updateAtmosScript: shellDir + "/scripts/update-atmos.sh"
  readonly property string setAtmosChannelScript: shellDir + "/scripts/set-atmos-channel.sh"
  readonly property string setSnapperPolicyScript: shellDir + "/scripts/set-snapper-policy.sh"
  readonly property string setFstrimScript: shellDir + "/scripts/set-fstrim.sh"
  readonly property string setMimeDefaultScript: shellDir + "/scripts/set-mime-default.sh"
  readonly property string setSshdScript: shellDir + "/scripts/set-sshd.sh"
  readonly property string setPasswordlessSudoScript: shellDir + "/scripts/set-passwordless-sudo.sh"
  readonly property string createHookScript: shellDir + "/scripts/create-hook.sh"
  readonly property string setHookSampleScript: shellDir + "/scripts/set-hook-sample.sh"
  readonly property string looknfeelLuaFile: Quickshell.env("HOME") + "/.config/hypr/looknfeel.lua"
  readonly property string inputLuaFile: Quickshell.env("HOME") + "/.config/hypr/input.lua"
  readonly property string pacmanConfFile: "/etc/pacman.conf"
  readonly property string localtimeFile: "/etc/localtime"
  readonly property string hostnameFile: "/etc/hostname"
  readonly property string passwdFile: "/etc/passwd"
  readonly property string groupFile: "/etc/group"
  readonly property string faceIconFile: Quickshell.env("HOME") + "/.face.icon"
  readonly property string faceFile: Quickshell.env("HOME") + "/.face"
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
  property string jobStdoutBuf: ""
  property var jobStdoutLineCb: null
  property var jobFinishedCb: null
  property var wifiQrRows: []
  property int wifiQrSize: 0
  property string wifiQrSsid: ""
  property string wifiQrError: ""
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
  readonly property string hostname: AccountsStore.hostname
  readonly property string fullName: AccountsStore.fullName
  readonly property string currentUser: AccountsStore.currentUser
  readonly property string avatarPath: AccountsStore.avatarPath
  readonly property var accountUsers: AccountsStore.users
  readonly property var accountGroups: AccountsStore.groups
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
  property int hyprEmulateDiscreteScroll: 1
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
  property int sudoMinutes: 15
  property bool sudoPromptOpen: false
  property bool sudoEnabling: false
  property string sudoError: ""
  property var sudoPendingJob: null
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
    if (!("hardware" in parsed) && !("disks" in parsed)) {
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
    AccountsStore.applyPatch(data)
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
    if ("monitors" in parsed) monitors = adoptArray(monitors, parsed.monitors)
    if ("keyboardBrightness" in parsed) {
      keyboardBrightness = Math.round(Number(parsed.keyboardBrightness)) || 0
      if (keyboardBrightness < 0) keyboardBrightness = 0
      if (keyboardBrightness > 100) keyboardBrightness = 100
    }
    if ("internalEnabled" in parsed) internalEnabled = parsed.internalEnabled === true
    if ("mirroring" in parsed) mirroring = parsed.mirroring === true
    if ("touchpadEnabled" in parsed) touchpadEnabled = parsed.touchpadEnabled !== false
    if ("touchscreenEnabled" in parsed) touchscreenEnabled = parsed.touchscreenEnabled !== false
    if ("barPosition" in parsed) barPosition = String(parsed.barPosition || "top")
    if ("barTransparent" in parsed) barTransparent = parsed.barTransparent === true
    if ("barVisible" in parsed) barVisible = parsed.barVisible !== false
    if ("clockPresent" in parsed) clockPresent = parsed.clockPresent === true
    if ("indicatorsPresent" in parsed) indicatorsPresent = parsed.indicatorsPresent === true
    if ("agentsPresent" in parsed) agentsPresent = parsed.agentsPresent === true
    if ("trayPresent" in parsed) trayPresent = parsed.trayPresent === true
    if ("isLaptop" in parsed) isLaptop = parsed.isLaptop === true
    if ("batteryPresent" in parsed) batteryPresent = parsed.batteryPresent === true
    if ("weatherPresent" in parsed) weatherPresent = parsed.weatherPresent === true
    if ("internalPresent" in parsed) internalPresent = parsed.internalPresent === true
    if ("externalPresent" in parsed) externalPresent = parsed.externalPresent === true
    if ("touchpadPresent" in parsed) touchpadPresent = parsed.touchpadPresent === true
    if ("touchscreenPresent" in parsed) touchscreenPresent = parsed.touchscreenPresent === true
    if ("keyboardBacklightPresent" in parsed) keyboardBacklightPresent = parsed.keyboardBacklightPresent === true
    if ("hyprSquareAspect" in parsed) hyprSquareAspect = parsed.hyprSquareAspect === true
    if ("hyprWorkspaceGesture" in parsed) hyprWorkspaceGesture = parsed.hyprWorkspaceGesture === true
    if ("clockFormat" in parsed) clockFormat = String(parsed.clockFormat || "")
    if ("clockFormatAlt" in parsed) clockFormatAlt = String(parsed.clockFormatAlt || "")
    if ("clockWeekStart" in parsed) {
      clockWeekStart = String(parsed.clockWeekStart || "").toLowerCase()
      if (clockWeekStart !== "sunday" && clockWeekStart !== "monday" && clockWeekStart !== "tuesday" && clockWeekStart !== "wednesday" && clockWeekStart !== "thursday" && clockWeekStart !== "friday" && clockWeekStart !== "saturday")
        clockWeekStart = ""
    }
    if ("clockBirthYear" in parsed) {
      clockBirthYear = Math.round(Number(parsed.clockBirthYear)) || 0
      if (clockBirthYear < 1) clockBirthYear = 0
    }
    if ("clockLifeExpectancy" in parsed) {
      clockLifeExpectancy = Math.round(Number(parsed.clockLifeExpectancy)) || 0
      if (clockLifeExpectancy < 1 || clockLifeExpectancy > 150) clockLifeExpectancy = 0
    }
    if ("indicatorsAlwaysShow" in parsed) indicatorsAlwaysShow = parsed.indicatorsAlwaysShow === true
    if ("indicatorsItems" in parsed) indicatorsItems = adoptArray(indicatorsItems, root.normalizedIndicatorItems(parsed.indicatorsItems))
    if ("agentsRefreshIntervalSec" in parsed) {
      agentsRefreshIntervalSec = Number(parsed.agentsRefreshIntervalSec) || 900
      if (agentsRefreshIntervalSec < 30) agentsRefreshIntervalSec = 900
    }
    if ("agentsSync" in parsed) agentsSync = parsed.agentsSync === true
    if ("agentsSyncDir" in parsed) agentsSyncDir = String(parsed.agentsSyncDir || "")
    if ("agentsSyncFileName" in parsed) agentsSyncFileName = String(parsed.agentsSyncFileName || "")
    if ("agentsSyncDeviceId" in parsed) agentsSyncDeviceId = String(parsed.agentsSyncDeviceId || "")
    if ("spacerSize" in parsed) {
      spacerSize = Math.round(Number(parsed.spacerSize))
      if (!isFinite(spacerSize) || spacerSize < 0) spacerSize = 12
      if (spacerSize > 64) spacerSize = 64
    }
    if ("spacerPresent" in parsed) spacerPresent = parsed.spacerPresent === true
    if ("trayHidden" in parsed) trayHidden = adoptArray(trayHidden, root.normalizedStringIds(parsed.trayHidden))
    if ("trayPinned" in parsed) trayPinned = adoptArray(trayPinned, root.normalizedStringIds(parsed.trayPinned))
    if ("browser" in parsed) browser = String(parsed.browser || "")
    if ("terminal" in parsed) terminal = String(parsed.terminal || "")
    if ("editor" in parsed) editor = String(parsed.editor || "")
    if ("agent" in parsed) agent = String(parsed.agent || "")
    if ("dns" in parsed) dns = String(parsed.dns || "")
    if ("idleScreensaver" in parsed) idleScreensaver = Number(parsed.idleScreensaver) || 0
    if ("idleLock" in parsed) idleLock = Number(parsed.idleLock) || 0
    if ("screensaverEnabled" in parsed) screensaverEnabled = parsed.screensaverEnabled !== false
    if ("timezone" in parsed) {
      timezone = String(parsed.timezone || "")
      if (!/^[A-Za-z0-9/_+-]+$/.test(timezone) || timezone.indexOf("..") !== -1) timezone = ""
    }
    if ("ntp" in parsed) ntp = parsed.ntp === true
    if ("ntpSynchronized" in parsed) ntpSynchronized = parsed.ntpSynchronized === true
    if ("hostname" in parsed || "fullName" in parsed || "currentUser" in parsed || "avatarPath" in parsed || "users" in parsed || "groups" in parsed)
      AccountsStore.applyPatch(parsed)
    if ("keyboardLayout" in parsed) {
      keyboardLayout = String(parsed.keyboardLayout || "")
      if (keyboardLayout.indexOf(",") !== -1) keyboardLayout = keyboardLayout.split(",")[0]
      if (!/^[a-z0-9]{1,8}$/.test(keyboardLayout)) keyboardLayout = ""
    }
    if ("locale" in parsed) {
      locale = String(parsed.locale || "")
      if (locale !== "C.UTF-8" && !/^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(locale))
        locale = ""
    }
    if ("parallelDownloads" in parsed) {
      parallelDownloads = Math.round(Number(parsed.parallelDownloads)) || 5
      if (parallelDownloads < 1) parallelDownloads = 5
      if (parallelDownloads > 20) parallelDownloads = 20
    }
    if ("hyprLook" in parsed) root.applyHyprLook(parsed.hyprLook)
    if ("hyprInput" in parsed) root.applyHyprInput(parsed.hyprInput)
    if ("hyprLookManaged" in parsed) hyprLookManaged = parsed.hyprLookManaged === true
    if ("hyprInputManaged" in parsed) hyprInputManaged = parsed.hyprInputManaged === true
    if ("hyprNoGaps" in parsed) hyprNoGaps = parsed.hyprNoGaps === true
    if ("hyprSquareAspect" in parsed) hyprSquareAspect = parsed.hyprSquareAspect === true
    if ("hyprWorkspaceGesture" in parsed) hyprWorkspaceGesture = parsed.hyprWorkspaceGesture === true
    if ("plugins" in parsed) plugins = adoptArray(plugins, parsed.plugins)
    if ("desktopApps" in parsed) desktopApps = adoptArray(desktopApps, parsed.desktopApps)
    if ("tuiApps" in parsed) tuiApps = adoptArray(tuiApps, parsed.tuiApps)
    if ("webApps" in parsed) webApps = adoptArray(webApps, parsed.webApps)
    if ("hyprWorkspaceLayout" in parsed) {
      hyprWorkspaceLayout = String(parsed.hyprWorkspaceLayout || "dwindle")
      if (hyprWorkspaceLayout !== "scrolling") hyprWorkspaceLayout = "dwindle"
    }
    if ("bluetoothDevices" in parsed) bluetoothDevices = adoptArray(bluetoothDevices, parsed.bluetoothDevices)
    if ("wifiConnections" in parsed) wifiConnections = adoptArray(wifiConnections, parsed.wifiConnections)
    if ("wifiConnected" in parsed) wifiConnected = parsed.wifiConnected === true
    if ("netKind" in parsed) {
      netKind = String(parsed.netKind || "disconnected")
      if (netKind !== "ethernet" && netKind !== "wifi") netKind = "disconnected"
    }
    if ("netSsid" in parsed) netSsid = String(parsed.netSsid || "")
    if ("hooks" in parsed) hooks = adoptArray(hooks, parsed.hooks)
    if ("reminderCount" in parsed) {
      reminderCount = Math.round(Number(parsed.reminderCount)) || 0
      if (reminderCount < 0) reminderCount = 0
    }
    if ("reminderActive" in parsed) reminderActive = parsed.reminderActive === true
    if ("reminders" in parsed) reminders = adoptArray(reminders, parsed.reminders)
    if ("recordingActive" in parsed) recordingActive = parsed.recordingActive === true
    if ("webcamOverlay" in parsed) webcamOverlay = parsed.webcamOverlay === true
    if ("autostart" in parsed) autostart = adoptArray(autostart, parsed.autostart)
    if ("autostartManaged" in parsed) autostartManaged = parsed.autostartManaged === true
    if ("bindings" in parsed) bindings = adoptArray(bindings, parsed.bindings)
    if ("bindingsManaged" in parsed) bindingsManaged = parsed.bindingsManaged === true
    if ("windowRules" in parsed) windowRules = adoptArray(windowRules, parsed.windowRules)
    if ("windowRulesManaged" in parsed) windowRulesManaged = parsed.windowRulesManaged === true
    if ("atmosChannel" in parsed) {
      atmosChannel = AtmosUpdate.parseChannel(parsed.atmosChannel)
      if (!atmosChannel) atmosChannel = "alpha"
    }
    if ("snapperNumberLimit" in parsed) {
      snapperNumberLimit = Math.round(Number(parsed.snapperNumberLimit)) || 5
      if (snapperNumberLimit < 1) snapperNumberLimit = 5
      if (snapperNumberLimit > 50) snapperNumberLimit = 50
    }
    if ("snapperTimeline" in parsed) snapperTimeline = parsed.snapperTimeline === true
    if ("fstrimEnabled" in parsed) fstrimEnabled = parsed.fstrimEnabled === true
    if ("mimePdf" in parsed) {
      mimePdf = String(parsed.mimePdf || "")
      if (!/^[A-Za-z0-9._-]+\.desktop$/.test(mimePdf)) mimePdf = ""
    }
    if ("mimeImage" in parsed) {
      mimeImage = String(parsed.mimeImage || "")
      if (!/^[A-Za-z0-9._-]+\.desktop$/.test(mimeImage)) mimeImage = ""
    }
    if ("mimeVideo" in parsed) {
      mimeVideo = String(parsed.mimeVideo || "")
      if (!/^[A-Za-z0-9._-]+\.desktop$/.test(mimeVideo)) mimeVideo = ""
    }
    if ("bluetooth" in parsed) bluetooth = parsed.bluetooth === true
    if ("wifiBandSelected" in parsed) wifiBandSelected = String(parsed.wifiBandSelected || "auto")
    if ("wifiRadio" in parsed) wifiRadio = parsed.wifiRadio === true
    if ("wifiHw" in parsed) wifiHw = parsed.wifiHw === true
    if ("wifiIface" in parsed) {
      wifiIface = String(parsed.wifiIface || "")
      if (!/^[a-zA-Z0-9._-]+$/.test(wifiIface)) wifiIface = ""
    }
    if ("wifiBand" in parsed) wifiBand = String(parsed.wifiBand || "")
    if ("wifiBands" in parsed) wifiBands = adoptArray(wifiBands, parsed.wifiBands)
    if ("netIface" in parsed) netIface = String(parsed.netIface || "")
    if ("netIp" in parsed) netIp = String(parsed.netIp || "")
    if ("netSpeed" in parsed) netSpeed = String(parsed.netSpeed || "")
    if ("netSignal" in parsed) netSignal = String(parsed.netSignal || "")
    if ("tailscaleInstalled" in parsed) tailscaleInstalled = parsed.tailscaleInstalled === true
    if ("tailscaleRunning" in parsed) tailscaleRunning = parsed.tailscaleRunning === true
    if ("tailscalePeers" in parsed) tailscalePeers = adoptArray(tailscalePeers, parsed.tailscalePeers)
    if ("disks" in parsed) disks = adoptArray(disks, parsed.disks)
    if ("luksDevices" in parsed) luksDevices = adoptArray(luksDevices, parsed.luksDevices)
    if ("swapDevices" in parsed) swapDevices = adoptArray(swapDevices, parsed.swapDevices)
    if ("snapshots" in parsed) snapshots = adoptArray(snapshots, parsed.snapshots)
    if ("snapperPresent" in parsed) snapperPresent = parsed.snapperPresent === true
    if ("snapperConfigs" in parsed) snapperConfigs = adoptArray(snapperConfigs, parsed.snapperConfigs)
    if ("hibernationAvailable" in parsed) hibernationAvailable = parsed.hibernationAvailable === true
    if ("hibernationSupported" in parsed) hibernationSupported = parsed.hibernationSupported === true
    if ("hibernationConfigured" in parsed) hibernationConfigured = parsed.hibernationConfigured === true
    if ("fstrimEnabled" in parsed) fstrimEnabled = parsed.fstrimEnabled === true
    if ("timezones" in parsed) timezones = adoptArray(timezones, parsed.timezones)
    if ("ntpAvailable" in parsed) ntpAvailable = parsed.ntpAvailable === true
    if ("locales" in parsed) locales = adoptArray(locales, parsed.locales)
    if ("keyboardLayouts" in parsed) keyboardLayouts = adoptArray(keyboardLayouts, parsed.keyboardLayouts)
    if ("audioOutputVolume" in parsed) {
      audioOutputVolume = Math.round(Number(parsed.audioOutputVolume)) || 0
      if (audioOutputVolume < 0) audioOutputVolume = 0
      if (audioOutputVolume > 100) audioOutputVolume = 100
    }
    if ("audioOutputMuted" in parsed) audioOutputMuted = parsed.audioOutputMuted === true
    if ("audioInputVolume" in parsed) {
      audioInputVolume = Math.round(Number(parsed.audioInputVolume)) || 0
      if (audioInputVolume < 0) audioInputVolume = 0
      if (audioInputVolume > 100) audioInputVolume = 100
    }
    if ("audioInputMuted" in parsed) audioInputMuted = parsed.audioInputMuted === true
    if ("audioSink" in parsed) audioSink = String(parsed.audioSink || "")
    if ("audioSource" in parsed) audioSource = String(parsed.audioSource || "")
    if ("audioTuningOn" in parsed) audioTuningOn = parsed.audioTuningOn === true
    if ("suspendEnabled" in parsed) suspendEnabled = parsed.suspendEnabled !== false
    if ("powerProfile" in parsed) powerProfile = String(parsed.powerProfile || "")
    if ("powerProfileAc" in parsed) powerProfileAc = String(parsed.powerProfileAc || "")
    if ("powerProfileBattery" in parsed) powerProfileBattery = String(parsed.powerProfileBattery || "")
    if ("powerShowPercentage" in parsed) powerShowPercentage = parsed.powerShowPercentage === true
    if ("crashCapture" in parsed) crashCapture = parsed.crashCapture !== false
    if ("doNotDisturb" in parsed) doNotDisturb = parsed.doNotDisturb === true
    if ("weatherLocation" in parsed) weatherLocation = String(parsed.weatherLocation || "")
    if ("weatherAuto" in parsed) weatherAuto = parsed.weatherAuto !== false
    if ("weatherCoords" in parsed) {
      weatherCoords = String(parsed.weatherCoords || "")
      if (!/^-?[0-9]+(\.[0-9]+)?,-?[0-9]+(\.[0-9]+)?$/.test(weatherCoords)) weatherCoords = ""
    }
    if ("weatherUnit" in parsed) {
      weatherUnit = String(parsed.weatherUnit || "auto")
      if (weatherUnit !== "metric" && weatherUnit !== "imperial") weatherUnit = "auto"
    }
    if ("weatherRefreshMinutes" in parsed) {
      weatherRefreshMinutes = Number(parsed.weatherRefreshMinutes) || 15
      if (weatherRefreshMinutes < 1) weatherRefreshMinutes = 15
    }
  }

  function refresh() {
    scheduleRefresh("all")
  }

  function scheduleRefresh(group) {
    pendingRefreshGroups = WorkQueue.addPendingRefresh(pendingRefreshGroups, group || "all")
    refreshTimer.restart()
  }

  property var pendingRefreshGroups: []

  function enqueueRead(group) {
    WorkQueue.enqueueRead(ioQueue, group || "all")
    kickIo()
  }

  function startSession(hub) {
    var first = WorkQueue.snapshotGroupForHub(hub)
    WorkQueue.enqueueRead(ioQueue, first)
    if (first !== "all") WorkQueue.enqueueRead(ioQueue, "rest")
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
      jobStdoutBuf = ""
      jobStdoutLineCb = typeof job.onStdoutLine === "function" ? job.onStdoutLine : null
      jobFinishedCb = typeof job.onFinished === "function" ? job.onFinished : null
      jobBusy = true
      if (jobKind === "wifi-qr") {
        wifiQrError = ""
        wifiQrRows = []
        wifiQrSize = 0
        wifiQrSsid = ""
      }
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

  function runCommand(argv, opts) {
    if (!(argv instanceof Array) || argv.length === 0) return
    opts = opts || {}
    enqueueIo({
      kind: "mut",
      argv: argv,
      key: opts.key ? String(opts.key) : "",
      apply: opts.apply && typeof opts.apply === "object" ? opts.apply : null,
      refresh: opts.refresh === "all" ? "all" : "none",
      sudo: opts.sudo === true
    })
  }

  function applyWritePatch(job) {
    if (!job || !job.apply) return
    applySnapshot(JSON.stringify(job.apply))
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
    hyprEmulateDiscreteScroll = Math.round(Number(input.emulateDiscreteScroll))
    if (!isFinite(hyprEmulateDiscreteScroll) || hyprEmulateDiscreteScroll < 0 || hyprEmulateDiscreteScroll > 2)
      hyprEmulateDiscreteScroll = 1
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
    if (Object.prototype.hasOwnProperty.call(input, "kbGroupToggle"))
      hyprKbGroupToggle = input.kbGroupToggle === true
    else
      hyprKbGroupToggle = String(input.kbOptions || "").indexOf("grp:alts_toggle") !== -1
    if (Object.prototype.hasOwnProperty.call(input, "workspaceGesture"))
      hyprWorkspaceGesture = input.workspaceGesture === true
  }

  function lookState(patch) {
    var look = {
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
    }
    if (patch && typeof patch === "object") {
      var k
      for (k in patch) {
        if (Object.prototype.hasOwnProperty.call(patch, k)) look[k] = patch[k]
      }
    }
    return look
  }

  function inputState(patch) {
    var input = {
      sensitivity: hyprSensitivity,
      accelProfile: hyprAccelProfile,
      emulateDiscreteScroll: hyprEmulateDiscreteScroll,
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
    }
    if (patch && typeof patch === "object") {
      var k
      for (k in patch) {
        if (Object.prototype.hasOwnProperty.call(patch, k)) input[k] = patch[k]
      }
    }
    input.kbLayout = input.kbLayoutOverride
    return input
  }

  function lookPayload() {
    return JSON.stringify(lookState(null))
  }

  function inputPayload() {
    return JSON.stringify(inputState(null))
  }

  function writeHyprLook(patch) {
    var look = lookState(patch)
    runCommand(["bash", setHyprLookScript, JSON.stringify(look)], {
      key: "hyprLook",
      apply: { hyprLook: look, hyprLookManaged: true },
      refresh: "none"
    })
  }

  function writeHyprInput(patch) {
    var input = inputState(patch)
    runCommand(["bash", setHyprInputScript, JSON.stringify(input)], {
      key: "hyprInput",
      apply: { hyprInput: input, hyprInputManaged: true },
      refresh: "none"
    })
  }

  function runGumJob(argv, kind, opts) {
    if (!(argv instanceof Array) || argv.length === 0) return
    var cmd = ["bash", "-c", "PATH=\"$1:$PATH\" exec \"$@\"", "prefs-job", gumStubDir]
    for (var i = 0; i < argv.length; i++) cmd.push(argv[i])
    runJob(cmd, "", kind, opts)
  }

  function enqueueIo(job) {
    if (!job) return
    if (job.sudo && !passwordlessSudo) {
      sudoPendingJob = job
      sudoError = ""
      sudoPromptOpen = true
      return
    }
    WorkQueue.enqueueWrite(ioQueue, job)
    kickIo()
  }

  function requestSudoMode() {
    sudoError = ""
    sudoPromptOpen = true
  }

  function confirmSudoMode(password) {
    password = String(password || "")
    if (!password || password.indexOf("\n") !== -1) {
      sudoError = "Password cannot be empty."
      sudoPromptOpen = true
      return
    }
    sudoError = ""
    if (passwordlessSudo) {
      sudoPromptOpen = false
      var pending = sudoPendingJob
      sudoPendingJob = null
      if (pending) {
        pending.sudo = false
        WorkQueue.enqueueWrite(ioQueue, pending)
        kickIo()
      }
      return
    }
    sudoEnabling = true
    enablePasswordlessSudo(sudoMinutes, password)
  }

  function cancelSudoMode() {
    sudoPromptOpen = false
    sudoPendingJob = null
    sudoEnabling = false
    sudoError = ""
  }

  function runJob(argv, stdinText, kind, opts) {
    if (!(argv instanceof Array) || argv.length === 0) return
    opts = opts || {}
    kind = String(kind || "")
    enqueueIo({
      kind: "job",
      argv: argv,
      stdin: String(stdinText || ""),
      jobKind: kind,
      key: opts.key ? String(opts.key) : kind,
      refresh: opts.refresh === "none" ? "none" : "all",
      sudo: opts.sudo === true,
      onStdoutLine: typeof opts.onStdoutLine === "function" ? opts.onStdoutLine : null,
      onFinished: typeof opts.onFinished === "function" ? opts.onFinished : null
    })
  }

  function cancelJob() {
    if (!jobProc.running) return
    jobProc.running = false
  }

  function commandFailureText(err, out) {
    var e = String(err || "").replace(/^\s+|\s+$/g, "")
    var o = String(out || "").replace(/^\s+|\s+$/g, "")
    if (e && o && o !== e) return e + "\n" + o
    return e || o
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
    Theme.applyNamedTheme(name)
    // Same detach as the shell "Switch theme" action: omarchy-theme-set
    // recolors the shell over IPC after the swap, then retints apps.
    runCommand(["bash", "-c", "omarchy theme set \"$1\" >/dev/null 2>&1 &", "theme-set", name], {
      key: "theme",
      apply: { theme: name },
      refresh: "none"
    })
  }
  function openThemeSwitcher() {
    runCommand(["bash", "-c", "theme=$(omarchy theme switcher || true); [[ -n $theme ]] && omarchy theme set \"$theme\" >/dev/null 2>&1 &"], {
      key: "theme",
      refresh: "none"
    })
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
    runCommand(["omarchy", "theme", "remove", name], {
      key: "theme-remove:" + name,
      apply: { extraThemes: SnapshotJs.patchRemoveMatching(extraThemes, "", name) },
      refresh: name === theme ? "all" : "none"
    })
  }
  function setBackgroundPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/") return
    runCommand(["omarchy", "theme", "bg", "set", path], {
      key: "background",
      apply: { background: path },
      refresh: "none"
    })
  }
  function nextBackground() {
    runCommand(["omarchy", "theme", "bg", "next"], {
      key: "background",
      refresh: "all"
    })
  }
  function openBackgroundSwitcher() {
    runCommand(["omarchy", "theme", "bg-switcher"], {
      key: "background",
      refresh: "all"
    })
  }
  function setBackgroundFromFile() {
    runCommand(["bash", "-c", "path=$(omarchy file select --title \"Set background\" --extensions \"jpg jpeg png gif webp bmp\" || true); [[ -n $path ]] && omarchy theme bg set \"$path\""], {
      key: "background",
      refresh: "all"
    })
  }
  function openBackgroundFolder() { runCommand(["omarchy", "theme", "bg", "install"]) }
  function cacheBackgrounds() { runCommand(["omarchy", "theme", "bg", "cache"]) }
  function setFont(name) {
    name = String(name || "")
    if (!name || name === font) return
    runCommand(["omarchy", "font", "set", name], {
      key: "font",
      apply: { font: name },
      refresh: "none"
    })
  }
  function setTextSize(size) {
    size = Math.round(Number(size))
    if (!isFinite(size) || size === textSize) return
    runCommand(["omarchy", "display", "text", "size", String(size)], {
      key: "textSize",
      apply: { textSize: size },
      refresh: "none"
    })
  }
  function resetTextSize() {
    runCommand(["omarchy", "display", "text", "size", "reset"], {
      key: "textSize",
      apply: { textSize: 12 },
      refresh: "none"
    })
  }
  function setMonitorScale(scale) {
    scale = String(scale || "")
    if (!scale) return
    var n = Number(scale)
    if (!isFinite(n) || n <= 0) return
    var list = monitors instanceof Array ? monitors : []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && list[i].focused === true) {
        if (Number(list[i].scale) === n) return
        break
      }
    }
    runCommand(["omarchy", "hyprland", "monitor", "scaling", scale], {
      key: "monitorScale",
      apply: { monitors: SnapshotJs.patchFocusedMonitorScale(monitors, n) },
      refresh: "none"
    })
  }
  function setDisplayBrightness(name, percent) {
    name = String(name || "")
    percent = Math.round(Number(percent))
    if (!/^[A-Za-z0-9._-]+$/.test(name)) return
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    runCommand(
      ["omarchy", "brightness", "display", "--no-osd", "--monitor", name, percent + "%"],
      {
        key: "brightness:" + name,
        apply: { monitors: SnapshotJs.patchMonitorBrightness(monitors, name, percent) },
        refresh: "none"
      }
    )
  }
  function setInternalDisplay(on) {
    if (on === internalEnabled) return
    runCommand(["omarchy", "hyprland", "monitor", "internal", on ? "on" : "off"], {
      key: "internalEnabled",
      apply: { internalEnabled: on },
      refresh: "none"
    })
  }
  function setInternalMirror(on) {
    if (on === mirroring) return
    runCommand(["omarchy", "hyprland", "monitor", "internal", "mirror", on ? "on" : "off"], {
      key: "mirroring",
      apply: { mirroring: on },
      refresh: "none"
    })
  }
  function setTouchpad(on) {
    if (on === touchpadEnabled) return
    runCommand(["omarchy", "toggle", "touchpad", on ? "on" : "off"], {
      key: "touchpadEnabled",
      apply: { touchpadEnabled: on },
      refresh: "none"
    })
  }
  function setTouchscreen(on) {
    if (on === touchscreenEnabled) return
    runCommand(["omarchy", "toggle", "touchscreen", on ? "on" : "off"], {
      key: "touchscreenEnabled",
      apply: { touchscreenEnabled: on },
      refresh: "none"
    })
  }
  function adjustKeyboardBacklight(direction) {
    if (direction !== "up" && direction !== "down" && direction !== "off" && direction !== "restore") return
    if (direction === "restore") {
      runCommand(["omarchy", "brightness", "keyboard", "--no-osd", "restore"], {
        key: "keyboardBrightness",
        refresh: "all"
      })
      return
    }
    var next = SnapshotJs.patchKeyboardBrightness(keyboardBrightness, direction)
    if (next === keyboardBrightness) return
    runCommand(["omarchy", "brightness", "keyboard", "--no-osd", direction], {
      key: "keyboardBrightness",
      apply: { keyboardBrightness: next },
      refresh: "none"
    })
  }
  function setBarPosition(position) {
    if (!position || position === barPosition) return
    runCommand(["omarchy", "bar", "position", position], {
      key: "barPosition",
      apply: { barPosition: position },
      refresh: "none"
    })
  }
  function setBarTransparent(on) {
    if (on === barTransparent) return
    runCommand(["omarchy", "bar", "transparent", on ? "true" : "false"], {
      key: "barTransparent",
      apply: { barTransparent: on },
      refresh: "none"
    })
  }
  // `omarchy toggle bar on` sets the bar-off flag and hides the bar.
  function setBarVisible(on) {
    if (on === barVisible) return
    runCommand(["omarchy", "toggle", "bar", on ? "off" : "on"], {
      key: "barVisible",
      apply: { barVisible: on },
      refresh: "none"
    })
  }
  function setClockFormat(fmt) {
    if (!fmt || fmt === clockFormat) return
    var key = (barPosition === "left" || barPosition === "right") ? "verticalFormat" : "format"
    runCommand(["omarchy", "bar", "set", "omarchy.clock", key, fmt], {
      key: "clockFormat",
      apply: { clockFormat: fmt },
      refresh: "none"
    })
  }
  function setClockFormatAlt(fmt) {
    if (!fmt || fmt === clockFormatAlt) return
    var key = (barPosition === "left" || barPosition === "right") ? "verticalFormatAlt" : "formatAlt"
    runCommand(["omarchy", "bar", "set", "omarchy.clock", key, fmt], {
      key: "clockFormatAlt",
      apply: { clockFormatAlt: fmt },
      refresh: "none"
    })
  }
  function setClockWeekStart(day) {
    day = String(day || "").toLowerCase()
    if (day !== "sunday" && day !== "monday" && day !== "tuesday" && day !== "wednesday" && day !== "thursday" && day !== "friday" && day !== "saturday") return
    if (day === clockWeekStart) return
    runCommand(["omarchy", "bar", "set", "omarchy.clock", "weekStartDay", day], {
      key: "clockWeekStart",
      apply: { clockWeekStart: day },
      refresh: "none"
    })
  }
  function setClockBirthYear(year) {
    if (typeof year === "number") year = String(Math.round(year))
    year = String(year || "").replace(/^\s+|\s+$/g, "")
    if (year.length === 0 || year === "0") {
      if (clockBirthYear === 0) return
      runCommand(["omarchy", "bar", "set", "omarchy.clock", "birthYear", "0", "--json"], {
        key: "clockBirthYear",
        apply: { clockBirthYear: 0 },
        refresh: "none"
      })
      return
    }
    if (!/^\d{4}$/.test(year)) return
    var born = parseInt(year, 10)
    var now = new Date().getFullYear()
    if (!(born >= now - 120 && born <= now)) return
    if (born === clockBirthYear) return
    runCommand(["omarchy", "bar", "set", "omarchy.clock", "birthYear", String(born), "--json"], {
      key: "clockBirthYear",
      apply: { clockBirthYear: born },
      refresh: "none"
    })
  }
  function setClockLifeExpectancy(years) {
    if (typeof years === "number") years = String(Math.round(years))
    years = String(years || "").replace(/^\s+|\s+$/g, "")
    if (years.length === 0 || years === "0") {
      if (clockLifeExpectancy === 0) return
      runCommand(["omarchy", "bar", "set", "omarchy.clock", "lifeExpectancy", "0", "--json"], {
        key: "clockLifeExpectancy",
        apply: { clockLifeExpectancy: 0 },
        refresh: "none"
      })
      return
    }
    if (!/^\d+$/.test(years)) return
    var span = parseInt(years, 10)
    if (!(span >= 1 && span <= 150)) return
    if (span === clockLifeExpectancy) return
    runCommand(["omarchy", "bar", "set", "omarchy.clock", "lifeExpectancy", String(span), "--json"], {
      key: "clockLifeExpectancy",
      apply: { clockLifeExpectancy: span },
      refresh: "none"
    })
  }
  function setIndicatorsAlwaysShow(on) {
    if (on === indicatorsAlwaysShow) return
    runCommand(["omarchy", "bar", "set", "omarchy.indicators", "alwaysShow", on ? "true" : "false", "--json"], {
      key: "indicatorsAlwaysShow",
      apply: { indicatorsAlwaysShow: on },
      refresh: "none"
    })
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
    runCommand(["bash", setBarWidgetScript, "omarchy.indicators", "items", JSON.stringify(next)], {
      key: "indicatorsItems",
      apply: { indicatorsItems: next },
      refresh: "none"
    })
  }
  function setAgentsRefreshIntervalSec(seconds) {
    seconds = Math.round(Number(seconds))
    if (!(seconds >= 30) || seconds === agentsRefreshIntervalSec) return
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "refreshIntervalSec", String(seconds), "--json"], {
      key: "agentsRefreshIntervalSec",
      apply: { agentsRefreshIntervalSec: seconds },
      refresh: "none"
    })
  }
  function setAgentsSync(on) {
    if (on === agentsSync) return
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncMode", on ? "On" : "Off"], {
      key: "agentsSync",
      apply: { agentsSync: on },
      refresh: "none"
    })
  }
  function setAgentsSyncDir(path) {
    path = String(path || "").replace(/^\s+|\s+$/g, "")
    if (path === agentsSyncDir) return
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncDir", path], {
      key: "agentsSyncDir",
      apply: { agentsSyncDir: path },
      refresh: "none"
    })
  }
  function setAgentsSyncFileName(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "").split("/").pop()
    if (name === agentsSyncFileName) return
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncFileName", name], {
      key: "agentsSyncFileName",
      apply: { agentsSyncFileName: name },
      refresh: "none"
    })
  }
  function setAgentsSyncDeviceId(id) {
    id = String(id || "").replace(/^\s+|\s+$/g, "")
    if (id === agentsSyncDeviceId) return
    runCommand(["omarchy", "bar", "set", "omarchy.agents", "syncDeviceId", id], {
      key: "agentsSyncDeviceId",
      apply: { agentsSyncDeviceId: id },
      refresh: "none"
    })
  }
  function setSpacerSize(size) {
    size = Math.round(Number(size))
    if (!isFinite(size) || size < 0 || size > 64 || size === spacerSize) return
    runCommand(["omarchy", "bar", "set", "omarchy.spacer", "size", String(size), "--json"], {
      key: "spacerSize",
      apply: { spacerSize: size },
      refresh: "none"
    })
  }
  function addSpacer() {
    if (spacerPresent) return
    runCommand(["omarchy", "bar", "put", "omarchy.spacer"], {
      key: "spacerPresent",
      apply: { spacerPresent: true },
      refresh: "none"
    })
  }
  function removeSpacer() {
    if (!spacerPresent) return
    runCommand(["omarchy", "plugin", "disable", "omarchy.spacer"], {
      key: "spacerPresent",
      apply: { spacerPresent: false },
      refresh: "none"
    })
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
    runCommand(["omarchy", "remove", "launcher", "entry", id, name], {
      key: "desktop-remove:" + id,
      apply: { desktopApps: SnapshotJs.patchRemoveMatching(desktopApps, "id", id) },
      refresh: "none",
      sudo: true
    })
  }
  function removeTui(name) {
    name = String(name || "")
    if (!name) return
    runCommand(["omarchy", "tui", "remove", name], {
      key: "tui-remove:" + name,
      apply: {
        tuiApps: SnapshotJs.patchRemoveMatching(
          SnapshotJs.patchRemoveMatching(tuiApps, "id", name),
          "name",
          name
        )
      },
      refresh: "none"
    })
  }
  function removeWebApp(name) {
    name = String(name || "")
    if (!name) return
    runCommand(["omarchy", "webapp", "remove", name], {
      key: "webapp-remove:" + name,
      apply: {
        webApps: SnapshotJs.patchRemoveMatching(
          SnapshotJs.patchRemoveMatching(webApps, "id", name),
          "name",
          name
        )
      },
      refresh: "none"
    })
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
    runCommand(["bash", setBarWidgetScript, "omarchy.tray", "hidden", JSON.stringify(next)], {
      key: "trayHidden",
      apply: { trayHidden: next },
      refresh: "none"
    })
  }
  function clearTrayHidden() {
    setTrayHidden([])
  }
  function setTrayPinned(list) {
    var next = normalizedStringIds(list)
    var current = trayPinned instanceof Array ? trayPinned : []
    if (JSON.stringify(next) === JSON.stringify(current)) return
    runCommand(["bash", setBarWidgetScript, "omarchy.tray", "pinned", JSON.stringify(next)], {
      key: "trayPinned",
      apply: { trayPinned: next },
      refresh: "none"
    })
  }
  function clearTrayPinned() {
    setTrayPinned([])
  }
  function setBrowser(name) {
    if (!name || name === browser) return
    runCommand(["omarchy", "default", "browser", name], {
      key: "browser",
      apply: { browser: name },
      refresh: "none"
    })
  }
  function setTerminal(name) {
    if (!name || name === terminal) return
    runCommand(["omarchy", "default", "terminal", name], {
      key: "terminal",
      apply: { terminal: name },
      refresh: "none"
    })
  }
  function setEditor(name) {
    if (!name || name === editor) return
    runCommand(["omarchy", "default", "editor", name], {
      key: "editor",
      apply: { editor: name },
      refresh: "none"
    })
  }
  function setAgent(name) {
    if (!name || name === agent) return
    runCommand(["omarchy", "default", "agent", name], {
      key: "agent",
      apply: { agent: name },
      refresh: "none"
    })
  }
  function setDns(name) {
    if (name !== "Cloudflare" && name !== "Google" && name !== "DHCP") return
    if (name === dns) return
    runCommand(["omarchy", "dns", name], {
      key: "dns",
      apply: { dns: name },
      refresh: "none"
    })
  }
  function setCustomDns(servers) {
    servers = String(servers || "").replace(/^\s+|\s+$/g, "")
    if (!servers) return
    runJob(["bash", setDnsCustomScript, servers], "", "dns-custom")
  }
  function openAether() { runCommand(["aether"]) }

  function setIdle(screensaver, lock) {
    var saver = Math.round(Number(screensaver)) || 0
    var lockSec = Math.round(Number(lock)) || 0
    runCommand(["bash", setIdleScript, String(screensaver), String(lock)], {
      key: "idle",
      apply: { idleScreensaver: saver, idleLock: lockSec },
      refresh: "none"
    })
  }

  function setStayAwake(on) {
    if (on === stayAwake) return
    runCommand(["omarchy", "toggle", "idle", on ? "stay-awake" : "allow-idle"], {
      key: "stayAwake",
      apply: { stayAwake: on },
      refresh: "none"
    })
  }

  function setNightlight(on) {
    if (on === nightlight) return
    runCommand(["omarchy", "toggle", "nightlight"], {
      key: "nightlight",
      apply: { nightlight: on },
      refresh: "none"
    })
  }

  function setScreensaverEnabled(on) {
    if (on === screensaverEnabled) return
    runCommand(["omarchy", "toggle", "screensaver-off", on ? "off" : "on"], {
      key: "screensaverEnabled",
      apply: { screensaverEnabled: on },
      refresh: "none"
    })
  }

  function setScreensaverBranding(action) {
    if (action !== "image" && action !== "text" && action !== "reset") return
    if (action === "reset") {
      if (!screensaverBranded) return
      runCommand(["omarchy", "branding", "screensaver", "reset"], {
        key: "screensaverBranding",
        apply: { screensaverBranded: false },
        refresh: "none"
      })
      return
    }
    runCommand(["omarchy", "branding", "screensaver", action], {
      key: "screensaverBranding",
      refresh: "all"
    })
  }

  function setAboutBranding(action) {
    if (action !== "image" && action !== "text" && action !== "reset") return
    if (action === "reset") {
      if (!aboutBranded) return
      runCommand(["omarchy", "branding", "about", "reset"], {
        key: "aboutBranding",
        apply: { aboutBranded: false },
        refresh: "none"
      })
      return
    }
    runCommand(["omarchy", "branding", "about", action], {
      key: "aboutBranding",
      refresh: "all"
    })
  }

  function setTimezone(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === timezone) return
    if (!/^[A-Za-z0-9/_+-]+$/.test(name) || name.indexOf("..") !== -1) return
    runCommand(["bash", setTimezoneScript, name], {
      key: "timezone",
      apply: { timezone: name },
      refresh: "none",
      sudo: true
    })
  }

  function setNtp(on) {
    if (on === ntp) return
    runCommand(["bash", setNtpScript, on ? "true" : "false"], {
      key: "ntp",
      apply: { ntp: on, ntpSynchronized: on ? ntpSynchronized : false },
      refresh: "none",
      sudo: true
    })
  }

  function setHostname(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === hostname) return
    if (name.length > 253) return
    if (!/^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)*$/.test(name)) return
    runCommand(["bash", setHostnameScript, name], {
      key: "hostname",
      apply: { hostname: name },
      refresh: "none",
      sudo: true
    })
  }

  function setFullName(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (name === fullName) return
    if (!AccountsJs.isFullName(name)) return
    runCommand(["bash", setFullNameScript, name], {
      key: "fullName",
      apply: { fullName: name },
      refresh: "none",
      sudo: true
    })
  }

  function setAvatarPath(path) {
    path = String(path || "")
    if (!currentUser) return
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runCommand(["bash", setAvatarScript, "set", currentUser, path], {
      key: "avatar",
      apply: { avatarPath: path },
      refresh: "none",
      sudo: true
    })
  }

  function clearAvatar() {
    if (!currentUser) return
    runCommand(["bash", setAvatarScript, "clear", currentUser], {
      key: "avatar",
      apply: { avatarPath: "" },
      refresh: "none",
      sudo: true
    })
  }

  function addAccountUser(name, full, password, wheel) {
    name = AccountsJs.parseUsername(name)
    full = String(full || "").replace(/^\s+|\s+$/g, "")
    password = String(password || "")
    if (!name || !password || password.indexOf("\n") !== -1) return
    if (!AccountsJs.isFullName(full)) return
    runJob(["bash", manageAccountScript, "add-user", name, full, wheel === true ? "true" : "false"], password + "\n", "account-add", { sudo: true })
  }

  function removeAccountUser(name) {
    name = AccountsJs.parseUsername(name)
    if (!name || name === currentUser) return
    runJob(["bash", manageAccountScript, "remove-user", name], "", "account-remove", { sudo: true })
  }

  function setAccountPassword(name, password) {
    name = AccountsJs.parseUsername(name)
    password = String(password || "")
    if (!name || !password || password.indexOf("\n") !== -1) return
    runJob(["bash", manageAccountScript, "set-password", name], password + "\n", "account-password", { sudo: true })
  }

  function addAccountGroup(name) {
    name = AccountsJs.parseGroupName(name)
    if (!name) return
    runJob(["bash", manageAccountScript, "add-group", name], "", "account-group-add", { sudo: true })
  }

  function removeAccountGroup(name) {
    name = AccountsJs.parseGroupName(name)
    if (!name || name === "wheel" || name === "docker") return
    runJob(["bash", manageAccountScript, "remove-group", name], "", "account-group-remove", { sudo: true })
  }

  function setGroupMember(group, name, on) {
    group = AccountsJs.parseGroupName(group)
    name = AccountsJs.parseUsername(name)
    if (!group || !name) return
    if (on !== true && group === "wheel" && name === currentUser) return
    runCommand(["bash", manageAccountScript, "set-member", group, name, on === true ? "on" : "off"], {
      key: "account-member-" + group + "-" + name,
      refresh: "all",
      sudo: true
    })
  }

  function setKeyboardLayout(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (name.indexOf(",") !== -1) name = name.split(",")[0]
    if (!name || name === keyboardLayout) return
    if (!/^[a-z0-9]{1,8}$/.test(name)) return
    runCommand(["bash", setKeyboardLayoutScript, name], {
      key: "keyboardLayout",
      apply: { keyboardLayout: name },
      refresh: "none",
      sudo: true
    })
  }

  function setLocale(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === locale) return
    if (name !== "C.UTF-8" && !/^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(name)) return
    runCommand(["bash", setLocaleScript, name], {
      key: "locale",
      apply: { locale: name },
      refresh: "none",
      sudo: true
    })
  }

  function setParallelDownloads(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 1 || n > 20 || n === parallelDownloads) return
    runCommand(["bash", setParallelDownloadsScript, String(n)], {
      key: "parallelDownloads",
      apply: { parallelDownloads: n },
      refresh: "none",
      sudo: true
    })
  }

  function setHyprGapsIn(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 64 || n === hyprGapsIn) return
    writeHyprLook({ gapsIn: n })
  }
  function setHyprGapsOut(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 64 || n === hyprGapsOut) return
    writeHyprLook({ gapsOut: n })
  }
  function setHyprBorderSize(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 16 || n === hyprBorderSize) return
    writeHyprLook({ borderSize: n })
  }
  function setHyprRounding(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 32 || n === hyprRounding) return
    writeHyprLook({ rounding: n })
  }
  function setHyprBlur(on) {
    if (on === hyprBlur) return
    writeHyprLook({ blur: on })
  }
  function setHyprShadow(on) {
    if (on === hyprShadow) return
    writeHyprLook({ shadow: on })
  }
  function setHyprLayout(name) {
    if (name !== "dwindle" && name !== "scrolling") return
    if (name === hyprLayout) return
    writeHyprLook({ layout: name })
  }
  function setHyprColumnWidth(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0.2 || n > 1 || n === hyprColumnWidth) return
    writeHyprLook({ columnWidth: n })
  }
  function setHyprDimInactive(on) {
    if (on === hyprDimInactive) return
    writeHyprLook({ dimInactive: on })
  }
  function setHyprDimStrength(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0 || n > 1 || n === hyprDimStrength) return
    writeHyprLook({ dimStrength: n })
  }
  function setHyprAnimations(on) {
    if (on === hyprAnimations) return
    writeHyprLook({ animations: on })
  }
  function setHyprCursorHideOnKey(on) {
    if (on === hyprCursorHideOnKey) return
    writeHyprLook({ cursorHideOnKey: on })
  }
  function setHyprCursorWarp(on) {
    if (on === hyprCursorWarp) return
    writeHyprLook({ cursorWarp: on })
  }
  function setHyprAllowTearing(on) {
    if (on === hyprAllowTearing) return
    writeHyprLook({ allowTearing: on })
  }
  function setHyprResizeOnBorder(on) {
    if (on === hyprResizeOnBorder) return
    writeHyprLook({ resizeOnBorder: on })
  }
  function setHyprCursorSize(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 8 || n > 64 || n === hyprCursorSize) return
    writeHyprLook({ cursorSize: n })
  }
  function setHyprActiveOpacity(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0.2 || n > 1 || n === hyprActiveOpacity) return
    writeHyprLook({ activeOpacity: n })
  }
  function setHyprPreserveSplit(on) {
    if (on === hyprPreserveSplit) return
    writeHyprLook({ preserveSplit: on })
  }
  function setHyprFocusOnActivate(on) {
    if (on === hyprFocusOnActivate) return
    writeHyprLook({ focusOnActivate: on })
  }
  function resetHyprLook() {
    if (!hyprLookManaged) return
    runCommand(["bash", setHyprLookScript, "--reset"], {
      key: "hyprLookManaged",
      apply: { hyprLookManaged: false },
      refresh: "none"
    })
  }
  function setHyprNoGaps(on) {
    if (on === hyprNoGaps) return
    runCommand(["omarchy", "hyprland", "toggle", "window-no-gaps", on ? "on" : "off"], {
      key: "hyprNoGaps",
      apply: { hyprNoGaps: on },
      refresh: "none"
    })
  }
  function setHyprSquareAspect(on) {
    if (on === hyprSquareAspect) return
    runCommand(["omarchy", "hyprland", "toggle", "single-window-aspect-ratio", on ? "on" : "off"], {
      key: "hyprSquareAspect",
      apply: { hyprSquareAspect: on },
      refresh: "none"
    })
  }
  function toggleWorkspaceLayout() {
    var next = hyprWorkspaceLayout === "scrolling" ? "dwindle" : "scrolling"
    runCommand(["omarchy", "hyprland", "workspace", "layout", "toggle"], {
      key: "hyprWorkspaceLayout",
      apply: { hyprWorkspaceLayout: next },
      refresh: "none"
    })
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
    writeHyprInput({ sensitivity: n })
  }
  function setHyprAccelProfile(name) {
    if (name !== "flat" && name !== "adaptive" && name !== "") return
    if (name === hyprAccelProfile) return
    writeHyprInput({ accelProfile: name })
  }
  function setHyprEmulateDiscreteScroll(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 2 || n === hyprEmulateDiscreteScroll) return
    writeHyprInput({ emulateDiscreteScroll: n })
  }
  function setHyprNaturalScroll(on) {
    if (on === hyprNaturalScroll) return
    writeHyprInput({ naturalScroll: on })
  }
  function setHyprScrollFactor(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0.1 || n > 3 || n === hyprScrollFactor) return
    writeHyprInput({ scrollFactor: n })
  }
  function setHyprClickfinger(on) {
    if (on === hyprClickfinger) return
    writeHyprInput({ clickfinger: on })
  }
  function setHyprDisableWhileTyping(on) {
    if (on === hyprDisableWhileTyping) return
    writeHyprInput({ disableWhileTyping: on })
  }
  function setHyprDrag3fg(on) {
    var n = on ? 1 : 0
    if (n === hyprDrag3fg) return
    writeHyprInput({ drag3fg: n })
  }
  function setHyprRepeatRate(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 10 || n > 100 || n === hyprRepeatRate) return
    writeHyprInput({ repeatRate: n })
  }
  function setHyprRepeatDelay(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 100 || n > 1000 || n === hyprRepeatDelay) return
    writeHyprInput({ repeatDelay: n })
  }
  function setHyprNumlock(on) {
    if (on === hyprNumlock) return
    writeHyprInput({ numlock: on })
  }
  function setHyprFollowMouse(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 0 || n > 3 || n === hyprFollowMouse) return
    writeHyprInput({ followMouse: n })
  }
  function setHyprKeyPressDpms(on) {
    if (on === hyprKeyPressDpms) return
    writeHyprInput({ keyPressDpms: on })
  }
  function setHyprMouseMoveDpms(on) {
    if (on === hyprMouseMoveDpms) return
    writeHyprInput({ mouseMoveDpms: on })
  }
  function setHyprKbOverride(layouts, variants, groupToggle) {
    layouts = String(layouts || "").replace(/^\s+|\s+$/g, "").toLowerCase()
    variants = String(variants || "").replace(/^\s+|\s+$/g, "")
    if (layouts && !/^[a-z0-9]{1,8}(,[a-z0-9]{1,8})*$/.test(layouts)) return
    writeHyprInput({
      kbLayoutOverride: layouts,
      kbVariantOverride: layouts ? variants : "",
      kbGroupToggle: groupToggle === true
    })
  }
  function setHyprWorkspaceGesture(on) {
    if (on === hyprWorkspaceGesture) return
    writeHyprInput({ workspaceGesture: on })
  }
  function resetHyprInput() {
    if (!hyprInputManaged) return
    runCommand(["bash", setHyprInputScript, "--reset"], {
      key: "hyprInputManaged",
      apply: { hyprInputManaged: false },
      refresh: "none"
    })
  }

  function setNightlightTemperature(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 3000 || n > 6500) return
    if (n === nightlightTemperature) return
    runCommand(["bash", setNightlightTempScript, String(n)], {
      key: "nightlightTemperature",
      apply: { nightlightTemperature: n, nightlight: n < 6000 },
      refresh: "none"
    })
  }

  function setupFingerprint() {
    runGumJob(["omarchy", "setup", "security", "fingerprint"], "security-fingerprint", { sudo: true })
  }
  function removeFingerprint() {
    if (!fingerprintConfigured) return
    runGumJob(["omarchy", "remove", "security", "fingerprint"], "security-fingerprint-remove", { sudo: true })
  }
  function setupFido2() {
    runGumJob(["omarchy", "setup", "security", "fido2"], "security-fido2", { sudo: true })
  }
  function removeFido2() {
    if (!fido2Configured) return
    runGumJob(["omarchy", "remove", "security", "fido2"], "security-fido2-remove", { sudo: true })
  }
  function setupSshd(key) {
    key = String(key || "").replace(/^\s+|\s+$/g, "")
    if (!key || key.length > 8192) return
    if (key.indexOf("\n") !== -1) return
    runGumJob(["omarchy", "setup", "security", "sshd", "--key=" + key], "security-sshd", { sudo: true })
  }
  function disableSshd() {
    if (!sshdEnabled && !sshdActive) return
    runJob(["bash", setSshdScript, "disable"], "", "security-sshd-disable", { sudo: true })
  }
  function enablePasswordlessSudo(minutes, password) {
    minutes = Math.round(Number(minutes))
    if (!isFinite(minutes) || minutes < 1 || minutes > 240) minutes = 15
    password = String(password || "")
    if (!password) {
      requestSudoMode()
      return
    }
    if (password.indexOf("\n") !== -1) return
    sudoEnabling = true
    runJob(
      ["bash", "-c", "export ATMOS_SUDO_ASK=1; exec \"$1\" on \"$2\"", "atmos-sudo", setPasswordlessSudoScript, String(minutes)],
      password + "\n",
      "passwordless-sudo"
    )
  }
  function disablePasswordlessSudo() {
    if (!passwordlessSudo) return
    runJob(["bash", setPasswordlessSudoScript, "off"], "", "passwordless-sudo-off", { sudo: true })
  }
  function setupSudolessDocker() {
    runGumJob(["omarchy", "setup", "security", "sudoless", "docker"], "security-docker", { sudo: true })
  }
  function removeSudolessDocker() {
    if (!sudolessDocker) return
    runGumJob(["omarchy", "remove", "security", "sudoless", "docker"], "security-docker-remove", { sudo: true })
  }

  function setOmarchyChannel(name) {
    if (name !== "stable" && name !== "rc" && name !== "edge" && name !== "dev") return
    if (name === omarchyChannel) return
    runGumJob(["omarchy", "channel", "set", name], "channel-set", { sudo: true })
  }
  function runOmarchyUpdate() {
    runGumJob(["omarchy", "update"], "omarchy-update", { sudo: true })
  }
  function checkOmarchyUpdate() {
    runJob(["omarchy", "update", "available"], "", "update-check")
  }
  function setAtmosChannel(name) {
    if (AtmosUpdate.parseChannel(name) !== "alpha") return
    if (name === atmosChannel) return
    runCommand(["bash", setAtmosChannelScript, "alpha"], {
      key: "atmosChannel",
      apply: { atmosChannel: "alpha" },
      refresh: "none"
    })
  }
  function checkAtmosUpdate() {
    runJob(["bash", updateAtmosScript, "check"], "", "atmos-update-check")
  }
  function runAtmosUpdate() {
    runJob(["bash", updateAtmosScript, "apply"], "", "atmos-update")
  }
  function updateFirmware() {
    runGumJob(["omarchy", "update", "firmware"], "update-firmware", { sudo: true })
  }
  function updateOrphanPkgs() {
    runGumJob(["omarchy", "update", "orphan", "pkgs"], "update-orphans", { sudo: true })
  }
  function prunePkgCache() {
    runGumJob(["omarchy", "update", "pkg", "prune"], "update-prune", { sudo: true })
  }

  function installVoxtype() {
    runGumJob(["omarchy", "voxtype", "install"], "voxtype-install", { sudo: true })
  }
  function removeVoxtype() {
    if (!voxtypeInstalled) return
    runGumJob(["omarchy", "voxtype", "remove"], "voxtype-remove", { sudo: true })
  }
  function toggleHybridGpu() {
    if (!hybridGpuAvailable) return
    runGumJob(["omarchy", "toggle", "hybrid", "gpu"], "hybrid-gpu", { sudo: true })
  }
  function installTailscale() {
    runGumJob(["omarchy", "install", "service", "tailscale"], "tailscale-install", { sudo: true })
  }
  function removeTailscale() {
    if (!tailscaleInstalled) return
    runGumJob(["omarchy", "remove", "service", "tailscale"], "tailscale-remove", { sudo: true })
  }
  function setPluginEnabled(id, on) {
    id = String(id || "")
    if (!/^[A-Za-z0-9._-]+$/.test(id)) return
    var list = plugins instanceof Array ? plugins : []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].id) === id) {
        if ((list[i].enabled === true) === (on === true)) return
        break
      }
    }
    runCommand(["omarchy", "plugin", on ? "enable" : "disable", id], {
      key: "plugin:" + id,
      apply: { plugins: SnapshotJs.patchPluginEnabled(plugins, id, on === true) },
      refresh: "none"
    })
  }
  function setSnapperNumberLimit(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 1 || n > 50 || n === snapperNumberLimit) return
    runCommand(["bash", setSnapperPolicyScript, "number-limit", String(n)], {
      key: "snapperNumberLimit",
      apply: { snapperNumberLimit: n },
      refresh: "none",
      sudo: true
    })
  }
  function setSnapperTimeline(on) {
    if (on === snapperTimeline) return
    runCommand(["bash", setSnapperPolicyScript, "timeline", on ? "on" : "off"], {
      key: "snapperTimeline",
      apply: { snapperTimeline: on },
      refresh: "none",
      sudo: true
    })
  }
  function setFstrim(on) {
    if (on === fstrimEnabled) return
    runCommand(["bash", setFstrimScript, on ? "on" : "off"], {
      key: "fstrimEnabled",
      apply: { fstrimEnabled: on },
      refresh: "none",
      sudo: true
    })
  }
  function setupDirectBoot() {
    if (!directBootAvailable) return
    runGumJob(["omarchy", "setup", "direct", "boot"], "direct-boot", { sudo: true })
  }
  function setMimeDefault(kind, desktop) {
    if (kind !== "pdf" && kind !== "image" && kind !== "video") return
    desktop = String(desktop || "")
    if (!/^[A-Za-z0-9._-]+\.desktop$/.test(desktop)) return
    var patch = {}
    if (kind === "pdf") patch.mimePdf = desktop
    else if (kind === "image") patch.mimeImage = desktop
    else patch.mimeVideo = desktop
    runCommand(["bash", setMimeDefaultScript, kind, desktop], {
      key: "mime:" + kind,
      apply: patch,
      refresh: "none"
    })
  }

  function setBluetooth(on) {
    if (on === bluetooth) return
    runCommand(["omarchy", "bluetooth", "power", on ? "on" : "off"], {
      key: "bluetooth",
      apply: { bluetooth: on },
      refresh: "none"
    })
  }

  function setWifiBand(band) {
    if (band !== "auto" && band !== "2.4" && band !== "5" && band !== "6") return
    if (band === wifiBandSelected) return
    runCommand(["omarchy", "network", "band", band], {
      key: "wifiBandSelected",
      apply: { wifiBandSelected: band },
      refresh: "none"
    })
  }

  function copyWifiPassword() {
    if (!wifiIface || !/^[a-zA-Z0-9._-]+$/.test(wifiIface)) return
    runCommand(["bash", "-c", "omarchy network password \"$1\" | wl-copy -n", "wifi-password", wifiIface])
  }
  function fetchWifiQr() {
    var argv = ["omarchy", "network", "qr", "--meta"]
    if (wifiIface && /^[a-zA-Z0-9._-]+$/.test(wifiIface)) argv.push(wifiIface)
    runJob(argv, "", "wifi-qr", { refresh: "none" })
  }
  function applyWifiQr(exitCode, out, err) {
    lastError = ""
    if (exitCode !== 0) {
      wifiQrError = String(err || "Could not build a QR code").replace(/^\s+|\s+$/g, "")
      wifiQrRows = []
      wifiQrSize = 0
      wifiQrSsid = ""
      return
    }
    var parsed = RichUi.parseQrOutput(out)
    if (!parsed.ok) {
      wifiQrError = parsed.error
      wifiQrRows = []
      wifiQrSize = 0
      wifiQrSsid = ""
      return
    }
    wifiQrError = ""
    wifiQrSsid = String(parsed.ssid || "")
    wifiQrRows = parsed.rows
    wifiQrSize = parsed.size
  }
  function copyText(text) {
    text = RichUi.clipboardPayload(text, { singleLine: true, maxLength: 1024 })
    if (!text) return
    runCommand(["bash", "-c", "printf '%s' \"$1\" | wl-copy -n", "copy-text", text])
  }

  function copyLastError() {
    var text = RichUi.clipboardPayload(lastError, { maxLength: 8192 })
    if (!text) return
    runCommand(["bash", "-c", "printf '%s' \"$1\" | wl-copy -n", "copy-text", text])
  }

  function clearLastError() {
    lastError = ""
  }

  function askAgentAboutError() {
    var prompt = RichUi.agentErrorPrompt(lastError)
    if (!prompt) return
    runCommand(["bash", "-c", "omarchy agent prompt \"$1\" >/dev/null 2>&1 &", "agent-prompt", prompt])
    lastError = ""
  }

  function showDebugError() {
    lastError = "Debug: this is the error banner. Copy puts it on the clipboard. Dismiss clears it."
  }
  function setWifiRadio(on) {
    if (on === wifiRadio) return
    runCommand(["bash", setWifiConnectionScript, "radio", on ? "on" : "off"], {
      key: "wifiRadio",
      apply: { wifiRadio: on },
      refresh: "none"
    })
  }
  function connectEnterpriseWifi(ssid, identity, password) {
    ssid = String(ssid || "")
    identity = String(identity || "")
    password = String(password || "")
    if (!ssid || !identity || !password) return
    if (ssid.length > 64 || identity.length > 256 || password.length > 256) return
    if (/[\r\n\0]/.test(ssid) || /[\r\n\0]/.test(identity) || /[\r\n\0]/.test(password)) return
    runJob(["bash", enterpriseWifiScript, ssid, identity], password + "\n", "wifi-enterprise")
  }
  function wifiUuidForSsid(ssid) {
    ssid = String(ssid || "")
    var list = wifiConnections instanceof Array ? wifiConnections : []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].name || "") === ssid)
        return String(list[i].uuid || "")
    }
    return ""
  }
  function joinWifi(ssid, password) {
    ssid = String(ssid || "")
    password = String(password || "")
    if (!ssid || ssid.length > 64 || ssid.charAt(0) === "-") return
    if (/[\r\n\0]/.test(ssid) || /[\r\n\0]/.test(password)) return
    if (password.length > 256) return
    var uuid = wifiUuidForSsid(ssid)
    if (uuid && !password) {
      activateWifiConnection(uuid)
      return
    }
    runJob(["bash", setWifiConnectionScript, "join", ssid], password.length ? password + "\n" : "", "wifi-join")
  }
  function activateWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    var list = wifiConnections instanceof Array ? wifiConnections : []
    var name = ""
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].uuid || "") === uuid) {
        name = String(list[i].name || "")
        break
      }
    }
    runCommand(["bash", setWifiConnectionScript, "up", uuid], {
      key: "wifi:" + uuid,
      apply: {
        wifiConnections: SnapshotJs.patchWifiActive(wifiConnections, uuid, true),
        wifiConnected: true,
        netKind: "wifi",
        netSsid: name
      },
      refresh: "none"
    })
  }
  function deactivateWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    runCommand(["bash", setWifiConnectionScript, "down", uuid], {
      key: "wifi:" + uuid,
      apply: {
        wifiConnections: SnapshotJs.patchWifiActive(wifiConnections, uuid, false),
        wifiConnected: false,
        netKind: "disconnected",
        netSsid: ""
      },
      refresh: "none"
    })
  }
  function forgetWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    var list = wifiConnections instanceof Array ? wifiConnections : []
    var active = false
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].uuid || "") === uuid && list[i].active === true) {
        active = true
        break
      }
    }
    var forgetApply = { wifiConnections: SnapshotJs.patchRemoveMatching(wifiConnections, "uuid", uuid) }
    if (active) {
      forgetApply.wifiConnected = false
      forgetApply.netKind = "disconnected"
      forgetApply.netSsid = ""
    }
    runCommand(["bash", setWifiConnectionScript, "delete", uuid], {
      key: "wifi:" + uuid,
      apply: forgetApply,
      refresh: "none"
    })
  }
  function deactivateWifiSsid(ssid) {
    ssid = String(ssid || "")
    if (!ssid || ssid.length > 64 || /[\r\n\0]/.test(ssid)) return
    var uuid = wifiUuidForSsid(ssid)
    if (uuid) {
      deactivateWifiConnection(uuid)
      return
    }
    runCommand(["bash", setWifiConnectionScript, "down-ssid", ssid], {
      key: "wifi:" + ssid,
      refresh: "all"
    })
  }
  function forgetWifiSsid(ssid) {
    ssid = String(ssid || "")
    if (!ssid || ssid.length > 64 || /[\r\n\0]/.test(ssid)) return
    var uuid = wifiUuidForSsid(ssid)
    if (uuid) {
      forgetWifiConnection(uuid)
      return
    }
    runCommand(["bash", setWifiConnectionScript, "delete-ssid", ssid], {
      key: "wifi:" + ssid,
      refresh: "all"
    })
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
    runCommand(["omarchy", "bluetooth", "device", "pair", address], {
      key: "bluetooth:" + address,
      refresh: "all"
    })
  }
  function connectBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(["omarchy", "bluetooth", "device", "connect", address], {
      key: "bluetooth:" + address,
      apply: { bluetoothDevices: SnapshotJs.patchRowField(bluetoothDevices, "address", address, "connected", true) },
      refresh: "none"
    })
  }
  function disconnectBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(["omarchy", "bluetooth", "device", "disconnect", address], {
      key: "bluetooth:" + address,
      apply: { bluetoothDevices: SnapshotJs.patchRowField(bluetoothDevices, "address", address, "connected", false) },
      refresh: "none"
    })
  }
  function forgetBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(["omarchy", "bluetooth", "device", "forget", address], {
      key: "bluetooth:" + address,
      apply: { bluetoothDevices: SnapshotJs.patchRemoveMatching(bluetoothDevices, "address", address) },
      refresh: "none"
    })
  }
  function setAudioOutputVolume(percent) {
    percent = Math.round(Number(percent))
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    if (percent === audioOutputVolume && !audioOutputMuted) return
    runCommand(["bash", setAudioScript, "output-volume", String(percent)], {
      key: "audioOutputVolume",
      apply: { audioOutputVolume: percent, audioOutputMuted: false },
      refresh: "none"
    })
  }
  function toggleAudioOutputMute() {
    runCommand(["omarchy", "audio", "output", "volume", "mute-toggle"], {
      key: "audioOutputMuted",
      apply: { audioOutputMuted: !audioOutputMuted },
      refresh: "none"
    })
  }
  function setAudioInputVolume(percent) {
    percent = Math.round(Number(percent))
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    if (percent === audioInputVolume && !audioInputMuted) return
    runCommand(["bash", setAudioScript, "input-volume", String(percent)], {
      key: "audioInputVolume",
      apply: { audioInputVolume: percent, audioInputMuted: false },
      refresh: "none"
    })
  }
  function toggleAudioInputMute() {
    runCommand(["omarchy", "audio", "input", "mute"], {
      key: "audioInputMuted",
      apply: { audioInputMuted: !audioInputMuted },
      refresh: "none"
    })
  }
  function setAudioSink(name) {
    name = String(name || "")
    if (!name || name === audioSink) return
    var list = audioSinks
    for (var i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].name) === name) {
        runCommand(["omarchy", "audio", "output", "set", "default", String(list[i].id), name], {
          key: "audioSink",
          apply: { audioSink: name },
          refresh: "none"
        })
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
        runCommand(["omarchy", "audio", "input", "set", "default", String(list[j].id), name], {
          key: "audioSource",
          apply: { audioSource: name },
          refresh: "none"
        })
        return
      }
    }
  }
  function switchAudioOutput() {
    runCommand(["omarchy", "audio", "output", "switch"], {
      key: "audioSink",
      refresh: "all"
    })
  }
  function setAudioTuning(on) {
    if (on === audioTuningOn) return
    runCommand(["omarchy", "audio", "tuning", on ? "on" : "off"], {
      key: "audioTuningOn",
      apply: { audioTuningOn: on },
      refresh: "none"
    })
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
    runJob(["bash", luksChangeKeyScript], device + "\n" + currentPass + "\n" + newPass + "\n", "luks", { sudo: true })
  }
  function createSnapshot() {
    runJob(["omarchy", "snapshot", "create"], "", "snapshot-create", { sudo: true })
  }
  function restoreSnapshot(config, id) {
    config = String(config || "")
    id = String(id || "")
    if (!/^[A-Za-z0-9_-]+$/.test(config)) return
    if (!/^[0-9]+$/.test(id)) return
    runJob(["bash", rollbackSnapshotScript, config, id], "", "snapshot-rollback", { sudo: true })
  }
  function setupHibernation() {
    runJob(["omarchy", "hibernation", "setup", "--force"], "", "hibernation-setup", { sudo: true })
  }
  function removeHibernation() {
    runJob(["bash", "-c", "PATH=\"$1:$PATH\" exec omarchy hibernation remove", "hibernation-remove", gumStubDir], "", "hibernation-remove", { sudo: true })
  }

  function setSuspendEnabled(on) {
    if (on === suspendEnabled) return
    runCommand(["omarchy", "toggle", "suspend-off", on ? "off" : "on"], {
      key: "suspendEnabled",
      apply: { suspendEnabled: on },
      refresh: "none"
    })
  }

  function setPowerProfile(name) {
    if (!name || name === powerProfile) return
    runCommand(["omarchy", "powerprofiles", "set", "autodetect", name], {
      key: "powerProfile",
      apply: { powerProfile: name },
      refresh: "none"
    })
  }

  function setPowerProfileAc(name) {
    if (!name || name === powerProfileAc) return
    runCommand(["omarchy", "powerprofiles", "set", "ac", name], {
      key: "powerProfileAc",
      apply: { powerProfileAc: name },
      refresh: "none"
    })
  }

  function setPowerProfileBattery(name) {
    if (!name || name === powerProfileBattery) return
    runCommand(["omarchy", "powerprofiles", "set", "battery", name], {
      key: "powerProfileBattery",
      apply: { powerProfileBattery: name },
      refresh: "none"
    })
  }

  function setPowerShowPercentage(on) {
    if (on === powerShowPercentage) return
    runCommand(["omarchy", "bar", "set", "omarchy.power", "showPercentage", on ? "true" : "false", "--json"], {
      key: "powerShowPercentage",
      apply: { powerShowPercentage: on },
      refresh: "none"
    })
  }

  function showBatteryNotification() {
    runCommand(["omarchy", "notification", "battery"])
  }

  function setCrashCapture(on) {
    if (on === crashCapture) return
    runCommand(["omarchy", "toggle", "crash", "capture"], {
      key: "crashCapture",
      apply: { crashCapture: on },
      refresh: "none"
    })
  }

  function setDoNotDisturb(on) {
    if (on === doNotDisturb) return
    runCommand(["omarchy", "toggle", "notification", "silencing"], {
      key: "doNotDisturb",
      apply: { doNotDisturb: on },
      refresh: "none"
    })
  }

  function setWeatherLocation(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name) return
    if (!weatherAuto && name === weatherLocation) return
    runCommand(["omarchy", "weather", "location", "--set", name], {
      key: "weatherLocation",
      apply: { weatherLocation: name, weatherAuto: false },
      refresh: "none"
    })
  }

  function clearWeatherLocation() {
    if (weatherAuto) return
    runCommand(["omarchy", "weather", "location", "--clear"], {
      key: "weatherLocation",
      apply: { weatherLocation: "", weatherAuto: true },
      refresh: "none"
    })
  }

  function setWeatherCoordinates(coords) {
    coords = String(coords || "").replace(/^\s+|\s+$/g, "")
    if (!/^-?[0-9]+(\.[0-9]+)?,-?[0-9]+(\.[0-9]+)?$/.test(coords)) return
    var name = String(weatherLocation || "").replace(/^\s+|\s+$/g, "")
    if (!name || weatherAuto) return
    if (coords === weatherCoords) return
    runCommand(["omarchy", "weather", "location", "--set", name, coords], {
      key: "weatherCoords",
      apply: { weatherCoords: coords },
      refresh: "none"
    })
  }

  function setWeatherUnit(unit) {
    if (unit !== "auto" && unit !== "metric" && unit !== "imperial") return
    if (unit === weatherUnit) return
    runCommand(["omarchy", "bar", "set", "omarchy.weather", "unit", unit], {
      key: "weatherUnit",
      apply: { weatherUnit: unit },
      refresh: "none"
    })
  }

  function setWeatherRefreshMinutes(minutes) {
    minutes = Math.round(Number(minutes))
    if (!(minutes >= 1) || minutes === weatherRefreshMinutes) return
    runCommand(["omarchy", "bar", "set", "omarchy.weather", "refreshMinutes", String(minutes), "--json"], {
      key: "weatherRefreshMinutes",
      apply: { weatherRefreshMinutes: minutes },
      refresh: "none"
    })
  }

  function setReminder(minutes, message) {
    minutes = String(minutes || "").replace(/^\s+|\s+$/g, "")
    if (!/^[1-9][0-9]*$/.test(minutes)) return
    message = String(message || "").replace(/^\s+|\s+$/g, "")
    var argv = ["omarchy", "reminder", minutes]
    if (message.length > 0) argv.push(message)
    var mins = parseInt(minutes, 10)
    runCommand(argv, {
      key: "reminder",
      apply: {
        reminderActive: true,
        reminderCount: reminderCount + 1,
        reminders: SnapshotJs.patchAppendReminder(reminders, mins, message)
      },
      refresh: "none"
    })
  }

  function clearReminders() {
    if (!reminderActive) return
    runCommand(["omarchy", "reminder", "clear"], {
      key: "reminder",
      apply: { reminderActive: false, reminderCount: 0, reminders: [] },
      refresh: "none"
    })
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
    runCommand(argv, {
      key: "recordingActive",
      apply: { recordingActive: true, webcamOverlay: webcam === true },
      refresh: "none"
    })
  }
  function stopScreenrecording() {
    runCommand(["omarchy", "capture", "screenrecording", "--stop-recording"], {
      key: "recordingActive",
      apply: { recordingActive: false },
      refresh: "none"
    })
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
    runGumJob(argv, kind || "software", { sudo: true })
  }
  function installDevEnv(lang) {
    lang = String(lang || "")
    if (!/^[a-z]+$/.test(lang)) return
    runGumJob(["omarchy", "install", "dev", "env", lang], "dev-env-install", { sudo: true })
  }
  function removeDevEnv(lang) {
    lang = String(lang || "")
    if (!/^[a-z]+$/.test(lang)) return
    runGumJob(["omarchy", "remove", "dev", "env", lang], "dev-env-remove", { sudo: true })
  }
  function installDockerDb(name) {
    name = String(name || "")
    if (!/^[A-Za-z]+$/.test(name)) return
    runGumJob(["omarchy", "install", "docker", "dbs", name], "docker-db-install", { sudo: true })
  }

  function isHookId(name) {
    name = String(name || "")
    if (name === "theme-set" || name === "font-set" || name === "post-boot" || name === "post-update" || name === "pre-refresh-pacman" || name === "battery-low")
      return true
    return /^[a-z0-9][a-z0-9-]*$/.test(name)
  }

  function hookDest(type, name) {
    type = String(type || "")
    name = String(name || "")
    if (!isHookId(type) || !name || name.indexOf("/") !== -1 || name.indexOf("..") !== -1) return ""
    return Quickshell.env("HOME") + "/.config/omarchy/hooks/" + type + ".d/" + name
  }

  function installHook(type, path) {
    type = String(type || "")
    path = String(path || "")
    if (!isHookId(type) || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    var base = path.split("/").pop()
    var dest = hookDest(type, base)
    if (!dest) return
    runCommand(["omarchy", "hook", "install", type, path], {
      key: "hook:" + dest,
      apply: {
        hooks: SnapshotJs.patchAppendHook(hooks, {
          type: type,
          name: base,
          path: dest,
          sample: base.length >= 7 && base.substring(base.length - 7) === ".sample",
          flat: false
        })
      },
      refresh: "none"
    })
  }

  function createHook(type, name, command) {
    type = String(type || "")
    name = HooksJs.sanitizeName(name)
    command = HooksJs.sanitizeLine(command)
    if (!isHookId(type) || !name || !command) return
    var dest = hookDest(type, name)
    if (!dest) return
    runCommand(["bash", createHookScript, type, name, command], {
      key: "hook:" + dest,
      apply: {
        hooks: SnapshotJs.patchAppendHook(hooks, {
          type: type,
          name: name,
          path: dest,
          sample: false,
          flat: false
        })
      },
      refresh: "none"
    })
  }

  function removeHook(path) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0) return
    if (path.indexOf("..") !== -1) return
    if (path.length >= 7 && path.substring(path.length - 7) === ".sample") return
    runCommand(["rm", "-f", path], {
      key: "hook:" + path,
      apply: { hooks: SnapshotJs.patchRemoveMatching(hooks, "path", path) },
      refresh: "none"
    })
  }

  function setHookSample(path, enabled) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0 || path.indexOf("..") !== -1) return
    runCommand(["bash", setHookSampleScript, enabled ? "enable" : "disable", path], {
      key: "hook:" + path,
      apply: { hooks: SnapshotJs.patchHookSample(hooks, path, enabled === true) },
      refresh: "none"
    })
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
    var temp = nightlightTemperature > 0 ? nightlightTemperature : 4000
    runCommand(["bash", setHyprsunsetScript, JSON.stringify({
      day: day,
      night: night,
      nightOn: nightOn === true,
      temperature: temp
    })], {
      key: "nightlightSchedule",
      apply: { nightlightDay: day, nightlightNight: night, nightlightNightOn: nightOn === true },
      refresh: "none"
    })
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
    runCommand(["bash", setHyprAutostartScript, JSON.stringify({ commands: commands })], {
      key: "autostart",
      apply: {
        autostart: SnapshotJs.patchReplaceManaged(autostart, commands),
        autostartManaged: true
      },
      refresh: "none"
    })
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
    runCommand(["bash", setHyprBindingsScript, JSON.stringify({ items: items })], {
      key: "bindings",
      apply: {
        bindings: SnapshotJs.patchReplaceManaged(bindings, items),
        bindingsManaged: true
      },
      refresh: "none"
    })
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
    runCommand(["bash", setHyprWindowsScript, JSON.stringify({ items: items })], {
      key: "windowRules",
      apply: {
        windowRules: SnapshotJs.patchReplaceManaged(windowRules, items),
        windowRulesManaged: true
      },
      refresh: "none"
    })
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

  function resetAtmos() {
    runJob(["bash", resetAtmosScript], "", "reset-atmos")
  }

  function setPlymouth(name) {
    if (!name || name === "default") return
    if (name === plymouth) return
    runCommand(["omarchy", "plymouth", "set", "by", "theme", name], {
      key: "plymouth",
      apply: { plymouth: name },
      refresh: "none",
      sudo: true
    })
  }

  function resetPlymouth() {
    if (plymouth === "default") return
    runJob(["omarchy", "plymouth", "reset"], "", "plymouth-reset", { sudo: true })
  }
  function setPlymouthFromPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runJob(["bash", "-c", "bg=$(omarchy theme color background); text=$(omarchy theme color foreground); omarchy plymouth set \"$bg\" \"$text\" \"$1\"", "plymouth-set", path], "", "plymouth-set", { sudo: true })
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

  Component.onCompleted: {
    Theme.currentThemeSwapped.connect(root.applyThemeNameFromFile)
    startSession(Quickshell.env("ATMOS_PAGE") || "appearance")
  }

  readonly property var watchSpecs: [
    { path: userShellJson, group: "look" },
    { path: defaultShellJson, group: "look" },
    { path: userShellToml, group: "look" },
    { path: weatherJson, group: "look" },
    { path: notificationsJson, group: "look" },
    { path: currentBackgroundFile, group: "look" },
    { path: screensaverBrandFile, group: "look" },
    { path: defaultScreensaverBrandFile, group: "look" },
    { path: aboutBrandFile, group: "look" },
    { path: defaultAboutBrandFile, group: "look" },
    { path: plymouthLogoFile, group: "look" },
    { path: defaultPlymouthLogoFile, group: "look" },
    { path: extraThemesDir, group: "look" },
    { path: packagedThemesDir, group: "look" },
    { path: fontconfigFile, group: "look" },
    { path: indicatorsDir, group: "look" },
    { path: reminderDir, group: "look" },
    { path: looknfeelLuaFile, group: "look" },
    { path: monitorsLuaFile, group: "look" },
    { path: hyprTogglesDir, group: "look" },
    { path: touchpadDisabledFile, group: "look" },
    { path: touchscreenDisabledFile, group: "look" },
    { path: togglesDir, group: "all" },
    { path: powerProfileAcFile, group: "rest" },
    { path: powerProfileBatteryFile, group: "rest" },
    { path: powerProfilesStateFile, group: "rest" },
    { path: applicationsDir, group: "rest" },
    { path: defaultEditorFile, group: "rest" },
    { path: defaultAgentFile, group: "rest" },
    { path: defaultTerminalFile, group: "rest" },
    { path: defaultBrowserFile, group: "rest" },
    { path: dnsConfFile, group: "rest" },
    { path: bluetoothRfkillDir, group: "rest" },
    { path: networkManagerDevicesDir, group: "rest" },
    { path: inputLuaFile, group: "rest" },
    { path: localtimeFile, group: "rest" },
    { path: vconsoleFile, group: "rest" },
    { path: localeConfFile, group: "rest" },
    { path: pacmanConfFile, group: "rest" }
  ]

  function applyThemeNameFromFile(slug) {
    slug = String(slug || "").replace(/^\s+|\s+$/g, "")
    var name = ThemeJs.themeNameFromSlug(slug, root.themes)
    if (!name) return
    if (name !== root.theme) {
      root.applySnapshot(JSON.stringify({ theme: name }))
      root.scheduleRefresh("look")
    }
  }

  function syncThemeFromDisk() {
    var slug = Theme.currentThemeSlug()
    if (!slug) return
    Theme.handleCurrentChanged()
    root.applyThemeNameFromFile(slug)
  }

  function syncThemeFromDiskIfStale() {
    var slug = Theme.currentThemeSlug()
    if (!slug) return
    if (ThemeJs.themeSlug(slug) === ThemeJs.themeSlug(root.theme)) return
    root.syncThemeFromDisk()
  }

  onThemesChanged: {
    var mapped = ThemeJs.themeNameFromSlug(root.theme, root.themes)
    if (mapped && mapped !== root.theme) root.applySnapshot(JSON.stringify({ theme: mapped }))
  }

  property Instantiator fileWatchers: Instantiator {
    model: root.watchSpecs
    delegate: FileView {
      path: modelData.path
      watchChanges: true
      printErrors: false
      onFileChanged: {
        reload()
        root.scheduleRefresh(modelData.group)
      }
    }
  }

  property Timer refreshTimer: Timer {
    interval: 180
    repeat: false
    onTriggered: {
      var groups = root.pendingRefreshGroups
      root.pendingRefreshGroups = []
      var i
      for (i = 0; i < groups.length; i++) root.enqueueRead(groups[i])
    }
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
      var job = root.ioJob
      if (exitCode === 0) {
        root.lastError = ""
        if (WorkQueue.shouldApplyRead(job, root.ioQueue))
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
    stdout: StdioCollector {
      id: mutOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: mutErr
      waitForEnd: true
    }
    onExited: function(exitCode) {
      var job = root.ioJob
      if (exitCode !== 0) {
        var msg = root.commandFailureText(mutErr.text, mutOut.text)
        if (root.stderrLooksLikeFailure(msg))
          root.lastError = msg || "Command failed"
        else
          root.lastError = msg ? "" : "Command failed"
      } else {
        root.applyWritePatch(job)
        if (job && job.refresh !== "none")
          WorkQueue.enqueueRead(root.ioQueue, "all")
      }
      root.ioFinished()
    }
  }

  property Process jobProc: Process {
    command: ["true"]
    stdinEnabled: false
    stdout: SplitParser {
      onRead: function(line) {
        root.jobStdoutBuf += String(line) + "\n"
        var cb = root.jobStdoutLineCb
        if (typeof cb === "function") cb(line)
      }
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
      var job = root.ioJob
      root.jobBusy = false
      var out = String(root.jobStdoutBuf || "").replace(/^\s+|\s+$/g, "")
      var err = String(jobErr.text || "").replace(/^\s+|\s+$/g, "")
      root.jobLog = out
      var finished = root.jobFinishedCb
      root.jobFinishedCb = null
      root.jobStdoutLineCb = null
      if (typeof finished === "function") finished(exitCode, out, err)
      if (root.sudoEnabling && root.jobKind === "passwordless-sudo") {
        root.sudoEnabling = false
        if (exitCode === 0) {
          root.passwordlessSudo = true
          root.sudoPromptOpen = false
          root.sudoError = ""
          var pending = root.sudoPendingJob
          root.sudoPendingJob = null
          if (pending) {
            pending.sudo = false
            WorkQueue.enqueueWrite(root.ioQueue, pending)
          }
        } else {
          root.sudoError = "Wrong password."
          root.sudoPromptOpen = true
          root.lastError = ""
          root.jobKind = ""
          root.ioFinished()
          return
        }
      }
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
      if (root.jobKind === "wifi-qr") {
        root.applyWifiQr(exitCode, out, err)
        root.jobKind = ""
        root.ioFinished()
        return
      }
      if (exitCode !== 0) {
        var failText = root.commandFailureText(err, out)
        if (root.stderrLooksLikeFailure(failText))
          root.lastError = failText || "Command failed"
        else
          root.lastError = failText ? "" : "Command failed"
      } else {
        root.lastError = ""
        if (!job || job.refresh !== "none")
          WorkQueue.enqueueRead(root.ioQueue, "all")
      }
      root.jobKind = ""
      root.ioFinished()
    }
  }
}
