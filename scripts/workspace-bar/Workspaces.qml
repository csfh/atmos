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
    var n = settings && settings.count != null ? Number(settings.count) : 5
    if (!isFinite(n) || n < 1) n = 5
    if (n > 10) n = 10
    return Math.round(n)
  }

  readonly property bool showNames: !settings || settings.showNames !== false

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

  GridLayout {
    id: grid
    anchors.fill: parent
    anchors.rightMargin: root.trailingGap
    columns: root.vertical ? 1 : root.shownCount
    columnSpacing: root.vertical ? 0 : Style.space(1)
    rowSpacing: root.vertical ? Style.space(2) : 0

    Repeater {
      model: root.shownCount

      WidgetButton {
        required property int modelData
        readonly property int workspaceId: modelData + 1

        readonly property var workspace: root.workspaceById(workspaceId)
        readonly property bool occupied: workspace !== null && workspace.toplevels.values.length > 0
        readonly property bool focused: Hyprland.focusedWorkspace !== null && Hyprland.focusedWorkspace.id === workspaceId
        readonly property string label: root.workspaceLabel(workspaceId, workspace, focused)
        readonly property bool named: {
          var name = workspace && workspace.name ? String(workspace.name) : ""
          return root.showNames && name.length > 0 && name !== String(workspaceId)
        }

        bar: root.bar
        text: label
        opacity: occupied || focused ? 1 : 0.5
        horizontalMargin: 6
        verticalPadding: 6
        fixedWidth: named ? 0 : (root.vertical ? root.barSize : Style.space(20))
        fixedHeight: root.barSize
        onPressed: function() { root.focusWorkspace(workspaceId) }
      }
    }
  }
}
