import QtQuick
import QtQuick.Controls
import "../services"
import "../services/PopupToggle.js" as PopupToggle
import "../services/RichUi.js" as RichUi

Item {
  id: root

  property string value: ""
  property var options: []
  property bool enabled: true
  property bool searchable: false
  property bool _holding: false
  property string _heldValue: ""

  readonly property string shownValue: _holding ? _heldValue : value

  readonly property bool useSearch: searchable || (options && options.length >= 8)

  // SettingRow keeps the row rail while this list is open on Overlay.
  readonly property bool overlayOpen: popup.opened

  signal changed(string value)

  // Opt-in live preview. Emitted as the pointer (or highlight) moves over
  // options, and with "" when it leaves the list or the popup closes.
  // Callers that do not subscribe are unaffected.
  signal previewed(string value)

  // Last value handed out, so a repeat of the same option does not re-apply
  // on every mouse move across one row.
  property string hoveredOption: ""

  function hoverOption(value) {
    var next = String(value || "")
    if (next === root.hoveredOption) return
    root.hoveredOption = next
    root.previewed(next)
  }

  // Pick already committed through changed(). Clearing without emitting
  // "" means the subscriber must not restore over that paint.
  function clearHover() {
    root.hoveredOption = ""
  }

  implicitWidth: Theme.controlColumnWidth
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: Theme.controlOpacity(enabled)

  Accessible.role: Accessible.ComboBox
  Accessible.name: displayLabel
  Accessible.onPressAction: root.togglePopup()

  property string filter: ""
  property string displayLabel: ""
  property var shownOptions: []

  function optionValue(item) { return RichUi.optionValue(item) }
  function optionLabel(item) { return RichUi.optionLabel(item) }

  function refreshDisplayLabel() {
    var list = options || []
    for (var i = 0; i < list.length; i++) {
      if (optionValue(list[i]) === shownValue) {
        displayLabel = optionLabel(list[i])
        return
      }
    }
    displayLabel = shownValue
  }

  function refreshShownOptions() {
    shownOptions = RichUi.filterOptions(options, useSearch ? filter : "")
  }

  onValueChanged: {
    if (_holding && value === _heldValue) _holding = false
    refreshDisplayLabel()
  }
  onShownValueChanged: refreshDisplayLabel()
  onOptionsChanged: {
    refreshDisplayLabel()
    refreshShownOptions()
  }
  onFilterChanged: refreshShownOptions()
  onUseSearchChanged: refreshShownOptions()
  Component.onCompleted: {
    refreshDisplayLabel()
    refreshShownOptions()
  }

  function currentIndexOfValue() {
    var listOpts = shownOptions || []
    for (var i = 0; i < listOpts.length; i++) {
      if (optionValue(listOpts[i]) === shownValue) return i
    }
    return listOpts.length > 0 ? 0 : -1
  }

  function pickValue(next) {
    _heldValue = next
    _holding = true
    changed(next)
    if (value === next) _holding = false
    refreshDisplayLabel()
    root.clearHover()
    popup.close()
  }

  function pickCurrent() {
    if (list.currentIndex < 0 || list.currentIndex >= (shownOptions || []).length) return
    pickValue(optionValue(shownOptions[list.currentIndex]))
  }

  function togglePopup() {
    PopupToggle.toggle(popup, root.enabled)
  }

  // Popup lives on Overlay so PrefsFlickable clip cannot crop it. y on the
  // declaring item would also be wrong after that reparent, so map the
  // trigger into overlay coordinates and flip above when the list would
  // run off the bottom of the window.
  function placePopup() {
    var gap = 4
    var overlay = Overlay.overlay
    popup.width = trigger.width
    if (!overlay) {
      popup.x = 0
      popup.y = trigger.height + gap
      return
    }
    popup.parent = overlay
    var pos = trigger.mapToItem(overlay, 0, 0)
    var x = pos.x
    if (x + popup.width > overlay.width) x = overlay.width - popup.width
    if (x < 0) x = 0
    popup.x = x
    var h = popup.height > 0 ? popup.height : 280
    var below = pos.y + trigger.height + gap
    if (h > overlay.height - below && pos.y - gap - h >= 0)
      popup.y = pos.y - gap - h
    else
      popup.y = Math.max(0, below)
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

    Behavior on color {
      ColorAnimation { duration: Theme.motionFast }
    }
    Behavior on border.color {
      ColorAnimation { duration: Theme.motionFast }
    }

    activeFocusOnTab: root.enabled

    Keys.onReturnPressed: root.togglePopup()
    Keys.onSpacePressed: root.togglePopup()

    Text {
      anchors.left: parent.left
      anchors.right: chevron.left
      anchors.verticalCenter: parent.verticalCenter
      anchors.leftMargin: Theme.fieldInset
      anchors.rightMargin: 8
      text: root.displayLabel
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      elide: Text.ElideRight
    }

    Text {
      id: chevron
      anchors.right: parent.right
      anchors.rightMargin: Theme.fieldInset
      anchors.verticalCenter: parent.verticalCenter
      text: "▾"
      color: Theme.muted
      font.pixelSize: Theme.captionSize
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
    width: trigger.width
    padding: 0
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnReleaseOutside

    onOpened: {
      root.filter = ""
      list.currentIndex = currentIndexOfValue()
      if (root.useSearch) searchField.forceActiveFocus()
      else list.forceActiveFocus()
      Qt.callLater(root.placePopup)
    }
    onClosed: {
      root.filter = ""
      // Dismiss by Escape or click-away puts the live value back. pickValue
      // already cleared hoveredOption, so this is a no-op after a click.
      root.hoverOption("")
    }

    background: Rectangle {
      color: Theme.background
      border.width: Theme.borderWidth
      border.color: Theme.borderColor()
      radius: Theme.radius
    }

    contentItem: Column {
      width: popup.width

      // One revert site, so there is no race between a row losing the
      // pointer and its neighbour gaining it.
      HoverHandler {
        id: listHover
        onHoveredChanged: if (!hovered) root.hoverOption("")
      }

      Rectangle {
        visible: root.useSearch
        width: parent.width
        height: Theme.rowHeight - 10
        color: "transparent"

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
          onTextChanged: {
            root.filter = text
            list.currentIndex = list.count > 0 ? 0 : -1
          }
          Keys.onDownPressed: list.forceActiveFocus()
          Keys.onReturnPressed: root.pickCurrent()
          Keys.onEnterPressed: root.pickCurrent()

          Text {
            anchors.fill: parent
            visible: searchField.text.length === 0 && !searchField.activeFocus
            text: "Find an option"
            color: Theme.muted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            verticalAlignment: Text.AlignVCenter
          }
        }
      }

      ListView {
        id: list
        clip: true
        width: parent.width
        implicitHeight: Math.min(280, Math.max(36, contentHeight))
        model: root.shownOptions
        flickableDirection: Flickable.VerticalFlick
        boundsBehavior: Flickable.StopAtBounds
        interactive: contentHeight > height
        keyNavigationEnabled: true
        highlightFollowsCurrentItem: true
        Keys.onReturnPressed: root.pickCurrent()
        Keys.onSpacePressed: root.pickCurrent()
        onCurrentIndexChanged: {
          if (!popup.opened || currentIndex < 0) return
          if (currentIndex >= (root.shownOptions || []).length) return
          root.hoverOption(root.optionValue(root.shownOptions[currentIndex]))
        }

        // Same viewport wheel handler as PrefsFlickable. Delegate MouseAreas
        // swallow the wheel, and a WheelHandler on this ListView never fires.
        MouseArea {
          parent: list
          width: list.width
          height: list.height
          acceptedButtons: Qt.NoButton

          onWheel: function(wheel) {
            var max = Math.max(0, list.contentHeight - list.height)
            var dy = wheel.pixelDelta.y
            if (wheel.pixelDelta.y === 0)
              dy = (wheel.angleDelta.y / 120) * Theme.rowHeight * 3
            if (max > 0 && dy !== 0) {
              list.cancelFlick()
              list.contentY = Math.max(0, Math.min(max, list.contentY - dy))
            }
            // Consume even when the list already fits, so the page behind
            // the open popup does not move.
            wheel.accepted = true
          }
        }
        delegate: Rectangle {
          required property var modelData
          required property int index
          width: list.width
          height: Theme.rowHeight - 10
          color: optionMouse.containsMouse || index === list.currentIndex || root.optionValue(modelData) === root.shownValue
            ? Theme.fill(Theme.selectedFill)
            : "transparent"

          Text {
            anchors.fill: parent
            anchors.leftMargin: Theme.fieldInset
            anchors.rightMargin: Theme.fieldInset
            verticalAlignment: Text.AlignVCenter
            text: root.optionLabel(modelData)
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
            // Enter only. The matching revert is on the list and on the
            // popup, never on the row -- moving between two adjacent rows
            // can fire the old row's onExited after the new row's onEntered,
            // which undoes the preview the instant it is set and makes the
            // whole feature look like it does nothing.
            onEntered: root.hoverOption(root.optionValue(modelData))
            onClicked: root.pickValue(root.optionValue(modelData))
          }
        }

        Text {
          visible: list.count === 0
          anchors.centerIn: parent
          text: "Nothing matches that."
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.captionSize
        }
      }
    }
  }
}
