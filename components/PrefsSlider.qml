import QtQuick
import QtQuick.Controls
import "../services"
import "../services/RichUi.js" as RichUi

// Full-width: ticks on the track, current value on the SettingRow label line
// (stretchControl). Compact: showTicks false, pair with PrefsSpinBox.
Item {
  id: root

  property real value: 0
  property real from: 0
  property real to: 100
  property real stepSize: 1
  property string valueText: ""
  property bool enabled: true
  property bool showValue: false
  property bool showTicks: true
  property bool live: false
  property int liveInterval: 100
  property var formatTick: null

  signal moved(real value)
  signal changed(real value)

  readonly property real _step: stepSize > 0 ? stepSize : 1
  property var ticks: []
  property bool _holding: false
  property real _heldValue: 0
  property var liveState: RichUi.sliderLiveState(liveInterval)

  function rebuildTicks() {
    ticks = RichUi.sliderTickValues(from, to, _step)
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

  readonly property int valueLineHeight: showValue ? Theme.metaSize : 0
  readonly property int trackBoxHeight: Theme.sliderTrack
  readonly property int tickLineHeight: showTicks ? Math.max(10, Theme.captionSize - 2) + Theme.sliderTickGap : 0

  implicitWidth: 260
  implicitHeight: valueLineHeight + (showValue ? Theme.sliderTickGap : 0) + trackBoxHeight + tickLineHeight
  width: implicitWidth
  height: implicitHeight
  opacity: Theme.controlOpacity(enabled)

  Accessible.role: Accessible.Slider
  Accessible.name: displayValue

  readonly property string displayValue: {
    if (slider.pressed || root._holding)
      return root.captionFor(slider.value)
    if (root.valueText.length > 0) return root.valueText
    return root.formatValue(root.value)
  }

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
    return RichUi.formatSliderNumber(v)
  }

  function captionFor(v) {
    return RichUi.formatSliderCaption(
      v,
      root.valueText,
      typeof formatTick === "function" ? formatTick(v) : ""
    )
  }

  function applyLiveWrite(result) {
    if (result.emit !== undefined)
      root.changed(result.emit)
    if (result.delayMs > 0 && !liveTimer.running) {
      liveTimer.interval = Math.max(1, Math.round(result.delayMs))
      liveTimer.start()
    }
  }

  function xForValue(v) {
    var span = to - from
    var t = span === 0 ? 0 : (v - from) / span
    if (t < 0) t = 0
    if (t > 1) t = 1
    var hw = Theme.sliderHandle
    return slider.leftPadding + t * (slider.availableWidth - hw) + hw / 2
  }

  readonly property var fittedTicks: {
    var all = root.ticks
    if (!root.showTicks || !all || all.length === 0) return []
    var labels = []
    for (var i = 0; i < all.length; i++) labels.push(root.formatValue(all[i]))
    return RichUi.sliderFitTicks(
      all,
      root.from,
      root.to,
      root.width,
      labels,
      Math.max(6, Theme.captionSize * 0.55),
      Theme.space
    )
  }

  Column {
    anchors.fill: parent
    spacing: root.showValue ? Theme.sliderTickGap : 0

    Text {
      visible: root.showValue
      height: visible ? implicitHeight : 0
      text: root.displayValue
      color: Theme.foreground
      font.family: Theme.fontFamily
      font.pixelSize: Theme.metaSize
    }

    Item {
      width: parent.width
      height: root.trackBoxHeight + root.tickLineHeight

      Slider {
        id: slider
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: Theme.sliderTrack
        from: root.from
        to: root.to
        stepSize: root._step
        snapMode: Slider.SnapAlways
        enabled: root.enabled
        onMoved: {
          var v = root.snapValue(value)
          root.moved(v)
          if (root.live) root.applyLiveWrite(RichUi.sliderLivePush(root.liveState, Date.now(), v))
        }
        onPressedChanged: {
          if (pressed) {
            root._holding = false
            return
          }
          var v = root.snapValue(value)
          liveTimer.stop()
          if (root.live)
            root.applyLiveWrite(RichUi.sliderLiveFlush(root.liveState, Date.now(), v))
          else
            root.changed(v)
          if (root.snapValue(root.value) === v) return
          root._heldValue = v
          root._holding = true
        }

        background: Item {
          x: slider.leftPadding
          y: slider.topPadding + slider.availableHeight / 2 - Theme.sliderBar / 2
          implicitWidth: 200
          implicitHeight: Theme.sliderBar
          width: slider.availableWidth
          height: Theme.sliderBar

          Rectangle {
            anchors.fill: parent
            radius: Theme.radius
            color: Theme.fill(0.2)
          }

          Rectangle {
            width: slider.visualPosition * parent.width
            height: parent.height
            color: Theme.accent
            radius: Theme.radius
          }
        }

        handle: Rectangle {
          x: slider.leftPadding + slider.visualPosition * (slider.availableWidth - width)
          y: slider.topPadding + slider.availableHeight / 2 - height / 2
          implicitWidth: Theme.sliderHandle
          implicitHeight: Theme.sliderHandle
          radius: Theme.sliderHandle / 2
          color: Theme.foreground
          border.color: Theme.accent
          scale: slider.hovered || slider.pressed ? 1.12 : 1

          Behavior on scale {
            NumberAnimation { duration: Theme.motionFast; easing.type: Easing.OutCubic }
          }
        }
      }

      Repeater {
        model: root.fittedTicks

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
            y: slider.height + Theme.sliderTickGap
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

  Timer {
    id: liveTimer
    repeat: false
    onTriggered: root.applyLiveWrite(RichUi.sliderLiveTake(root.liveState, Date.now()))
  }
}
