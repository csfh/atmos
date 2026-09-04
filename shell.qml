import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import Quickshell
import Quickshell.Io
import "services"
import "services/Accounts.js" as AccountsJs
import "services/Layout.js" as LayoutJs
import "services/RichUi.js" as RichUi
import "components"
import "pages"

ShellRoot {
  id: root

  property string launchPath: Quickshell.env("ATMOS_PAGE") || "appearance"
  property string currentPage: "appearance"
  property string query: ""

  readonly property string profileTitle: AccountsJs.profileTitle(Omarchy.fullName, Omarchy.currentUser)
  readonly property string profileHost: AccountsJs.profileHost(Omarchy.currentUser, Omarchy.hostname)

  readonly property var pages: [
    { id: "appearance", title: "Appearance", group: "look", icon: "palette-line", keywords: "theme background wallpaper font text size reset default scale aether palette night light nightlight warmth temperature kelvin schedule hyprsunset plymouth boot screen unlock sddm login refresh reapply templates switcher preview thumbnail picker install git extra clone update pull remove uninstall delete custom folder file image path directory cache logo png render" },
    { id: "display", title: "Displays", group: "look", icon: "computer-line", keywords: "monitor scale hidpi brightness backlight ddc keyboard laptop lid internal mirror clone edp hdmi dp resolution refresh touchpad trackpad pointer mouse input touch touchscreen tablet digitizer" },
    { id: "windows", title: "Windows", group: "look", icon: "layout-grid-line", keywords: "gaps border rounding blur shadow tiling dwindle scrolling niri column opacity transparency fullscreen tight square aspect dim animations cursor tearing looknfeel preserve split focus activate keybinding hotkey shortcut bind unbind chord window rule float tile class" },
    { id: "bar", title: "Bar", group: "look", icon: "layout-top-2-line", keywords: "position transparent menu bar top bottom left right show hide visible clock format time alternate date week start calendar sunday monday birth year age life expectancy indicators always show status icons dictation recording reminder night light dnd stay awake agents usage refresh sync snapshot file hostname device id spacer gap padding tray hidden icons unhide pin unpin plugin widget" },
    { id: "notifications", title: "Notifications", group: "look", icon: "notification-3-line", keywords: "do not disturb dnd silent mute quiet toast reminder timer notify later test time battery weather" },
    { id: "input", title: "Input", group: "input", icon: "keyboard-box-line", keywords: "mouse pointer sensitivity acceleration natural scroll inertia wheel high-res discrete smooth touchpad clickfinger repeat delay numlock follow dpms gesture swipe layout xkb" },
    { id: "accessibility", title: "Accessibility", group: "input", icon: "accessibility-line", keywords: "a11y motion animations reduce text size font cursor pointer hide typing touchscreen herdr screen reader" },
    { id: "sound", title: "Sound", group: "input", icon: "volume-up-line", keywords: "audio volume mute speaker headphone sink source microphone mic input output pipewire pactl wpctl tuning dsp restart voxtype dictation speech" },
    { id: "capture", title: "Capture", group: "input", icon: "screenshot-2-line", keywords: "screenshot record recording screen video ocr qr webcam grim slurp pictures videos" },
    { id: "hardware", title: "Hardware", group: "device", icon: "cpu-line", keywords: "gpu graphics nvidia vulkan hybrid supergfx igpu cuda radeon cpu processor intel amd memory ram dimm ddr chipset motherboard bios uefi firmware tpm npu xdna neural tpu ai accelerator dmi chassis laptop desktop framework usb nic thermal battery" },
    { id: "disks", title: "Disks", group: "device", icon: "hard-drive-2-line", keywords: "drive nvme ssd lsblk luks encryption snapper snapshot btrfs hibernation zram swap speedtest df usage trim fstrim timeline retention" },
    { id: "network", title: "Network", group: "device", icon: "wifi-line", keywords: "dns cloudflare google dhcp custom nameserver resolver bluetooth radio wifi band ghz wlan qr share ssid password psk copy ethernet status ip speedtest join scan pair headset forget restart rfkill connectivity tailscale vpn tailnet localsend taildrop clipboard file folder" },
    { id: "power", title: "Power", group: "device", icon: "battery-2-charge-line", keywords: "power profile performance balanced battery saver percentage charge laptop ac plugged adapter charger unplugged discharging status notify draw" },
    { id: "idle", title: "Idle and lock", group: "device", icon: "lock-2-line", keywords: "screensaver lock timeout idle security stay awake caffeine allow disable suspend sleep branding logo ascii lid clamshell" },
    { id: "defaults", title: "Defaults", group: "apps", icon: "links-line", keywords: "browser terminal editor agent chrome firefox nvim pdf mime image video" },
    { id: "applications", title: "Applications", group: "apps", icon: "apps-2-line", keywords: "desktop app tui webapp web app add install create remove uninstall launcher shortcut delete autostart startup launch" },
    { id: "software", title: "Software", group: "apps", icon: "box-3-line", keywords: "install remove browser terminal editor service gaming steam heroic lutris docker chatgpt 1password dropbox tailscale nordvpn signal spotify sunshine" },
    { id: "hooks", title: "Hooks", group: "apps", icon: "code-s-slash-line", keywords: "hook script theme-set font-set post-boot post-update pacman battery-low" },
    { id: "security", title: "Security", group: "admin", icon: "shield-keyhole-line", keywords: "fingerprint fido2 yubikey ssh sshd sudo passwordless docker pam u2f" },
    { id: "accounts", title: "Accounts", group: "admin", icon: "user-3-line", keywords: "account avatar face icon picture sddm user group password passwd login wheel admin docker useradd userdel groupadd usermod chfn gecos full name real name display name chpasswd" },
    { id: "system", title: "System", group: "admin", icon: "settings-3-line", keywords: "crash capture diagnostics coredump weather location city forecast coordinates latitude longitude gps units celsius fahrenheit metric imperial refresh interval about logo branding fastfetch timezone tz utc region city date time zoneinfo timedatectl hostname computer machine device name hostnamectl keyboard layout keymap xkb qwerty language input localectl ntp timesync synchronize automatic clock network time locale lang utf-8 i18n translation pacman parallel downloads packages mirrors aur update channel firmware orphan prune version printer cups print restore hyprland shell restart atmos git pull reset sentinel overrides" }
  ]

  readonly property var groupedPages: {
    var q = root.query
    var matched = []
    var list = root.pages
    var i
    for (i = 0; i < list.length; i++) {
      if (root.pageMatches(list[i], q)) matched.push(list[i])
    }
    return LayoutJs.clusterByGroup(matched, q.length === 0)
  }

  function pageMatches(page, q) {
    var nq = String(q || "").toLowerCase()
    if (!nq) return true
    if (!page.haystack)
      page.haystack = (page.title + " " + page.keywords).toLowerCase()
    return page.haystack.indexOf(nq) !== -1
  }

  function pageComponent(id) {
    return pageById[id] || appearancePage
  }

  function hubId(id) {
    var raw = String(id || "")
    var slash = raw.indexOf("/")
    return slash === -1 ? raw : raw.substring(0, slash)
  }

  function subId(id) {
    var raw = String(id || "")
    var slash = raw.indexOf("/")
    return slash === -1 ? "" : raw.substring(slash + 1)
  }

  function loadHub(id) {
    currentPage = id
    if (pageStack.depth > 0)
      pageStack.clear(StackView.Immediate)
    pageStack.push(pageComponent(id), {}, StackView.Immediate)
  }

  function openPage(id) {
    if (searchField.text.length > 0)
      searchField.text = ""
    var hub = hubId(id)
    var sub = subId(id)
    for (var i = 0; i < pages.length; i++) {
      if (pages[i].id === hub) {
        loadHub(hub)
        window.visible = true
        window.minimized = false
        if (sub.length > 0) {
          Qt.callLater(function() {
            if (pageStack.currentItem && pageStack.currentItem.openSubpage)
              pageStack.currentItem.openSubpage(sub)
          })
        }
        return "ok"
      }
    }
    return "unknown"
  }

  function showWindow() {
    window.visible = true
    window.minimized = false
    Omarchy.refresh()
    return "ok"
  }

  function submitSudoPassword() {
    Omarchy.confirmSudoMode(sudoPassword.currentText())
  }

  QtObject {
    id: prefsNavigator
    function go(path) {
      searchField.text = ""
      Qt.callLater(function() { root.openPage(path) })
    }
  }

  function onSearchPane() {
    return pageStack.currentItem && pageStack.currentItem.searchPane === true
  }

  function syncSearchPane() {
    if (root.query.length > 0) {
      if (!root.onSearchPane()) {
        if (pageStack.depth > 0)
          pageStack.clear(StackView.Immediate)
        pageStack.push(searchPage, {}, StackView.Immediate)
      }
      return
    }
    if (root.onSearchPane())
      root.loadHub(root.currentPage)
  }

  Component { id: appearancePage; AppearancePage { query: root.query; stack: pageStack } }
  Component { id: displayPage; DisplaysPage { query: root.query } }
  Component { id: hardwarePage; HardwarePage { query: root.query } }
  Component { id: windowsPage; WindowsPage { query: root.query; stack: pageStack } }
  Component { id: inputPage; InputPage { query: root.query } }
  Component { id: accessibilityPage; AccessibilityPage { query: root.query } }
  Component { id: soundPage; SoundPage { query: root.query } }
  Component { id: capturePage; CapturePage { query: root.query } }
  Component { id: disksPage; DisksPage { query: root.query } }
  Component { id: barPage; BarPage { query: root.query } }
  Component { id: notificationsPage; NotificationsPage { query: root.query } }
  Component { id: defaultsPage; DefaultsPage { query: root.query } }
  Component { id: applicationsPage; ApplicationsPage { query: root.query } }
  Component { id: softwarePage; SoftwarePage { query: root.query } }
  Component { id: networkPage; NetworkPage { query: root.query; stack: pageStack } }
  Component { id: powerPage; PowerPage { query: root.query } }
  Component { id: idlePage; IdlePage { query: root.query } }
  Component { id: securityPage; SecurityPage { query: root.query } }
  Component { id: accountsPage; AccountsPage { query: root.query } }
  Component { id: hooksPage; HooksPage { query: root.query } }
  Component { id: systemPage; SystemPage { query: root.query } }
  Component { id: searchPage; SearchPage { query: root.query; navigator: prefsNavigator } }

  readonly property var pageById: ({
    appearance: appearancePage,
    display: displayPage,
    hardware: hardwarePage,
    windows: windowsPage,
    input: inputPage,
    accessibility: accessibilityPage,
    sound: soundPage,
    capture: capturePage,
    disks: disksPage,
    bar: barPage,
    notifications: notificationsPage,
    defaults: defaultsPage,
    applications: applicationsPage,
    software: softwarePage,
    network: networkPage,
    power: powerPage,
    idle: idlePage,
    security: securityPage,
    accounts: accountsPage,
    hooks: hooksPage,
    system: systemPage
  })

  IpcHandler {
    target: "prefs"

    function ping(): string { return "ok" }
    function show(): string { return root.showWindow() }
    function hide(): string {
      window.visible = false
      return "ok"
    }
    function open(page: string): string { return root.openPage(page) }
  }

  FloatingWindow {
    id: window
    title: "Atmos"
    color: Theme.background
    implicitWidth: 960
    implicitHeight: 680
    minimumSize: Qt.size(800, 560)
    visible: true

    onClosed: Qt.quit()

    // FileView on the Theme singleton does not see omarchy-theme-set replacing
    // ~/.local/state/omarchy/current/theme. inotifywait watches that directory
    // the same way the shell plugin registry watches plugins.
    Process {
      id: currentDirWatcher
      running: true
      command: [
        "inotifywait", "-m", "-q",
        "-e", "close_write,create,delete,move,modify,attrib",
        "--format", "%e %f",
        Theme.currentDir
      ]
      stdout: SplitParser {
        onRead: function(line) { currentDirDebounce.restart() }
      }
      onExited: currentDirWatcherRestart.restart()
    }

    Timer {
      id: currentDirWatcherRestart
      interval: 1000
      onTriggered: currentDirWatcher.running = true
    }

    Timer {
      id: currentDirDebounce
      interval: 120
      onTriggered: Omarchy.syncThemeFromDisk()
    }

    Timer {
      interval: 800
      running: true
      repeat: true
      onTriggered: Omarchy.syncThemeFromDiskIfStale()
    }

    Rectangle {
      id: sidebar
      anchors.left: parent.left
      anchors.top: parent.top
      anchors.bottom: parent.bottom
      width: Theme.sidebarWidth
      color: Theme.fill(0.03)

      Item {
        anchors.fill: parent
        anchors.margins: Theme.spaceMd

        FileDialog {
          id: sidebarAvatarDialog
          title: "Choose a face"
          nameFilters: ["Images (*.png *.jpg *.jpeg)"]
          onAccepted: Omarchy.setAvatarPath(RichUi.pathFromUrl(selectedFile))
        }

        Column {
          id: sidebarTitle
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: parent.top
          spacing: Theme.space

          Rectangle {
            id: avatarWell
            width: 88
            height: 88
            radius: width / 2
            anchors.horizontalCenter: parent.horizontalCenter
            clip: true
            layer.enabled: true
            layer.smooth: true
            color: Theme.fill(Theme.normalFill)
            border.width: avatarMouse.containsMouse ? 1 : 0
            border.color: Theme.accent

            Image {
              anchors.fill: parent
              visible: Omarchy.avatarPath.length > 0
              fillMode: Image.PreserveAspectCrop
              asynchronous: true
              source: Omarchy.avatarPath.length ? ("file://" + encodeURI(Omarchy.avatarPath)) : ""
            }

            PrefsIcon {
              visible: Omarchy.avatarPath.length === 0
              anchors.centerIn: parent
              name: "user-3-line"
              size: 36
              color: avatarMouse.containsMouse ? Theme.foreground : Theme.muted
            }

            MouseArea {
              id: avatarMouse
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor
              enabled: Omarchy.currentUser.length > 0 && !Omarchy.jobBusy
              onClicked: sidebarAvatarDialog.open()
            }

            Accessible.role: Accessible.Button
            Accessible.name: Omarchy.avatarPath.length ? "Change face" : "Set a face"
            Accessible.onPressAction: if (avatarMouse.enabled) sidebarAvatarDialog.open()
          }

          Item {
            width: parent.width
            height: profileCopy.implicitHeight

            Column {
              id: profileCopy
              width: parent.width
              spacing: 0

              Text {
                id: profileName
                width: parent.width
                text: root.profileTitle
                color: Theme.foreground
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
              }

              Text {
                id: profileSub
                width: parent.width
                visible: root.profileHost.length > 0
                text: root.profileHost
                color: Theme.muted
                font.family: Theme.fontFamily
                font.pixelSize: Theme.captionSize
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
              }
            }

            MouseArea {
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor
              onClicked: root.openPage("accounts")
            }

            Accessible.role: Accessible.Button
            Accessible.name: "Accounts"
            Accessible.onPressAction: root.openPage("accounts")
          }
        }

        Rectangle {
          id: searchBox
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: sidebarTitle.bottom
          anchors.topMargin: Theme.spaceMd
          height: Theme.controlHeight
          radius: Theme.radius
          color: searchField.activeFocus || searchHover.hovered ? Theme.fill(Theme.hoverFill) : Theme.fill(Theme.normalFill)
          border.width: 1
          border.color: searchField.activeFocus || searchHover.hovered ? Theme.accent : Theme.borderColor()

          Behavior on color {
            ColorAnimation { duration: 90 }
          }
          Behavior on border.color {
            ColorAnimation { duration: 90 }
          }

          HoverHandler {
            id: searchHover
          }

          TextInput {
            id: searchField
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 10
            color: Theme.foreground
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            clip: true
            selectByMouse: true
            verticalAlignment: TextInput.AlignVCenter
            activeFocusOnTab: true
            onTextChanged: {
              root.query = text
              root.syncSearchPane()
            }
            Keys.onEscapePressed: {
              if (text.length > 0) text = ""
            }

            Text {
              anchors.fill: parent
              visible: searchField.text.length === 0 && !searchField.activeFocus
              text: "Find a setting"
              color: Theme.muted
              font.family: Theme.fontFamily
              font.pixelSize: Theme.fontSize
              verticalAlignment: Text.AlignVCenter
            }
          }
        }

        Flickable {
          id: navFlick
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: searchBox.bottom
          anchors.topMargin: Theme.space
          anchors.bottom: parent.bottom
          clip: true
          contentWidth: width
          contentHeight: navColumn.implicitHeight
          boundsBehavior: Flickable.StopAtBounds
          flickableDirection: Flickable.VerticalFlick

          Column {
            id: navColumn
            width: navFlick.width
            spacing: 0

            Repeater {
              model: root.groupedPages
              delegate: Column {
                id: navGroup
                required property var modelData
                required property int index
                width: navColumn.width
                spacing: 2
                topPadding: index > 0 ? Theme.spaceMd : 0

                Item {
                  width: navColumn.width
                  visible: !!(navGroup.modelData && navGroup.modelData.title)
                  height: groupLabel.implicitHeight + 4

                  Text {
                    id: groupLabel
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    anchors.leftMargin: Theme.pad
                    anchors.rightMargin: Theme.pad
                    text: navGroup.modelData && navGroup.modelData.title ? navGroup.modelData.title : ""
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.captionSize
                    font.bold: true
                    elide: Text.ElideRight
                  }
                }

                Repeater {
                  model: navGroup.modelData && navGroup.modelData.pages ? navGroup.modelData.pages : []
                  delegate: Rectangle {
                    required property var modelData
                    width: navColumn.width
                    height: Theme.rowHeight
                    radius: Theme.radius
                    color: root.query.length === 0 && root.currentPage === modelData.id
                      ? Theme.fill(Theme.selectedFill)
                      : (navMouse.containsMouse ? Theme.fill(Theme.hoverFill) : "transparent")

                    PrefsIcon {
                      id: navIcon
                      anchors.left: parent.left
                      anchors.leftMargin: Theme.pad
                      anchors.verticalCenter: parent.verticalCenter
                      name: modelData && modelData.icon ? modelData.icon : ""
                      size: Theme.fontSize + 2
                      color: root.query.length === 0 && root.currentPage === modelData.id
                        ? Theme.foreground
                        : (navMouse.containsMouse ? Theme.foreground : Theme.muted)
                    }

                    Text {
                      anchors.left: navIcon.right
                      anchors.right: parent.right
                      anchors.verticalCenter: parent.verticalCenter
                      anchors.leftMargin: 8
                      anchors.rightMargin: Theme.pad
                      text: modelData.title
                      color: Theme.foreground
                      font.family: Theme.fontFamily
                      font.pixelSize: Theme.fontSize
                      font.bold: root.query.length === 0 && root.currentPage === modelData.id
                      elide: Text.ElideRight
                    }

                    MouseArea {
                      id: navMouse
                      anchors.fill: parent
                      hoverEnabled: true
                      cursorShape: Qt.PointingHandCursor
                      onClicked: {
                        root.currentPage = modelData.id
                        if (searchField.text.length > 0) searchField.text = ""
                        else root.loadHub(modelData.id)
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }

    Rectangle {
      id: divider
      anchors.left: sidebar.right
      anchors.top: parent.top
      anchors.bottom: parent.bottom
      width: 1
      color: Theme.borderColor()
    }

    Item {
      id: rightPane
      anchors.left: divider.right
      anchors.right: parent.right
      anchors.top: parent.top
      anchors.bottom: parent.bottom

      Item {
        id: header
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: pageStack.depth > 1 ? 48 : 0
        clip: true

        readonly property bool canGoBack: pageStack.depth > 1
        readonly property int backSlotWidth: Math.max(22, Theme.titleSize)

        Behavior on height {
          NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
        }

        Item {
          id: backSlot
          anchors.left: parent.left
          anchors.leftMargin: Theme.pad * 1.5
          anchors.verticalCenter: parent.verticalCenter
          width: header.backSlotWidth
          height: header.backSlotWidth

          Accessible.role: Accessible.Button
          Accessible.name: "Back"
          Accessible.ignored: !header.canGoBack
          Accessible.onPressAction: pageStack.pop()

          PrefsIcon {
            id: backIcon
            anchors.centerIn: parent
            anchors.horizontalCenterOffset: header.canGoBack ? 0 : -6
            name: Theme.iconChevronLeft
            size: Theme.titleSize
            color: backMouse.containsMouse && header.canGoBack ? Theme.foreground : Theme.accent
            opacity: header.canGoBack ? 1 : 0
            scale: backMouse.containsMouse && header.canGoBack ? 1.08 : 1

            Behavior on opacity {
              NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
            }
            Behavior on scale {
              NumberAnimation { duration: 120; easing.type: Easing.OutCubic }
            }
            Behavior on anchors.horizontalCenterOffset {
              NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
            }
            Behavior on color {
              ColorAnimation { duration: 120 }
            }
          }

          MouseArea {
            id: backMouse
            anchors.fill: parent
            anchors.margins: -6
            enabled: header.canGoBack
            hoverEnabled: true
            cursorShape: header.canGoBack ? Qt.PointingHandCursor : Qt.ArrowCursor
            onClicked: pageStack.pop()
          }
        }

      }

      Rectangle {
        id: errorBanner
        anchors.top: header.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.pad * 1.5
        anchors.rightMargin: Theme.pad * 1.5
        visible: Omarchy.lastError.length > 0
        height: visible ? errorText.implicitHeight + Theme.pad : 0
        color: Theme.urgent
        opacity: 0.18
        radius: Theme.radius

        Text {
          id: errorText
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: parent.top
          anchors.leftMargin: 8
          anchors.rightMargin: 8
          anchors.topMargin: 8
          anchors.bottomMargin: 8
          text: Omarchy.lastError
          color: Theme.urgent
          wrapMode: Text.WordWrap
          font.family: Theme.fontFamily
          font.pixelSize: Theme.captionSize
        }
      }

      StackView {
        id: pageStack
        anchors.top: errorBanner.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        clip: true

        pushEnter: Transition {
          NumberAnimation { property: "x"; from: 36; to: 0; duration: 220; easing.type: Easing.OutCubic }
          NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 180 }
        }
        pushExit: Transition {
          NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 160 }
        }
        popEnter: Transition {
          NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 180 }
        }
        popExit: Transition {
          NumberAnimation { property: "x"; from: 0; to: 36; duration: 200; easing.type: Easing.InCubic }
          NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 160 }
        }

        Component.onCompleted: {
          var hub = root.hubId(root.launchPath)
          if (!hub) hub = "appearance"
          root.currentPage = hub
          pageStack.push(root.pageComponent(hub), {}, StackView.Immediate)
          var sub = root.subId(root.launchPath)
          if (sub.length > 0) {
            Qt.callLater(function() {
              if (pageStack.currentItem && pageStack.currentItem.openSubpage)
                pageStack.currentItem.openSubpage(sub)
            })
          }
        }
      }
    }

    PrefsDialog {
      id: sudoModeDialog
      title: "Administrator password"
      closePolicy: Popup.CloseOnEscape

      PrefsText {
        width: parent.width
        text: Omarchy.sudoEnabling
          ? "Unlocking sudo…"
          : "This change needs your password. Sudo then stays unlocked for " + Omarchy.sudoMinutes + " minutes. Agents and anything else running as you can use it until then."
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
      }

      PrefsPassword {
        id: sudoPassword
        width: parent.width
        placeholder: "Password"
        enabled: !Omarchy.sudoEnabling
        onSubmitted: function() { root.submitSudoPassword() }
      }

      PrefsText {
        width: parent.width
        visible: Omarchy.sudoError.length > 0
        text: Omarchy.sudoError
        color: Theme.urgent
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
      }

      Row {
        anchors.right: parent.right
        spacing: Theme.space

        PrefsButton {
          text: "Cancel"
          enabled: !Omarchy.sudoEnabling
          onClicked: {
            sudoModeDialog.close()
            Omarchy.cancelSudoMode()
          }
        }

        PrefsButton {
          text: "Continue"
          primary: true
          enabled: !Omarchy.sudoEnabling
          onClicked: root.submitSudoPassword()
        }
      }

      onClosed: {
        sudoPassword.clear()
        if (Omarchy.sudoPromptOpen && !Omarchy.sudoEnabling)
          Omarchy.cancelSudoMode()
      }
    }

    Connections {
      target: Omarchy
      function onSudoPromptOpenChanged() {
        if (Omarchy.sudoPromptOpen) {
          if (!sudoModeDialog.visible) sudoPassword.clear()
          sudoModeDialog.open()
          Qt.callLater(function() { sudoPassword.focusInput() })
        } else if (sudoModeDialog.visible) {
          sudoModeDialog.close()
        }
      }
      function onSudoErrorChanged() {
        if (Omarchy.sudoError.length > 0) {
          sudoPassword.clear()
          Qt.callLater(function() { sudoPassword.focusInput() })
        }
      }
    }

    Shortcut {
      sequences: ["Ctrl+F", "/"]
      onActivated: searchField.forceActiveFocus()
    }

    Shortcut {
      sequences: ["Escape"]
      onActivated: {
        if (pageStack.depth > 1) pageStack.pop()
        else if (searchField.text.length > 0) searchField.text = ""
      }
    }
  }
}
