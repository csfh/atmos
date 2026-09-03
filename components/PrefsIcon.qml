import QtQuick
import QtQuick.Effects
import "../services"

Item {
  id: root

  property string name: ""
  property color color: Theme.muted
  property int size: Theme.fontSize

  implicitWidth: size
  implicitHeight: size
  width: implicitWidth
  height: implicitHeight

  Image {
    id: src
    anchors.fill: parent
    visible: false
    source: root.name.length ? Qt.resolvedUrl("../icons/" + root.name + ".svg") : ""
    sourceSize.width: root.size
    sourceSize.height: root.size
    fillMode: Image.PreserveAspectFit
    smooth: true
    layer.enabled: true
  }

  MultiEffect {
    anchors.fill: parent
    source: src
    colorization: 1
    colorizationColor: root.color
  }
}
