import QtQuick
import "../services"
import "../services/TextWrap.js" as TextWrap

Item {
  id: root

  property string text: ""
  property alias color: label.color
  property alias font: label.font
  property alias horizontalAlignment: label.horizontalAlignment
  property alias wrapMode: label.wrapMode
  property alias lineCount: label.lineCount
  property string wrapStyle: "pretty"

  implicitWidth: width
  implicitHeight: label.implicitHeight
  height: implicitHeight

  TextMetrics {
    id: metrics
    font: label.font
  }

  Text {
    id: label
    width: parent.width
    wrapMode: Text.WordWrap
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontSize
  }

  function measureWord(word) {
    metrics.text = word
    return metrics.advanceWidth > 0 ? metrics.advanceWidth : metrics.width
  }

  function refresh() {
    var raw = root.text
    var w = root.width
    if (!raw || w < 16) {
      if (label.text !== raw) label.text = raw
      return
    }
    metrics.text = " "
    var space = metrics.advanceWidth > 0 ? metrics.advanceWidth : metrics.width
    var next = root.wrapStyle === "balance"
      ? TextWrap.balanceWrap(raw, measureWord, w, space)
      : TextWrap.prettyWrap(raw, measureWord, w, space)
    if (label.text !== next) label.text = next
  }

  readonly property int _fontKey: label.font.pixelSize + label.font.weight

  onTextChanged: refresh()
  onWidthChanged: refresh()
  onWrapStyleChanged: refresh()
  on_FontKeyChanged: refresh()
  Component.onCompleted: refresh()
}
