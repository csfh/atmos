import QtQuick
import Quickshell.Hyprland
import qs.Commons
import qs.Ui

// atmos:workspace-labels
BarWidget {
  id: root
  moduleName: "omarchy.workspaces"

  property int shownCount: 5

  function workspaceById(id) {
    var values = Hyprland.workspaces.values
    for (var i = 0; i < values.length; i++) {
      if (values[i].id === id) return values[i]
    }

    return null
  }

  function clampShown(raw) {
    var n = Math.round(Number(raw))
    if (!isFinite(n) || n < 1) n = 5
    if (n > 10) n = 10
    return n
  }

  function applySettings() {
    var s = settings
    root.shownCount = clampShown(s && s.count != null ? s.count : 5)
  }

  onSettingsChanged: applySettings()
  Component.onCompleted: applySettings()

  readonly property bool showNames: !settings || settings.showNames !== false

  readonly property var shownIds: {
    var ids = []
    var i
    var n = root.shownCount
    for (i = 1; i <= n; i++) ids.push(i)
    return ids
  }

  function focusWorkspace(id) {
    if (!root.bar) return
    root.bar.run("hyprctl dispatch " + Util.shellQuote("hl.dsp.focus({ workspace = \"" + id + "\" })"))
  }

  function workspaceLabel(id, workspace, focused) {
    var name = workspace && workspace.name ? String(workspace.name) : ""
    if (root.showNames && name && name !== String(id)) return name
    if (focused) return "\uDB85\uDCFB"
    return id === 10 ? "0" : String(id)
  }

  readonly property real trailingGap: root.vertical ? 0 : Style.spaceReal(1.5)

  implicitWidth: grid.implicitWidth + trailingGap
  implicitHeight: grid.implicitHeight

  Grid {
    id: grid
    anchors.fill: parent
    anchors.rightMargin: root.trailingGap
    columns: root.vertical ? 1 : Math.max(1, root.shownCount)
    columnSpacing: root.vertical ? 0 : Style.space(1)
    rowSpacing: root.vertical ? Style.space(2) : 0

    Repeater {
      model: root.shownIds

      WidgetButton {
        required property int modelData

        readonly property var workspace: root.workspaceById(modelData)
        readonly property bool occupied: workspace !== null && workspace.toplevels.values.length > 0
        readonly property bool focused: Hyprland.focusedWorkspace !== null && Hyprland.focusedWorkspace.id === modelData
        readonly property string label: root.workspaceLabel(modelData, workspace, focused)
        readonly property bool named: {
          var name = workspace && workspace.name ? String(workspace.name) : ""
          return root.showNames && name.length > 0 && name !== String(modelData)
        }

        bar: root.bar
        text: label
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
