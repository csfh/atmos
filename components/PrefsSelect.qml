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
  property bool _holding: false
  property string _heldValue: ""

  readonly property string shownValue: _holding ? _heldValue : value

  readonly property bool useSearch: searchable || (options && options.length >= 8)

  signal changed(string value)

  implicitWidth: Theme.controlColumnWidth
  implicitHeight: Theme.controlHeight
  width: implicitWidth
  height: implicitHeight
  opacity: enabled ? 1 : 0.45

  Accessible.role: Accessible.ComboBox
  Accessible.name: displayLabel
  Accessible.onPressAction: if (enabled) popup.open()

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
    popup.close()
  }

  function pickCurrent() {
    if (list.currentIndex < 0 || list.currentIndex >= (shownOptions || []).length) return
    pickValue(optionValue(shownOptions[list.currentIndex]))
  }

  Rectangle {
    id: trigger
    anchors.fill: parent
    radius: Theme.radius
    color: triggerHover.containsMouse || trigger.focus ? Theme.fill(Theme.hoverFill) : Theme.fill(Theme.normalFill)
    border.width: 1
    border.color: trigger.focus || triggerHover.containsMouse ? Theme.accent : Theme.borderColor()

    Behavior on color {
      ColorAnimation { duration: 90 }
    }
    Behavior on border.color {
      ColorAnimation { duration: 90 }
    }

    activeFocusOnTab: root.enabled

    Keys.onReturnPressed: if (root.enabled) popup.open()
    Keys.onSpacePressed: if (root.enabled) popup.open()

    Text {
      anchors.left: parent.left
      anchors.right: chevron.left
      anchors.verticalCenter: parent.verticalCenter
      anchors.leftMargin: 10
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
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    onOpened: {
      root.filter = ""
      list.currentIndex = currentIndexOfValue()
      if (root.useSearch) searchField.forceActiveFocus()
      else list.forceActiveFocus()
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
        boundsBehavior: Flickable.StopAtBounds
        keyNavigationEnabled: true
        highlightFollowsCurrentItem: true
        Keys.onReturnPressed: root.pickCurrent()
        Keys.onSpacePressed: root.pickCurrent()
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
