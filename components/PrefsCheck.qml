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

  implicitWidth: Theme.checkSize
  implicitHeight: Theme.checkSize
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
    id: checkHover
    enabled: root.enabled
  }

  readonly property bool highlight: checkHover.hovered || root.activeFocus

  Rectangle {
    anchors.fill: parent
    radius: Theme.radius
    color: root.highlight ? Theme.fill(Theme.hoverFill) : "transparent"
    border.width: Theme.borderWidth
    border.color: root.shownChecked || root.highlight ? Theme.accent : Theme.borderColor()

    Behavior on color {
      ColorAnimation { duration: Theme.motionFast }
    }
    Behavior on border.color {
      ColorAnimation { duration: Theme.motionFast }
    }

    Text {
      anchors.centerIn: parent
      visible: root.shownChecked
      text: "✓"
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.labelSize
      font.bold: true
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
