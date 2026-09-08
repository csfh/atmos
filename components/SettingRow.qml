import QtQuick
import "../services"
import "../services/Favorites.js" as FavJs
import "../services/Hubs.js" as HubsJs
import "../services/ShellConfig.js" as ShellConfigJs

Item {
  id: root

  readonly property bool prefsRow: true

  property string label: ""
  property string description: ""
  property string hint: ""
  // Longer copy for the section "?" modal. Leave empty to use description.
  property string detail: ""
  property string query: ""
  property var keywords: []
  // Read-only trailing copy when there is no child control. Use this for
  // informational booleans (On/Off) instead of a disabled switch.
  property string valueText: ""
  property bool stretchControl: false
  property bool available: true
  // List rows (wifi SSIDs, devices) stay out of the section modal.
  property bool sectionHelp: true
  // Find a setting indexes SettingRow blocks unless this is false.
  property bool catalog: true
  // Hairline above the row. The first row in a framed card leaves this off.
  property bool split: true
  // Extra muted line under the description (counts, status).
  property string caption: ""
  // Leading control, such as an inclusion checkbox.
  property alias leading: leadingSlot.data
  // Clicking the copy column emits activated (for row-toggle checkboxes).
  property bool interactive: false

  signal activated()

  default property alias extra: controlSlot.data

  readonly property string searchHaystack: {
    var parts = [label, description, hint, detail, valueText, caption]
    var list = keywords || []
    for (var i = 0; i < list.length; i++) parts.push(list[i])
    return ShellConfigJs.joinSearchHaystack(parts)
  }

  readonly property bool matches: ShellConfigJs.haystackMatches(query, searchHaystack)
  readonly property bool shown: available && matches

  property string favoriteHub: ""
  readonly property string resolvedHubId: {
    if (root.favoriteHub.length) return root.favoriteHub
    var p = parent
    while (p) {
      if (p.hubId !== undefined && String(p.hubId).length) return String(p.hubId)
      p = p.parent
    }
    return ""
  }
  readonly property bool canFavorite: root.catalog && root.label.length > 0 && root.resolvedHubId.length > 0
  readonly property bool favorited: FavJs.isFavorite(Omarchy.favoriteItems, root.resolvedHubId, root.label)

  function toggleStar() {
    if (!root.canFavorite) return
    Omarchy.toggleFavorite({
      hub: root.resolvedHubId,
      hubTitle: HubsJs.hubTitle(root.resolvedHubId),
      label: root.label,
      description: root.description
    })
  }

  readonly property bool hovered: rowHover.hovered

  HoverHandler {
    id: rowHover
  }

  // children.length is a real property dependency. A JS walk of .children
  // alone can stay at 0 after default-property kids land, which hid toggles.
  readonly property int controlCount: controlSlot.children.length

  // Count the slot. A row that starts hidden (Installed themes waits on
  // extraThemes) used to walk child visibility, miss the select, and never
  // run that walk again when the row appeared.
  readonly property bool hasChild: root.controlCount > 0
  readonly property bool hasControl: {
    if (root.hasChild) return true
    if (root.valueText.length > 0 && !root.stack) return true
    return false
  }

  readonly property string shownValue: {
    var _n = root.controlCount
    if (root.stack && _n > 0) {
      var kids = controlSlot.children
      var kid = kids[0]
      if (kid && kid.displayValue !== undefined && String(kid.displayValue).length)
        return String(kid.displayValue)
    }
    return root.valueText
  }

  readonly property int maxControlCol: {
    var avail = parent ? parent.width : Theme.controlColumnWidth
    return Math.max(140, avail - 160 - Theme.spaceMd)
  }

  readonly property int controlCol: {
    var wanted = Theme.controlColumnWidth
    var natural = 0
    if (root.hasChild)
      natural = controlSlot.implicitWidth
    else if (root.valueText.length > 0)
      natural = Math.ceil(statusLabel.implicitWidth)
    if (natural > wanted) wanted = natural
    return Math.min(wanted, maxControlCol)
  }

  // Stack when the control asked for the full width, or the row is too
  // narrow for a side-by-side copy column. Use Theme.controlColumnWidth
  // rather than controlCol: controlCol follows the slot, and the slot's
  // width follows stack.
  readonly property bool stack: {
    if (root.stretchControl) return true
    var inner = root.width - Theme.copyInset * 2
    if (inner <= 0) return false
    return inner < 160 + Theme.spaceMd + Theme.controlColumnWidth
  }

  visible: shown
  width: parent ? parent.width : 640
  implicitWidth: width
  implicitHeight: visible ? body.implicitHeight + Theme.rowPad * 2 : 0
  height: implicitHeight

  Rectangle {
    width: parent.width
    height: 1
    visible: root.split
    color: Theme.splitColor()
  }

  MouseArea {
    anchors.fill: parent
    z: -1
    enabled: root.interactive
    hoverEnabled: root.interactive
    cursorShape: Qt.PointingHandCursor
    onClicked: root.activated()
  }

  Flow {
    id: body
    width: parent.width - Theme.copyInset * 2
    x: Theme.copyInset
    y: Theme.rowPad
    spacing: root.stack ? Theme.stackGap : Theme.spaceMd

    Item {
      id: copyHost
      width: (root.stack || !root.hasControl)
        ? body.width
        : Math.max(160, body.width - root.controlCol - body.spacing)
      implicitWidth: width
      implicitHeight: copyCol.implicitHeight
      height: implicitHeight

      Row {
        id: copyCol
        width: parent.width
        spacing: Theme.space

        Item {
          id: favoriteHost
          visible: root.canFavorite
          width: visible ? Theme.helpHit : 0
          height: Theme.helpHit
          activeFocusOnTab: visible

          Accessible.role: Accessible.Button
          Accessible.name: (root.favorited ? "Remove from favorites: " : "Add to favorites: ") + root.label
          Accessible.onPressAction: root.toggleStar()

          Keys.onReturnPressed: root.toggleStar()
          Keys.onSpacePressed: root.toggleStar()

          PrefsIcon {
            anchors.centerIn: parent
            name: root.favorited ? Theme.iconStarOn : Theme.iconStar
            size: Theme.helpIcon
            color: root.favorited || favoriteMouse.containsMouse || favoriteHost.activeFocus
              ? Theme.accent
              : Theme.muted
          }

          MouseArea {
            id: favoriteMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.toggleStar()
          }
        }

        Item {
          id: leadingHost
          visible: leadingSlot.children.length > 0
          width: visible ? leadingSlot.implicitWidth : 0
          height: Math.max(labelText.implicitHeight, leadingSlot.implicitHeight)

          Item {
            id: leadingSlot
            implicitWidth: children.length > 0 ? children[0].implicitWidth : 0
            implicitHeight: children.length > 0 ? children[0].implicitHeight : 0
            width: implicitWidth
            height: implicitHeight
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
          }
        }

        Column {
          width: parent.width - (favoriteHost.visible ? favoriteHost.width + copyCol.spacing : 0)
            - (leadingHost.visible ? leadingHost.width + copyCol.spacing : 0)
            - (valueHost.visible ? valueHost.width + copyCol.spacing : 0)
          spacing: Theme.labelGap

          PrefsText {
            id: labelText
            width: parent.width
            visible: root.label.length > 0
            text: root.label
            color: Theme.foreground
            font.family: Theme.fontFamily
            font.pixelSize: Theme.labelSize
            font.bold: true
            horizontalAlignment: Text.AlignLeft
          }

          PrefsText {
            id: descText
            width: parent.width
            visible: root.description.length > 0
            text: root.description
            color: Theme.muted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.descriptionSize
            horizontalAlignment: Text.AlignLeft
          }

          PrefsText {
            id: captionText
            width: parent.width
            visible: root.caption.length > 0
            text: root.caption
            color: Theme.muted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.metaSize
            opacity: Theme.metaOpacity
            horizontalAlignment: Text.AlignLeft
          }
        }

        Item {
          id: valueHost
          visible: root.stack && root.shownValue.length > 0
          width: visible
            ? Math.min(valueMetrics.width, Math.max(48, Math.round(copyHost.width * 0.4)))
            : 0
          height: valueMetrics.height

          TextMetrics {
            id: valueMetrics
            font.family: Theme.fontFamily
            font.pixelSize: Theme.labelSize
            text: root.shownValue
          }

          Text {
            id: valueBit
            width: parent.width
            text: root.shownValue
            color: Theme.muted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.labelSize
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignRight
          }
        }
      }
    }

    Item {
      id: controlHost
      visible: root.hasControl
      width: root.hasControl
        ? ((root.stack || root.stretchControl) ? body.width : root.controlCol)
        : 0
      implicitWidth: width
      implicitHeight: {
        if (!root.hasControl) return 0
        var h = Math.max(Theme.controlHeight, controlSlot.implicitHeight)
        if (root.valueText.length > 0 && !root.hasChild)
          h = Math.max(h, statusLabel.implicitHeight)
        if (!root.stack)
          h = Math.max(h, labelText.implicitHeight)
        return h
      }
      height: implicitHeight

      Item {
        id: controlSlot
        implicitWidth: children.length > 0 ? children[0].implicitWidth : 0
        implicitHeight: children.length > 0 ? children[0].implicitHeight : 0
        width: root.stack && root.stretchControl
          ? controlHost.width
          : Math.min(implicitWidth, parent ? parent.width : implicitWidth)
        height: implicitHeight
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
      }

      Text {
        id: statusLabel
        visible: root.valueText.length > 0 && !root.hasChild && !root.stack
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        width: Math.min(implicitWidth, parent ? parent.width : implicitWidth)
        text: root.valueText
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.labelSize
        elide: Text.ElideRight
        horizontalAlignment: Text.AlignRight
      }
    }
  }
}
