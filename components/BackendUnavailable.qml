import QtQuick
import "../services"
import "../services/BackendProtocol.js" as Protocol

// The one screen for a backend that is not there (docs/adr/0010). It covers
// the window while the server or the daemon does not answer, says why, and
// goes away by itself when Backend reconnects; nothing behind it runs a thing.
Rectangle {
  id: root

  readonly property var notice: Protocol.unavailableNotice(Backend.link, Backend.daemonDown)

  anchors.fill: parent
  z: 100
  visible: !Backend.available
  color: Theme.background
  focus: visible
  onVisibleChanged: if (visible) forceActiveFocus()
  // Nor a key: a field that had focus must not type behind the screen.
  Keys.onPressed: function(event) { event.accepted = true }

  // Nothing behind it takes a click or a key.
  MouseArea {
    anchors.fill: parent
    hoverEnabled: true
    onWheel: function(wheel) { wheel.accepted = true }
  }

  Column {
    anchors.centerIn: parent
    width: Math.min(parent.width - Theme.spaceLg * 2, 420)
    spacing: Theme.spaceMd

    PrefsIcon {
      anchors.horizontalCenter: parent.horizontalCenter
      name: "server-line"
      size: Theme.navIconSize * 2
      color: Theme.muted
    }

    PrefsText {
      width: parent.width
      text: root.notice.title
      horizontalAlignment: Text.AlignHCenter
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.titleSize
    }

    PrefsText {
      width: parent.width
      text: root.notice.detail
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      color: Theme.urgent
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
    }

    PrefsText {
      width: parent.width
      text: root.notice.status
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.fontSize
    }
  }
}
