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

  // Natural text width so a Row can host a short label. Block copy still
  // sets width: parent.width and wraps. implicitWidth: width hid those
  // labels at 0px.
  implicitWidth: Math.max(1, Math.ceil(fullMetrics.advanceWidth > 0 ? fullMetrics.advanceWidth : fullMetrics.width))
  implicitHeight: label.implicitHeight
  height: implicitHeight

  TextMetrics {
    id: fullMetrics
    font: label.font
    text: root.text
  }

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

  property var _wordWidths: ({})

  function measureWord(word) {
    var hit = _wordWidths[word]
    if (hit !== undefined)
      return hit
    metrics.text = word
    var w = metrics.advanceWidth > 0 ? metrics.advanceWidth : metrics.width
    _wordWidths[word] = w
    return w
  }

  function refresh() {
    var raw = root.text
    var w = root.width
    if (!raw || w < 16) {
      if (label.text !== raw) label.text = raw
      return
    }
    var space = measureWord(" ")
    var next = root.wrapStyle === "balance"
      ? TextWrap.balanceWrap(raw, measureWord, w, space)
      : TextWrap.prettyWrap(raw, measureWord, w, space)
    if (label.text !== next) label.text = next
  }

  readonly property string _fontKey: label.font.family + "\t" + label.font.pixelSize + "\t" + label.font.weight + "\t" + label.font.italic

  onTextChanged: refresh()
  onWidthChanged: refresh()
  onWrapStyleChanged: refresh()
  on_FontKeyChanged: {
    _wordWidths = ({})
    refresh()
  }
  Component.onCompleted: refresh()
}
