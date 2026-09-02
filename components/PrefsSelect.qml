import QtQuick
import QtQuick.Controls
import "../services"
import "../services/RichUi.js" as RichUi

Item {
  id: root

  property string value: ""
  property var options: []
  property bool enabled: true
  property bool searchable: false

  readonly property bool useSearch: searchable || (options && options.length >= 8)

  signal changed(string value)

  implicitWidth: Theme.controlColumnWidth
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: enabled ? 1 : 0.45

  Accessible.role: Accessible.ComboBox
  Accessible.name: currentLabel()
  Accessible.onPressAction: if (enabled) popup.open()

  property string filter: ""

  function optionValue(item) { return RichUi.optionValue(item) }
  function optionLabel(item) { return RichUi.optionLabel(item) }

  function currentLabel() {
    for (var i = 0; i < options.length; i++) {
      if (optionValue(options[i]) === value) return optionLabel(options[i])
    }
    return value
  }

  function filteredOptions() {
    return RichUi.filterOptions(options, useSearch ? filter : "")
  }

  Rectangle {
    id: trigger
    anchors.fill: parent
    radius: Theme.radius
    color: triggerHover.containsMouse || trigger.focus ? Theme.fill(Theme.hoverFill) : Theme.fill(Theme.normalFill)
    border.width: 1
    border.color: trigger.focus || triggerHover.containsMouse ? Theme.accent : Theme.borderColor()

    activeFocusOnTab: root.enabled

    Keys.onReturnPressed: if (root.enabled) popup.open()
    Keys.onSpacePressed: if (root.enabled) popup.open()

    Text {
      anchors.left: parent.left
      anchors.right: chevron.left
      anchors.verticalCenter: parent.verticalCenter
      anchors.leftMargin: 10
      anchors.rightMargin: 8
      text: root.currentLabel()
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
      elide: Text.ElideRight
    }

    Text {
      id: chevron
      anchors.right: parent.right
      anchors.rightMargin: 10
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
      onClicked: popup.open()
    }
  }

  Popup {
    id: popup
    y: trigger.height + 4
    width: trigger.width
    padding: 0
    modal: false
    focus: true

    onOpened: {
      root.filter = ""
      if (root.useSearch) searchField.forceActiveFocus()
    }
    onClosed: root.filter = ""

    background: Rectangle {
      color: Theme.background
      border.width: 1
      border.color: Theme.borderColor()
      radius: Theme.radius
    }

    contentItem: Column {
      width: popup.width

      Rectangle {
        visible: root.useSearch
        width: parent.width
        height: Theme.rowHeight - 10
        color: "transparent"

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
          onTextChanged: root.filter = text
          Keys.onDownPressed: list.forceActiveFocus()

          Text {
            visible: searchField.text.length === 0 && !searchField.activeFocus
            text: "Find an option"
            color: Theme.muted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
          }
        }
      }

      ListView {
        id: list
        clip: true
        width: parent.width
        implicitHeight: Math.min(280, Math.max(36, contentHeight))
        model: root.filteredOptions()
        boundsBehavior: Flickable.StopAtBounds
        delegate: Rectangle {
          required property var modelData
          required property int index
          width: list.width
          height: Theme.rowHeight - 10
          color: optionMouse.containsMouse || root.optionValue(modelData) === root.value
            ? Theme.fill(Theme.selectedFill)
            : "transparent"

          Text {
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 10
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
            onClicked: {
              root.changed(root.optionValue(modelData))
              popup.close()
            }
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
