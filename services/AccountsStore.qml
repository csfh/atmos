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

  readonly property string hostnameFile: "/etc/hostname"
  readonly property string passwdFile: "/etc/passwd"
  readonly property string groupFile: "/etc/group"
  readonly property string faceIconFile: Quickshell.env("HOME") + "/.face.icon"
  readonly property string faceFile: Quickshell.env("HOME") + "/.face"
  readonly property string homeDir: Quickshell.env("HOME") || ""

  function pathExists(path) {
    var target = String(path || "")
    if (!target) return false
    peekFile.path = ""
    peekFile.path = target
    peekFile.reload()
    peekFile.waitForJob()
    return peekFile.loaded
  }

  function readPath(path) {
    var target = String(path || "")
    if (!target) return ""
    peekFile.path = target
    peekFile.reload()
    peekFile.waitForJob()
    return peekFile.text() || ""
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
    var seeded = AccountsJs.seedFromDisk({
      currentUser: Quickshell.env("USER") || Quickshell.env("LOGNAME") || "",
      hostname: root.readPath(root.hostnameFile),
      passwd: root.readPath(root.passwdFile),
      group: root.readPath(root.groupFile),
      home: root.homeDir,
      exists: root.pathExists
    })
    root.applyPatch(seeded)
  }

  property FileView peekFile: FileView {
    printErrors: false
    blockLoading: true
  }

  property var watchPaths: [
    hostnameFile, passwdFile, groupFile, faceIconFile, faceFile
  ]

  property Instantiator fileWatchers: Instantiator {
    model: root.watchPaths
    delegate: FileView {
      path: modelData
      watchChanges: true
      printErrors: false
      onFileChanged: {
        reload()
        diskDebounce.restart()
      }
    }
  }

  property Timer diskDebounce: Timer {
    interval: 80
    repeat: false
    onTriggered: root.reloadFromDisk()
  }

  Component.onCompleted: root.reloadFromDisk()
}
