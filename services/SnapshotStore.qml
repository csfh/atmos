pragma Singleton
import QtQuick
import "SnapshotGroups.js" as SnapshotGroups
import "WorkQueue.js" as WorkQueue

// What to read and when. It decides which snapshot groups to fetch at the
// start of a session, batches refresh requests, and watches the files each
// group is built from. The backend pushes a stamp when one of them changes, and
// the changed file's group is read again. What a snapshot means is Omarchy's
// business; this only schedules the reads.
QtObject {
  id: root

  property var pending: []
  property var watchSig: ({})

  // The first file or two Omarchy reads itself get a signal with their text.
  signal stamped(string path, string text, bool first, bool changed)

  readonly property var specs: SnapshotGroups.watchSpecs({
    userShellJson: Paths.userShellJson,
    defaultShellJson: Paths.defaultShellJson,
    userShellToml: Paths.userShellToml,
    weatherJson: Paths.weatherJson,
    notificationsJson: Paths.notificationsJson,
    currentBackgroundFile: Paths.currentBackgroundFile,
    screensaverBrandFile: Paths.screensaverBrandFile,
    defaultScreensaverBrandFile: Paths.defaultScreensaverBrandFile,
    aboutBrandFile: Paths.aboutBrandFile,
    defaultAboutBrandFile: Paths.defaultAboutBrandFile,
    plymouthLogoFile: Paths.plymouthLogoFile,
    defaultPlymouthLogoFile: Paths.defaultPlymouthLogoFile,
    packagedThemesDir: Paths.packagedThemesDir,
    fontconfigFile: Paths.fontconfigFile,
    indicatorsDir: Paths.indicatorsDir,
    reminderDir: Paths.reminderDir,
    looknfeelLuaFile: Paths.looknfeelLuaFile,
    hyprsunsetConfFile: Paths.hyprsunsetConfFile,
    monitorsLuaFile: Paths.monitorsLuaFile,
    hyprTogglesDir: Paths.hyprTogglesDir,
    touchpadDisabledFile: Paths.touchpadDisabledFile,
    touchscreenDisabledFile: Paths.touchscreenDisabledFile,
    togglesDir: Paths.togglesDir,
    powerProfileAcFile: Paths.powerProfileAcFile,
    powerProfileBatteryFile: Paths.powerProfileBatteryFile,
    powerProfilesStateFile: Paths.powerProfilesStateFile,
    applicationsDir: Paths.applicationsDir,
    defaultEditorFile: Paths.defaultEditorFile,
    defaultAgentFile: Paths.defaultAgentFile,
    defaultTerminalFile: Paths.defaultTerminalFile,
    defaultBrowserFile: Paths.defaultBrowserFile,
    dnsConfFile: Paths.dnsConfFile,
    bluetoothRfkillDir: Paths.bluetoothRfkillDir,
    networkManagerDevicesDir: Paths.networkManagerDevicesDir,
    inputLuaFile: Paths.inputLuaFile,
    autostartLuaFile: Paths.autostartLuaFile,
    bindingsLuaFile: Paths.bindingsLuaFile,
    windowsLuaFile: Paths.windowsLuaFile,
    envFile: Paths.envFile,
    presentationFile: Paths.presentationFile,
    localtimeFile: Paths.localtimeFile,
    vconsoleFile: Paths.vconsoleFile,
    localeConfFile: Paths.localeConfFile,
    pacmanConfFile: Paths.pacmanConfFile,
    gtkSettingsFile: Paths.gtkSettingsFile,
    swappinessFile: Paths.swappinessFile
  })

  function scheduleRefresh(group) {
    root.pending = WorkQueue.addPendingRefresh(root.pending, SnapshotGroups.normalizeGroup(group))
    refreshTimer.restart()
  }

  function enqueueRead(group) {
    IoQueue.enqueueRead(SnapshotGroups.normalizeGroup(group))
  }

  // The hub the session opens on first, then the rest.
  function startSession(hub) {
    var first = SnapshotGroups.snapshotGroupForHub(hub)
    WorkQueue.enqueueRead(IoQueue.queue, first)
    if (first !== "all") WorkQueue.enqueueRead(IoQueue.queue, "rest")
    IoQueue.kick()
  }

  function watchPaths() {
    var list = root.specs || []
    var paths = []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && list[i].path) paths.push(String(list[i].path))
    }
    paths.push(Paths.extraThemesDir)
    paths.push(Paths.favoritesFile)
    paths.push(Paths.inputLuaFile)
    return paths
  }

  // Tell the backend which files to watch. It pushes a stamp when one changes,
  // so there is no poll here.
  function syncWatch() {
    Backend.watch("omarchy", { paths: root.watchPaths() })
  }

  onSpecsChanged: root.syncWatch()

  function groupForPath(path) {
    var list = root.specs || []
    var i
    for (i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].path) === path) return String(list[i].group || "")
    }
    return ""
  }

  function adoptStamp(doc) {
    var items = doc && doc.items ? doc.items : []
    var next = {}
    var i, item, path, sig, prev
    for (i = 0; i < items.length; i++) {
      item = items[i]
      path = String(item.path || "")
      sig = String(item.sig || "")
      next[path] = sig
      prev = root.watchSig[path]
      var changed = prev !== undefined && prev !== sig
      root.stamped(path, String(item.text || ""), prev === undefined, changed)
      if (changed) {
        if (path === Paths.extraThemesDir) root.scheduleRefresh("look")
        else {
          var group = root.groupForPath(path)
          if (group) root.scheduleRefresh(group)
        }
      }
    }
    root.watchSig = next
  }

  property Timer refreshTimer: Timer {
    interval: 180
    repeat: false
    onTriggered: {
      var groups = root.pending
      root.pending = []
      var i
      for (i = 0; i < groups.length; i++) root.enqueueRead(groups[i])
    }
  }

  Component.onCompleted: {
    Backend.stamp.connect(root.adoptStamp)
    root.syncWatch()
  }
}
