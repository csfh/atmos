import QtQuick
import QtQuick.Controls
import "../services"
import "../services/Layout.js" as LayoutJs

Item {
  id: root

  property string title: ""
  property string body: ""
  property string command: ""
  property var topics: []

  readonly property bool hasTopics: topics && topics.length > 0
  readonly property bool hasContent: body.length > 0 || command.length > 0 || hasTopics
  readonly property string accessibleName: LayoutJs.helpAccessibleName(title)
  readonly property string tooltipText: {
    if (root.body.length > 0) return root.body
    if (root.hasTopics && root.topics[0] && root.topics[0].body)
      return String(root.topics[0].body)
    if (root.command.length > 0) return root.command
    return root.accessibleName
  }

  implicitWidth: visible ? Theme.helpHit : 0
  implicitHeight: Theme.helpHit
  visible: hasContent
  width: implicitWidth
  height: implicitHeight
  z: 2
  activeFocusOnTab: visible

  Accessible.role: Accessible.Button
  Accessible.name: root.accessibleName
  Accessible.onPressAction: popup.open()

  Keys.onReturnPressed: popup.open()
  Keys.onSpacePressed: popup.open()

  PrefsIcon {
    anchors.centerIn: parent
    name: Theme.iconInfo
    size: Theme.helpIcon
    color: helpMouse.containsMouse || root.activeFocus ? Theme.foreground : Theme.muted

    Behavior on color {
      ColorAnimation { duration: Theme.motionFast }
    }
  }

  MouseArea {
    id: helpMouse
    anchors.fill: parent
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: popup.open()
  }

  ToolTip {
    visible: root.hasContent && helpMouse.containsMouse && !popup.visible
    delay: 400
    timeout: 8000
    text: root.tooltipText
    contentItem: Text {
      width: 360
      text: root.tooltipText
      wrapMode: Text.Wrap
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }
  }

  component CommandBox: Rectangle {
    property string command: ""

    width: parent.width
    visible: command.length > 0
    implicitHeight: visible ? commandText.implicitHeight + 16 : 0
    height: implicitHeight
    color: Theme.fill(0.08)
    border.width: Theme.borderWidth
    border.color: Theme.borderColor()
    radius: Theme.radius

    Text {
      id: commandText
      anchors.left: parent.left
      anchors.right: parent.right
      anchors.verticalCenter: parent.verticalCenter
      anchors.margins: 8
      text: parent.command
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
      wrapMode: Text.WrapAnywhere
    }
  }

  Popup {
    id: popup
    modal: true
    focus: true
    padding: Theme.pad * 1.5
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    anchors.centerIn: Overlay.overlay
    width: Math.min(480, Overlay.overlay ? Overlay.overlay.width - 48 : 480)

    // The "?" sits in the group header inside PrefsFlickable. Reparent onto
    // Overlay so clip cannot crop the modal when the header is near an edge.
    onAboutToShow: {
      var overlay = Overlay.overlay
      if (overlay)
        parent = overlay
    }

    background: Rectangle {
      color: Theme.background
      border.width: Theme.borderWidth
      border.color: Theme.borderColor()
      radius: Theme.radius
    }

    Overlay.modal: Rectangle {
      color: Qt.rgba(0, 0, 0, 0.55)
    }

    Column {
      width: parent.width
      spacing: Theme.pad

      PrefsText {
        width: parent.width
        visible: root.title.length > 0
        text: root.title
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        font.bold: true
      }

      PrefsFlickable {
        width: parent.width
        visible: helpColumn.implicitHeight > 0
        height: visible ? Math.min(helpColumn.implicitHeight, 360) : 0
        contentHeight: helpColumn.implicitHeight
        clip: true

        Column {
          id: helpColumn
          width: parent.width
          spacing: Theme.pad

          PrefsText {
            width: parent.width
            visible: root.body.length > 0
            text: root.body
            color: Theme.foreground
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
          }

          CommandBox {
            command: root.command
          }

          Repeater {
            model: root.topics

            Column {
              required property var modelData
              required property int index

              width: helpColumn.width
              spacing: 6

              Rectangle {
                width: parent.width
                height: 1
                visible: index > 0 || root.body.length > 0 || root.command.length > 0
                color: Theme.splitColor()
              }

              PrefsText {
                width: parent.width
                visible: modelData && modelData.title && String(modelData.title).length > 0
                text: modelData && modelData.title ? modelData.title : ""
                color: Theme.foreground
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
                font.bold: true
              }

              PrefsText {
                width: parent.width
                visible: modelData && modelData.body && String(modelData.body).length > 0
                text: modelData && modelData.body ? modelData.body : ""
                color: Theme.foreground
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
              }

              CommandBox {
                command: modelData && modelData.command ? String(modelData.command) : ""
              }
            }
          }
        }
      }
    }
  }
}
