pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "Accounts.js" as AccountsJs
import "AtmosUpdate.js" as AtmosUpdate
import "Hardware.js" as HardwareJs
import "Hooks.js" as HooksJs
import "Hubs.js" as HubsJs
import "HyprPrefs.js" as HyprPrefs
import "HyprSunset.js" as HyprSunset
import "RichUi.js" as RichUi
import "Settings.js" as SettingsJs
import "Snapshot.js" as SnapshotJs
import "SnapshotGroups.js" as SnapshotGroups
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
  readonly property string autostartLuaFile: Quickshell.env("HOME") + "/.config/hypr/autostart.lua"
  readonly property string bindingsLuaFile: Quickshell.env("HOME") + "/.config/hypr/bindings.lua"
  readonly property string windowsLuaFile: Quickshell.env("HOME") + "/.config/hypr/atmos.lua"
  readonly property string hyprsunsetConfFile: Quickshell.env("HOME") + "/.config/hypr/hyprsunset.conf"
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
  property real hyprInactiveOpacity: 1
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
  property bool hyprWorkspaceGestureManaged: false
  property bool hyprWorkspaceGestureUnmanaged: false
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
  readonly property var snapshotAdapters: ({
    clampLook: HyprPrefs.clampLook,
    clampInput: HyprPrefs.clampInput,
    applyAccountPatch: AccountsJs.applyAccountPatch,
    normalizeHardware: HardwareJs.normalize,
    parseTime: HyprSunset.parseTime,
    parseChannel: AtmosUpdate.parseChannel,
    parseWeatherCoords: RichUi.parseWeatherCoords,
    allowedKey: SnapshotGroups.allowedKey
  })
  property var ioJob: null
  property bool snapshotReady: false

  function applySnapshot(raw) {
    var parsed = SnapshotJs.parseSnapshot(raw)
    if (!parsed) {
      lastError = "Could not parse Omarchy snapshot"
      return
    }
    var next = SnapshotJs.adopt(snapshotData, parsed, snapshotAdapters)
    snapshotData = next
    copyRecord(next)
    var accounts = SnapshotJs.accountStorePatch(parsed)
    if (accounts) AccountsStore.applyPatch(accounts)
  }

  function copyRecord(next) {
    if (!next || typeof next !== "object") next = {}
    var look = next.hyprLook && typeof next.hyprLook === "object" ? next.hyprLook : null
    var input = next.hyprInput && typeof next.hyprInput === "object" ? next.hyprInput : null
    theme = SnapshotJs.adoptValue(theme, next.theme)
    background = SnapshotJs.adoptValue(background, next.background)
    font = SnapshotJs.adoptValue(font, next.font)
    textSize = SnapshotJs.adoptValue(textSize, next.textSize)
    themes = SnapshotJs.adoptArray(themes, next.themes)
    extraThemes = SnapshotJs.adoptArray(extraThemes, next.extraThemes)
    desktopApps = SnapshotJs.adoptArray(desktopApps, next.desktopApps)
    tuiApps = SnapshotJs.adoptArray(tuiApps, next.tuiApps)
    webApps = SnapshotJs.adoptArray(webApps, next.webApps)
    fonts = SnapshotJs.adoptArray(fonts, next.fonts)
    barPosition = SnapshotJs.adoptValue(barPosition, next.barPosition)
    barTransparent = SnapshotJs.adoptValue(barTransparent, next.barTransparent)
    barVisible = SnapshotJs.adoptValue(barVisible, next.barVisible)
    clockFormat = SnapshotJs.adoptValue(clockFormat, next.clockFormat)
    clockFormatAlt = SnapshotJs.adoptValue(clockFormatAlt, next.clockFormatAlt)
    clockWeekStart = SnapshotJs.adoptValue(clockWeekStart, next.clockWeekStart)
    clockPresent = SnapshotJs.adoptValue(clockPresent, next.clockPresent)
    clockBirthYear = SnapshotJs.adoptValue(clockBirthYear, next.clockBirthYear)
    clockLifeExpectancy = SnapshotJs.adoptValue(clockLifeExpectancy, next.clockLifeExpectancy)
    indicatorsPresent = SnapshotJs.adoptValue(indicatorsPresent, next.indicatorsPresent)
    indicatorsAlwaysShow = SnapshotJs.adoptValue(indicatorsAlwaysShow, next.indicatorsAlwaysShow)
    indicatorsItems = SnapshotJs.adoptArray(indicatorsItems, next.indicatorsItems)
    agentsPresent = SnapshotJs.adoptValue(agentsPresent, next.agentsPresent)
    agentsRefreshIntervalSec = SnapshotJs.adoptValue(agentsRefreshIntervalSec, next.agentsRefreshIntervalSec)
    agentsSync = SnapshotJs.adoptValue(agentsSync, next.agentsSync)
    agentsSyncDir = SnapshotJs.adoptValue(agentsSyncDir, next.agentsSyncDir)
    agentsSyncFileName = SnapshotJs.adoptValue(agentsSyncFileName, next.agentsSyncFileName)
    agentsSyncDeviceId = SnapshotJs.adoptValue(agentsSyncDeviceId, next.agentsSyncDeviceId)
    spacerPresent = SnapshotJs.adoptValue(spacerPresent, next.spacerPresent)
    spacerSize = SnapshotJs.adoptValue(spacerSize, next.spacerSize)
    trayPresent = SnapshotJs.adoptValue(trayPresent, next.trayPresent)
    trayHidden = SnapshotJs.adoptArray(trayHidden, next.trayHidden)
    trayPinned = SnapshotJs.adoptArray(trayPinned, next.trayPinned)
    browser = SnapshotJs.adoptValue(browser, next.browser)
    terminal = SnapshotJs.adoptValue(terminal, next.terminal)
    editor = SnapshotJs.adoptValue(editor, next.editor)
    agent = SnapshotJs.adoptValue(agent, next.agent)
    dns = SnapshotJs.adoptValue(dns, next.dns)
    idleScreensaver = SnapshotJs.adoptValue(idleScreensaver, next.idleScreensaver)
    idleLock = SnapshotJs.adoptValue(idleLock, next.idleLock)
    stayAwake = SnapshotJs.adoptValue(stayAwake, next.stayAwake)
    nightlight = SnapshotJs.adoptValue(nightlight, next.nightlight)
    nightlightTemperature = SnapshotJs.adoptValue(nightlightTemperature, next.nightlightTemperature)
    screensaverEnabled = SnapshotJs.adoptValue(screensaverEnabled, next.screensaverEnabled)
    screensaverBranded = SnapshotJs.adoptValue(screensaverBranded, next.screensaverBranded)
    aboutBranded = SnapshotJs.adoptValue(aboutBranded, next.aboutBranded)
    bluetooth = SnapshotJs.adoptValue(bluetooth, next.bluetooth)
    wifiConnected = SnapshotJs.adoptValue(wifiConnected, next.wifiConnected)
    wifiBand = SnapshotJs.adoptValue(wifiBand, next.wifiBand)
    wifiBandSelected = SnapshotJs.adoptValue(wifiBandSelected, next.wifiBandSelected)
    wifiBands = SnapshotJs.adoptArray(wifiBands, next.wifiBands)
    wifiIface = SnapshotJs.adoptValue(wifiIface, next.wifiIface)
    netKind = SnapshotJs.adoptValue(netKind, next.netKind)
    netIface = SnapshotJs.adoptValue(netIface, next.netIface)
    netSsid = SnapshotJs.adoptValue(netSsid, next.netSsid)
    netSignal = SnapshotJs.adoptValue(netSignal, next.netSignal)
    netIp = SnapshotJs.adoptValue(netIp, next.netIp)
    netSpeed = SnapshotJs.adoptValue(netSpeed, next.netSpeed)
    wifiHw = SnapshotJs.adoptValue(wifiHw, next.wifiHw)
    wifiRadio = SnapshotJs.adoptValue(wifiRadio, next.wifiRadio)
    wifiConnections = SnapshotJs.adoptArray(wifiConnections, next.wifiConnections)
    bluetoothDevices = SnapshotJs.adoptArray(bluetoothDevices, next.bluetoothDevices)
    audioSinks = SnapshotJs.adoptArray(audioSinks, next.audioSinks)
    audioSources = SnapshotJs.adoptArray(audioSources, next.audioSources)
    audioOutputVolume = SnapshotJs.adoptValue(audioOutputVolume, next.audioOutputVolume)
    audioOutputMuted = SnapshotJs.adoptValue(audioOutputMuted, next.audioOutputMuted)
    audioInputVolume = SnapshotJs.adoptValue(audioInputVolume, next.audioInputVolume)
    audioInputMuted = SnapshotJs.adoptValue(audioInputMuted, next.audioInputMuted)
    audioTuningMatch = SnapshotJs.adoptValue(audioTuningMatch, next.audioTuningMatch)
    audioTuningOn = SnapshotJs.adoptValue(audioTuningOn, next.audioTuningOn)
    disks = SnapshotJs.adoptArray(disks, next.disks)
    hardware = SnapshotJs.adoptValue(hardware, next.hardware)
    luksDevices = SnapshotJs.adoptArray(luksDevices, next.luksDevices)
    swapDevices = SnapshotJs.adoptArray(swapDevices, next.swapDevices)
    snapperPresent = SnapshotJs.adoptValue(snapperPresent, next.snapperPresent)
    snapperConfigs = SnapshotJs.adoptArray(snapperConfigs, next.snapperConfigs)
    snapshots = SnapshotJs.adoptArray(snapshots, next.snapshots)
    hibernationAvailable = SnapshotJs.adoptValue(hibernationAvailable, next.hibernationAvailable)
    hibernationSupported = SnapshotJs.adoptValue(hibernationSupported, next.hibernationSupported)
    hibernationConfigured = SnapshotJs.adoptValue(hibernationConfigured, next.hibernationConfigured)
    audioSink = SnapshotJs.adoptValue(audioSink, next.audioSink)
    audioSource = SnapshotJs.adoptValue(audioSource, next.audioSource)
    suspendEnabled = SnapshotJs.adoptValue(suspendEnabled, next.suspendEnabled)
    powerProfile = SnapshotJs.adoptValue(powerProfile, next.powerProfile)
    powerProfileAc = SnapshotJs.adoptValue(powerProfileAc, next.powerProfileAc)
    powerProfileBattery = SnapshotJs.adoptValue(powerProfileBattery, next.powerProfileBattery)
    powerProfiles = SnapshotJs.adoptArray(powerProfiles, next.powerProfiles)
    powerPresent = SnapshotJs.adoptValue(powerPresent, next.powerPresent)
    powerShowPercentage = SnapshotJs.adoptValue(powerShowPercentage, next.powerShowPercentage)
    isLaptop = SnapshotJs.adoptValue(isLaptop, next.isLaptop)
    batteryPresent = SnapshotJs.adoptValue(batteryPresent, next.batteryPresent)
    monitors = SnapshotJs.adoptArray(monitors, next.monitors)
    internalPresent = SnapshotJs.adoptValue(internalPresent, next.internalPresent)
    internalEnabled = SnapshotJs.adoptValue(internalEnabled, next.internalEnabled)
    externalPresent = SnapshotJs.adoptValue(externalPresent, next.externalPresent)
    mirroring = SnapshotJs.adoptValue(mirroring, next.mirroring)
    touchpadPresent = SnapshotJs.adoptValue(touchpadPresent, next.touchpadPresent)
    touchpadEnabled = SnapshotJs.adoptValue(touchpadEnabled, next.touchpadEnabled)
    touchscreenPresent = SnapshotJs.adoptValue(touchscreenPresent, next.touchscreenPresent)
    touchscreenEnabled = SnapshotJs.adoptValue(touchscreenEnabled, next.touchscreenEnabled)
    keyboardBacklightPresent = SnapshotJs.adoptValue(keyboardBacklightPresent, next.keyboardBacklightPresent)
    keyboardBrightness = SnapshotJs.adoptValue(keyboardBrightness, next.keyboardBrightness)
    crashCapture = SnapshotJs.adoptValue(crashCapture, next.crashCapture)
    doNotDisturb = SnapshotJs.adoptValue(doNotDisturb, next.doNotDisturb)
    weatherLocation = SnapshotJs.adoptValue(weatherLocation, next.weatherLocation)
    weatherCoords = SnapshotJs.adoptValue(weatherCoords, next.weatherCoords)
    weatherAuto = SnapshotJs.adoptValue(weatherAuto, next.weatherAuto)
    weatherPresent = SnapshotJs.adoptValue(weatherPresent, next.weatherPresent)
    weatherUnit = SnapshotJs.adoptValue(weatherUnit, next.weatherUnit)
    weatherRefreshMinutes = SnapshotJs.adoptValue(weatherRefreshMinutes, next.weatherRefreshMinutes)
    reminderCount = SnapshotJs.adoptValue(reminderCount, next.reminderCount)
    reminderActive = SnapshotJs.adoptValue(reminderActive, next.reminderActive)
    reminders = SnapshotJs.adoptArray(reminders, next.reminders)
    plymouth = SnapshotJs.adoptValue(plymouth, next.plymouth)
    plymouthThemes = SnapshotJs.adoptArray(plymouthThemes, next.plymouthThemes)
    hasAether = SnapshotJs.adoptValue(hasAether, next.hasAether)
    browsers = SnapshotJs.adoptValue(browsers, next.browsers)
    terminals = SnapshotJs.adoptValue(terminals, next.terminals)
    editors = SnapshotJs.adoptValue(editors, next.editors)
    timezone = SnapshotJs.adoptValue(timezone, next.timezone)
    timezones = SnapshotJs.adoptArray(timezones, next.timezones)
    ntp = SnapshotJs.adoptValue(ntp, next.ntp)
    ntpAvailable = SnapshotJs.adoptValue(ntpAvailable, next.ntpAvailable)
    ntpSynchronized = SnapshotJs.adoptValue(ntpSynchronized, next.ntpSynchronized)
    keyboardLayout = SnapshotJs.adoptValue(keyboardLayout, next.keyboardLayout)
    keyboardLayouts = SnapshotJs.adoptArray(keyboardLayouts, next.keyboardLayouts)
    locale = SnapshotJs.adoptValue(locale, next.locale)
    locales = SnapshotJs.adoptArray(locales, next.locales)
    parallelDownloads = SnapshotJs.adoptValue(parallelDownloads, next.parallelDownloads)
    hyprGapsIn = SnapshotJs.adoptValue(hyprGapsIn, look ? look.gapsIn : undefined)
    hyprGapsOut = SnapshotJs.adoptValue(hyprGapsOut, look ? look.gapsOut : undefined)
    hyprBorderSize = SnapshotJs.adoptValue(hyprBorderSize, look ? look.borderSize : undefined)
    hyprRounding = SnapshotJs.adoptValue(hyprRounding, look ? look.rounding : undefined)
    hyprBlur = SnapshotJs.adoptValue(hyprBlur, look ? look.blur : undefined)
    hyprShadow = SnapshotJs.adoptValue(hyprShadow, look ? look.shadow : undefined)
    hyprLayout = SnapshotJs.adoptValue(hyprLayout, look ? look.layout : undefined)
    hyprColumnWidth = SnapshotJs.adoptValue(hyprColumnWidth, look ? look.columnWidth : undefined)
    hyprDimInactive = SnapshotJs.adoptValue(hyprDimInactive, look ? look.dimInactive : undefined)
    hyprDimStrength = SnapshotJs.adoptValue(hyprDimStrength, look ? look.dimStrength : undefined)
    hyprAnimations = SnapshotJs.adoptValue(hyprAnimations, look ? look.animations : undefined)
    hyprCursorHideOnKey = SnapshotJs.adoptValue(hyprCursorHideOnKey, look ? look.cursorHideOnKey : undefined)
    hyprCursorWarp = SnapshotJs.adoptValue(hyprCursorWarp, look ? look.cursorWarp : undefined)
    hyprCursorSize = SnapshotJs.adoptValue(hyprCursorSize, look ? look.cursorSize : undefined)
    hyprAllowTearing = SnapshotJs.adoptValue(hyprAllowTearing, look ? look.allowTearing : undefined)
    hyprResizeOnBorder = SnapshotJs.adoptValue(hyprResizeOnBorder, look ? look.resizeOnBorder : undefined)
    hyprActiveOpacity = SnapshotJs.adoptValue(hyprActiveOpacity, look ? look.activeOpacity : undefined)
    hyprInactiveOpacity = SnapshotJs.adoptValue(hyprInactiveOpacity, look ? look.inactiveOpacity : undefined)
    hyprPreserveSplit = SnapshotJs.adoptValue(hyprPreserveSplit, look ? look.preserveSplit : undefined)
    hyprFocusOnActivate = SnapshotJs.adoptValue(hyprFocusOnActivate, look ? look.focusOnActivate : undefined)
    hyprLookManaged = SnapshotJs.adoptValue(hyprLookManaged, next.hyprLookManaged)
    hyprInputManaged = SnapshotJs.adoptValue(hyprInputManaged, next.hyprInputManaged)
    hyprWorkspaceGesture = SnapshotJs.adoptValue(hyprWorkspaceGesture, next.hyprWorkspaceGesture !== undefined ? next.hyprWorkspaceGesture : (input ? input.workspaceGesture : undefined))
    hyprWorkspaceGestureManaged = SnapshotJs.adoptValue(hyprWorkspaceGestureManaged, next.hyprWorkspaceGestureManaged)
    hyprWorkspaceGestureUnmanaged = SnapshotJs.adoptValue(hyprWorkspaceGestureUnmanaged, next.hyprWorkspaceGestureUnmanaged)
    hyprNoGaps = SnapshotJs.adoptValue(hyprNoGaps, next.hyprNoGaps)
    hyprSquareAspect = SnapshotJs.adoptValue(hyprSquareAspect, next.hyprSquareAspect)
    hyprWorkspaceLayout = SnapshotJs.adoptValue(hyprWorkspaceLayout, next.hyprWorkspaceLayout)
    hyprSensitivity = SnapshotJs.adoptValue(hyprSensitivity, input ? input.sensitivity : undefined)
    hyprAccelProfile = SnapshotJs.adoptValue(hyprAccelProfile, input ? input.accelProfile : undefined)
    hyprEmulateDiscreteScroll = SnapshotJs.adoptValue(hyprEmulateDiscreteScroll, input ? input.emulateDiscreteScroll : undefined)
    hyprNaturalScroll = SnapshotJs.adoptValue(hyprNaturalScroll, input ? input.naturalScroll : undefined)
    hyprScrollFactor = SnapshotJs.adoptValue(hyprScrollFactor, input ? input.scrollFactor : undefined)
    hyprClickfinger = SnapshotJs.adoptValue(hyprClickfinger, input ? input.clickfinger : undefined)
    hyprDisableWhileTyping = SnapshotJs.adoptValue(hyprDisableWhileTyping, input ? input.disableWhileTyping : undefined)
    hyprDrag3fg = SnapshotJs.adoptValue(hyprDrag3fg, input ? input.drag3fg : undefined)
    hyprRepeatRate = SnapshotJs.adoptValue(hyprRepeatRate, input ? input.repeatRate : undefined)
    hyprRepeatDelay = SnapshotJs.adoptValue(hyprRepeatDelay, input ? input.repeatDelay : undefined)
    hyprNumlock = SnapshotJs.adoptValue(hyprNumlock, input ? input.numlock : undefined)
    hyprFollowMouse = SnapshotJs.adoptValue(hyprFollowMouse, input ? input.followMouse : undefined)
    hyprKeyPressDpms = SnapshotJs.adoptValue(hyprKeyPressDpms, input ? input.keyPressDpms : undefined)
    hyprMouseMoveDpms = SnapshotJs.adoptValue(hyprMouseMoveDpms, input ? input.mouseMoveDpms : undefined)
    hyprKbLayout = SnapshotJs.adoptValue(hyprKbLayout, input ? ("kbLayoutOverride" in input ? input.kbLayoutOverride : input.kbLayout) : undefined)
    hyprKbVariant = SnapshotJs.adoptValue(hyprKbVariant, input ? ("kbVariantOverride" in input ? input.kbVariantOverride : input.kbVariant) : undefined)
    hyprKbOptions = SnapshotJs.adoptValue(hyprKbOptions, input ? input.kbOptions : undefined)
    fingerprintAvailable = SnapshotJs.adoptValue(fingerprintAvailable, next.fingerprintAvailable)
    fingerprintConfigured = SnapshotJs.adoptValue(fingerprintConfigured, next.fingerprintConfigured)
    fido2Configured = SnapshotJs.adoptValue(fido2Configured, next.fido2Configured)
    sshdEnabled = SnapshotJs.adoptValue(sshdEnabled, next.sshdEnabled)
    sshdActive = SnapshotJs.adoptValue(sshdActive, next.sshdActive)
    passwordlessSudo = SnapshotJs.adoptValue(passwordlessSudo, next.passwordlessSudo)
    sudolessDocker = SnapshotJs.adoptValue(sudolessDocker, next.sudolessDocker)
    omarchyVersion = SnapshotJs.adoptValue(omarchyVersion, next.omarchyVersion)
    omarchyChannel = SnapshotJs.adoptValue(omarchyChannel, next.omarchyChannel)
    updateAvailable = SnapshotJs.adoptValue(updateAvailable, next.updateAvailable)
    updateSummary = SnapshotJs.adoptValue(updateSummary, next.updateSummary)
    atmosRevision = SnapshotJs.adoptValue(atmosRevision, next.atmosRevision)
    atmosChannel = SnapshotJs.adoptValue(atmosChannel, next.atmosChannel)
    atmosInstalled = SnapshotJs.adoptValue(atmosInstalled, next.atmosInstalled)
    voxtypeInstalled = SnapshotJs.adoptValue(voxtypeInstalled, next.voxtypeInstalled)
    hybridGpuAvailable = SnapshotJs.adoptValue(hybridGpuAvailable, next.hybridGpuAvailable)
    hybridGpuMode = SnapshotJs.adoptValue(hybridGpuMode, next.hybridGpuMode)
    hwNvidia = SnapshotJs.adoptValue(hwNvidia, next.hwNvidia)
    hwNvidiaGsp = SnapshotJs.adoptValue(hwNvidiaGsp, next.hwNvidiaGsp)
    hwNvidiaWithoutGsp = SnapshotJs.adoptValue(hwNvidiaWithoutGsp, next.hwNvidiaWithoutGsp)
    hwVulkan = SnapshotJs.adoptValue(hwVulkan, next.hwVulkan)
    hwIntel = SnapshotJs.adoptValue(hwIntel, next.hwIntel)
    hwIntelPtl = SnapshotJs.adoptValue(hwIntelPtl, next.hwIntelPtl)
    hwWebcam = SnapshotJs.adoptValue(hwWebcam, next.hwWebcam)
    hwFramework16 = SnapshotJs.adoptValue(hwFramework16, next.hwFramework16)
    hwAsusRog = SnapshotJs.adoptValue(hwAsusRog, next.hwAsusRog)
    hwSurface = SnapshotJs.adoptValue(hwSurface, next.hwSurface)
    dmiVendor = SnapshotJs.adoptValue(dmiVendor, next.dmiVendor)
    dmiProduct = SnapshotJs.adoptValue(dmiProduct, next.dmiProduct)
    dmiFamily = SnapshotJs.adoptValue(dmiFamily, next.dmiFamily)
    cpuStat = SnapshotJs.adoptValue(cpuStat, next.cpuStat)
    memoryStat = SnapshotJs.adoptValue(memoryStat, next.memoryStat)
    cpuIdentity = SnapshotJs.adoptValue(cpuIdentity, next.cpuIdentity)
    gpuIdentity = SnapshotJs.adoptValue(gpuIdentity, next.gpuIdentity)
    npuIdentity = SnapshotJs.adoptValue(npuIdentity, next.npuIdentity)
    tailscaleInstalled = SnapshotJs.adoptValue(tailscaleInstalled, next.tailscaleInstalled)
    tailscaleRunning = SnapshotJs.adoptValue(tailscaleRunning, next.tailscaleRunning)
    plugins = SnapshotJs.adoptArray(plugins, next.plugins)
    snapperNumberLimit = SnapshotJs.adoptValue(snapperNumberLimit, next.snapperNumberLimit)
    snapperTimeline = SnapshotJs.adoptValue(snapperTimeline, next.snapperTimeline)
    fstrimEnabled = SnapshotJs.adoptValue(fstrimEnabled, next.fstrimEnabled)
    directBootAvailable = SnapshotJs.adoptValue(directBootAvailable, next.directBootAvailable)
    directBoot = SnapshotJs.adoptValue(directBoot, next.directBoot)
    mimePdf = SnapshotJs.adoptValue(mimePdf, next.mimePdf)
    mimeImage = SnapshotJs.adoptValue(mimeImage, next.mimeImage)
    mimeVideo = SnapshotJs.adoptValue(mimeVideo, next.mimeVideo)
    mimePdfOptions = SnapshotJs.adoptArray(mimePdfOptions, next.mimePdfOptions)
    mimeImageOptions = SnapshotJs.adoptArray(mimeImageOptions, next.mimeImageOptions)
    mimeVideoOptions = SnapshotJs.adoptArray(mimeVideoOptions, next.mimeVideoOptions)
    picturesDir = SnapshotJs.adoptValue(picturesDir, next.picturesDir)
    videosDir = SnapshotJs.adoptValue(videosDir, next.videosDir)
    recordingActive = SnapshotJs.adoptValue(recordingActive, next.recordingActive)
    webcamOverlay = SnapshotJs.adoptValue(webcamOverlay, next.webcamOverlay)
    services = SnapshotJs.adoptValue(services, next.services)
    gaming = SnapshotJs.adoptValue(gaming, next.gaming)
    extras = SnapshotJs.adoptValue(extras, next.extras)
    hooks = SnapshotJs.adoptArray(hooks, next.hooks)
    autostart = SnapshotJs.adoptArray(autostart, next.autostart)
    autostartManaged = SnapshotJs.adoptValue(autostartManaged, next.autostartManaged)
    bindings = SnapshotJs.adoptArray(bindings, next.bindings)
    bindingsManaged = SnapshotJs.adoptValue(bindingsManaged, next.bindingsManaged)
    windowRules = SnapshotJs.adoptArray(windowRules, next.windowRules)
    windowRulesManaged = SnapshotJs.adoptValue(windowRulesManaged, next.windowRulesManaged)
    keybindings = SnapshotJs.adoptArray(keybindings, next.keybindings)
    focusedClass = SnapshotJs.adoptValue(focusedClass, next.focusedClass)
    cupsActive = SnapshotJs.adoptValue(cupsActive, next.cupsActive)
    printerSetup = SnapshotJs.adoptValue(printerSetup, next.printerSetup)
    nightlightDay = SnapshotJs.adoptValue(nightlightDay, next.nightlightDay)
    nightlightNight = SnapshotJs.adoptValue(nightlightNight, next.nightlightNight)
    nightlightNightOn = SnapshotJs.adoptValue(nightlightNightOn, next.nightlightNightOn)
    tailscalePeers = SnapshotJs.adoptArray(tailscalePeers, next.tailscalePeers)
    hyprKbGroupToggle = SnapshotJs.adoptValue(hyprKbGroupToggle, input ? input.kbGroupToggle : undefined)
  }

  function refresh() {
    scheduleRefresh("all")
  }

  function scheduleRefresh(group) {
    pendingRefreshGroups = WorkQueue.addPendingRefresh(pendingRefreshGroups, SnapshotGroups.normalizeGroup(group))
    refreshTimer.restart()
  }

  property var pendingRefreshGroups: []

  function enqueueRead(group) {
    WorkQueue.enqueueRead(ioQueue, SnapshotGroups.normalizeGroup(group))
    kickIo()
  }

  function startSession(hub) {
    var first = SnapshotGroups.snapshotGroupForHub(hub)
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

  function snapshotRefreshGroup(value) {
    var g = String(value || "none")
    if (g === "none" || g === "") return "none"
    return SnapshotGroups.normalizeGroup(g)
  }

  function runCommand(argv, opts) {
    if (!(argv instanceof Array) || argv.length === 0) return
    opts = opts || {}
    enqueueIo({
      kind: "mut",
      argv: argv,
      key: opts.key ? String(opts.key) : "",
      apply: opts.apply && typeof opts.apply === "object" ? opts.apply : null,
      refresh: snapshotRefreshGroup(opts.refresh),
      sudo: opts.sudo === true
    })
  }

  function scriptOpts() {
    return {
      root: shellDir,
      scripts: {
        look: setHyprLookScript,
        input: setHyprInputScript,
        bindings: setHyprBindingsScript,
        windows: setHyprWindowsScript,
        autostart: setHyprAutostartScript,
        idle: setIdleScript,
        hyprsunset: setHyprsunsetScript,
        nightlightTemp: setNightlightTempScript,
        mime: setMimeDefaultScript,
        audio: setAudioScript,
        barWidget: setBarWidgetScript,
        hostname: setHostnameScript,
        timezone: setTimezoneScript,
        locale: setLocaleScript,
        keyboard: setKeyboardLayoutScript,
        ntp: setNtpScript,
        fullName: setFullNameScript,
        parallelDownloads: setParallelDownloadsScript,
        wifiRadio: setWifiConnectionScript
      }
    }
  }

  function runSettingCommand(cmd, key) {
    if (!cmd || cmd.skip) return
    runCommand(cmd.argv, {
      key: cmd.coalesceKey || key,
      apply: cmd.apply,
      refresh: "none",
      sudo: cmd.sudo === true
    })
  }

  function dispatchSetting(key, value) {
    runSettingCommand(SettingsJs.commandFor(key, value, snapshotData, scriptOpts()), key)
  }

  function inputLuaText() {
    inputLuaView.reload()
    if (typeof inputLuaView.waitForJob === "function")
      inputLuaView.waitForJob()
    var src = typeof inputLuaView.text === "function" ? inputLuaView.text() : inputLuaView.text
    return String(src || "")
  }

  function applyHyprWorkspaceGestureFromFile() {
    var state = HyprPrefs.inputWorkspaceGestureState(root.inputLuaText())
    hyprWorkspaceGesture = state.workspaceGesture === true
    hyprWorkspaceGestureManaged = state.workspaceGestureManaged === true
    hyprWorkspaceGestureUnmanaged = state.workspaceGestureUnmanaged === true
  }

  function liveWorkspaceGestureUnmanaged() {
    return HyprPrefs.inputHasUnmanagedWorkspaceGesture(root.inputLuaText())
  }

  function applyWritePatch(job) {
    if (!job || !job.apply) return
    applySnapshot(JSON.stringify(job.apply))
    // refresh: "none" leaves these flags stale. Commenting the stock line
    // out and then touching Sensitivity would take ownership in the file
    // while the row stayed disabled. Re-scan after every input write.
    if (job.key === "hyprInput" || job.key === "hyprInputManaged")
      root.applyHyprWorkspaceGestureFromFile()
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
      inactiveOpacity: hyprInactiveOpacity,
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
    var snap = SnapshotJs.mergeSnapshot(snapshotData, { hyprLook: look })
    var field = "gapsIn"
    var value = look.gapsIn
    if (patch && typeof patch === "object") {
      for (var k in patch) {
        if (Object.prototype.hasOwnProperty.call(patch, k)) {
          field = k
          value = patch[k]
          break
        }
      }
    }
    runSettingCommand(SettingsJs.commandFor("hyprLook." + field, value, snap, scriptOpts()), "hyprLook." + field)
  }

  function writeHyprInput(patch) {
    var input = inputState(patch)
    var snap = SnapshotJs.mergeSnapshot(snapshotData, { hyprInput: input })
    var field = "sensitivity"
    var value = input.sensitivity
    if (patch && typeof patch === "object") {
      for (var k in patch) {
        if (Object.prototype.hasOwnProperty.call(patch, k)) {
          field = k
          value = patch[k]
          break
        }
      }
    }
    runSettingCommand(SettingsJs.commandFor("hyprInput." + field, value, snap, scriptOpts()), "hyprInput." + field)
  }

  function runGumJob(argv, kind, opts) {
    if (!(argv instanceof Array) || argv.length === 0) return
    // shift, or "$@" still carries $1 and exec is handed the stub directory
    // itself: prefs-job: .../scripts/stubs: Is a directory
    var cmd = ["bash", "-c", "PATH=\"$1:$PATH\"; shift; exec \"$@\"", "prefs-job", gumStubDir]
    for (var i = 0; i < argv.length; i++) cmd.push(argv[i])
    runJob(cmd, "", kind, opts)
  }

  // Job record: kind ("read"|"mut"|"job"), argv, stdin, key, apply, refresh,
  // sudo, jobKind, onStdoutLine, onFinished. apply is consumed for mut and job.
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
      apply: opts.apply && typeof opts.apply === "object" ? opts.apply : null,
      refresh: opts.refresh === "none" ? "none" : SnapshotGroups.normalizeGroup(opts.refresh || "all"),
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
    var cmd = SettingsJs.commandFor("theme", name, snapshotData, scriptOpts())
    if (!cmd || cmd.skip) return
    // Import argv stays omarchy theme set. Detach here so mutProc does not
    // wait for omarchy-theme-set to recolor the shell.
    var argv = ["bash", "-c", "\"$@\" >/dev/null 2>&1 &", "theme-set"]
    var i
    for (i = 0; i < cmd.argv.length; i++) argv.push(cmd.argv[i])
    runCommand(argv, {
      key: cmd.coalesceKey || "theme",
      apply: cmd.apply,
      refresh: "none",
      sudo: cmd.sudo === true
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
    url = RichUi.parseGitUrl(url)
    if (!url) return
    var name = RichUi.gitThemeName(url)
    var extras = extraThemes.slice()
    var allThemes = themes.slice()
    var apply = null
    if (name) {
      if (extras.indexOf(name) === -1) extras.push(name)
      if (allThemes.indexOf(name) === -1) allThemes.push(name)
      apply = { extraThemes: extras, themes: allThemes, theme: name }
    }
    runJob(["omarchy", "theme", "install", url], "", "theme-install", {
      refresh: "look",
      apply: apply
    })
  }
  function updateThemes() {
    runJob(["omarchy", "theme", "update"], "", "theme-update", { refresh: "look" })
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
    dispatchSetting("background", path)
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
    dispatchSetting("font", name)
  }
  function setTextSize(size) {
    size = Math.round(Number(size))
    if (!isFinite(size) || size === textSize) return
    dispatchSetting("textSize", size)
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
    dispatchSetting("touchpadEnabled", on)
  }
  function setTouchscreen(on) {
    if (on === touchscreenEnabled) return
    dispatchSetting("touchscreenEnabled", on)
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
    dispatchSetting("barPosition", position)
  }
  function setBarTransparent(on) {
    if (on === barTransparent) return
    dispatchSetting("barTransparent", on)
  }
  // `omarchy toggle bar on` sets the bar-off flag and hides the bar.
  function setBarVisible(on) {
    if (on === barVisible) return
    dispatchSetting("barVisible", on)
  }
  function setClockFormat(fmt) {
    if (!fmt || fmt === clockFormat) return
    dispatchSetting("clockFormat", fmt)
  }
  function setClockFormatAlt(fmt) {
    if (!fmt || fmt === clockFormatAlt) return
    dispatchSetting("clockFormatAlt", fmt)
  }
  function setClockWeekStart(day) {
    day = String(day || "").toLowerCase()
    if (day !== "sunday" && day !== "monday" && day !== "tuesday" && day !== "wednesday" && day !== "thursday" && day !== "friday" && day !== "saturday") return
    if (day === clockWeekStart) return
    dispatchSetting("clockWeekStart", day)
  }
  function setClockBirthYear(year) {
    if (typeof year === "number") year = String(Math.round(year))
    year = String(year || "").replace(/^\s+|\s+$/g, "")
    if (year.length === 0 || year === "0") {
      if (clockBirthYear === 0) return
      dispatchSetting("clockBirthYear", 0)
      return
    }
    if (!/^\d{4}$/.test(year)) return
    var born = parseInt(year, 10)
    var now = new Date().getFullYear()
    if (!(born >= now - 120 && born <= now)) return
    if (born === clockBirthYear) return
    dispatchSetting("clockBirthYear", born)
  }
  function setClockLifeExpectancy(years) {
    if (typeof years === "number") years = String(Math.round(years))
    years = String(years || "").replace(/^\s+|\s+$/g, "")
    if (years.length === 0 || years === "0") {
      if (clockLifeExpectancy === 0) return
      dispatchSetting("clockLifeExpectancy", 0)
      return
    }
    if (!/^\d+$/.test(years)) return
    var span = parseInt(years, 10)
    if (!(span >= 1 && span <= 150)) return
    if (span === clockLifeExpectancy) return
    dispatchSetting("clockLifeExpectancy", span)
  }
  function setIndicatorsAlwaysShow(on) {
    if (on === indicatorsAlwaysShow) return
    dispatchSetting("indicatorsAlwaysShow", on)
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
    dispatchSetting("indicatorsItems", next)
  }
  function setAgentsRefreshIntervalSec(seconds) {
    seconds = Math.round(Number(seconds))
    if (!(seconds >= 30) || seconds === agentsRefreshIntervalSec) return
    dispatchSetting("agentsRefreshIntervalSec", seconds)
  }
  function setAgentsSync(on) {
    if (on === agentsSync) return
    dispatchSetting("agentsSync", on)
  }
  function setAgentsSyncDir(path) {
    path = String(path || "").replace(/^\s+|\s+$/g, "")
    if (path === agentsSyncDir) return
    dispatchSetting("agentsSyncDir", path)
  }
  function setAgentsSyncFileName(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "").split("/").pop()
    if (name === agentsSyncFileName) return
    dispatchSetting("agentsSyncFileName", name)
  }
  function setAgentsSyncDeviceId(id) {
    id = String(id || "").replace(/^\s+|\s+$/g, "")
    if (id === agentsSyncDeviceId) return
    dispatchSetting("agentsSyncDeviceId", id)
  }
  function setSpacerSize(size) {
    size = Math.round(Number(size))
    if (!isFinite(size) || size < 0 || size > 64 || size === spacerSize) return
    dispatchSetting("spacerSize", size)
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
    dispatchSetting("trayHidden", next)
  }
  function clearTrayHidden() {
    setTrayHidden([])
  }
  function setTrayPinned(list) {
    var next = normalizedStringIds(list)
    var current = trayPinned instanceof Array ? trayPinned : []
    if (JSON.stringify(next) === JSON.stringify(current)) return
    dispatchSetting("trayPinned", next)
  }
  function clearTrayPinned() {
    setTrayPinned([])
  }
  function setBrowser(name) {
    if (!name || name === browser) return
    dispatchSetting("browser", name)
  }
  function setTerminal(name) {
    if (!name || name === terminal) return
    dispatchSetting("terminal", name)
  }
  function setEditor(name) {
    if (!name || name === editor) return
    dispatchSetting("editor", name)
  }
  function setAgent(name) {
    if (!name || name === agent) return
    dispatchSetting("agent", name)
  }
  function setDns(name) {
    if (name !== "Cloudflare" && name !== "Google" && name !== "DHCP") return
    if (name === dns) return
    dispatchSetting("dns", name)
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
    var snap = SnapshotJs.mergeSnapshot(snapshotData, {
      idleScreensaver: saver,
      idleLock: lockSec
    })
    runSettingCommand(SettingsJs.commandFor("idleScreensaver", saver, snap, scriptOpts()), "idleScreensaver")
  }

  function setStayAwake(on) {
    if (on === stayAwake) return
    dispatchSetting("stayAwake", on)
  }

  function setNightlight(on) {
    if (on === nightlight) return
    dispatchSetting("nightlight", on)
  }

  function setScreensaverEnabled(on) {
    if (on === screensaverEnabled) return
    dispatchSetting("screensaverEnabled", on)
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
    dispatchSetting("timezone", name)
  }

  function setNtp(on) {
    if (on === ntp) return
    dispatchSetting("ntp", on)
  }

  function setHostname(name) {
    name = RichUi.parseHostname(name)
    if (!name || name === hostname) return
    dispatchSetting("hostname", name)
  }

  function setFullName(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (name === fullName) return
    if (!AccountsJs.isFullName(name)) return
    dispatchSetting("fullName", name)
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
    dispatchSetting("keyboardLayout", name)
  }

  function setLocale(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === locale) return
    if (name !== "C.UTF-8" && !/^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(name)) return
    dispatchSetting("locale", name)
  }

  function setParallelDownloads(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 1 || n > 20 || n === parallelDownloads) return
    dispatchSetting("parallelDownloads", n)
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
  function setHyprInactiveOpacity(n) {
    n = Math.round(Number(n) * 100) / 100
    if (!isFinite(n) || n < 0.2 || n > 1 || n === hyprInactiveOpacity) return
    writeHyprLook({ inactiveOpacity: n })
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
    dispatchSetting("hyprNoGaps", on)
  }
  function setHyprSquareAspect(on) {
    if (on === hyprSquareAspect) return
    dispatchSetting("hyprSquareAspect", on)
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
    var rawLayouts = String(layouts || "").replace(/^\s+|\s+$/g, "")
    layouts = HyprPrefs.sanitizeLayoutList(layouts)
    if (rawLayouts && !layouts) return
    var rawVariants = String(variants || "").replace(/^\s+|\s+$/g, "")
    variants = layouts ? HyprPrefs.sanitizeVariantList(variants, layouts.split(",").length) : ""
    if (rawVariants && layouts && !variants) return
    groupToggle = groupToggle === true
    if (layouts === hyprKbLayout && variants === hyprKbVariant && groupToggle === hyprKbGroupToggle) return
    writeHyprInput({
      kbLayoutOverride: layouts,
      kbVariantOverride: variants,
      kbGroupToggle: groupToggle
    })
  }
  function setHyprWorkspaceGesture(on) {
    if (hyprWorkspaceGestureUnmanaged) return
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
    dispatchSetting("nightlightTemperature", n)
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
    key = RichUi.parseSshPublicKey(key)
    if (!key) return
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
    var key = kind === "pdf" ? "mimePdf" : kind === "image" ? "mimeImage" : "mimeVideo"
    dispatchSetting(key, desktop)
  }

  function setBluetooth(on) {
    if (on === bluetooth) return
    dispatchSetting("bluetooth", on)
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
    dispatchSetting("wifiRadio", on)
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
    runCommand(["omarchy", "restart", "wifi"], { refresh: "network" })
  }
  function restartBluetooth() {
    runCommand(["omarchy", "restart", "bluetooth"], { refresh: "network" })
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
    dispatchSetting("audioOutputVolume", percent)
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
    dispatchSetting("audioInputVolume", percent)
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
    dispatchSetting("audioTuningOn", on)
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
    dispatchSetting("suspendEnabled", on)
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
    dispatchSetting("powerProfileAc", name)
  }

  function setPowerProfileBattery(name) {
    if (!name || name === powerProfileBattery) return
    dispatchSetting("powerProfileBattery", name)
  }

  function setPowerShowPercentage(on) {
    if (on === powerShowPercentage) return
    dispatchSetting("powerShowPercentage", on)
  }

  function showBatteryNotification() {
    runCommand(["omarchy", "notification", "battery"])
  }

  function setCrashCapture(on) {
    if (on === crashCapture) return
    dispatchSetting("crashCapture", on)
  }

  function setDoNotDisturb(on) {
    if (on === doNotDisturb) return
    dispatchSetting("doNotDisturb", on)
  }

  function setWeatherLocation(name) {
    name = RichUi.parseWeatherLocation(name)
    if (!name) return
    if (!weatherAuto && name === weatherLocation) return
    dispatchSetting("weatherLocation", name)
  }

  function clearWeatherLocation() {
    if (weatherAuto) return
    dispatchSetting("weatherLocation", "")
  }

  function setWeatherCoordinates(coords) {
    coords = RichUi.parseWeatherCoords(coords)
    if (!coords) return
    var name = RichUi.parseWeatherLocation(weatherLocation)
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
    dispatchSetting("weatherUnit", unit)
  }

  function setWeatherRefreshMinutes(minutes) {
    minutes = Math.round(Number(minutes))
    if (!(minutes >= 1) || minutes === weatherRefreshMinutes) return
    dispatchSetting("weatherRefreshMinutes", minutes)
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
    day = HyprSunset.parseTime(day)
    night = HyprSunset.parseTime(night)
    if (!day || !night) return
    var snap = SnapshotJs.mergeSnapshot(snapshotData, {
      nightlightDay: day,
      nightlightNight: night,
      nightlightNightOn: nightOn === true
    })
    runSettingCommand(SettingsJs.commandFor("nightlightDay", day, snap, scriptOpts()), "nightlightDay")
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
    dispatchSetting("autostart", commands)
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
    dispatchSetting("bindings", items)
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
    dispatchSetting("windowRules", items)
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
    dispatchSetting("plymouth", name)
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
    SnapshotGroups.setSnapshotGroupForHub(HubsJs.snapshotGroupForHub)
    Theme.currentThemeSwapped.connect(root.applyThemeNameFromFile)
    startSession(Quickshell.env("ATMOS_PAGE") || "appearance")
  }

  readonly property var watchSpecs: SnapshotGroups.watchSpecs({
    userShellJson: userShellJson,
    defaultShellJson: defaultShellJson,
    userShellToml: userShellToml,
    weatherJson: weatherJson,
    notificationsJson: notificationsJson,
    currentBackgroundFile: currentBackgroundFile,
    screensaverBrandFile: screensaverBrandFile,
    defaultScreensaverBrandFile: defaultScreensaverBrandFile,
    aboutBrandFile: aboutBrandFile,
    defaultAboutBrandFile: defaultAboutBrandFile,
    plymouthLogoFile: plymouthLogoFile,
    defaultPlymouthLogoFile: defaultPlymouthLogoFile,
    packagedThemesDir: packagedThemesDir,
    fontconfigFile: fontconfigFile,
    indicatorsDir: indicatorsDir,
    reminderDir: reminderDir,
    looknfeelLuaFile: looknfeelLuaFile,
    hyprsunsetConfFile: hyprsunsetConfFile,
    monitorsLuaFile: monitorsLuaFile,
    hyprTogglesDir: hyprTogglesDir,
    touchpadDisabledFile: touchpadDisabledFile,
    touchscreenDisabledFile: touchscreenDisabledFile,
    togglesDir: togglesDir,
    powerProfileAcFile: powerProfileAcFile,
    powerProfileBatteryFile: powerProfileBatteryFile,
    powerProfilesStateFile: powerProfilesStateFile,
    applicationsDir: applicationsDir,
    defaultEditorFile: defaultEditorFile,
    defaultAgentFile: defaultAgentFile,
    defaultTerminalFile: defaultTerminalFile,
    defaultBrowserFile: defaultBrowserFile,
    dnsConfFile: dnsConfFile,
    bluetoothRfkillDir: bluetoothRfkillDir,
    networkManagerDevicesDir: networkManagerDevicesDir,
    inputLuaFile: inputLuaFile,
    autostartLuaFile: autostartLuaFile,
    bindingsLuaFile: bindingsLuaFile,
    windowsLuaFile: windowsLuaFile,
    localtimeFile: localtimeFile,
    vconsoleFile: vconsoleFile,
    localeConfFile: localeConfFile,
    pacmanConfFile: pacmanConfFile
  })

  function applyThemeNameFromFile(slug) {
    slug = String(slug || "").replace(/^\s+|\s+$/g, "")
    var name = ThemeJs.themeNameFromSlug(slug, root.themes)
    if (!name) return
    if (name !== root.theme) {
      root.applySnapshot(JSON.stringify({ theme: name }))
      root.scheduleRefresh("look")
    }
  }

  onThemesChanged: {
    var mapped = ThemeJs.themeNameFromSlug(root.theme, root.themes)
    if (mapped && mapped !== root.theme) root.applySnapshot(JSON.stringify({ theme: mapped }))
  }

  FileView {
    id: inputLuaView
    path: root.inputLuaFile
    watchChanges: false
    printErrors: false
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

  // FileView misses nested git clones in extraThemesDir. inotifywait follows
  // create/delete/move/close_write; the 1s timer restarts a dead watcher.
  property Process extraThemesWatcher: Process {
    running: true
    command: [
      "inotifywait", "-m", "-q",
      "-e", "create,delete,move,close_write",
      "--format", "%e %f",
      extraThemesDir
    ]
    stdout: SplitParser {
      onRead: function(line) { extraThemesDebounce.restart() }
    }
    onExited: extraThemesWatcherRestart.restart()
  }

  property Timer extraThemesWatcherRestart: Timer {
    interval: 1000
    onTriggered: extraThemesWatcher.running = true
  }

  property Timer extraThemesDebounce: Timer {
    interval: 180
    onTriggered: root.scheduleRefresh("look")
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
        if (job && job.refresh && job.refresh !== "none")
          WorkQueue.enqueueRead(root.ioQueue, SnapshotGroups.normalizeGroup(job.refresh))
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
        // Closed after writing, or a job that reads its input to EOF never
        // gets one. enqueueIo re-arms this per job, so it stays correct for
        // the next one.
        stdinEnabled = false;
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
        root.applyWritePatch(job)
        if (job && job.refresh && job.refresh !== "none")
          WorkQueue.enqueueRead(root.ioQueue, SnapshotGroups.normalizeGroup(job.refresh))
      }
      root.jobKind = ""
      root.ioFinished()
    }
  }
}
