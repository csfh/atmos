import QtQuick
import "../services"

Item {
  id: root

  property string text: ""

  width: parent ? parent.width : 640
  implicitHeight: label.implicitHeight + Theme.space
  height: implicitHeight

  PrefsText {
    id: label
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.bottom: parent.bottom
    text: root.text
    color: Theme.foreground
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontSize
    font.bold: true
  }
}
