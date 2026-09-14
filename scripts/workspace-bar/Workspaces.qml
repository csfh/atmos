import QtQuick
import QtQuick.Layouts
import Quickshell.Hyprland
import qs.Commons
import qs.Ui

// atmos:workspace-labels
BarWidget {
  id: root
  moduleName: "omarchy.workspaces"

  function workspaceById(id) {
    var values = Hyprland.workspaces.values
    for (var i = 0; i < values.length; i++) {
      if (values[i].id === id) return values[i]
    }

    return null
  }

  readonly property int shownCount: {
    var n = 5
    if (settings && settings.count !== undefined && settings.count !== null) {
      n = Math.round(Number(settings.count))
      if (!isFinite(n) || n < 1) n = 5
      if (n > 10) n = 10
    }
    return n
  }

  readonly property bool showNames: !(settings && settings.showNames === false)

  // Stock omarchy.workspaces paints [1..5] plus any live Hypr id 1–10.
  // Atmos persists 1–10, so empty extras always exist. Keep those off the
  // strip unless they have windows or are focused — same bar as stock Hypr.
  readonly property var shownIds: {
    var ids = []
    var i
    var n = root.shownCount
    for (i = 1; i <= n; i++) ids.push(i)
    var values = Hyprland.workspaces.values
    var focusedId = Hyprland.focusedWorkspace !== null ? Hyprland.focusedWorkspace.id : 0
    for (i = 0; i < values.length; i++) {
      var workspace = values[i]
      var id = workspace.id
      if (!(id > n && id > 0 && id <= 10)) continue
      if (ids.indexOf(id) !== -1) continue
      var occupied = workspace.toplevels && workspace.toplevels.values && workspace.toplevels.values.length > 0
      if (occupied || id === focusedId) ids.push(id)
    }
    ids.sort(function(left, right) { return left - right })
    return ids
  }

  function focusWorkspace(id) {
    if (!root.bar) return
    root.bar.run("hyprctl dispatch " + Util.shellQuote("hl.dsp.focus({ workspace = \"" + id + "\" })"))
  }

  readonly property real trailingGap: root.vertical ? 0 : Style.spaceReal(1.5)

  implicitWidth: grid.implicitWidth + trailingGap
  implicitHeight: grid.implicitHeight

  GridLayout {
    id: grid
    anchors.fill: parent
    anchors.rightMargin: root.trailingGap
    columns: root.vertical ? 1 : Math.max(1, root.shownIds.length)
    columnSpacing: root.vertical ? 0 : Style.space(1)
    rowSpacing: root.vertical ? Style.space(2) : 0

    Repeater {
      model: root.shownIds

      WidgetButton {
        required property int modelData

        readonly property var workspace: root.workspaceById(modelData)
        readonly property bool occupied: workspace !== null && workspace.toplevels.values.length > 0
        readonly property bool focused: Hyprland.focusedWorkspace !== null && Hyprland.focusedWorkspace.id === modelData
        readonly property string workspaceName: workspace && workspace.name ? String(workspace.name) : ""
        readonly property bool named: root.showNames && workspaceName.length > 0 && workspaceName !== String(modelData)

        bar: root.bar
        text: named ? workspaceName : (focused ? "\uDB85\uDCFB" : (modelData === 10 ? "0" : String(modelData)))
        opacity: occupied || focused ? 1 : 0.5
        horizontalMargin: 6
        verticalPadding: 6
        fixedWidth: named ? 0 : (root.vertical ? root.barSize : Style.space(20))
        fixedHeight: root.barSize
        onPressed: function() { root.focusWorkspace(modelData) }
      }
    }
  }
}
