import QtQuick
import "../services"
import "../services/Charts.js" as ChartsJs

// Whether a live chart is in the PrefsFlickable viewport. Off-screen
// charts must not retain FrameClock or rebuild their model.
// inView is written from refresh(), not a binding: mapToItem + x/y/width
// in a property binding loops with layout.
QtObject {
  id: root

  property Item target: null
  property var flickItem: null
  property int pad: 64
  property bool retainClock: true
  property bool clockHeld: false
  property bool inView: false

  function findFlickable(item) {
    var flick = item ? item.parent : null
    while (flick) {
      if (flick instanceof Flickable) return flick
      flick = flick.parent
    }
    return null
  }

  function isShown(item) {
    var vis = item
    while (vis) {
      if (vis.visible === false) return false
      vis = vis.parent
    }
    return true
  }

  function compute() {
    var item = root.target
    if (!item) return false
    if (!root.isShown(item)) return false
    var flick = root.flickItem
    if (!flick) return true
    var pos = item.mapToItem(flick.contentItem, 0, 0)
    return ChartsJs.inViewport(
      pos.x,
      pos.y,
      item.width,
      item.height,
      flick.contentX,
      flick.contentY,
      flick.width,
      flick.height,
      root.pad
    )
  }

  function hookFlick() {
    root.flickItem = root.findFlickable(root.target)
  }

  function refresh() {
    var next = root.compute()
    if (root.inView !== next) root.inView = next
  }

  function syncClock() {
    if (root.retainClock && root.inView) {
      if (!root.clockHeld) {
        FrameClock.retain()
        root.clockHeld = true
      }
      return
    }
    if (root.clockHeld) {
      FrameClock.release()
      root.clockHeld = false
    }
  }

  onTargetChanged: {
    root.hookFlick()
    root.refresh()
  }
  onFlickItemChanged: root.refresh()
  onInViewChanged: root.syncClock()
  Component.onCompleted: {
    root.hookFlick()
    root.refresh()
    root.syncClock()
  }
  Component.onDestruction: {
    if (root.clockHeld) FrameClock.release()
  }

  property Connections targetConn: Connections {
    target: root.target
    ignoreUnknownSignals: true
    function onXChanged() {
      root.refresh()
    }
    function onYChanged() {
      root.refresh()
    }
    function onWidthChanged() {
      root.refresh()
    }
    function onHeightChanged() {
      root.refresh()
    }
    function onVisibleChanged() {
      root.refresh()
    }
    function onParentChanged() {
      root.hookFlick()
      root.refresh()
    }
  }

  property Connections flickConn: Connections {
    target: root.flickItem
    ignoreUnknownSignals: true
    function onContentXChanged() {
      root.refresh()
    }
    function onContentYChanged() {
      root.refresh()
    }
    function onWidthChanged() {
      root.refresh()
    }
    function onHeightChanged() {
      root.refresh()
    }
    function onContentHeightChanged() {
      root.refresh()
    }
  }
}
