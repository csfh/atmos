import QtQuick
import "../services"

// Shimmer placeholder while live samples arrive. Theme-aware fill only;
// no private palette. Pulse instead of slide so it stays cheap.
Rectangle {
  id: root

  property bool active: true

  implicitWidth: 260
  implicitHeight: 14
  radius: Theme.radius
  color: Theme.fill(Theme.normalFill)
  opacity: active ? 1 : 0
  visible: opacity > 0

  Behavior on opacity {
    NumberAnimation { duration: Theme.motionFast }
  }

  SequentialAnimation on color {
    running: root.active && root.visible
    loops: Animation.Infinite
    ColorAnimation {
      to: Theme.fill(Theme.hoverFill)
      duration: 700
      easing.type: Easing.InOutQuad
    }
    ColorAnimation {
      to: Theme.fill(Theme.normalFill)
      duration: 700
      easing.type: Easing.InOutQuad
    }
  }

  Accessible.role: Accessible.StaticText
  Accessible.name: I18n.tr("Loading")
}
