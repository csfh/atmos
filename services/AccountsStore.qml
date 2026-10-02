pragma Singleton
import QtQuick
import Quickshell
import "Accounts.js" as AccountsJs
import "Requests.js" as Requests

QtObject {
  id: root

  property string hostname: ""
  property string fullName: ""
  property string currentUser: ""
  property string avatarPath: ""
  property var users: []
  property var groups: []

  readonly property string homeDir: Quickshell.env("HOME") || ""

  readonly property string userName: Quickshell.env("USER") || Quickshell.env("LOGNAME") || ""

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

  // Passwd, group and hostname are pushed when they change. This asks once.
  function reloadFromDisk() {
    Backend.request(Requests.hostAccounts(root.userName, root.homeDir), function(env) {
      if (env && env.ok === true && env.result) root.adoptAccounts(env.result)
    })
  }

  function adoptAccounts(doc) {
    var present = doc && doc.exists ? doc.exists : {}
    var seeded = AccountsJs.seedFromDisk({
      currentUser: root.userName,
      hostname: doc ? String(doc.hostname || "") : "",
      passwd: doc ? String(doc.passwd || "") : "",
      group: doc ? String(doc.group || "") : "",
      home: root.homeDir,
      exists: function(path) { return present[String(path)] === true }
    })
    root.applyPatch(seeded)
  }

  Component.onCompleted: {
    Backend.accounts.connect(root.adoptAccounts)
    Backend.watch("accounts", { accounts: { user: root.userName, home: root.homeDir } })
    root.reloadFromDisk()
  }
}
