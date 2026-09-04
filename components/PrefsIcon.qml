import QtQuick
import QtQuick.Shapes
import "../services"
import "../services/RemixIcons.js" as RemixIcons

Item {
  id: root

  property string name: ""
  property color color: Theme.muted
  property int size: Theme.fontSize

  readonly property int box: RemixIcons.viewBoxSize()

  implicitWidth: size
  implicitHeight: size
  width: implicitWidth
  height: implicitHeight

  Shape {
    width: root.box
    height: root.box
    preferredRendererType: Shape.CurveRenderer
    transform: Scale {
      xScale: root.box > 0 ? root.size / root.box : 1
      yScale: root.box > 0 ? root.size / root.box : 1
    }

    ShapePath {
      fillColor: root.color
      strokeColor: "transparent"
      fillRule: ShapePath.WindingFill
      PathSvg {
        path: RemixIcons.pathFor(root.name)
      }
    }
  }
}
