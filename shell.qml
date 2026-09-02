import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import "services"
import "components"
import "pages"

ShellRoot {
  id: root

  property string launchPath: Quickshell.env("ATMOS_PAGE") || "appearance"
  property string currentPage: "appearance"
  property string query: ""

  readonly property var pages: [
    { id: "appearance", title: "Appearance", keywords: "theme background wallpaper font text size reset default scale aether palette night light nightlight warmth temperature kelvin schedule hyprsunset plymouth boot screen unlock sddm login refresh reapply templates switcher preview thumbnail picker install git extra clone update pull remove uninstall delete custom folder file image path directory cache logo png render" },
    { id: "display", title: "Displays", keywords: "monitor scale hidpi brightness backlight ddc keyboard laptop lid internal mirror clone edp hdmi dp resolution refresh touchpad trackpad pointer mouse input touch touchscreen tablet digitizer gpu hybrid nvidia supergfx" },
    { id: "windows", title: "Windows", keywords: "gaps border rounding blur shadow tiling dwindle scrolling niri column opacity transparency fullscreen tight square aspect dim animations cursor tearing looknfeel preserve split focus activate keybinding hotkey shortcut bind unbind chord window rule float tile class" },
    { id: "input", title: "Input", keywords: "mouse pointer sensitivity acceleration natural scroll touchpad clickfinger repeat delay numlock follow dpms gesture swipe layout xkb" },
    { id: "accessibility", title: "Accessibility", keywords: "a11y motion animations reduce text size font cursor pointer hide typing touchscreen herdr screen reader" },
    { id: "sound", title: "Sound", keywords: "audio volume mute speaker headphone sink source microphone mic input output pipewire pactl wpctl tuning dsp restart voxtype dictation speech" },
    { id: "capture", title: "Capture", keywords: "screenshot record recording screen video ocr qr webcam grim slurp pictures videos" },
    { id: "disks", title: "Disks", keywords: "drive nvme ssd lsblk luks encryption snapper snapshot btrfs hibernation zram swap speedtest df usage trim fstrim timeline retention" },
    { id: "bar", title: "Bar", keywords: "position transparent menu bar top bottom left right show hide visible clock format time alternate date week start calendar sunday monday birth year age life expectancy indicators always show status icons dictation recording reminder night light dnd stay awake agents usage refresh sync snapshot file hostname device id spacer gap padding tray hidden icons unhide pin unpin plugin widget" },
    { id: "notifications", title: "Notifications", keywords: "do not disturb dnd silent mute quiet toast reminder timer notify later test time battery weather" },
    { id: "defaults", title: "Defaults", keywords: "browser terminal editor agent chrome firefox nvim pdf mime image video" },
    { id: "applications", title: "Applications", keywords: "desktop app tui webapp web app add install create remove uninstall launcher shortcut delete autostart startup launch" },
    { id: "software", title: "Software", keywords: "install remove browser terminal editor service gaming steam heroic lutris docker chatgpt 1password dropbox tailscale nordvpn signal spotify sunshine" },
    { id: "network", title: "Network", keywords: "dns cloudflare google dhcp custom nameserver resolver bluetooth radio wifi band ghz wlan qr share ssid password psk copy ethernet status ip speedtest join scan pair headset forget restart rfkill connectivity tailscale vpn tailnet localsend taildrop clipboard file folder" },
    { id: "power", title: "Power", keywords: "power profile performance balanced battery saver percentage charge laptop ac plugged adapter charger unplugged discharging status notify draw" },
    { id: "idle", title: "Idle and lock", keywords: "screensaver lock timeout idle security stay awake caffeine allow disable suspend sleep branding logo ascii lid clamshell" },
    { id: "security", title: "Security", keywords: "fingerprint fido2 yubikey ssh sshd sudo passwordless docker pam u2f" },
    { id: "hooks", title: "Hooks", keywords: "hook script theme-set font-set post-boot post-update pacman battery-low" },
    { id: "system", title: "System", keywords: "crash capture diagnostics coredump weather location city forecast coordinates latitude longitude gps units celsius fahrenheit metric imperial refresh interval about logo branding fastfetch timezone tz utc region city date time zoneinfo timedatectl hostname computer machine device name hostnamectl keyboard layout keymap xkb qwerty language input localectl ntp timesync synchronize automatic clock network time locale lang utf-8 i18n translation pacman parallel downloads packages mirrors aur full name user gecos account chfn update channel firmware orphan prune version printer cups print restore hyprland shell restart" }
  ]

  function pageMatches(page, q) {
    if (!q || q.length === 0) return true
    var haystack = (page.title + " " + page.keywords).toLowerCase()
    return haystack.indexOf(q.toLowerCase()) !== -1
  }

  function pageComponent(id) {
    if (id === "display") return displayPage
    if (id === "windows") return windowsPage
    if (id === "input") return inputPage
    if (id === "accessibility") return accessibilityPage
    if (id === "sound") return soundPage
    if (id === "capture") return capturePage
    if (id === "disks") return disksPage
    if (id === "bar") return barPage
    if (id === "notifications") return notificationsPage
    if (id === "defaults") return defaultsPage
    if (id === "applications") return applicationsPage
    if (id === "software") return softwarePage
    if (id === "network") return networkPage
    if (id === "power") return powerPage
    if (id === "idle") return idlePage
    if (id === "security") return securityPage
    if (id === "hooks") return hooksPage
    if (id === "system") return systemPage
    return appearancePage
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
  Component { id: hooksPage; HooksPage { query: root.query } }
  Component { id: systemPage; SystemPage { query: root.query } }
  Component { id: searchPage; SearchPage { query: root.query; navigator: prefsNavigator } }

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

        Text {
          id: sidebarTitle
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: parent.top
          text: "Atmos"
          color: Theme.foreground
          font.family: Theme.fontFamily
          font.pixelSize: Theme.titleSize
          font.bold: true
        }

        Rectangle {
          id: searchBox
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: sidebarTitle.bottom
          anchors.topMargin: Theme.space
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
            spacing: 2

            Repeater {
              model: root.pages
              delegate: Rectangle {
                required property var modelData
                width: navColumn.width
                height: Theme.rowHeight
                visible: root.query.length > 0 || root.pageMatches(modelData, root.query)
                radius: Theme.radius
                color: root.query.length === 0 && root.currentPage === modelData.id
                  ? Theme.fill(Theme.selectedFill)
                  : (navMouse.containsMouse ? Theme.fill(Theme.hoverFill) : "transparent")

                Text {
                  anchors.left: parent.left
                  anchors.right: parent.right
                  anchors.verticalCenter: parent.verticalCenter
                  anchors.leftMargin: Theme.pad
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

          Text {
            id: backIcon
            anchors.centerIn: parent
            anchors.horizontalCenterOffset: header.canGoBack ? 0 : -6
            text: Theme.iconChevronLeft
            color: backMouse.containsMouse && header.canGoBack ? Theme.foreground : Theme.accent
            font.family: Theme.fontFamily
            font.pixelSize: Theme.titleSize
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

      Text {
        z: 15
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.topMargin: Theme.space
        anchors.rightMargin: Theme.spaceLg
        visible: Omarchy.busy || Omarchy.jobBusy
        text: Omarchy.jobBusy ? "Working…" : "Updating…"
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.captionSize
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
          anchors.fill: parent
          anchors.margins: 8
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
