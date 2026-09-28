import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import "services"
import "services/Accounts.js" as AccountsJs
import "services/Hubs.js" as HubsJs
import "services/Layout.js" as LayoutJs
import "services/NavStatus.js" as NavStatusJs
import "services/RichUi.js" as RichUi
import "services/Systemd.js" as SystemdJs
import "components"
import "pages"
import "pages/windows" as Win
import "pages/network" as Net

ShellRoot {
  id: root

  property string launchPath: Quickshell.env("ATMOS_PAGE") || "home"
  property string currentPage: "home"
  property string query: ""
  onQueryChanged: if (query.length > 0) placeNavHighlight(null)

  readonly property string profileTitle: AccountsJs.profileTitle(Omarchy.fullName, Omarchy.currentUser)
  readonly property string profileHost: AccountsJs.profileHost(Omarchy.currentUser, Omarchy.hostname)

  readonly property var pages: HubsJs.navPages()

  // Fields the sidebar badges read, boxed so the binding list lives in
  // one place. A change still re-runs forHub on every row; that is cheap
  // at this size.
  readonly property var navState: ({
    ready: Omarchy.snapshotReady,
    systemdUnits: Omarchy.systemdUnits,
    bluetoothDevices: Omarchy.bluetoothDevices,
    monitors: Omarchy.monitors,
    netKind: Omarchy.netKind,
    netSsid: Omarchy.netSsid,
    updateAvailable: Omarchy.updateAvailable,
    atmosUpdateAvailable: Omarchy.atmosUpdateAvailable,
    isFailed: SystemdJs.isFailed
  })

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

  // The nav in the order it is drawn. Follows the live search filter.
  readonly property var flatNavPages: LayoutJs.flattenNavPages(root.groupedPages)

  // True while a text field owns the keyboard, so a plain letter types
  // instead of starting a search. Window shortcuts still fire when a
  // PrefsField, PrefsPassword, PrefsSelect filter, or the sudo box has
  // focus, so this watches the real focus item.
  readonly property bool typing: {
    var item = window.activeFocusItem
    return !!(item && (item instanceof TextInput || item instanceof TextEdit))
  }

  // Up, Down, Home, End, and Enter move the sidebar or the search hits.
  // They stay live while the search field itself is focused. A slider,
  // button, or other text field keeps the key.
  readonly property bool listKeys: {
    if (modalOpen) return false
    if (searchField.activeFocus) return true
    if (typing) return false
    var item = window.activeFocusItem
    return !item || item === window.contentItem
  }

  // Page-level PrefsDialogs (add binding, LUKS, …) never appear as ids here.
  // Walk the focus parent chain so a j cannot change hub under a modal or
  // while a key-grab is listening.
  function focusIsUnderPopup(item) {
    var p = item
    while (p) {
      if (p instanceof Popup) return true
      p = p.parent
    }
    return false
  }

  readonly property bool modalOpen: errorDialog.visible
    || sudoModeDialog.visible
    || keysDialog.visible
    || focusIsUnderPopup(window.activeFocusItem)

  readonly property bool navBusy: typing || modalOpen

  function searchPageItem() {
    var page = pageStack.currentItem
    return page && page.searchPane === true ? page : null
  }

  function moveList(delta) {
    var page = root.searchPageItem()
    if (root.query.length > 0 && page && page.moveHit) {
      page.moveHit(delta)
      return
    }
    root.moveNav(delta)
  }

  function jumpList(toEnd) {
    var page = root.searchPageItem()
    if (root.query.length > 0 && page && page.jumpHit) {
      page.jumpHit(toEnd)
      return
    }
    root.jumpNav(toEnd)
  }

  function activateList() {
    var page = root.searchPageItem()
    if (root.query.length > 0 && page && page.activateHit) {
      page.activateHit()
      return
    }
    if (root.query.length === 0)
      root.loadHub(root.currentPage)
  }

  // A printable key focuses search and inserts itself. The field then
  // owns the rest of the word. Space does not start an empty query.
  function startSearch(event) {
    if (!event) return
    event.accepted = false
    if (event.isAutoRepeat) return
    if (root.navBusy || searchField.activeFocus) return
    var mods = event.modifiers
    if (mods & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier)) return
    var text = event.text || ""
    if (text.length !== 1) return
    var code = text.charCodeAt(0)
    if (code <= 31 || code === 127) return
    if (text === " " && searchField.text.length === 0) return
    searchField.forceActiveFocus()
    searchField.insert(searchField.length, text)
    event.accepted = true
  }

  function moveNav(delta) {
    var list = root.flatNavPages
    var next = LayoutJs.stepNavIndex(list, root.currentPage, delta)
    if (next < 0) return
    var id = list[next]
    if (id === root.currentPage) return
    root.currentPage = id
    if (searchField.text.length > 0) searchField.text = ""
    else root.loadHub(id)
  }

  function jumpNav(toEnd) {
    var list = root.flatNavPages
    var next = LayoutJs.jumpNavIndex(list, toEnd)
    if (next < 0) return
    var id = list[next]
    if (id === root.currentPage) return
    root.currentPage = id
    if (searchField.text.length > 0) searchField.text = ""
    else root.loadHub(id)
  }

  function pageMatches(page, q) {
    var nq = String(q || "").toLowerCase()
    if (!nq) return true
    if (!page.haystack)
      page.haystack = (page.title + " " + page.keywords).toLowerCase()
    return page.haystack.indexOf(nq) !== -1
  }

  function pageComponent(id) {
    return pageById[root.hubId(root.canonicalHub(id))] || homePage
  }

  // The chart grid used to be its own hub. Old launchers still say dashboard.
  function canonicalHub(id) {
    var raw = String(id || "")
    if (raw === "dashboard" || raw.indexOf("dashboard/") === 0) return "monitor"
    if (raw === "machine" || raw === "system/machine") return "hardware"
    return raw
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
    id = root.canonicalHub(id)
    Disclosure.leaveHub(hubId(id))
    currentPage = id
    if (pageStack.depth > 0)
      pageStack.clear(StackView.Immediate)
    pageStack.push(pageComponent(id), {}, StackView.Immediate)
    Qt.callLater(function() { root.revealCurrentNav() })
  }

  function placeNavHighlight(item) {
    if (!item || root.query.length > 0) {
      navHighlight.visible = false
      return
    }
    var pos = item.mapToItem(navContent, 0, 0)
    if (pos.y !== pos.y) return
    navHighlight.height = item.height
    var slide = highlightSlide.enabled && navHighlight.visible
    if (!slide) highlightSlide.enabled = false
    navHighlight.y = pos.y
    navHighlight.visible = true
    if (!slide) highlightSlide.enabled = true
    root.revealNavItem(item)
  }

  // Home, End, and a held Down select rows below the fold. Keep the
  // current hub in the nav viewport.
  function revealNavItem(item) {
    if (!item || !navFlick) return
    var pos = item.mapToItem(navContent, 0, 0)
    if (pos.y !== pos.y) return
    var top = pos.y
    var bottom = top + item.height
    var viewH = navFlick.height
    var maxY = Math.max(0, navFlick.contentHeight - viewH)
    var y = navFlick.contentY
    if (top < y)
      navFlick.contentY = Math.max(0, Math.min(maxY, top))
    else if (bottom > y + viewH)
      navFlick.contentY = Math.max(0, Math.min(maxY, bottom - viewH))
  }

  // The delegate for a hub id, or null while the nav is still building.
  // Group delegates are plain Columns, so walk one level down.
  function navItemFor(id) {
    if (!navColumn) return null
    var gi
    for (gi = 0; gi < navColumn.children.length; gi++) {
      var group = navColumn.children[gi]
      if (!group || !group.children) continue
      var pi
      for (pi = 0; pi < group.children.length; pi++) {
        var item = group.children[pi]
        if (item && item.modelData && item.modelData.id === id) return item
      }
    }
    return null
  }

  // Re-assert the rail on the current hub. Placement reads live geometry,
  // so a call that lands mid-layout (fresh launch, late badges) is a no-op
  // until positions settle; the settle hooks below call this again.
  function revealCurrentNav() {
    if (root.query.length > 0) {
      navHighlight.visible = false
      return
    }
    var item = root.navItemFor(root.currentPage)
    if (item) root.placeNavHighlight(item)
  }

  function openPage(id) {
    if (searchField.text.length > 0)
      searchField.text = ""
    id = root.canonicalHub(id)
    var hub = hubId(id)
    var sub = subId(id)
    for (var i = 0; i < pages.length; i++) {
      if (pages[i].id === hub) {
        Disclosure.finishReveal(hub)
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
    Disclosure.finishReveal("")
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
    function go(path, label) {
      if (label)
        Disclosure.revealFromSearch(path, label)
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

  Component { id: homePage; HomePage { query: root.query; navigator: prefsNavigator } }
  Component { id: monitorPage; MonitorPage { query: root.query; stack: pageStack; navigator: prefsNavigator } }
  Component { id: favoritesPage; FavoritesPage { query: root.query; navigator: prefsNavigator } }
  Component { id: appearancePage; AppearancePage { query: root.query; stack: pageStack; navigator: prefsNavigator } }
  Component { id: displayPage; DisplaysPage { query: root.query } }
  Component { id: hardwarePage; HardwarePage { query: root.query; navigator: prefsNavigator } }
  Component { id: driversPage; DriversPage { query: root.query } }
  Component { id: windowsPage; WindowsPage { query: root.query; stack: pageStack; navigator: prefsNavigator } }
  Component { id: workspacesPage; WorkspacesPage { query: root.query } }
  Component { id: inputPage; InputPage { query: root.query } }
  Component { id: keybindingsPage; Win.BindingsPage { query: root.query } }
  Component { id: profilesPage; ProfilesPage { query: root.query; navigator: prefsNavigator } }
  Component { id: accessibilityPage; AccessibilityPage { query: root.query } }
  Component { id: soundPage; SoundPage { query: root.query } }
  Component { id: capturePage; CapturePage { query: root.query } }
  Component { id: disksPage; DisksPage { query: root.query } }
  Component { id: barPage; BarPage { query: root.query } }
  Component { id: notificationsPage; NotificationsPage { query: root.query } }
  Component { id: defaultsPage; DefaultsPage { query: root.query } }
  Component { id: applicationsPage; ApplicationsPage { query: root.query; stack: pageStack; navigator: prefsNavigator } }
  Component { id: softwarePage; SoftwarePage { query: root.query } }
  Component { id: networkPage; NetworkPage { query: root.query; stack: pageStack; navigator: prefsNavigator } }
  Component { id: bluetoothPage; Net.BluetoothPage { query: root.query } }
  Component { id: powerPage; PowerPage { query: root.query } }
  Component { id: idlePage; IdlePage { query: root.query } }
  Component { id: tweaksPage; TweaksPage { query: root.query } }
  Component { id: servicesPage; ServicesPage { query: root.query } }
  Component { id: securityPage; SecurityPage { query: root.query } }
  Component { id: accountsPage; AccountsPage { query: root.query } }
  Component { id: hooksPage; HooksPage { query: root.query } }
  Component { id: systemPage; SystemPage { query: root.query; stack: pageStack; navigator: prefsNavigator } }
  Component { id: exportPage; ExportPage { query: root.query } }
  Component { id: searchPage; SearchPage { query: root.query; navigator: prefsNavigator } }

  readonly property var pageById: ({
    home: homePage,
    monitor: monitorPage,
    favorites: favoritesPage,
    appearance: appearancePage,
    display: displayPage,
    hardware: hardwarePage,
    drivers: driversPage,
    windows: windowsPage,
    workspaces: workspacesPage,
    input: inputPage,
    keybindings: keybindingsPage,
    accessibility: accessibilityPage,
    sound: soundPage,
    capture: capturePage,
    disks: disksPage,
    bar: barPage,
    notifications: notificationsPage,
    profiles: profilesPage,
    defaults: defaultsPage,
    applications: applicationsPage,
    software: softwarePage,
    network: networkPage,
    bluetooth: bluetoothPage,
    power: powerPage,
    idle: idlePage,
    tweaks: tweaksPage,
    services: servicesPage,
    security: securityPage,
    accounts: accountsPage,
    hooks: hooksPage,
    system: systemPage,
    export: exportPage,
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
    onVisibleChanged: {
      if (visible) Qt.callLater(function() { root.revealCurrentNav() })
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

          Item {
            id: avatarWell
            width: 88
            height: 88
            anchors.horizontalCenter: parent.horizontalCenter

            Item {
              id: avatarMask
              anchors.fill: parent
              visible: false
              layer.enabled: true
              layer.smooth: true

              Rectangle {
                anchors.fill: parent
                radius: width / 2
                color: "white"
              }
            }

            Rectangle {
              anchors.fill: parent
              radius: width / 2
              color: Theme.fill(Theme.normalFill)
              visible: Omarchy.avatarPath.length === 0
            }

            Item {
              anchors.fill: parent
              visible: Omarchy.avatarPath.length > 0
              layer.enabled: true
              layer.smooth: true
              layer.effect: MultiEffect {
                maskEnabled: true
                maskSource: avatarMask
                maskThresholdMin: 0.3
                maskSpreadAtMin: 0.3
              }

              Image {
                anchors.fill: parent
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                source: Omarchy.avatarPath.length ? ("file://" + encodeURI(Omarchy.avatarPath)) : ""
              }
            }

            PrefsIcon {
              visible: Omarchy.avatarPath.length === 0
              anchors.centerIn: parent
              name: "user-3-line"
              size: 36
              color: avatarMouse.containsMouse ? Theme.foreground : Theme.muted
            }

            Rectangle {
              anchors.fill: parent
              radius: width / 2
              color: "transparent"
              border.width: avatarMouse.containsMouse ? Theme.borderWidth : 0
              border.color: Theme.accent
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
            height: profileNameSlot.height + profileHostSlot.height

            Column {
              id: profileCopy
              width: parent.width
              spacing: 0

              Item {
                id: profileNameSlot
                width: parent.width
                height: Theme.fontSize + 6

                Text {
                  id: profileName
                  width: parent.width
                  height: parent.height
                  text: root.profileTitle
                  color: Theme.foreground
                  font.family: Theme.fontFamily
                  font.pixelSize: Theme.fontSize
                  font.bold: true
                  horizontalAlignment: Text.AlignHCenter
                  verticalAlignment: Text.AlignVCenter
                  elide: Text.ElideRight
                }
              }

              Item {
                id: profileHostSlot
                width: parent.width
                height: Theme.captionSize + 4

                Text {
                  id: profileSub
                  width: parent.width
                  height: parent.height
                  text: root.profileHost
                  color: Theme.muted
                  font.family: Theme.fontFamily
                  font.pixelSize: Theme.captionSize
                  horizontalAlignment: Text.AlignHCenter
                  verticalAlignment: Text.AlignTop
                  elide: Text.ElideRight
                  opacity: root.profileHost.length > 0 ? 1 : 0
                }
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
          border.width: Theme.borderWidth
          border.color: searchField.activeFocus || searchHover.hovered ? Theme.accent : Theme.borderColor()

          Behavior on color {
            ColorAnimation { duration: Theme.motionFast }
          }
          Behavior on border.color {
            ColorAnimation { duration: Theme.motionFast }
          }

          HoverHandler {
            id: searchHover
          }

          TextInput {
            id: searchField
            anchors.fill: parent
            anchors.leftMargin: Theme.fieldInset
            anchors.rightMargin: Theme.fieldInset
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
            Keys.onEscapePressed: function(event) {
              // Blur first so the filter stays. The window Escape shortcut
              // clears it on the next press.
              searchField.focus = false
              event.accepted = true
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

        PrefsFlickable {
          id: navFlick
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: searchBox.bottom
          anchors.topMargin: Theme.space
          anchors.bottom: parent.bottom
          clip: true
          contentHeight: navColumn.implicitHeight

          Item {
            id: navContent
            width: navFlick.width
            height: navColumn.implicitHeight

            Rectangle {
              id: navHighlight
              width: Theme.railWidth
              height: Theme.rowHeight
              x: 0
              y: 0
              z: 2
              visible: false
              color: Theme.accent

              Behavior on y {
                id: highlightSlide
                enabled: false
                NumberAnimation { duration: Theme.motionNav; easing.type: Easing.OutCubic }
              }
            }

          Column {
            id: navColumn
            width: parent.width
            z: 1
            spacing: 0
            onImplicitHeightChanged: root.revealCurrentNav()

            Repeater {
              model: root.groupedPages
              delegate: Column {
                id: navGroup
                required property var modelData
                required property int index
                width: navColumn.width
                spacing: Theme.sidebarItemSpacing
                topPadding: index > 0 ? Theme.sidebarGroupSpacing : 0

                Item {
                  width: navColumn.width
                  visible: !!(navGroup.modelData && navGroup.modelData.title)
                  height: groupLabel.implicitHeight + Theme.titleGap

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
                    font.pixelSize: Theme.sectionSize
                    font.bold: true
                    elide: Text.ElideRight
                  }
                }

                Repeater {
                  model: navGroup.modelData && navGroup.modelData.pages ? navGroup.modelData.pages : []
                  delegate: Rectangle {
                    id: navItem
                    required property var modelData
                    width: navColumn.width
                    height: Theme.rowHeight
                    radius: Theme.radius
                    readonly property bool selected: root.query.length === 0 && root.currentPage === modelData.id
                    readonly property bool hovered: navMouse.containsMouse
                    activeFocusOnTab: true
                    color: (navItem.hovered || navItem.activeFocus) ? Theme.fill(Theme.hoverFill) : "transparent"

                    Accessible.role: Accessible.Button
                    Accessible.name: modelData && modelData.title ? modelData.title : ""
                    Accessible.checkable: true
                    Accessible.checked: navItem.selected
                    Accessible.onPressAction: navItem.activate()
                    Keys.onReturnPressed: navItem.activate()
                    Keys.onSpacePressed: navItem.activate()

                    function activate() {
                      root.currentPage = modelData.id
                      if (searchField.text.length > 0) searchField.text = ""
                      else root.loadHub(modelData.id)
                    }

                    onSelectedChanged: if (selected) root.placeNavHighlight(navItem)
                    Component.onCompleted: if (selected) Qt.callLater(function() { root.placeNavHighlight(navItem) })

                    PrefsIcon {
                      id: navIcon
                      anchors.left: parent.left
                      anchors.leftMargin: Theme.pad
                      anchors.verticalCenter: parent.verticalCenter
                      name: modelData && modelData.icon ? modelData.icon : ""
                      size: Theme.navIconSize
                      color: navItem.selected || navItem.hovered || navItem.activeFocus
                        ? Theme.foreground
                        : Theme.muted
                    }

                    Text {
                      anchors.left: navIcon.right
                      anchors.right: navBadge.visible ? navBadge.left : parent.right
                      anchors.verticalCenter: parent.verticalCenter
                      anchors.leftMargin: Theme.space
                      anchors.rightMargin: navBadge.visible ? Theme.space : Theme.pad
                      text: modelData.title
                      color: Theme.foreground
                      font.family: Theme.fontFamily
                      font.pixelSize: Theme.labelSize
                      font.bold: navItem.selected
                      elide: Text.ElideRight
                    }

                    // Live state at the row edge. Silent when there is
                    // nothing to say, which is the common case.
                    Text {
                      id: navBadge
                      anchors.right: parent.right
                      anchors.rightMargin: Theme.pad
                      anchors.verticalCenter: parent.verticalCenter
                      readonly property var badge: NavStatusJs.forHub(
                        modelData ? modelData.id : "", root.navState)
                      visible: !!badge
                      text: badge ? badge.text : ""
                      color: badge && badge.tone === "warn" ? Theme.urgent : Theme.muted
                      font.family: Theme.fontFamily
                      font.pixelSize: Theme.badgeSize
                      Accessible.role: Accessible.StaticText
                      Accessible.name: badge ? badge.title : ""
                    }

                    MouseArea {
                      id: navMouse
                      anchors.fill: parent
                      hoverEnabled: true
                      cursorShape: Qt.PointingHandCursor
                      onClicked: {
                        navItem.forceActiveFocus()
                        navItem.activate()
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
      readonly property bool atmosRightPane: true
      anchors.left: divider.right
      anchors.right: parent.right
      anchors.top: parent.top
      anchors.bottom: parent.bottom

      PrefsGuardBar {
        id: guardBar
      }

      Item {
        id: header
        anchors.top: guardBar.bottom
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

      StackView {
        id: pageStack
        anchors.top: header.bottom
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
          var launched = root.canonicalHub(root.launchPath)
          var hub = root.hubId(launched)
          if (!hub) hub = "appearance"
          root.currentPage = hub
          pageStack.push(root.pageComponent(hub), {}, StackView.Immediate)
          Qt.callLater(function() { root.revealCurrentNav() })
          var sub = launched === root.launchPath ? root.subId(root.launchPath) : ""
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
      id: errorDialog
      title: "Error"
      closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

      PrefsText {
        width: parent.width
        text: Omarchy.lastError
        color: Theme.urgent
        wrapMode: Text.WordWrap
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
      }

      PrefsButton {
        text: "Ask my Agent to work on this"
        primary: true
        onClicked: Omarchy.askAgentAboutError()
      }

      Row {
        anchors.right: parent.right
        spacing: Theme.space

        PrefsButton {
          text: "Copy"
          onClicked: Omarchy.copyLastError()
        }

        PrefsButton {
          text: "Dismiss"
          onClicked: {
            errorDialog.close()
            Omarchy.clearLastError()
          }
        }
      }

      onClosed: {
        if (Omarchy.lastError.length > 0)
          Omarchy.clearLastError()
      }
    }

    Connections {
      target: Omarchy
      function onLastErrorChanged() {
        if (Omarchy.lastError.length > 0) {
          if (!errorDialog.visible) errorDialog.open()
        } else if (errorDialog.visible) {
          errorDialog.close()
        }
      }
    }

    // A keyboard-first app has to teach its own keys rather than send you
    // to a README.
    PrefsDialog {
      id: keysDialog
      title: "Keyboard"
      closePolicy: Popup.CloseOnEscape

      Repeater {
        model: [
          { keys: "A letter", what: "Search settings" },
          { keys: "Up  /  Down", what: "Move through hubs or search hits" },
          { keys: "Ctrl+J  /  Ctrl+K", what: "Move through hubs or search hits" },
          { keys: "Home  /  End", what: "Jump to the first or last" },
          { keys: "/  /  Ctrl+F", what: "Focus search" },
          { keys: "Enter", what: "Open the highlighted hub or setting" },
          { keys: "Tab", what: "Move through controls on the page" },
          { keys: "Escape", what: "Revert a pending change, go back, or leave search" },
          { keys: "Ctrl+/", what: "This sheet" }
        ]
        delegate: Item {
          required property var modelData
          width: parent ? parent.width : 0
          height: Theme.rowHeight

          PrefsText {
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            width: Theme.spinWidth + Theme.spaceMd
            text: modelData.keys
            color: Theme.accent
            font.family: Theme.fontFamily
            font.pixelSize: Theme.labelSize
            font.bold: true
          }
          PrefsText {
            anchors.left: parent.left
            anchors.leftMargin: Theme.spinWidth + Theme.spaceLg
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            text: modelData.what
            color: Theme.foreground
            font.family: Theme.fontFamily
            font.pixelSize: Theme.labelSize
          }
        }
      }

      Row {
        anchors.right: parent.right
        PrefsButton {
          text: "Close"
          primary: true
          onClicked: keysDialog.close()
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

    Item {
      Component.onCompleted: {
        var item = window.contentItem
        if (!item) return
        item.Keys.priority = Keys.BeforeItem
        item.Keys.pressed.connect(function(event) { root.startSearch(event) })
      }
    }

    Shortcut {
      sequences: ["Ctrl+F", "/"]
      enabled: !root.modalOpen
      onActivated: searchField.forceActiveFocus()
    }

    Shortcut {
      sequences: ["Ctrl+/"]
      enabled: !root.modalOpen
      onActivated: keysDialog.open()
    }

    // Letters belong to search. These move the list, including while the
    // search field is focused. Another text field keeps its own keys.
    Shortcut {
      sequences: ["Down", "Ctrl+J"]
      enabled: root.listKeys
      onActivated: root.moveList(1)
    }
    Shortcut {
      sequences: ["Up", "Ctrl+K"]
      enabled: root.listKeys
      onActivated: root.moveList(-1)
    }
    Shortcut {
      sequences: ["Home"]
      enabled: root.listKeys
      onActivated: root.jumpList(false)
    }
    Shortcut {
      sequences: ["End"]
      enabled: root.listKeys
      onActivated: root.jumpList(true)
    }
    Shortcut {
      sequences: ["Return", "Enter"]
      enabled: root.listKeys && (searchField.activeFocus || !window.activeFocusItem || window.activeFocusItem === window.contentItem)
      onActivated: root.activateList()
    }

    Shortcut {
      sequences: ["Escape"]
      onActivated: {
        if (Omarchy.guardOpen) {
          Omarchy.revertGuard()
          return
        }
        var page = pageStack.currentItem
        if (page && page.chartOpen) {
          if (page.zoomOutChart && page.zoomOutChart()) return
          page.collapseChart()
          return
        }
        if (pageStack.depth > 1) pageStack.pop()
        else if (searchField.text.length > 0) searchField.text = ""
      }
    }

    TapHandler {
      acceptedButtons: Qt.BackButton
      enabled: pageStack.depth > 1 && !errorDialog.visible && !sudoModeDialog.visible
      onTapped: pageStack.pop()
    }
  }
}
