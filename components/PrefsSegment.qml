import QtQuick
import "../services"
import "../services/RichUi.js" as RichUi

// One tap per option, every segment the same width. The selected segment
// carries the selected fill and an accent edge; the rest read as quiet
// hairline chrome until hovered, the same state language as PrefsButton.
Row {
  id: root

  property var options: []
  property string value: ""
  property bool enabled: true

  signal changed(string value)

  spacing: Theme.stackGap
  opacity: Theme.controlOpacity(enabled)

  Accessible.role: Accessible.List

  property int cellWidth: 64

  TextMetrics {
    id: cellMetrics
    font.family: Theme.fontFamily
    font.pixelSize: Theme.labelSize
  }

  function measureCells() {
    var list = root.options || []
    var widest = 0
    for (var i = 0; i < list.length; i++) {
      cellMetrics.text = RichUi.optionLabel(list[i])
      widest = Math.max(widest, Math.ceil(cellMetrics.width))
    }
    root.cellWidth = Math.max(64, widest + Theme.pad * 2)
  }

  onOptionsChanged: measureCells()
  Component.onCompleted: measureCells()

  Repeater {
    model: root.options

    Rectangle {
      id: cell
      required property var modelData

      readonly property string optValue: RichUi.optionValue(modelData)
      readonly property string optLabel: RichUi.optionLabel(modelData)
      readonly property bool selected: optValue === root.value
      readonly property bool hot: (mouse.containsMouse || cell.activeFocus) && root.enabled

      width: root.cellWidth
      height: Theme.controlHeight
      radius: Theme.radius
      opacity: Theme.controlOpacity(root.enabled)
      activeFocusOnTab: root.enabled

      property color bodyColor: {
        if (selected) return Theme.fill(Theme.selectedFill)
        if (hot) return Theme.fill(Theme.hoverFill)
        return Theme.fill(Theme.normalFill)
      }
      property color edgeColor: {
        if (selected || hot) return Theme.accent
        return Theme.borderColor()
      }

      color: cell.bodyColor
      border.width: Theme.borderWidth
      border.color: cell.edgeColor

      Keys.onReturnPressed: if (root.enabled) root.changed(optValue)
      Keys.onSpacePressed: if (root.enabled) root.changed(optValue)

      Behavior on bodyColor {
        ColorAnimation { duration: Theme.motionFast }
      }
      Behavior on edgeColor {
        ColorAnimation { duration: Theme.motionFast }
      }

      Accessible.role: Accessible.RadioButton
      Accessible.name: optLabel
      Accessible.checkable: true
      Accessible.checked: selected
      Accessible.onPressAction: if (root.enabled) root.changed(optValue)

      Text {
        anchors.centerIn: parent
        text: cell.optLabel
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.labelSize
      }

      MouseArea {
        id: mouse
        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: {
          cell.forceActiveFocus()
          root.changed(cell.optValue)
        }
      }
    }
  }
}
