import QtQuick
import "../services"

Item {
  id: root

  property bool checked: false
  property bool enabled: true
  property bool _holding: false
  property bool _heldChecked: false

  readonly property bool shownChecked: _holding ? _heldChecked : checked

  signal toggled(bool next)

  implicitWidth: Theme.toggleWidth
  implicitHeight: Theme.toggleHeight
  width: implicitWidth
  height: implicitHeight
  opacity: Theme.controlOpacity(enabled)
  activeFocusOnTab: enabled

  Accessible.role: Accessible.CheckBox
  Accessible.checkable: true
  Accessible.checked: shownChecked
  Accessible.onPressAction: if (enabled) root.emitToggle()

  Keys.onReturnPressed: if (enabled) root.emitToggle()
  Keys.onSpacePressed: if (enabled) root.emitToggle()

  HoverHandler {
    id: toggleHover
    enabled: root.enabled
  }

  readonly property bool highlight: toggleHover.hovered || root.activeFocus

  Rectangle {
    anchors.fill: parent
    radius: height / 2
    color: root.shownChecked ? Theme.accentFill(root.highlight ? 1 : 0.85) : Theme.fill(root.highlight ? Theme.hoverFill : Theme.toggleOffFill)
    border.width: Theme.borderWidth
    border.color: root.shownChecked || root.highlight ? Theme.accent : Theme.borderColor()

    Behavior on color {
      ColorAnimation { duration: Theme.motionFast }
    }
    Behavior on border.color {
      ColorAnimation { duration: Theme.motionFast }
    }

    Rectangle {
      width: Theme.toggleThumb
      height: Theme.toggleThumb
      radius: Theme.toggleThumb / 2
      anchors.verticalCenter: parent.verticalCenter
      x: root.shownChecked ? parent.width - width - 4 : 4
      color: Theme.foreground

      Behavior on x { NumberAnimation { duration: Theme.motionMed } }
    }
  }

  MouseArea {
    anchors.fill: parent
    enabled: root.enabled
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: root.emitToggle()
  }

  function emitToggle() {
    var next = !root.shownChecked
    root._heldChecked = next
    root._holding = true
    root.toggled(next)
    if (root.checked === next) root._holding = false
  }

  onCheckedChanged: {
    if (_holding && checked === _heldChecked)
      _holding = false
  }
}
