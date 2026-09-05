import QtQuick
import "../services"

Item {
  id: root

  property real value: 0
  property real from: 0
  property real to: 100
  property real stepSize: 1
  property bool enabled: true
  property string valueText: ""

  signal changed(real value)

  readonly property int _intValue: Math.round(value)
  readonly property string displayValue: valueText.length > 0 ? valueText : String(_intValue)

  implicitWidth: Theme.controlColumnWidth
  implicitHeight: Math.max(slider.implicitHeight, spin.implicitHeight)
  width: parent ? parent.width : implicitWidth
  height: implicitHeight

  PrefsSlider {
    id: slider
    width: Math.max(80, parent.width - spin.width - Theme.space)
    anchors.left: parent.left
    anchors.verticalCenter: parent.verticalCenter
    from: root.from
    to: root.to
    stepSize: root.stepSize
    value: root.value
    valueText: root.valueText
    enabled: root.enabled
    showValue: false
    showTicks: false
    onChanged: function(v) { root.changed(v) }
  }

  PrefsSpinBox {
    id: spin
    anchors.right: parent.right
    anchors.verticalCenter: parent.verticalCenter
    from: Math.round(root.from)
    to: Math.round(root.to)
    stepSize: Math.max(1, Math.round(root.stepSize))
    value: root._intValue
    enabled: root.enabled
    onChanged: function(v) { root.changed(v) }
  }
}
