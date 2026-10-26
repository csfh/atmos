pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "Accounts.js" as AccountsJs
import "AtmosUpdate.js" as AtmosUpdate
import "Diagnostics.js" as DiagnosticsJs
import "Failure.js" as FailureJs
import "Favorites.js" as FavoritesJs
import "Guard.js" as GuardJs
import "Hardware.js" as HardwareJs
import "History.js" as HistoryJs
import "Hooks.js" as HooksJs
import "Hubs.js" as HubsJs
import "HyprPrefs.js" as HyprPrefs
import "HyprSunset.js" as HyprSunset
import "RichUi.js" as RichUi
import "Settings.js" as SettingsJs
import "NetworkPrefs.js" as NetworkPrefs
import "Processes.js" as ProcessesJs
import "Requests.js" as Requests
import "SettingTable.js" as SettingTableJs
import "Monitors.js" as MonitorsJs
import "Snapshot.js" as SnapshotJs
import "SnapshotGroups.js" as SnapshotGroups
import "Theme.js" as ThemeJs
import "WorkQueue.js" as WorkQueue

QtObject {
  id: root

  readonly property string shellDir: Quickshell.shellDir

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
  property var diagnostics: ({})
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
  // The queue and its processes live in IoQueue. Pages read these three.
  readonly property bool jobBusy: IoQueue.jobBusy
  readonly property string jobKind: IoQueue.jobKind
  readonly property string jobLog: IoQueue.jobLog
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
  property var hyprLook: HyprPrefs.defaultLook()
  property bool hyprLookManaged: false
  property var hyprInput: HyprPrefs.defaultInput()
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
  property string atmosChannel: "stable"
  property bool atmosInstalled: false
  property bool atmosPackaged: false
  property string atmosVersion: ""
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
  property bool windowOpaque: false
  property var workspaces: []
  property bool workspacesManaged: false
  property bool workspaceWrapSwitch: true
  property bool workspaceWheelSwitch: true
  property bool workspaceBarNames: false
  property int workspaceBarCount: 5
  property var monitorRules: []
  property bool monitorRulesManaged: false
  property var tweaks: ({})
  property var envVars: []
  property string envPathPrepend: ""
  property var envDetected: ({})
  property var systemdUnits: []
  property bool presentationMode: false
  property var favoriteItems: []
  property string powerGovernor: ""
  property string amdPstate: ""
  property int chargeLimit: 0
  property bool chargeLimitAvailable: false
  property string netGateway: ""
  property var netDnsServers: []
  property var keybindings: []
  property string focusedClass: ""
  property bool cupsActive: false
  property bool printerSetup: false
  property string nightlightDay: "07:00"
  property string nightlightNight: "20:00"
  property bool nightlightNightOn: false
  property var tailscalePeers: []

  property var snapshotData: ({})
  readonly property var snapshotAdapters: ({
    clampLook: HyprPrefs.clampLook,
    clampInput: HyprPrefs.clampInput,
    applyAccountPatch: AccountsJs.applyAccountPatch,
    normalizeHardware: HardwareJs.normalize,
    normalizeDiagnostics: DiagnosticsJs.normalize,
    parseTime: HyprSunset.parseTime,
    parseChannel: AtmosUpdate.parseChannel,
    parseWeatherCoords: RichUi.parseWeatherCoords,
    allowedKey: SnapshotGroups.allowedKey
  })
  property bool snapshotReady: false
  // Keys a producer could not read, and why. They keep the value they had.
  property var snapshotFailed: ({})

  function applySnapshot(raw) {
    var parsed = typeof raw === "string" ? SnapshotJs.parseSnapshot(raw) : raw
    if (!SnapshotJs.isPlainObject(parsed)) {
      lastError = "Could not parse Omarchy snapshot"
      return
    }
    var next = SnapshotJs.adopt(snapshotData, parsed, snapshotAdapters)
    snapshotData = next
    copyRecord(next)
    var accounts = SnapshotJs.accountStorePatch(parsed)
    if (accounts) AccountsStore.applyPatch(accounts)
    if (parsed.favorites !== undefined) root.loadFavorites(parsed.favorites)
  }

  function copyRecord(next) {
    if (!next || typeof next !== "object") next = {}
    var keys = SnapshotGroups.copyableBagKeys()
    var i, key, cur, nxt
    for (i = 0; i < keys.length; i++) {
      key = keys[i]
      cur = root[key]
      nxt = next[key]
      if (Array.isArray(cur) || Array.isArray(nxt))
        root[key] = SnapshotJs.adoptArray(cur, nxt)
      else
        root[key] = SnapshotJs.adoptValue(cur, nxt)
    }
  }

  function refresh() {
    scheduleRefresh("all")
  }

  function scheduleRefresh(group) {
    SnapshotStore.scheduleRefresh(group)
  }

  function startSession() {
    SnapshotFeed.start()
    DisplayFeed.start()
    IoQueue.resumeFromDaemon()
  }

  function applyDisplays(parsed) {
    var hw, disks, units, apps, diag
    hw = root.displayDoc(parsed.hardware)
    if (hw) root.hardware = HardwareJs.normalize(hw)
    disks = root.displayDoc(parsed.disks)
    if (disks) {
      if (disks.disks) root.disks = disks.disks
      if (disks.luksDevices) root.luksDevices = disks.luksDevices
      if (disks.swapDevices) root.swapDevices = disks.swapDevices
      if (disks.snapshots) root.snapshots = disks.snapshots
    }
    units = root.displayDoc(parsed.services)
    if (units && Array.isArray(units.items)) root.systemdUnits = units.items
    apps = root.displayDoc(parsed.software)
    if (apps) {
      if (apps.desktop) root.desktopApps = apps.desktop
      if (apps.tui) root.tuiApps = apps.tui
      if (apps.web) root.webApps = apps.web
    }
    diag = root.displayDoc(parsed.diagnostics)
    if (diag) root.diagnostics = DiagnosticsJs.normalize(diag)
  }

  function domainValue(cmd, key) {
    if (!cmd || !key) return undefined
    return cmd.value
  }

  function unstamp(value) {
    if (!value || typeof value !== "object" || Array.isArray(value)) return value
    var out = {}
    var key
    for (key in value) {
      if (!Object.prototype.hasOwnProperty.call(value, key)) continue
      if (key === "platform" || key === "collector") continue
      out[key] = value[key]
    }
    return out
  }

  // display.snapshot answers ok even when one kind failed. That kind is
  // {platform, collector, error: {code, message, context}}. Applying it would wipe the last good page.
  function displayDoc(value) {
    if (!value || typeof value !== "object" || Array.isArray(value)) return null
    if (value.error !== undefined && value.error !== null) return null
    return root.unstamp(value)
  }

  // IoQueue starts a job and says so; these do the domain work around it.
  function onSetRequested(job) {
    job.sentAt = Date.now()
    Backend.request(Requests.settingsSet(String(job.domain || job.key || ""), job.value, job.readback), function(env) {
      root.mutationAnswered(job, env)
    })
  }

  function onWriteStarting(job) {
    lastError = ""
    Feedback.begin()
  }

  function onJobStarting(job) {
    lastError = ""
    Feedback.begin()
    if (IoQueue.jobKind === "wifi-qr") {
      wifiQrError = ""
      wifiQrRows = []
      wifiQrSize = 0
      wifiQrSsid = ""
    }
  }

  function onInteractiveExited(exitCode, readback, refresh, out, err, summary) {
    if (exitCode === 143) return
    if (exitCode !== 0) {
      // An interactive tool that exits non-zero having printed nothing was
      // closed by the user, not broken.
      if (summary && !summary.message && !(summary.warnings && summary.warnings.length)) return
      var banner = FailureJs.applyBanner(summary, exitCode, root.commandFailureText(err, out))
      if (banner) root.lastError = banner
      return
    }
    if (readback) Ops.read(readback)
    if (refresh && refresh !== "none")
      root.scheduleRefresh(refresh)
  }

  function onJobExited(job, exitCode, out, err, summary) {
    Feedback.finish(exitCode === 0, (summary && summary.message) || err || out)
    if (root.sudoEnabling && IoQueue.jobKind === "passwordless-sudo") {
      root.sudoEnabling = false
      if (exitCode === 0) {
        root.sudoPromptOpen = false
        root.sudoError = ""
        var pending = root.sudoPendingJob
        root.sudoPendingJob = null
        // Sudo mode is on when the machine says so, not when the job exits.
        Ops.read({ keys: ["passwordlessSudo"], expect: { passwordlessSudo: true } }, function() {
          if (!pending) return
          pending.sudo = false
          root.rememberGuard(pending)
          WorkQueue.enqueueWrite(IoQueue.queue, pending)
          IoQueue.kick()
        })
      } else {
        root.sudoError = "Wrong password."
        root.sudoPromptOpen = true
        root.lastError = ""
        IoQueue.jobKind = ""
        IoQueue.finished()
        return
      }
    }
    if (IoQueue.jobKind === "update-check") {
      root.lastError = ""
      IoQueue.jobKind = ""
      IoQueue.finished()
      return
    }
    if (IoQueue.jobKind === "atmos-update-check" || IoQueue.jobKind === "atmos-update") {
      var parsed = AtmosUpdate.parseCheckOutput(out + "\n" + err)
      var applied = IoQueue.jobKind === "atmos-update" && exitCode === 0
      root.atmosUpdateAvailable = parsed.status === "behind"
      root.atmosUpdateSummary = parsed.summary
      if (parsed.short) root.atmosRevision = parsed.short
      if (parsed.channel) root.atmosChannel = parsed.channel
      root.lastError = (exitCode !== 0 && parsed.status !== "behind") ? (parsed.summary || err || "Atmos update failed") : ""
      IoQueue.jobKind = ""
      IoQueue.finished()
      return
    }
    if (IoQueue.jobKind === "wifi-qr") {
      root.applyWifiQr(exitCode, out, err)
      IoQueue.jobKind = ""
      IoQueue.finished()
      return
    }
    if (exitCode !== 0) {
      root.lastError = FailureJs.applyBanner(summary, exitCode, root.commandFailureText(err, out))
    } else {
      root.lastError = ""
    }
    // The daemon invalidates what a job can change when it ends and pushes it.
    IoQueue.jobKind = ""
    IoQueue.finished()
  }

  function mutationAnswered(job, env) {
    if (env && env.ok === true) {
      Feedback.finish(true, "")
      // The keys read back after the write, plus the setting's own value.
      var shown = env.result && env.result.snapshot ? SnapshotJs.plain(env.result.snapshot) : {}
      var domain = String((job && (job.domain || job.key)) || "")
      var own = env.result ? env.result.value : undefined
      if (domain && domain.indexOf(".") === -1 && shown[domain] === undefined && SnapshotJs.isScalar(own))
        shown[domain] = own
      Ops.noteWritten(shown, job && job.sentAt)
      root.applySnapshot(shown)
      if (job && job.refresh && job.refresh !== "none") root.scheduleRefresh(job.refresh)
    } else {
      var msg = FailureJs.errorText(env && env.error) || "Command failed"
      root.lastError = msg
      Feedback.finish(false, msg)
    }
    IoQueue.finished()
  }

  function snapshotRefreshGroup(value) {
    var g = String(value || "none")
    if (g === "none" || g === "") return "none"
    return SnapshotGroups.normalizeGroup(g)
  }

  // Every mutation passes through here, which makes it the one honest place
  // to record from or to hold. Hooking each page instead would mean trusting
  // every future control to remember, and the changes that got missed would
  // be exactly the ones nobody thought about -- the ones you most want when
  // working out what broke yesterday. The lists stay in this window's
  // memory: after multi-window Atmos each launch is its own process.
  property var changeHistory: []
  property var heldChanges: []

  // In-memory only. If this window exits during the 12s, the new setting stays.
  property var guardState: ({ open: false, deadline: 0, reverts: [] })
  property double guardClock: 0
  property bool reverting: false
  readonly property bool guardOpen: !!(guardState && guardState.open === true)
  readonly property int guardSeconds: GuardJs.secondsLeft(guardState, guardClock)
  signal guardArmed()

  function recordChange(label, opts) {
    var o = opts || {}
    changeHistory = HistoryJs.push(
      changeHistory,
      HistoryJs.entry(label, {
        key: o.key || "",
        file: HistoryJs.targetFile(label, Quickshell.env("HOME")),
        source: "you",
        sudo: o.sudo === true
      })
    )
  }

  function holdChange(label, opts) {
    var o = opts || {}
    var item = HistoryJs.entry(label, {
      key: o.key || "",
      file: HistoryJs.targetFile(label, Quickshell.env("HOME")),
      source: "preview",
      sudo: o.sudo === true
    })
    // The rendered text is for reading; the label and opts (which carry the
    // typed op) are what Apply replays. Keeping only the text makes Apply a
    // no-op that looks like it worked, which is the worst way for this to fail.
    item.argv = label
    item.opts = o
    heldChanges = HistoryJs.push(heldChanges, item)
  }

  function discardHeld() {
    heldChanges = []
  }

  // Replays one held change: a typed op through runCommand, a setting through
  // runSetting.
  function replay(label, opts) {
    if (!(opts && opts.op)) return root.runSetting(label, opts)
    var o = {}
    var k
    for (k in opts) if (k !== "op") o[k] = opts[k]
    root.runCommand(opts.op, o)
  }

  // The only path by which a held command ever runs, so "preview" cannot
  // quietly become "apply later". Replay copies every option the original
  // call carried (key, refresh, sudo, readback, the op itself, ...) and sets
  // bypassPreview so Apply cannot re-hold itself.
  function applyHeld() {
    var held = heldChanges
    heldChanges = []
    HistoryJs.applyHeld(held, replay)
  }

  function stripGuard(opts) {
    if (!opts || !opts.guard) return opts || {}
    var next = {}
    var k
    for (k in opts) {
      if (k === "guard") continue
      next[k] = opts[k]
    }
    return next
  }

  // Preview stops a write and shows it instead; a revert never holds or arms.
  // Returns true when the change was held.
  function previewHeld(label, opts) {
    if (root.reverting) {
      opts.bypassPreview = true
      opts.guard = null
    }
    if (Preview.active !== true || opts.bypassPreview === true) return false
    holdChange(label, root.stripGuard(opts))
    return true
  }

  // A typed write: a name and arguments the backend checks and runs on its
  // lane. The window builds no command line (docs/adr/0004, 0008).
  function runCommand(op, opts) {
    if (!(op && op.op)) return
    opts = opts || {}
    var label = Requests.label(op)
    var withOp = root.stripGuard(opts)
    withOp.op = op
    if (root.previewHeld(label, withOp)) return
    recordChange(label, root.stripGuard(opts))
    var run = {}
    var k
    for (k in opts) run[k] = opts[k]
    run.op = op
    enqueueIo({ kind: "op", opts: run, sudo: opts.sudo === true, guard: opts.guard && opts.guard.id ? opts.guard : null })
  }

  // A setting: settings.set with the value the row chose, then the keys read
  // back. `label` is for the history only.
  function runSetting(label, opts) {
    if (!(label instanceof Array) || label.length === 0) return
    opts = opts || {}
    if (opts.hasValue !== true) return
    if (root.previewHeld(label, opts)) return
    recordChange(label, root.stripGuard(opts))
    enqueueIo({
      kind: "mut",
      key: opts.key ? String(opts.key) : "",
      domain: opts.domain ? String(opts.domain) : "",
      hasValue: true,
      value: opts.value,
      readback: opts.readback instanceof Array ? opts.readback : null,
      refresh: snapshotRefreshGroup(opts.refresh),
      sudo: opts.sudo === true,
      guard: opts.guard && opts.guard.id ? opts.guard : null
    })
  }

  function scriptOpts() {
    return {
      root: shellDir,
      atmosd: Backend.bin
    }
  }

  function runSettingCommand(cmd, key, guard) {
    if (!cmd || cmd.skip) return
    var value = root.domainValue(cmd, key)
    var opts = {
      key: cmd.coalesceKey || key,
      domain: key,
      readback: cmd.readback,
      refresh: "none",
      sudo: cmd.sudo === true
    }
    if (value !== undefined) {
      opts.hasValue = true
      opts.value = value
    }
    if (guard && guard.id) opts.guard = guard
    runSetting(cmd.argv, opts)
  }

  // The settings that need no more than "skip a no-op, then send it". See
  // SettingTable.js for the list. Anything with a rule of its own keeps a
  // named setter.
  function set(key, value) {
    var step = SettingTableJs.plan(key, value, root[key])
    if (step.dispatch) dispatchSetting(key, step.value)
  }

  function dispatchSetting(key, value, guard) {
    runSettingCommand(
      SettingsJs.commandFor(key, value, snapshotData, scriptOpts()),
      key,
      root.reverting ? null : guard
    )
  }

  function copyJson(value) {
    try {
      return JSON.parse(JSON.stringify(value))
    } catch (e) {
      return value
    }
  }

  // Arm only once the job is on the queue. A sudo prompt that the user
  // cancels never reaches here, so the bar does not open for a write that
  // did not happen.
  function rememberGuard(job) {
    if (!job || !job.guard || !job.guard.id || root.reverting) return
    var now = Date.now()
    var step = GuardJs.open(root.guardState, job.guard, now)
    job.guard = null
    if (!step.armed) return
    root.guardState = step.state
    root.guardClock = now
    root.guardArmed()
  }

  function tickGuard() {
    var now = Date.now()
    var step = GuardJs.tick(root.guardState, now)
    root.guardClock = now
    if (!step.fire || !step.fire.length) {
      root.guardState = step.state
      return
    }
    root.guardState = step.state
    root.applyReverts(step.fire)
  }

  function keepGuard() {
    root.guardState = GuardJs.keep(root.guardState)
    root.guardClock = Date.now()
  }

  function revertGuard() {
    var step = GuardJs.revert(root.guardState)
    root.guardState = step.state
    root.guardClock = Date.now()
    root.applyReverts(step.fire)
  }

  function applyReverts(list) {
    if (!list || !list.length) return
    root.reverting = true
    var i
    for (i = 0; i < list.length; i++) root.dispatchRevert(list[i])
    root.reverting = false
  }

  function dispatchRevert(desc) {
    if (!desc || !desc.kind) return
    if (desc.kind === "monitorRules") {
      root.writeMonitorRules(desc.rules)
      return
    }
    if (desc.kind === "monitorScale") {
      var scale = String(desc.scale || "")
      var n = Number(scale)
      if (!scale || !isFinite(n) || n <= 0) return
      root.runCommand(Requests.displayScale(scale), { key: "monitorScale", refresh: "none" })
      return
    }
    if (desc.kind === "keyboardLayout") {
      root.dispatchSetting("keyboardLayout", desc.name, null)
      return
    }
    if (desc.kind === "kbOverride") {
      root.writeHyprKbOverride(desc.layouts, desc.variants, desc.groupToggle === true)
      return
    }
    if (desc.kind === "touchpad") {
      if (WorkQueue.hasQueuedKey(IoQueue.queue, "touchpadEnabled")) {
        WorkQueue.dropWriteKey(IoQueue.queue, "touchpadEnabled")
        return
      }
      root.dispatchSetting("touchpadEnabled", true, null)
    }
  }

  function firstPatchField(patch) {
    if (!patch || typeof patch !== "object") return ""
    var k
    for (k in patch) {
      if (Object.prototype.hasOwnProperty.call(patch, k)) return k
    }
    return ""
  }

  function writeHyprLook(patch) {
    var field = firstPatchField(patch)
    if (!field) return
    var look = HyprPrefs.clampLook(SnapshotJs.mergeSnapshot(hyprLook, patch))
    var snap = SnapshotJs.mergeSnapshot(snapshotData, { hyprLook: look })
    runSettingCommand(SettingsJs.commandFor("hyprLook." + field, look[field], snap, scriptOpts()), "hyprLook." + field)
  }

  function writeHyprInput(patch) {
    var field = firstPatchField(patch)
    if (!field) return
    var input = HyprPrefs.clampInput(SnapshotJs.mergeSnapshot(hyprInput, patch))
    input.kbLayout = input.kbLayoutOverride
    var snap = SnapshotJs.mergeSnapshot(snapshotData, { hyprInput: input })
    runSettingCommand(SettingsJs.commandFor("hyprInput." + field, input[field], snap, scriptOpts()), "hyprInput." + field)
  }

  // Job record: kind ("read"|"mut"|"job"|"op"), op, key, readback,
  // refresh, sudo, jobKind, onStdoutLine, onFinished.
  function enqueueIo(job) {
    if (!job) return
    if (job.sudo && !passwordlessSudo) {
      sudoPendingJob = job
      sudoError = ""
      sudoPromptOpen = true
      return
    }
    root.rememberGuard(job)
    if (job.kind === "op") return Ops.run(job.opts)
    WorkQueue.enqueueWrite(IoQueue.queue, job)
    IoQueue.kick()
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
        enqueueIo(pending)
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

  // A job: a registry name and its arguments (Requests.job), run by the daemon
  // so it outlives this window. `kind` names it for the pages that show it
  // running; a password or a plan is an argument, never a command line.
  function runJob(op, kind, opts) {
    if (!(op && op.op === "job")) return
    opts = opts || {}
    kind = String(kind || "")
    enqueueIo({
      kind: "job",
      op: op,
      jobKind: kind,
      key: opts.key ? String(opts.key) : kind,
      refresh: opts.refresh === "none" ? "none" : SnapshotGroups.normalizeGroup(opts.refresh || "all"),
      sudo: opts.sudo === true,
      onStdoutLine: typeof opts.onStdoutLine === "function" ? opts.onStdoutLine : null,
      onFinished: typeof opts.onFinished === "function" ? opts.onFinished : null
    })
  }

  function cancelJob() {
    IoQueue.cancelJob()
  }

  // Blocking pickers and region tools (file select, slurp, theme switcher)
  // stay off the queue. The queue is a single lock; holding it while a dialog
  // waits stalls every other setting.
  function runInteractive(op, opts) {
    if (!(op && op.op === "job")) return
    opts = opts || {}
    lastError = ""
    IoQueue.startInteractive(
      op,
      opts.kind,
      opts.readback && typeof opts.readback === "object" ? opts.readback : null,
      snapshotRefreshGroup(opts.refresh)
    )
  }

  function commandFailureText(err, out) {
    return FailureJs.commandText(err, out)
  }

  function setTheme(name) {
    name = String(name || "")
    if (!name || name === theme) return
    // A hover snapshot is stale once we commit. Drop it so a later
    // restorePreview cannot paint the pre-click chrome back over this.
    Theme.discardPreview()
    Theme.applyNamedTheme(name)
    dispatchSetting("theme", name)
  }
  function openThemeSwitcher() {
    runInteractive(Requests.job("theme.switcher", {}), {
      kind: "theme-switcher",
      refresh: "none"
    })
  }
  function refreshTheme() { runCommand(Requests.write("theme.refresh", {})) }
  function openThemeFolder() {
    if (!theme) return
    runCommand(Requests.write("theme.folder", { theme: theme }))
  }
  function installTheme(url) {
    url = RichUi.parseGitUrl(url)
    if (!url) return
    runJob(Requests.job("theme.install", { url: url }), "theme-install", { refresh: "look" })
  }
  function updateThemes() {
    runJob(Requests.job("theme.update", {}), "theme-update", { refresh: "look" })
  }
  function removeTheme(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name.indexOf("/") !== -1 || name.indexOf(".") === 0) return
    runCommand(Requests.write("theme.remove", { name: name }), { key: "theme-remove:" + name, refresh: name === theme ? "all" : "none" })
  }
  function setBackgroundPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/") return
    dispatchSetting("background", path)
  }
  function nextBackground() {
    runCommand(Requests.write("theme.bg-next", {}), { key: "background", refresh: "all" })
  }
  function openBackgroundSwitcher() {
    runInteractive(Requests.job("theme.bg-switcher", {}), {
      kind: "background-switcher",
      refresh: "all"
    })
  }
  function setBackgroundFromFile() {
    runInteractive(Requests.job("theme.bg-file", {}), {
      kind: "background-file",
      refresh: "all"
    })
  }
  function openBackgroundFolder() { runCommand(Requests.write("theme.bg-folder", {})) }
  function cacheBackgrounds() { runCommand(Requests.write("theme.bg-cache", {})) }
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
    runCommand(Requests.write("display.text-size-reset", {}), { key: "textSize", refresh: "none" })
  }
  function setMonitorScale(scale) {
    scale = String(scale || "")
    if (!scale) return
    var n = Number(scale)
    if (!isFinite(n) || n <= 0) return
    var list = SnapshotJs.toList(monitors)
    var i
    var previous = ""
    for (i = 0; i < list.length; i++) {
      if (list[i] && list[i].focused === true) {
        if (!root.reverting && Number(list[i].scale) === n) return
        previous = String(list[i].scale)
        break
      }
    }
    var opts = {
      key: "monitorScale",
      op: Requests.displayScale(scale),
      refresh: "none"
    }
    if (!root.reverting && previous)
      opts.guard = { id: "monitorScale", revert: { kind: "monitorScale", scale: previous } }
    runCommand(Requests.displayScale(scale), opts)
  }
  function setDisplayBrightness(name, percent) {
    name = String(name || "")
    percent = Math.round(Number(percent))
    if (!/^[A-Za-z0-9._-]+$/.test(name)) return
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    runCommand(Requests.displayBrightness(name, percent), { key: "brightness:" + name })
  }
  function setInternalDisplay(on) {
    if (on === internalEnabled) return
    runCommand(Requests.write("monitor.internal", { on: on === true }), { key: "internalEnabled", refresh: "none" })
  }
  function setInternalMirror(on) {
    if (on === mirroring) return
    runCommand(Requests.write("monitor.mirror", { on: on === true }), { key: "mirroring", refresh: "none" })
  }
  function setTouchpad(on) {
    if (on === touchpadEnabled) return
    var guard = null
    if (on !== true)
      guard = { id: "touchpad", revert: { kind: "touchpad" } }
    dispatchSetting("touchpadEnabled", on === true, guard)
  }
  function adjustKeyboardBacklight(direction) {
    if (direction !== "up" && direction !== "down" && direction !== "off" && direction !== "restore") return
    if (direction === "restore") {
      runCommand(Requests.write("keyboard.backlight-restore", {}), { key: "keyboardBrightness", refresh: "all" })
      return
    }
    runCommand(Requests.write("keyboard.backlight", { direction: direction }), { key: "keyboardBrightness", refresh: "none" })
  }
  // `omarchy toggle bar on` sets the bar-off flag and hides the bar.
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
    var current = SnapshotJs.toList(indicatorsItems)
    if (JSON.stringify(next) === JSON.stringify(current)) return
    dispatchSetting("indicatorsItems", next)
  }
  function setAgentsRefreshIntervalSec(seconds) {
    seconds = Math.round(Number(seconds))
    if (!(seconds >= 30) || seconds === agentsRefreshIntervalSec) return
    dispatchSetting("agentsRefreshIntervalSec", seconds)
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
    runCommand(Requests.write("bar.spacer", { on: true }), { key: "spacerPresent", refresh: "none" })
  }
  function removeSpacer() {
    if (!spacerPresent) return
    runCommand(Requests.write("bar.spacer", { on: false }), { key: "spacerPresent", refresh: "none" })
  }
  function installDesktopApp(name, command, icon) {
    name = String(name || "")
    command = String(command || "")
    icon = String(icon || "application-x-executable")
    if (!name || !command) return
    if (name.indexOf("/") !== -1 || name.charAt(0) === "-") return
    runJob(Requests.job("app.desktop-install", { name: name, command: command, icon: icon }), "desktop-install")
  }
  function installTui(name, command, style, icon) {
    name = String(name || "")
    command = String(command || "")
    style = String(style || "tile")
    icon = String(icon || "utilities-terminal")
    if (!name || !command || !icon) return
    if (style !== "float" && style !== "tile") return
    if (name.indexOf("/") !== -1 || name.charAt(0) === "-") return
    runJob(Requests.job("app.tui-install", { name: name, command: command, style: style, icon: icon }), "tui-install")
  }
  function installWebApp(name, url, icon) {
    name = String(name || "")
    url = String(url || "")
    icon = String(icon || "")
    if (!name || !url) return
    if (name.indexOf("/") !== -1 || name.charAt(0) === "-") return
    runJob(Requests.job("app.webapp-install", icon ? { name: name, url: url, icon: icon } : { name: name, url: url }), "webapp-install")
  }
  function removeDesktopApp(id, name) {
    id = String(id || "")
    name = String(name || id)
    if (!id) return
    runCommand(Requests.write("app.remove-desktop", { id: id, name: name }), { key: "desktop-remove:" + id, refresh: "none", sudo: true })
  }
  function removeTui(name) {
    name = String(name || "")
    if (!name) return
    runCommand(Requests.write("tui.remove", { name: name }), { key: "tui-remove:" + name, refresh: "none" })
  }
  function removeWebApp(name) {
    name = String(name || "")
    if (!name) return
    runCommand(Requests.write("webapp.remove", { name: name }), { key: "webapp-remove:" + name, refresh: "none" })
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
    var current = SnapshotJs.toList(trayHidden)
    if (JSON.stringify(next) === JSON.stringify(current)) return
    dispatchSetting("trayHidden", next)
  }
  function clearTrayHidden() {
    setTrayHidden([])
  }
  function setTrayPinned(list) {
    var next = normalizedStringIds(list)
    var current = SnapshotJs.toList(trayPinned)
    if (JSON.stringify(next) === JSON.stringify(current)) return
    dispatchSetting("trayPinned", next)
  }
  function clearTrayPinned() {
    setTrayPinned([])
  }
  function setDns(name) {
    if (name !== "Cloudflare" && name !== "Google" && name !== "DHCP") return
    if (name === dns) return
    dispatchSetting("dns", name)
  }
  function setCustomDns(servers) {
    servers = String(servers || "").replace(/^\s+|\s+$/g, "")
    if (!servers) return
    runJob(Requests.job("dns.custom", { servers: servers }), "dns-custom")
  }
  function openAether() { runCommand(Requests.write("aether", {})) }

  function setIdle(screensaver, lock) {
    var saver = Math.round(Number(screensaver)) || 0
    var lockSec = Math.round(Number(lock)) || 0
    var snap = SnapshotJs.mergeSnapshot(snapshotData, {
      idleScreensaver: saver,
      idleLock: lockSec
    })
    runSettingCommand(SettingsJs.commandFor("idleScreensaver", saver, snap, scriptOpts()), "idleScreensaver")
  }

  function setScreensaverBranding(action) {
    if (action !== "image" && action !== "text" && action !== "reset") return
    if (action === "reset") {
      if (!screensaverBranded) return
      runCommand(Requests.write("branding.reset", { which: "screensaver" }), { key: "screensaverBranding", refresh: "none" })
      return
    }
    runInteractive(Requests.job("branding.screensaver", { action: action }), {
      kind: "screensaver-branding",
      refresh: "all"
    })
  }

  function setAboutBranding(action) {
    if (action !== "image" && action !== "text" && action !== "reset") return
    if (action === "reset") {
      if (!aboutBranded) return
      runCommand(Requests.write("branding.reset", { which: "about" }), { key: "aboutBranding", refresh: "none" })
      return
    }
    runInteractive(Requests.job("branding.about", { action: action }), {
      kind: "about-branding",
      refresh: "all"
    })
  }

  function setTimezone(name) {
    name = String(name || "").replace(/^\s+|\s+$/g, "")
    if (!name || name === timezone) return
    if (!/^[A-Za-z0-9/_+-]+$/.test(name) || name.indexOf("..") !== -1) return
    dispatchSetting("timezone", name)
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
    runCommand(Requests.write("avatar.set", { user: currentUser, path: path }), { key: "avatar", refresh: "none", sudo: true })
  }

  function clearAvatar() {
    if (!currentUser) return
    runCommand(Requests.write("avatar.clear", { user: currentUser }), { key: "avatar", refresh: "none", sudo: true })
  }

  function addAccountUser(name, full, password, wheel) {
    name = AccountsJs.parseUsername(name)
    full = String(full || "").replace(/^\s+|\s+$/g, "")
    password = String(password || "")
    if (!name || !password || password.indexOf("\n") !== -1) return
    if (!AccountsJs.isFullName(full)) return
    runJob(Requests.job("account.add-user", { name: name, full: full, wheel: wheel === true, password: password }), "account-add", { sudo: true })
  }

  function removeAccountUser(name) {
    name = AccountsJs.parseUsername(name)
    if (!name || name === currentUser) return
    runJob(Requests.job("account.remove-user", { name: name }), "account-remove", { sudo: true })
  }

  function setAccountPassword(name, password) {
    name = AccountsJs.parseUsername(name)
    password = String(password || "")
    if (!name || !password || password.indexOf("\n") !== -1) return
    runJob(Requests.job("account.set-password", { name: name, password: password }), "account-password", { sudo: true })
  }

  function addAccountGroup(name) {
    name = AccountsJs.parseGroupName(name)
    if (!name) return
    runJob(Requests.job("account.add-group", { name: name }), "account-group-add", { sudo: true })
  }

  function removeAccountGroup(name) {
    name = AccountsJs.parseGroupName(name)
    if (!name || name === "wheel" || name === "docker") return
    runJob(Requests.job("account.remove-group", { name: name }), "account-group-remove", { sudo: true })
  }

  function setGroupMember(group, name, on) {
    group = AccountsJs.parseGroupName(group)
    name = AccountsJs.parseUsername(name)
    if (!group || !name) return
    if (on !== true && group === "wheel" && name === currentUser) return
    runCommand(Requests.write("account.member", { group: group, user: name, on: on === true }), {
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
    dispatchSetting("keyboardLayout", name, {
      id: "keyboardLayout",
      revert: { kind: "keyboardLayout", name: keyboardLayout }
    })
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

  function resetHyprLook() {
    if (!hyprLookManaged) return
    runCommand(Requests.write("hypr.reset", { group: "look" }), { key: "hyprLookManaged", refresh: "none" })
  }
  function toggleWorkspaceLayout() {
    runCommand(Requests.write("hypr.workspace-layout-toggle", {}), { key: "hyprWorkspaceLayout", refresh: "none" })
  }
  function toggleWindowTransparency() {
    runCommand(Requests.write("window.transparency-toggle", {}))
  }
  function toggleTiledFullscreen() {
    runCommand(Requests.write("window.tiled-fullscreen-toggle", {}))
  }

  function writeHyprKbOverride(layouts, variants, groupToggle) {
    var plan = HyprPrefs.kbOverridePlan(hyprInput, layouts, variants, groupToggle, SnapshotJs.mergeSnapshot)
    var snap = SnapshotJs.mergeSnapshot(snapshotData, { hyprInput: plan.input })
    var cmds = SettingsJs.planCommands(plan.rows, snap, scriptOpts())
    if (!cmds || !cmds.length || cmds[0].skip) return
    runCommand(Requests.write("hypr.apply", { group: "input", payload: cmds[0].stdin }), HyprPrefs.kbRunOptions(cmds[0], plan.revert, root.reverting))
  }

  function setHyprKbOverride(layouts, variants, groupToggle) {
    var rawLayouts = String(layouts || "").replace(/^\s+|\s+$/g, "")
    layouts = HyprPrefs.sanitizeLayoutList(layouts)
    if (rawLayouts && !layouts) return
    var rawVariants = String(variants || "").replace(/^\s+|\s+$/g, "")
    variants = layouts ? HyprPrefs.sanitizeVariantList(variants, layouts.split(",").length) : ""
    if (rawVariants && layouts && !variants) return
    groupToggle = groupToggle === true
    var input = hyprInput && typeof hyprInput === "object" ? hyprInput : {}
    if (layouts === (input.kbLayoutOverride || "") && variants === (input.kbVariantOverride || "") && groupToggle === (input.kbGroupToggle === true)) return
    writeHyprKbOverride(layouts, variants, groupToggle)
  }
  function setHyprWorkspaceGesture(on) {
    if (hyprWorkspaceGestureUnmanaged) return
    if (on === hyprWorkspaceGesture) return
    writeHyprInput({ workspaceGesture: on })
  }
  function resetHyprInput() {
    if (!hyprInputManaged) return
    runCommand(Requests.write("hypr.reset", { group: "input" }), { key: "hyprInputManaged", refresh: "none" })
  }

  function setNightlightTemperature(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 3000 || n > 6500) return
    if (n === nightlightTemperature) return
    dispatchSetting("nightlightTemperature", n)
  }

  function setupFingerprint() {
    runJob(Requests.job("security.fingerprint-setup", {}), "security-fingerprint", { sudo: true })
  }
  function removeFingerprint() {
    if (!fingerprintConfigured) return
    runJob(Requests.job("security.fingerprint-remove", {}), "security-fingerprint-remove", { sudo: true })
  }
  function setupFido2() {
    runJob(Requests.job("security.fido2-setup", {}), "security-fido2", { sudo: true })
  }
  function removeFido2() {
    if (!fido2Configured) return
    runJob(Requests.job("security.fido2-remove", {}), "security-fido2-remove", { sudo: true })
  }
  function setupSshd(key) {
    key = RichUi.parseSshPublicKey(key)
    if (!key) return
    runJob(Requests.job("security.sshd-setup", { key: key }), "security-sshd", { sudo: true })
  }
  function disableSshd() {
    if (!sshdEnabled && !sshdActive) return
    runJob(Requests.job("sshd.disable", {}), "security-sshd-disable", { sudo: true })
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
    runJob(Requests.job("sudo.enable", { minutes: minutes, password: password }), "passwordless-sudo")
  }
  function disablePasswordlessSudo() {
    if (!passwordlessSudo) return
    runJob(Requests.job("sudo.disable", {}), "passwordless-sudo-off", { sudo: true })
  }
  function setupSudolessDocker() {
    runJob(Requests.job("security.docker-setup", {}), "security-docker", { sudo: true })
  }
  function removeSudolessDocker() {
    if (!sudolessDocker) return
    runJob(Requests.job("security.docker-remove", {}), "security-docker-remove", { sudo: true })
  }

  function setOmarchyChannel(name) {
    if (name !== "stable" && name !== "rc" && name !== "edge" && name !== "dev") return
    if (name === omarchyChannel) return
    runJob(Requests.job("channel.set", { name: name }), "channel-set", { sudo: true })
  }
  function runOmarchyUpdate() {
    runJob(Requests.job("update.omarchy", {}), "omarchy-update", { sudo: true })
  }
  function checkOmarchyUpdate() {
    runJob(Requests.job("update.omarchy-check", {}), "update-check")
  }
  function setAtmosChannel(name) {
    if (AtmosUpdate.parseChannel(name) !== "stable") return
    if (name === atmosChannel) return
    runCommand(Requests.write("atmos.channel", { channel: "stable" }), { key: "atmosChannel", refresh: "none" })
  }
  function checkAtmosUpdate() {
    runJob(Requests.job("update.atmos-check", {}), "atmos-update-check")
  }
  function runAtmosUpdate() {
    runJob(Requests.job("update.atmos-apply", {}), "atmos-update")
  }
  function updateFirmware() {
    runJob(Requests.job("update.firmware", {}), "update-firmware", { sudo: true })
  }
  function updateOrphanPkgs() {
    runJob(Requests.job("update.orphans", {}), "update-orphans", { sudo: true })
  }
  function prunePkgCache() {
    runJob(Requests.job("update.prune", {}), "update-prune", { sudo: true })
  }

  function installVoxtype() {
    runJob(Requests.job("voxtype.install", {}), "voxtype-install", { sudo: true })
  }
  function removeVoxtype() {
    if (!voxtypeInstalled) return
    runJob(Requests.job("voxtype.remove", {}), "voxtype-remove", { sudo: true })
  }
  function toggleHybridGpu() {
    if (!hybridGpuAvailable) return
    runJob(Requests.job("hybrid-gpu.toggle", {}), "hybrid-gpu", { sudo: true })
  }
  function installTailscale() {
    runJob(Requests.job("tailscale.install", {}), "tailscale-install", { sudo: true })
  }
  function removeTailscale() {
    if (!tailscaleInstalled) return
    runJob(Requests.job("tailscale.remove", {}), "tailscale-remove", { sudo: true })
  }
  function setPluginEnabled(id, on) {
    id = String(id || "")
    if (!/^[A-Za-z0-9._-]+$/.test(id)) return
    var list = SnapshotJs.toList(plugins)
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].id) === id) {
        if ((list[i].enabled === true) === (on === true)) return
        break
      }
    }
    runCommand(Requests.write("plugin.set", { id: id, on: on === true }), { key: "plugin:" + id, refresh: "none" })
  }
  function setSnapperNumberLimit(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 1 || n > 50 || n === snapperNumberLimit) return
    runCommand(Requests.write("snapper.number-limit", { n: n }), { key: "snapperNumberLimit", refresh: "none", sudo: true })
  }
  function setSnapperTimeline(on) {
    if (on === snapperTimeline) return
    runCommand(Requests.write("snapper.timeline", { on: on === true }), { key: "snapperTimeline", refresh: "none", sudo: true })
  }
  function setFstrim(on) {
    if (on === fstrimEnabled) return
    runCommand(Requests.write("fstrim", { on: on === true }), { key: "fstrimEnabled", refresh: "none", sudo: true })
  }
  function setupDirectBoot() {
    if (!directBootAvailable) return
    runJob(Requests.job("direct-boot.setup", {}), "direct-boot", { sudo: true })
  }
  function setMimeDefault(kind, desktop) {
    if (kind !== "pdf" && kind !== "image" && kind !== "video") return
    desktop = String(desktop || "")
    if (!/^[A-Za-z0-9._-]+\.desktop$/.test(desktop)) return
    var key = kind === "pdf" ? "mimePdf" : kind === "image" ? "mimeImage" : "mimeVideo"
    dispatchSetting(key, desktop)
  }


  function setWifiBand(band) {
    if (band !== "auto" && band !== "2.4" && band !== "5" && band !== "6") return
    if (band === wifiBandSelected) return
    runCommand(Requests.write("wifi.band", { band: band }), { key: "wifiBandSelected", refresh: "none" })
  }

  function copyWifiPassword() {
    if (!wifiIface || !/^[a-zA-Z0-9._-]+$/.test(wifiIface)) return
    runCommand(Requests.write("wifi.password-copy", { iface: wifiIface }))
  }
  function fetchWifiQr() {
    var args = {}
    if (wifiIface && /^[a-zA-Z0-9._-]+$/.test(wifiIface)) args.iface = wifiIface
    runJob(Requests.job("wifi.qr", args), "wifi-qr", { refresh: "none" })
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
    runCommand(Requests.write("clipboard.copy", { text: text }))
  }

  function writeWorkspaces(items, wrap, wheel) {
    var list = Array.isArray(items) ? items : workspaces
    var wrapOn = wrap === true || wrap === false ? wrap : workspaceWrapSwitch !== false
    var wheelOn = wheel === true || wheel === false ? wheel : workspaceWheelSwitch !== false
    // The planner needs the new switches; the window's own state is not touched.
    var snap = SnapshotJs.mergeSnapshot(snapshotData, { workspaceWrapSwitch: wrapOn, workspaceWheelSwitch: wheelOn })
    runSettingCommand(SettingsJs.commandFor("workspaces", list, snap, scriptOpts()), "workspaces")
  }
  function writeMonitorRules(items) {
    var guard = null
    if (!root.reverting) {
      guard = {
        id: "monitorRules",
        revert: {
          kind: "monitorRules",
          rules: root.copyJson(Array.isArray(monitorRules) ? monitorRules : [])
        }
      }
    }
    dispatchSetting("monitorRules", items, guard)
  }

  function patchMonitorRule(output, patch) {
    var next = MonitorsJs.patchRules(monitorRules, monitors, output, patch, function(live) {
      return MonitorsJs.modeFromHyprctl(RichUi.currentMonitorModeValue(live))
    })
    if (next) writeMonitorRules(next)
  }

  function setConnectionMetered(uuid, value) {
    var op = NetworkPrefs.opFor("metered", { uuid: uuid, value: value })
    if (!op) return
    runCommand(Requests.write(op.name, op.args), { key: "wifi-metered:" + uuid, refresh: "network" })
  }
  function setConnectionPriority(uuid, value) {
    var op = NetworkPrefs.opFor("priority", { uuid: uuid, value: value })
    if (!op) return
    runCommand(Requests.write(op.name, op.args), { key: "wifi-priority:" + uuid, refresh: "network" })
  }
  function setConnectionMac(uuid, value) {
    var op = NetworkPrefs.opFor("mac", { uuid: uuid, value: value })
    if (!op) return
    runCommand(Requests.write(op.name, op.args), { key: "wifi-mac:" + uuid, refresh: "network" })
  }
  function setConnectionIpv4(uuid, spec) {
    spec = spec && typeof spec === "object" ? spec : {}
    spec.uuid = uuid
    var op = NetworkPrefs.opFor("ipv4", spec)
    if (!op) return
    runCommand(Requests.write(op.name, op.args), { key: "wifi-ipv4:" + uuid, refresh: "network" })
  }
  function importWireGuard(path) {
    var op = NetworkPrefs.opFor("wireguard-import", { path: path })
    if (!op) return
    runCommand(Requests.write(op.name, op.args), { key: "wireguard-import", refresh: "network" })
  }
  function setHotspot(on, ssid, password) {
    var op = NetworkPrefs.opFor("hotspot", { on: on === true, ssid: ssid, password: password })
    if (!op) return
    runCommand(Requests.write(op.name, op.args), { key: "wifi-hotspot", refresh: "network" })
  }
  function applyMonitorLayout(name) {
    var key = String(name || "")
    // Never write a layout that leaves no display on. The buttons already
    // gate this; refuse here too so no other caller can zero the outputs.
    if (key === "laptop" && !internalPresent) return
    if (key === "docked" && !externalPresent) return
    var items = MonitorsJs.layoutFromLive(key, monitors)
    if (items.length) writeMonitorRules(items)
  }
  function loadFavorites(raw) {
    favoriteItems = FavoritesJs.parseDocument(raw)
  }

  function toggleFavorite(row) {
    var next = FavoritesJs.toggleItem(favoriteItems, row)
    // The file's stamp brings the new list back; the window does not set it.
    runCommand(Requests.write("favorites.write", { json: FavoritesJs.serialize(next) }), { key: "favorites" })
  }

  function setPresentationMode(on) {
    on = on === true
    if (on === presentationMode) return
    dispatchSetting("presentationMode", on)
  }
  function setChargeLimit(n) {
    n = Math.round(Number(n))
    if (!isFinite(n) || n < 50 || n > 100 || n === chargeLimit) return
    dispatchSetting("chargeLimit", n)
  }
  function setEnvVars(vars, pathPrepend) {
    if (Array.isArray(vars)) dispatchSetting("envVars", vars)
    if (pathPrepend != null) dispatchSetting("envPathPrepend", String(pathPrepend))
  }
  function setTweak(id, on) {
    dispatchSetting("tweaks." + String(id || ""), on === true)
  }
  function signalProcess(pid, signal) {
    var args = ProcessesJs.signalArgs(pid, signal)
    if (!args) return
    runCommand(Requests.write("process.signal", args), { key: "process:" + pid, refresh: "none" })
  }

  function systemdAction(action, unit, scope) {
    if (action !== "start" && action !== "stop" && action !== "restart" && action !== "enable" && action !== "disable") return
    unit = String(unit || "")
    if (!/^[A-Za-z0-9:_.@\\-]+$/.test(unit)) return
    runCommand(Requests.write(scope === "user" ? "systemd.user" : "systemd.system", { action: action, unit: unit }), { key: "systemd:" + unit, refresh: "all", sudo: scope !== "user" })
  }
  function applyProfileValues(values) {
    if (!values || typeof values !== "object") return
    var key
    for (key in values) {
      if (!Object.prototype.hasOwnProperty.call(values, key)) continue
      dispatchSetting(key, values[key])
    }
  }
  function copyLastError() {
    var text = RichUi.clipboardPayload(lastError, { maxLength: 8192 })
    if (!text) return
    runCommand(Requests.write("clipboard.copy", { text: text }))
  }

  function copyDiagnosticReport() {
    var header = DiagnosticsJs.reportText(diagnostics)
    if (!header) return
    runJob(Requests.job("diag.report", { action: "copy", header: header }), "diag-report", { refresh: "none" })
  }

  function saveDiagnosticReport(path) {
    var dest = String(path || "")
    if (!dest || dest.indexOf("\n") !== -1 || dest.charAt(0) !== "/") return
    var header = DiagnosticsJs.reportText(diagnostics)
    if (!header) return
    runJob(Requests.job("diag.report", { action: "save", dest: dest, header: header }), "diag-report", { refresh: "none" })
  }

  function askAgentAboutDiagnostics() {
    var header = DiagnosticsJs.reportText(diagnostics)
    if (!header) return
    runJob(Requests.job("diag.report", { action: "agent", header: header }), "diag-report", { refresh: "none" })
  }

  function clearLastError() {
    lastError = ""
  }

  function askAgentAboutError() {
    var prompt = RichUi.agentErrorPrompt(lastError)
    if (!prompt) return
    runCommand(Requests.write("agent.prompt", { text: prompt }))
    lastError = ""
  }

  function showDebugError() {
    lastError = "Debug: this is the error banner. Copy puts it on the clipboard. Dismiss clears it."
  }
  function connectEnterpriseWifi(ssid, identity, password) {
    ssid = String(ssid || "")
    identity = String(identity || "")
    password = String(password || "")
    if (!ssid || !identity || !password) return
    if (ssid.length > 64 || identity.length > 256 || password.length > 256) return
    if (/[\r\n\0]/.test(ssid) || /[\r\n\0]/.test(identity) || /[\r\n\0]/.test(password)) return
    runJob(Requests.job("wifi.enterprise", { ssid: ssid, identity: identity, password: password }), "wifi-enterprise")
  }
  function wifiUuidForSsid(ssid) {
    ssid = String(ssid || "")
    var list = SnapshotJs.toList(wifiConnections)
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
    runJob(Requests.job("wifi.join", password.length ? { ssid: ssid, password: password } : { ssid: ssid }), "wifi-join")
  }
  function activateWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    var list = SnapshotJs.toList(wifiConnections)
    runCommand(Requests.write("wifi.connection", { action: "up", uuid: uuid }), { key: "wifi:" + uuid, refresh: "none" })
  }
  function deactivateWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    runCommand(Requests.write("wifi.connection", { action: "down", uuid: uuid }), { key: "wifi:" + uuid, refresh: "none" })
  }
  function forgetWifiConnection(uuid) {
    uuid = String(uuid || "")
    if (!/^[0-9a-fA-F-]{36}$/.test(uuid)) return
    runCommand(Requests.write("wifi.connection", { action: "delete", uuid: uuid }), { key: "wifi:" + uuid, refresh: "none" })
  }
  function deactivateWifiSsid(ssid) {
    ssid = String(ssid || "")
    if (!ssid || ssid.length > 64 || /[\r\n\0]/.test(ssid)) return
    var uuid = wifiUuidForSsid(ssid)
    if (uuid) {
      deactivateWifiConnection(uuid)
      return
    }
    runCommand(Requests.write("wifi.ssid", { action: "down-ssid", ssid: ssid }), { key: "wifi:" + ssid, refresh: "all" })
  }
  function forgetWifiSsid(ssid) {
    ssid = String(ssid || "")
    if (!ssid || ssid.length > 64 || /[\r\n\0]/.test(ssid)) return
    var uuid = wifiUuidForSsid(ssid)
    if (uuid) {
      forgetWifiConnection(uuid)
      return
    }
    runCommand(Requests.write("wifi.ssid", { action: "delete-ssid", ssid: ssid }), { key: "wifi:" + ssid, refresh: "all" })
  }
  function restartWifi() {
    runCommand(Requests.write("restart", { what: "wifi" }), { refresh: "network" })
  }
  function restartBluetooth() {
    runCommand(Requests.write("restart", { what: "bluetooth" }), { refresh: "network" })
  }
  function pairBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(Requests.write("bluetooth.pair", { address: address }), { key: "bluetooth:" + address, refresh: "all" })
  }
  function connectBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(Requests.write("bluetooth.device", { action: "connect", address: address }), { key: "bluetooth:" + address, refresh: "none" })
  }
  function disconnectBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(Requests.write("bluetooth.device", { action: "disconnect", address: address }), { key: "bluetooth:" + address, refresh: "none" })
  }
  function trustBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(Requests.write("bluetooth.trust", { address: address }), { key: "bluetooth-trust:" + address, refresh: "network" })
  }
  function forgetBluetoothDevice(address) {
    address = String(address || "")
    if (!/^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$/.test(address)) return
    runCommand(Requests.write("bluetooth.device", { action: "forget", address: address }), { key: "bluetooth:" + address, refresh: "none" })
  }
  function setAudioOutputVolume(percent) {
    percent = Math.round(Number(percent))
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    if (percent === audioOutputVolume && !audioOutputMuted) return
    dispatchSetting("audioOutputVolume", percent)
  }
  function toggleAudioOutputMute() {
    runCommand(Requests.write("audio.output.mute-toggle", {}), { key: "audioOutputMuted", refresh: "none" })
  }
  function setAudioInputVolume(percent) {
    percent = Math.round(Number(percent))
    if (!isFinite(percent) || percent < 0 || percent > 100) return
    if (percent === audioInputVolume && !audioInputMuted) return
    dispatchSetting("audioInputVolume", percent)
  }
  function toggleAudioInputMute() {
    runCommand(Requests.write("audio.input.mute-toggle", {}), { key: "audioInputMuted", refresh: "none" })
  }
  function setAudioSink(name) {
    name = String(name || "")
    if (!name || name === audioSink) return
    var list = audioSinks
    for (var i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].name) === name) {
        runCommand(Requests.write("audio.output.default", { id: String(list[i].id), name: name }), { key: "audioSink", refresh: "none" })
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
        runCommand(Requests.write("audio.input.default", { id: String(list[j].id), name: name }), { key: "audioSource", refresh: "none" })
        return
      }
    }
  }
  function switchAudioOutput() {
    runCommand(Requests.write("audio.output-switch", {}), { key: "audioSink", refresh: "all" })
  }
  function restartAudio() {
    runCommand(Requests.write("restart", { what: "audio" }))
  }
  function validMountPath(dir) {
    return RichUi.validMountPath(dir)
  }
  function openUserDir(dir) {
    dir = String(dir || "")
    if (!dir || dir.length > 1024) return
    if (dir.charAt(0) !== "/" || dir.indexOf("..") !== -1) return
    if (/[\r\n\0]/.test(dir)) return
    runCommand(Requests.write("open.path", { path: dir }))
  }
  function changeDrivePassword(device, currentPass, newPass) {
    device = String(device || "")
    currentPass = String(currentPass || "")
    newPass = String(newPass || "")
    if (!device || device.charAt(0) !== "/" || device.indexOf("..") !== -1) return
    if (!currentPass || !newPass) return
    runJob(Requests.job("luks.change-key", { device: device, current: currentPass, new: newPass }), "luks", { sudo: true })
  }
  function createSnapshot() {
    runJob(Requests.job("snapshot.create", {}), "snapshot-create", { sudo: true })
  }
  function restoreSnapshot(config, id) {
    config = String(config || "")
    id = String(id || "")
    if (!/^[A-Za-z0-9_-]+$/.test(config)) return
    if (!/^[0-9]+$/.test(id)) return
    runJob(Requests.job("snapshot.rollback", { config: config, id: id }), "snapshot-rollback", { sudo: true })
  }
  function setupHibernation() {
    runJob(Requests.job("hibernation.setup", {}), "hibernation-setup", { sudo: true })
  }
  function removeHibernation() {
    runJob(Requests.job("hibernation.remove", {}), "hibernation-remove", { sudo: true })
  }


  function setPowerProfile(name) {
    if (!name || name === powerProfile) return
    runCommand(Requests.write("power.profile", { name: name }), { key: "powerProfile", refresh: "none" })
  }




  function showBatteryNotification() {
    runCommand(Requests.write("notification", { kind: "battery" }))
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
    runCommand(Requests.write("weather.coordinates", { name: name, coords: coords }), { key: "weatherCoords", refresh: "none" })
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
    runCommand(Requests.write("reminder.set", { minutes: parseInt(minutes, 10), message: message }), { key: "reminder", refresh: "none" })
  }

  function clearReminders() {
    if (!reminderActive) return
    runCommand(Requests.write("reminder.clear", {}), { key: "reminder", refresh: "none" })
  }

  function showReminders() {
    runCommand(Requests.write("reminder.show", {}))
  }

  function sendTestNotification() {
    runCommand(Requests.write("notification.test", {}))
  }
  function sendTimeNotification() {
    runCommand(Requests.write("notification", { kind: "time" }))
  }
  function sendWeatherNotification() {
    runCommand(Requests.write("notification", { kind: "weather" }))
  }

  function captureScreenshot(mode, dest) {
    mode = String(mode || "smart")
    dest = String(dest || "slurp")
    if (mode !== "smart" && mode !== "region" && mode !== "windows" && mode !== "fullscreen") return
    if (dest !== "slurp" && dest !== "copy" && dest !== "save") return
    runInteractive(Requests.job("capture.screenshot", { mode: mode, dest: dest }), { kind: "screenshot" })
  }
  function startScreenrecording(desktopAudio, microphone, webcam, webcamSize, fullscreen) {
    var size = String(webcamSize || "medium")
    if (size !== "small" && size !== "medium" && size !== "large") size = "medium"
    var args = { fullscreen: !!fullscreen, desktopAudio: !!desktopAudio, microphone: !!microphone, webcam: !!webcam }
    if (webcam) args.webcamSize = size
    runInteractive(Requests.job("capture.recording", args), {
      kind: "recording",
      readback: { keys: ["recordingActive", "webcamOverlay"], expect: { recordingActive: true } },
      refresh: "none"
    })
  }
  // A picker still open is cancelled; once the recorder runs, the tool ends it.
  function stopScreenrecording() {
    if (!recordingActive && IoQueue.interactiveKind === "recording" && IoQueue.interactiveRunning) IoQueue.stopInteractive()
    else runCommand(Requests.write("recording.stop", {}), { key: "recordingActive", refresh: "none" })
  }
  function captureText() {
    runInteractive(Requests.job("capture.text", {}), { kind: "capture-text" })
  }
  function captureQr() {
    runInteractive(Requests.job("capture.qr", {}), { kind: "capture-qr" })
  }
  function resizeWebcam(action) {
    action = String(action || "")
    if (action !== "smaller" && action !== "larger" && action !== "reset" && action !== "small" && action !== "medium" && action !== "large") return
    runCommand(Requests.write("webcam.resize", { action: action }))
  }

  function shareClipboard() {
    runCommand(Requests.write("share.clipboard", {}))
  }
  function shareFile(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runCommand(Requests.write("share.file", { path: path }))
  }
  function shareFolder(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runCommand(Requests.write("share.folder", { path: path }))
  }
  function tailscaleSend(machine, path) {
    machine = String(machine || "").replace(/^\s+|\s+$/g, "")
    if (!machine || !/^[A-Za-z0-9._-]+$/.test(machine)) return
    path = String(path || "")
    if (path.length > 0) {
      if (path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
      runInteractive(Requests.job("tailscale.send", { machine: machine, path: path }), { kind: "tailscale-send" })
      return
    }
    runInteractive(Requests.job("tailscale.send", { machine: machine }), { kind: "tailscale-send" })
  }
  function tailscaleReceive() {
    runInteractive(Requests.job("tailscale.receive", {}), { kind: "tailscale-receive" })
  }

  function runSoftware(argv, kind) {
    if (!(argv instanceof Array) || argv.length < 2) return
    if (argv[0] !== "omarchy") return
    runJob(Requests.job("software.run", { action: argv[1], words: argv.slice(2) }), kind || "software", { sudo: true })
  }
  function installDevEnv(lang) {
    lang = String(lang || "")
    if (!/^[a-z]+$/.test(lang)) return
    runJob(Requests.job("dev-env.install", { lang: lang }), "dev-env-install", { sudo: true })
  }
  function removeDevEnv(lang) {
    lang = String(lang || "")
    if (!/^[a-z]+$/.test(lang)) return
    runJob(Requests.job("dev-env.remove", { lang: lang }), "dev-env-remove", { sudo: true })
  }
  function installDockerDb(name) {
    name = String(name || "")
    if (!/^[A-Za-z]+$/.test(name)) return
    runJob(Requests.job("docker-db.install", { name: name }), "docker-db-install", { sudo: true })
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
    runCommand(Requests.write("hook.install", { type: type, path: path }), { key: "hook:" + dest, refresh: "none" })
  }

  function createHook(type, name, command) {
    type = String(type || "")
    name = HooksJs.sanitizeName(name)
    command = HooksJs.sanitizeLine(command)
    if (!isHookId(type) || !name || !command) return
    var dest = hookDest(type, name)
    if (!dest) return
    runCommand(Requests.write("hook.create", { type: type, name: name, command: command }), { key: "hook:" + dest, refresh: "none" })
  }

  function removeHook(path) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0) return
    if (path.indexOf("..") !== -1) return
    if (path.length >= 7 && path.substring(path.length - 7) === ".sample") return
    runCommand(Requests.write("hook.remove", { path: path }), { key: "hook:" + path, refresh: "none" })
  }

  function setHookSample(path, enabled) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0 || path.indexOf("..") !== -1) return
    runCommand(Requests.write("hook.sample", { path: path, enabled: enabled === true }), { key: "hook:" + path, refresh: "none" })
  }

  function runHook(name, arg) {
    name = String(name || "")
    if (!isHookId(name)) return
    arg = String(arg || "").replace(/^\s+|\s+$/g, "")
    if (arg)
      runCommand(Requests.write("hook.run", { name: name, arg: arg }))
    else
      runCommand(Requests.write("hook.run", { name: name }))
  }

  function openHookFolder(type) {
    type = String(type || "")
    if (!isHookId(type)) return
    runCommand(Requests.write("hook.folder", { type: type }))
  }

  function editHook(path) {
    path = String(path || "")
    var root = Quickshell.env("HOME") + "/.config/omarchy/hooks/"
    if (path.indexOf(root) !== 0 || path.indexOf("..") !== -1) return
    runCommand(Requests.write("open.path", { path: path }))
  }

  function editMonitorsLua() {
    var path = String(Paths.monitorsLuaFile || "")
    var root = Quickshell.env("HOME") + "/.config/hypr/"
    if (path.indexOf(root) !== 0 || path.indexOf("..") !== -1) return
    runCommand(Requests.write("open.path", { path: path }))
  }

  function launchHerdr() {
    if (!(extras && extras.herdr === true)) return
    runCommand(Requests.write("terminal.herdr", {}))
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

  function managedAutostart() {
    var list = Array.isArray(autostart) ? autostart : []
    var out = []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].managed === true && list[i].command)
        out.push({
          command: String(list[i].command),
          delay: Math.round(Number(list[i].delay || 0)) || 0,
          enabled: list[i].enabled !== false
        })
    }
    return out
  }
  function writeAutostart(items) {
    dispatchSetting("autostart", items)
  }
  function addAutostart(command, delay) {
    command = String(command || "").replace(/^\s+|\s+$/g, "")
    if (!command || command.length > 256 || command.indexOf("\n") !== -1) return
    var n = Math.round(Number(delay || 0))
    if (!isFinite(n) || n < 0) n = 0
    if (n > 600) n = 600
    var next = managedAutostart()
    for (var i = 0; i < next.length; i++) {
      if (next[i].command === command) return
    }
    next.push({ command: command, delay: n, enabled: true })
    writeAutostart(next)
  }
  function removeAutostart(command) {
    command = String(command || "")
    var cur = managedAutostart()
    var next = []
    for (var i = 0; i < cur.length; i++) {
      if (cur[i].command !== command) next.push(cur[i])
    }
    if (next.length === cur.length) return
    writeAutostart(next)
  }
  function setAutostartEnabled(command, on) {
    command = String(command || "")
    var cur = managedAutostart()
    var next = []
    var found = false
    for (var i = 0; i < cur.length; i++) {
      if (cur[i].command === command) {
        next.push({ command: cur[i].command, delay: cur[i].delay, enabled: on !== false })
        found = true
      } else next.push(cur[i])
    }
    if (!found) return
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
          workspace: String(list[i].workspace || ""),
          title: String(list[i].title || ""),
          pin: list[i].pin === true,
          fullscreen: list[i].fullscreen === true,
          opacity: String(list[i].opacity || "")
        })
    }
    return out
  }

  function writeWindowRules(items) {
    dispatchSetting("windowRules", items)
  }

  function addWindowRule(match, placement, center, width, height, workspace, extras) {
    match = String(match || "").replace(/^\s+|\s+$/g, "")
    placement = String(placement || "")
    if (placement !== "float" && placement !== "tile") placement = ""
    workspace = String(workspace || "").replace(/^\s+|\s+$/g, "")
    width = Math.round(Number(width)) || 0
    height = Math.round(Number(height)) || 0
    extras = extras && typeof extras === "object" ? extras : {}
    var title = String(extras.title || "").replace(/^\s+|\s+$/g, "")
    var opacity = String(extras.opacity || "").replace(/^\s+|\s+$/g, "")
    if (!match || match.length > 128 || match.indexOf("\n") !== -1 || match.indexOf("]]") !== -1) return
    if (!(width >= 100 && height >= 100)) {
      width = 0
      height = 0
    }
    if (!placement && center !== true && !width && !workspace && !title && extras.pin !== true && extras.fullscreen !== true && !opacity) return
    var cur = managedWindowRules()
    var next = []
    for (var i = 0; i < cur.length; i++) {
      if (cur[i].match !== match) next.push(cur[i])
    }
    next.push({
      match: match,
      title: title,
      placement: placement,
      center: center === true,
      width: width,
      height: height,
      workspace: workspace,
      pin: extras.pin === true,
      fullscreen: extras.fullscreen === true,
      opacity: opacity
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

  function openPrinters() {
    if (printerSetup)
      runCommand(Requests.write("printers.setup", {}))
    else
      openCupsAdmin()
  }

  function openCupsAdmin() {
    runCommand(Requests.write("cups.admin", {}))
  }

  function restartShell() {
    runCommand(Requests.write("restart", { what: "shell" }))
  }

  function refreshHyprland() {
    runJob(Requests.job("refresh.hyprland", {}), "refresh-hyprland")
  }

  function refreshShell() {
    runJob(Requests.job("refresh.shell", {}), "refresh-shell")
  }

  function resetAtmos() {
    runJob(Requests.job("reset.atmos", {}), "reset-atmos")
  }

  function setPlymouth(name) {
    if (!name || name === "default") return
    if (name === plymouth) return
    dispatchSetting("plymouth", name)
  }

  function resetPlymouth() {
    if (plymouth === "default") return
    runJob(Requests.job("plymouth.reset", {}), "plymouth-reset", { sudo: true })
  }
  function setPlymouthFromPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runJob(Requests.job("plymouth.set", { path: path }), "plymouth-set", { sudo: true })
  }
  function previewPlymouthFromPath(path) {
    path = String(path || "")
    if (!path || path.charAt(0) !== "/" || path.indexOf("..") !== -1) return
    runJob(Requests.job("plymouth.preview", { path: path }), "plymouth-preview")
  }

  function installedOptions(all, available) {
    return RichUi.installedOptions(all, available)
  }

  Component.onCompleted: {
    Theme.currentThemeSwapped.connect(root.applyThemeNameFromFile)
    IoQueue.setRequested.connect(root.onSetRequested)
    IoQueue.writeStarting.connect(root.onWriteStarting)
    IoQueue.jobStarting.connect(root.onJobStarting)
    IoQueue.jobExited.connect(root.onJobExited)
    IoQueue.interactiveExited.connect(root.onInteractiveExited)
    // The feeds put their topics to the daemon again by themselves.
    Backend.reconnected.connect(IoQueue.resumeFromDaemon)
    startSession()
  }

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
}
