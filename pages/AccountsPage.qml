import QtQuick
import QtQuick.Dialogs
import "../components"
import "../services"
import "../services/Accounts.js" as AccountsJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Accounts"
  description: "This login's name and face, other local users, and groups. Fingerprint, SSH, and sudoless Docker stay on Security."

  property string selectedGroup: ""
  property string pendingUser: ""
  property string pendingGroup: ""
  property string passwordUser: ""
  property string addError: ""
  property string addGroupError: ""
  property string passwordError: ""
  property bool addUserAdmin: false
  property string fullNameDraft: Omarchy.fullName
  readonly property string fullNameParsed: AccountsJs.parseFullName(root.fullNameDraft)
  readonly property bool fullNameValid: AccountsJs.isFullName(root.fullNameDraft)

  function groupByName(name) {
    var groups = Omarchy.accountGroups || []
    var i
    for (i = 0; i < groups.length; i++) {
      if (groups[i] && groups[i].name === name)
        return groups[i]
    }
    return null
  }

  function userExists(name) {
    var users = Omarchy.accountUsers || []
    var i
    for (i = 0; i < users.length; i++) {
      if (users[i] && users[i].name === name)
        return true
    }
    return false
  }

  function groupExists(name) {
    return root.groupByName(name) !== null
  }

  function userInGroup(groupName, userName) {
    return AccountsJs.groupHasMember(root.groupByName(groupName), userName)
  }

  function memberLocked(groupName, row) {
    return !!(
      groupName === "wheel"
      && row
      && row.current
      && root.userInGroup("wheel", row.name)
    )
  }

  function userBlurb(row) {
    var bits = []
    if (!row) return ""
    if (row.fullName) bits.push(row.fullName)
    bits.push("uid " + row.uid)
    if (row.wheel) bits.push("admin")
    if (row.current) bits.push("this session")
    return bits.join(" · ")
  }

  function groupBlurb(row) {
    var members = row && row.members ? row.members : []
    var n = members.length
    var gid = row && row.gid !== undefined ? row.gid : ""
    if (n === 1) return "gid " + gid + " · 1 member"
    return "gid " + gid + " · " + n + " members"
  }

  function openAddUser() {
    root.addError = ""
    root.addUserAdmin = false
    addUserName.clear()
    addUserFull.clear()
    addUserPassword.clear()
    addUserConfirm.clear()
    addUserDialog.open()
  }

  function submitAddUser() {
    var name = AccountsJs.parseUsername(addUserName.currentText())
    var full = String(addUserFull.currentText() || "").replace(/^\s+|\s+$/g, "")
    var password = addUserPassword.currentText()
    var confirm = addUserConfirm.currentText()
    if (!name) {
      root.addError = "Username must be lowercase, start with a letter or underscore, and stay under 32 characters."
      return
    }
    if (root.userExists(name)) {
      root.addError = name + " already exists."
      return
    }
    if (!AccountsJs.isFullName(full)) {
      root.addError = "Full name cannot contain a colon or comma."
      return
    }
    if (!password) {
      root.addError = "Password cannot be empty."
      return
    }
    if (password !== confirm) {
      root.addError = "Passwords do not match."
      return
    }
    root.addError = ""
    Omarchy.addAccountUser(name, full, password, root.addUserAdmin)
    addUserDialog.close()
  }

  function openPassword(name) {
    root.passwordUser = name
    root.passwordError = ""
    passwordField.clear()
    passwordConfirm.clear()
    passwordDialog.open()
  }

  function submitPassword() {
    var password = passwordField.currentText()
    var confirm = passwordConfirm.currentText()
    if (!password) {
      root.passwordError = "Password cannot be empty."
      return
    }
    if (password !== confirm) {
      root.passwordError = "Passwords do not match."
      return
    }
    root.passwordError = ""
    Omarchy.setAccountPassword(root.passwordUser, password)
    passwordDialog.close()
  }

  function openManageGroup(name) {
    root.selectedGroup = name || ""
    if (!root.selectedGroup) return
    manageGroupDialog.open()
  }

  function openAddGroup() {
    root.addGroupError = ""
    addGroupName.clear()
    addGroupDialog.open()
  }

  function submitAddGroup() {
    var name = AccountsJs.parseGroupName(addGroupName.currentText())
    if (!name) {
      root.addGroupError = "Group name must be lowercase, start with a letter or underscore, and stay under 32 characters."
      return
    }
    if (root.groupExists(name)) {
      root.addGroupError = name + " already exists."
      return
    }
    root.addGroupError = ""
    Omarchy.addAccountGroup(name)
    addGroupDialog.close()
  }

  function askRemoveUser(name) {
    root.pendingUser = name
    removeUserConfirm.ask()
  }

  function askRemoveGroup(name) {
    root.pendingGroup = name
    removeGroupConfirm.ask()
  }

  Component.onCompleted: {
    removeUserConfirm.parent = root.prefsOverlay
    removeGroupConfirm.parent = root.prefsOverlay
    clearAvatarConfirm.parent = root.prefsOverlay
    addUserDialog.parent = root.prefsOverlay
    addGroupDialog.parent = root.prefsOverlay
    manageGroupDialog.parent = root.prefsOverlay
    passwordDialog.parent = root.prefsOverlay
  }

  Connections {
    target: Omarchy
    function onFullNameChanged() { root.fullNameDraft = Omarchy.fullName }
  }

  FileDialog {
    id: avatarDialog
    title: "Choose a face"
    nameFilters: ["Images (*.png *.jpg *.jpeg)"]
    onAccepted: Omarchy.setAvatarPath(RichUi.pathFromUrl(selectedFile))
  }

  PrefsConfirm {
    id: clearAvatarConfirm
    title: "Clear face"
    message: "Remove the face icon for this login."
    confirmText: "Clear"
    onConfirmed: Omarchy.clearAvatar()
  }

  PrefsConfirm {
    id: removeUserConfirm
    title: "Remove user"
    message: "Remove " + root.pendingUser + " and their home directory."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeAccountUser(root.pendingUser)
  }

  PrefsConfirm {
    id: removeGroupConfirm
    title: "Remove group"
    message: "Remove the " + root.pendingGroup + " group. Logins stay."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeAccountGroup(root.pendingGroup)
  }

  PrefsDialog {
    id: addUserDialog
    title: "Add a user"

    PrefsText {
      width: parent.width
      text: Omarchy.jobKind === "account-add" && Omarchy.jobBusy
        ? "Creating the login…"
        : "A local login with a home from /etc/skel."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: addUserName
      width: parent.width
      placeholder: "username"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAddUser() }
    }

    PrefsField {
      id: addUserFull
      width: parent.width
      placeholder: "Full name (optional)"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAddUser() }
    }

    PrefsPassword {
      id: addUserPassword
      width: parent.width
      placeholder: "Password"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAddUser() }
    }

    PrefsPassword {
      id: addUserConfirm
      width: parent.width
      placeholder: "Confirm password"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAddUser() }
    }

    Item {
      width: parent.width
      implicitHeight: Math.max(adminCopy.implicitHeight, Theme.toggleHeight)
      height: implicitHeight

      PrefsToggle {
        x: 0
        y: Math.round((parent.height - height) / 2)
        checked: root.addUserAdmin
        enabled: !Omarchy.jobBusy
        Accessible.name: "Admin"
        onToggled: root.addUserAdmin = !root.addUserAdmin
      }

      Column {
        id: adminCopy
        width: parent.width - Theme.toggleWidth - Theme.spaceMd
        x: Theme.toggleWidth + Theme.spaceMd
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.labelGap

        PrefsText {
          width: parent.width
          text: "Admin"
          color: Theme.foreground
          font.family: Theme.fontFamily
          font.pixelSize: Theme.labelSize
          font.bold: true
        }

        PrefsText {
          width: parent.width
          text: "Puts this login in wheel."
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.descriptionSize
        }
      }
    }

    PrefsText {
      width: parent.width
      visible: root.addError.length > 0
      text: root.addError
      color: Theme.urgent
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Row {
      anchors.right: parent.right
      spacing: Theme.space

      PrefsButton {
        text: "Cancel"
        onClicked: addUserDialog.close()
      }

      PrefsButton {
        text: "Add"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.submitAddUser()
      }
    }
  }

  PrefsDialog {
    id: addGroupDialog
    title: "Add a group"

    PrefsText {
      width: parent.width
      text: "A local group. You can add logins to it after it exists."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: addGroupName
      width: parent.width
      placeholder: "group"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitAddGroup() }
    }

    PrefsText {
      width: parent.width
      visible: root.addGroupError.length > 0
      text: root.addGroupError
      color: Theme.urgent
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Row {
      anchors.right: parent.right
      spacing: Theme.space

      PrefsButton {
        text: "Cancel"
        onClicked: addGroupDialog.close()
      }

      PrefsButton {
        text: "Add"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.submitAddGroup()
      }
    }
  }

  PrefsDialog {
    id: passwordDialog
    title: "Set password"

    PrefsText {
      width: parent.width
      text: "New password for " + root.passwordUser + "."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsPassword {
      id: passwordField
      width: parent.width
      placeholder: "New password"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitPassword() }
    }

    PrefsPassword {
      id: passwordConfirm
      width: parent.width
      placeholder: "Confirm password"
      enabled: !Omarchy.jobBusy
      onSubmitted: function() { root.submitPassword() }
    }

    PrefsText {
      width: parent.width
      visible: root.passwordError.length > 0
      text: root.passwordError
      color: Theme.urgent
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Row {
      anchors.right: parent.right
      spacing: Theme.space

      PrefsButton {
        text: "Cancel"
        onClicked: passwordDialog.close()
      }

      PrefsButton {
        text: "Set"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.submitPassword()
      }
    }
  }

  PrefsDialog {
    id: manageGroupDialog
    title: "Manage " + root.selectedGroup
    onClosed: root.selectedGroup = ""

    PrefsText {
      width: parent.width
      text: root.selectedGroup === "wheel"
        ? "Add or remove logins in wheel. This session has to stay."
        : "Add or remove logins in this group."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsText {
      width: parent.width
      visible: Omarchy.accountUsers.length === 0
      text: "No human logins to add."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Column {
      id: memberCol
      width: parent.width
      visible: Omarchy.accountUsers.length > 0

      Repeater {
        model: Omarchy.accountUsers

        Column {
          required property var modelData
          required property int index
          width: parent ? parent.width : 0
          spacing: Theme.labelGap

          Rectangle {
            width: parent.width
            height: 1
            visible: index > 0
            color: Theme.splitColor()
          }

          Item {
            width: parent.width
            implicitHeight: Math.max(memberCopy.implicitHeight, Theme.toggleHeight)
            height: implicitHeight

            Column {
              id: memberCopy
              width: parent.width - Theme.toggleWidth - Theme.spaceMd
              anchors.left: parent.left
              anchors.verticalCenter: parent.verticalCenter
              spacing: Theme.labelGap

              PrefsText {
                width: parent.width
                text: modelData && modelData.name ? modelData.name : "user"
                color: Theme.foreground
                font.family: Theme.fontFamily
                font.pixelSize: Theme.labelSize
                font.bold: true
              }

              PrefsText {
                width: parent.width
                text: root.memberLocked(root.selectedGroup, modelData)
                  ? "This session has to stay in wheel."
                  : root.userBlurb(modelData)
                color: Theme.muted
                font.family: Theme.fontFamily
                font.pixelSize: Theme.descriptionSize
              }
            }

            PrefsToggle {
              x: parent.width - width
              y: Math.round((parent.height - height) / 2)
              checked: root.userInGroup(root.selectedGroup, modelData && modelData.name ? modelData.name : "")
              enabled: !Omarchy.jobBusy && !!(modelData && modelData.name) && !root.memberLocked(root.selectedGroup, modelData)
              onToggled: {
                var name = modelData && modelData.name ? modelData.name : ""
                if (!name || !root.selectedGroup) return
                Omarchy.setGroupMember(root.selectedGroup, name, !root.userInGroup(root.selectedGroup, name))
              }
            }
          }
        }
      }
    }

    Item {
      width: parent.width
      height: Theme.space
    }

    Row {
      anchors.right: parent.right
      spacing: Theme.space

      PrefsButton {
        text: "Done"
        primary: true
        onClicked: manageGroupDialog.close()
      }
    }
  }

  PrefsGroup {
    title: "This account"
    query: root.query
    detail: "Face is a PNG or JPEG copied to ~/.face.icon and AccountsService. Full name is the GECOS real name. Password changes this login."

    SettingRow {
      label: Omarchy.currentUser.length ? Omarchy.currentUser : "Face"
      description: AccountsJs.faceRowDescription(Omarchy.avatarPath)
      hint: "~/.face.icon"
      query: root.query
      keywords: ["avatar", "face", "icon", "picture", "sddm", "accountsservice"]

      Row {
        spacing: Theme.space

        Image {
          visible: Omarchy.avatarPath.length > 0
          width: 48
          height: 48
          fillMode: Image.PreserveAspectCrop
          asynchronous: true
          source: Omarchy.avatarPath.length ? ("file://" + encodeURI(Omarchy.avatarPath)) : ""
        }

        PrefsButton {
          text: "Choose…"
          primary: true
          enabled: !Omarchy.jobBusy && Omarchy.currentUser.length > 0
          onClicked: avatarDialog.open()
        }

        PrefsButton {
          visible: Omarchy.avatarPath.length > 0
          text: "Clear…"
          danger: true
          enabled: !Omarchy.jobBusy && Omarchy.avatarPath.length > 0
          onClicked: clearAvatarConfirm.ask()
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Full name"
      description: root.fullNameValid || root.fullNameDraft.length === 0
        ? "The name on this account. Login screens and user lists show it."
        : "No colon, comma, or leading hyphen (GECOS splits on comma). Set stays off until the name is valid."
      hint: "chfn --full-name"
      query: root.query
      keywords: ["user", "gecos", "real name", "display name", "account"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: fullNameField
          width: parent.width - fullNameSetBtn.width - parent.spacing
          value: root.fullNameDraft
          placeholder: "Your name"
          invalid: root.fullNameDraft.length > 0 && !root.fullNameValid
          onEdited: function(value) { root.fullNameDraft = value }
          onSubmitted: function(value) {
            root.fullNameDraft = value
            if (root.fullNameValid && root.fullNameParsed !== Omarchy.fullName)
              Omarchy.setFullName(root.fullNameParsed)
          }
        }

        PrefsButton {
          id: fullNameSetBtn
          text: "Set"
          primary: true
          enabled: root.fullNameValid && root.fullNameParsed !== Omarchy.fullName
          onClicked: Omarchy.setFullName(root.fullNameParsed)
        }
      }
    }

    SettingRow {
      label: "Password"
      description: "Change the password for this login."
      hint: "chpasswd"
      query: root.query
      keywords: ["password", "passwd", "login", "chpasswd"]

      PrefsButton {
        text: "Change…"
        enabled: !Omarchy.jobBusy && Omarchy.currentUser.length > 0
        onClicked: root.openPassword(Omarchy.currentUser)
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Users"
    query: root.query
    detail: "Human logins (UID 1000 and up) plus this session. Add copies /etc/skel. Remove deletes the home directory. You cannot remove the login you are using."

    SettingRow {
      label: "Add a user"
      description: "Create a local login. Admin adds them to wheel."
      hint: "useradd -m"
      query: root.query
      keywords: ["useradd", "add", "create", "login", "wheel", "admin"]

      PrefsButton {
        text: "Add…"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.openAddUser()
      }
    }

    SettingRow {
      available: Omarchy.accountUsers.length === 0
      sectionHelp: false
      label: "Logins"
      description: "No human logins."
      query: root.query
      keywords: ["user", "empty"]
    }

    Repeater {
      model: Omarchy.accountUsers

      CollectionRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "user"
        description: root.userBlurb(modelData)
        hint: modelData && modelData.home ? modelData.home : "/etc/passwd"
        query: root.query
        keywords: ["user", "login", "wheel", modelData && modelData.name ? modelData.name : ""]
        action: "Password…"
        actionEnabled: !Omarchy.jobBusy && !!(modelData && modelData.name)
        dangerAction: modelData && modelData.current ? "" : "Remove…"
        dangerEnabled: !Omarchy.jobBusy && !!(modelData && modelData.name && !modelData.current)
        onActioned: root.openPassword(modelData.name)
        onDangered: root.askRemoveUser(modelData.name)
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Groups"
    query: root.query
    detail: "wheel and docker always show. Extra groups are ones with a human member or a GID of 1000 or more. Manage opens a group so you can add or remove logins. You cannot drop this session from wheel or remove wheel and docker."

    SettingRow {
      label: "Add a group"
      description: "Create a local group, then add members from Manage…"
      hint: "groupadd"
      query: root.query
      keywords: ["groupadd", "add", "create", "group"]

      PrefsButton {
        text: "Add…"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: root.openAddGroup()
      }
    }

    SettingRow {
      available: Omarchy.accountGroups.length === 0
      sectionHelp: false
      label: "Groups"
      description: "No groups."
      query: root.query
      keywords: ["group", "empty"]
    }

    Repeater {
      model: Omarchy.accountGroups

      CollectionRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "group"
        description: root.groupBlurb(modelData)
        hint: "/etc/group"
        query: root.query
        keywords: ["group", "member", "wheel", "docker", modelData && modelData.name ? modelData.name : ""]
        action: "Manage…"
        actionEnabled: !!(modelData && modelData.name)
        dangerAction: modelData && AccountsJs.isRemovableGid(modelData.gid, modelData.name) ? "Remove…" : ""
        dangerEnabled: !Omarchy.jobBusy && !!(modelData && AccountsJs.isRemovableGid(modelData.gid, modelData.name))
        onActioned: root.openManageGroup(modelData.name)
        onDangered: root.askRemoveGroup(modelData.name)
      }
    }
  }
}
