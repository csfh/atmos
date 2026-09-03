import QtQuick
import "../services"

Item {
  id: root

  property bool checked: false
  property bool enabled: true
  property bool _holding: false
  property bool _heldChecked: false

  readonly property bool shownChecked: _holding ? _heldChecked : checked

  signal toggled()

  implicitWidth: 44
  implicitHeight: 24
  width: implicitWidth
  height: implicitHeight
  opacity: enabled ? 1 : 0.45
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
    color: root.shownChecked ? Theme.accentFill(root.highlight ? 1 : 0.85) : Theme.fill(root.highlight ? Theme.hoverFill : 0.12)
    border.width: 1
    border.color: root.shownChecked || root.highlight ? Theme.accent : Theme.borderColor()

    Behavior on color {
      ColorAnimation { duration: 90 }
    }
    Behavior on border.color {
      ColorAnimation { duration: 90 }
    }

    Rectangle {
      width: 16
      height: 16
      radius: 8
      anchors.verticalCenter: parent.verticalCenter
      x: root.shownChecked ? parent.width - width - 4 : 4
      color: Theme.foreground

      Behavior on x { NumberAnimation { duration: 120 } }
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
    root.toggled()
    if (root.checked === next) root._holding = false
  }

  onCheckedChanged: {
    if (_holding && checked === _heldChecked)
      _holding = false
  }
}
