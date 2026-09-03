import QtQuick
import QtQuick.Controls
import "../services"

Item {
  id: root

  property real value: 0
  property real from: 0
  property real to: 100
  property real stepSize: 1
  property string valueText: ""
  property bool enabled: true
  property bool showValue: true
  property bool showTicks: true
  property var formatTick: null

  signal moved(real value)
  signal changed(real value)

  readonly property real _step: stepSize > 0 ? stepSize : 1
  property var ticks: []
  property bool _holding: false
  property real _heldValue: 0

  function rebuildTicks() {
    var n = 0
    if (to > from)
      n = Math.round((to - from) / _step)
    var every = 1
    if (n > 0) {
      var candidates = [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 24, 30, 60]
      every = Math.max(1, Math.ceil(n / 12))
      for (var c = 0; c < candidates.length; c++) {
        if (n / candidates[c] <= 12) {
          every = candidates[c]
          break
        }
      }
    }
    var list = []
    if (n > 0) {
      for (var i = 0; i <= n; i++) {
        if (i !== 0 && i !== n && i % every !== 0)
          continue
        list.push(from + i * _step)
      }
    }
    ticks = list
  }

  onFromChanged: rebuildTicks()
  onToChanged: rebuildTicks()
  onStepSizeChanged: rebuildTicks()
  Component.onCompleted: rebuildTicks()

  onValueChanged: {
    if (!_holding) return
    if (snapValue(value) === snapValue(_heldValue))
      _holding = false
  }

  readonly property int valueLineHeight: showValue ? Theme.captionSize : 0
  readonly property int trackBoxHeight: 22
  readonly property int tickLineHeight: showTicks ? Math.max(10, Theme.captionSize - 2) + 8 : 0

  implicitWidth: 260
  implicitHeight: valueLineHeight + (showValue ? 2 : 0) + trackBoxHeight + tickLineHeight
  width: implicitWidth
  height: implicitHeight
  opacity: enabled ? 1 : 0.45

  Accessible.role: Accessible.Slider
  Accessible.name: valueText.length > 0 ? valueText : formatValue(value)

  function snapValue(v) {
    var n = Math.round((v - from) / _step)
    var out = from + n * _step
    if (out < from) out = from
    if (out > to) out = to
    return out
  }

  function formatValue(v) {
    if (typeof formatTick === "function")
      return formatTick(v)
    if (Math.abs(v - Math.round(v)) < 0.001)
      return String(Math.round(v))
    return String(Math.round(v * 100) / 100)
  }

  function xForValue(v) {
    var span = to - from
    var t = span === 0 ? 0 : (v - from) / span
    if (t < 0) t = 0
    if (t > 1) t = 1
    var hw = 14
    return slider.leftPadding + t * (slider.availableWidth - hw) + hw / 2
  }

  Column {
    anchors.fill: parent
    spacing: root.showValue ? 2 : 0

    Text {
      visible: root.showValue
      height: visible ? implicitHeight : 0
      text: slider.pressed || root._holding
        ? root.formatValue(slider.value)
        : (root.valueText.length > 0 ? root.valueText : root.formatValue(root.value))
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Item {
      width: parent.width
      height: root.trackBoxHeight + root.tickLineHeight

      Slider {
        id: slider
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 22
        from: root.from
        to: root.to
        stepSize: root._step
        snapMode: Slider.SnapAlways
        value: root.value
        enabled: root.enabled
        onMoved: root.moved(root.snapValue(value))
        onPressedChanged: {
          if (pressed) {
            root._holding = false
            return
          }
          var v = root.snapValue(value)
          root.changed(v)
          if (root.snapValue(root.value) === v) return
          root._heldValue = v
          root._holding = true
        }

        background: Item {
          x: slider.leftPadding
          y: slider.topPadding + slider.availableHeight / 2 - 2
          implicitWidth: 200
          implicitHeight: 4
          width: slider.availableWidth
          height: 4

          Rectangle {
            anchors.fill: parent
            radius: 2
            color: Theme.fill(0.2)
          }

          Rectangle {
            width: slider.visualPosition * parent.width
            height: parent.height
            color: Theme.accent
            radius: 2
          }
        }

        handle: Rectangle {
          x: slider.leftPadding + slider.visualPosition * (slider.availableWidth - width)
          y: slider.topPadding + slider.availableHeight / 2 - height / 2
          implicitWidth: 14
          implicitHeight: 14
          radius: 7
          color: Theme.foreground
          border.color: Theme.accent
          scale: slider.hovered || slider.pressed ? 1.12 : 1

          Behavior on scale {
            NumberAnimation { duration: 90; easing.type: Easing.OutCubic }
          }
        }
      }

      Repeater {
        model: root.showTicks ? root.ticks : []

        Item {
          width: 1
          height: 1
          x: root.xForValue(modelData)
          y: 0

          Rectangle {
            width: 1
            height: 6
            x: -0.5
            y: slider.height - 2
            color: Theme.muted
          }

          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            y: slider.height + 4
            text: root.formatValue(modelData)
            color: Theme.muted
            font.family: Theme.fontFamily
            font.pixelSize: Math.max(9, Theme.captionSize - 2)
          }
        }
      }
    }
  }

  Binding {
    target: slider
    property: "value"
    value: root.value
    when: !slider.pressed && !root._holding
  }
}
