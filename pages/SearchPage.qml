import QtQuick
import "../components"
import "../services"

Item {
  id: root

  readonly property bool searchPane: true
  property string query: ""
  property var navigator: null

  function pageHits(loader) {
    return loader.item && loader.item.hasSections === true
  }

  readonly property bool hasHits: pageHits(appearanceLoader)
    || pageHits(displayLoader)
    || pageHits(windowsLoader)
    || pageHits(inputLoader)
    || pageHits(accessibilityLoader)
    || pageHits(soundLoader)
    || pageHits(captureLoader)
    || pageHits(disksLoader)
    || pageHits(barLoader)
    || pageHits(notificationsLoader)
    || pageHits(defaultsLoader)
    || pageHits(applicationsLoader)
    || pageHits(softwareLoader)
    || pageHits(networkLoader)
    || pageHits(powerLoader)
    || pageHits(idleLoader)
    || pageHits(securityLoader)
    || pageHits(hooksLoader)
    || pageHits(systemLoader)

  function bindPage(item, withNavigator) {
    if (!item) return
    item.embed = true
    item.width = Qt.binding(function() { return pageColumn.width })
    item.query = Qt.binding(function() { return root.query })
    if (withNavigator)
      item.navigator = root.navigator
  }

  Flickable {
    id: flick
    anchors.fill: parent
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    contentWidth: width
    contentHeight: pageColumn.implicitHeight + Theme.spaceLg * 2

    Column {
      id: pageColumn
      width: Math.max(240, Math.min(Theme.contentMaxWidth, flick.width - Theme.spaceLg * 2))
      x: Math.max(Theme.spaceLg, Math.round((flick.width - width) / 2))
      y: Theme.spaceLg
      spacing: Theme.spaceLg

      Column {
        width: parent.width
        spacing: 4

        PrefsText {
          width: parent.width
          text: "Search"
          color: Theme.foreground
          font.family: Theme.fontFamily
          font.pixelSize: Theme.titleSize
          font.bold: true
        }

        PrefsText {
          width: parent.width
          text: root.hasHits
            ? "Matching settings across every page for “" + root.query + "”."
            : "Nothing on any page mentions “" + root.query + "”. Try another word."
          color: Theme.muted
          font.family: Theme.fontFamily
          font.pixelSize: Theme.fontSize
        }
      }

      Loader {
        id: appearanceLoader
        width: parent.width
        source: "AppearancePage.qml"
        onLoaded: root.bindPage(item, true)
      }
      Loader {
        id: displayLoader
        width: parent.width
        source: "DisplaysPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: windowsLoader
        width: parent.width
        source: "WindowsPage.qml"
        onLoaded: root.bindPage(item, true)
      }
      Loader {
        id: inputLoader
        width: parent.width
        source: "InputPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: accessibilityLoader
        width: parent.width
        source: "AccessibilityPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: soundLoader
        width: parent.width
        source: "SoundPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: captureLoader
        width: parent.width
        source: "CapturePage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: disksLoader
        width: parent.width
        source: "DisksPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: barLoader
        width: parent.width
        source: "BarPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: notificationsLoader
        width: parent.width
        source: "NotificationsPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: defaultsLoader
        width: parent.width
        source: "DefaultsPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: applicationsLoader
        width: parent.width
        source: "ApplicationsPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: softwareLoader
        width: parent.width
        source: "SoftwarePage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: networkLoader
        width: parent.width
        source: "NetworkPage.qml"
        onLoaded: root.bindPage(item, true)
      }
      Loader {
        id: powerLoader
        width: parent.width
        source: "PowerPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: idleLoader
        width: parent.width
        source: "IdlePage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: securityLoader
        width: parent.width
        source: "SecurityPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: hooksLoader
        width: parent.width
        source: "HooksPage.qml"
        onLoaded: root.bindPage(item, false)
      }
      Loader {
        id: systemLoader
        width: parent.width
        source: "SystemPage.qml"
        onLoaded: root.bindPage(item, false)
      }
    }
  }
}
