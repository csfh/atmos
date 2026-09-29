pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "Accounts.js" as AccountsJs

QtObject {
  id: root

  property string hostname: ""
  property string fullName: ""
  property string currentUser: ""
  property string avatarPath: ""
  property var users: []
  property var groups: []

  readonly property string homeDir: Quickshell.env("HOME") || ""

  function backendRequest() {
    var bin = String(Quickshell.env("ATMOS_BACKEND") || "")
    if (!bin.length) bin = String(Quickshell.shellDir || "") + "/bin/ratmos"
    var id = String(Quickshell.env("ATMOS_BACKEND_ID") || "omarchy")
    return [bin, "--backend", id, "request"]
  }

  function applyPatch(parsed) {
    var next = AccountsJs.applyAccountPatch({
      hostname: hostname,
      fullName: fullName,
      currentUser: currentUser,
      avatarPath: avatarPath,
      users: users,
      groups: groups
    }, parsed)
    hostname = next.hostname
    fullName = next.fullName
    currentUser = next.currentUser
    avatarPath = next.avatarPath
    users = next.users
    groups = next.groups
  }

  function reloadFromDisk() {
    if (accountProc.running) return
    accountStdin = JSON.stringify({
      op: "host.accounts",
      user: Quickshell.env("USER") || Quickshell.env("LOGNAME") || "",
      home: root.homeDir
    })
    accountProc.command = root.backendRequest()
    accountProc.stdinEnabled = true
    accountProc.running = true
  }

  function adoptAccounts(doc) {
    var present = doc && doc.exists ? doc.exists : {}
    var seeded = AccountsJs.seedFromDisk({
      currentUser: Quickshell.env("USER") || Quickshell.env("LOGNAME") || "",
      hostname: doc ? String(doc.hostname || "") : "",
      passwd: doc ? String(doc.passwd || "") : "",
      group: doc ? String(doc.group || "") : "",
      home: root.homeDir,
      exists: function(path) { return present[String(path)] === true }
    })
    root.applyPatch(seeded)
  }

  property string accountStdin: ""

  property Timer accountTimer: Timer {
    interval: 2000
    running: true
    repeat: true
    onTriggered: root.reloadFromDisk()
  }

  property Process accountProc: Process {
    command: ["true"]
    stdinEnabled: false
    stdout: StdioCollector { id: accountOut; waitForEnd: true }
    onStarted: {
      if (root.accountStdin.length > 0) {
        write(root.accountStdin)
        root.accountStdin = ""
        stdinEnabled = false
      }
    }
    onExited: function(code) {
      if (code !== 0) return
      var env = null
      try { env = JSON.parse(String(accountOut.text || "")) } catch (e) { env = null }
      if (!env || env.ok !== true || !env.result) return
      root.adoptAccounts(env.result)
    }
  }

  Component.onCompleted: root.reloadFromDisk()
}
