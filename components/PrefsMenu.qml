import QtQuick
import QtQuick.Controls
import "../services"
import "../services/PopupToggle.js" as PopupToggle

Item {
  id: root

  property var items: []
  property string accessibleName: "More actions"

  signal picked(string id)

  implicitWidth: Theme.controlHeight
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: Theme.controlOpacity(enabled)

  // SettingRow keeps the row rail while this menu is open on Overlay.
  readonly property bool overlayOpen: popup.opened

  Accessible.role: Accessible.Button
  Accessible.name: root.accessibleName
  Accessible.onPressAction: root.togglePopup()

  function itemId(item) {
    if (!item) return ""
    if (typeof item === "string") return item
    return String(item.id || "")
  }

  function itemLabel(item) {
    if (!item) return ""
    if (typeof item === "string") return item
    return String(item.label || item.id || "")
  }

  function placePopup() {
    var gap = 4
    var overlay = Overlay.overlay
    var w = Math.max(trigger.width, 200)
    popup.width = w
    if (!overlay) {
      popup.x = Math.min(0, root.width - w)
      popup.y = trigger.height + gap
      return
    }
    popup.parent = overlay
    var pos = trigger.mapToItem(overlay, 0, 0)
    var x = pos.x + trigger.width - w
    if (x + w > overlay.width) x = overlay.width - w
    if (x < 0) x = 0
    popup.x = x
    var h = popup.height > 0 ? popup.height : 200
    var below = pos.y + trigger.height + gap
    if (h > overlay.height - below && pos.y - gap - h >= 0)
      popup.y = pos.y - gap - h
    else
      popup.y = Math.max(0, below)
  }

  function togglePopup() {
    PopupToggle.toggle(popup, root.enabled)
  }

  function pick(id) {
    popup.close()
    if (id) root.picked(id)
  }

  Rectangle {
    id: trigger
    anchors.fill: parent
    radius: Theme.radius
    color: triggerHover.containsMouse || trigger.focus || popup.opened
      ? Theme.fill(Theme.hoverFill)
      : Theme.fill(Theme.normalFill)
    border.width: Theme.borderWidth
    border.color: trigger.focus || triggerHover.containsMouse || popup.opened
      ? Theme.accent
      : Theme.borderColor()
    activeFocusOnTab: root.enabled

    Keys.onReturnPressed: root.togglePopup()
    Keys.onSpacePressed: root.togglePopup()

    Behavior on color {
      ColorAnimation { duration: Theme.motionFast }
    }
    Behavior on border.color {
      ColorAnimation { duration: Theme.motionFast }
    }

    Text {
      anchors.centerIn: parent
      text: "…"
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.labelSize
    }

    MouseArea {
      id: triggerHover
      anchors.fill: parent
      enabled: root.enabled
      hoverEnabled: true
      cursorShape: Qt.PointingHandCursor
      property bool openedAtPress: false
      onPressed: openedAtPress = popup.opened
      onClicked: PopupToggle.clickTrigger(popup, openedAtPress)
    }
  }

  Popup {
    id: popup
    y: trigger.height + 4
    width: 200
    padding: 0
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnReleaseOutside

    onOpened: {
      list.currentIndex = list.count > 0 ? 0 : -1
      list.forceActiveFocus()
      Qt.callLater(root.placePopup)
    }

    background: Rectangle {
      color: Theme.background
      border.width: Theme.borderWidth
      border.color: Theme.borderColor()
      radius: Theme.radius
    }

    contentItem: ListView {
      id: list
      clip: true
      width: popup.width
      implicitHeight: Math.min(280, Math.max(Theme.controlHeight, contentHeight))
      model: root.items
      keyNavigationEnabled: true
      highlightFollowsCurrentItem: true
      boundsBehavior: Flickable.StopAtBounds
      Keys.onReturnPressed: {
        var listItems = root.items || []
        if (currentIndex >= 0 && currentIndex < listItems.length)
          root.pick(root.itemId(listItems[currentIndex]))
      }
      Keys.onSpacePressed: {
        var listItems = root.items || []
        if (currentIndex >= 0 && currentIndex < listItems.length)
          root.pick(root.itemId(listItems[currentIndex]))
      }

      delegate: Rectangle {
        required property var modelData
        required property int index
        width: list.width
        height: Theme.rowHeight - 10
        color: optionMouse.containsMouse || index === list.currentIndex
          ? Theme.fill(Theme.selectedFill)
          : "transparent"

        Text {
          anchors.fill: parent
          anchors.leftMargin: Theme.fieldInset
          anchors.rightMargin: Theme.fieldInset
          verticalAlignment: Text.AlignVCenter
          text: root.itemLabel(modelData)
          color: Theme.foreground
          font.family: Theme.fontFamily
          font.pixelSize: Theme.fontSize
          elide: Text.ElideRight
        }

        MouseArea {
          id: optionMouse
          anchors.fill: parent
          hoverEnabled: true
          cursorShape: Qt.PointingHandCursor
          onClicked: root.pick(root.itemId(modelData))
        }
      }
    }
  }
}
