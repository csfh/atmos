import QtQuick
import "../components"
import "../services"
import "windows" as Win
import "rows"

PrefsPage {
  id: root
  title: "Windows"
  description: "Gaps, corners, and the tiling layout. Keybindings and window rules have their own pages. Reset only removes the block Atmos wrote."

  property var stack: null
  property var navigator: null

  function openSubpage(id) {
    if (root.navigator && root.navigator.go) {
      root.navigator.go("windows/" + id)
      return
    }
    if (!stack) return
    if (id === "bindings") stack.push(bindingsPage)
    else if (id === "rules") stack.push(rulesPage)
  }

  function overrideCountText() {
    var n = 0
    var list = Omarchy.bindings || []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].managed === true) n++
    }
    if (n === 1) return "One Atmos override in bindings.lua."
    if (n > 1) return n + " Atmos overrides in bindings.lua."
    var total = Omarchy.keybindings.length
    if (total === 1) return "One chord Hyprland is running."
    if (total > 1) return total + " chords Hyprland is running."
    return "No chords listed yet."
  }

  function ruleCountText() {
    var n = 0
    var list = Omarchy.windowRules || []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].managed === true) n++
    }
    if (n === 1) return "One Atmos rule."
    if (n > 1) return n + " Atmos rules."
    return "No Atmos rules yet."
  }

  Component { id: bindingsPage; Win.BindingsPage {} }
  Component { id: rulesPage; Win.RulesPage {} }

  PrefsGroup {
    title: "Shortcuts and rules"
    query: root.query
    detail: "Keybindings lists what Hyprland is running and lets you add a personal override. Window rules float, tile, or pin a class without rewriting hyprland.lua."

    SettingRow {
      label: "Keybindings"
      description: root.overrideCountText()
      hint: "~/.config/hypr/bindings.lua"
      query: root.query
      keywords: ["keybinding", "hotkey", "shortcut", "bind", "unbind", "chord"]

      PrefsButton {
        text: "Configure…"
        enabled: true
        onClicked: root.openSubpage("bindings")
      }
    }

    SettingRow {
      label: "Window rules"
      description: root.ruleCountText()
      hint: "~/.config/hypr/atmos.lua"
      query: root.query
      keywords: ["window", "rule", "float", "tile", "class", "regex"]

      PrefsButton {
        text: "Configure…"
        enabled: true
        onClicked: root.openSubpage("rules")
      }
    }
  }

  PrefsGroup {
    title: "Look"
    query: root.query
    detail: "These write a managed block at the end of ~/.config/hypr/looknfeel.lua. Tight windows overrides the sliders until you turn it off."

    SettingRow {
      stretchControl: true
      label: "Inner gaps"
      description: "Space between windows. Tight windows sets this to zero."
      hint: "~/.config/hypr/looknfeel.lua · general.gaps_in"
      query: root.query
      keywords: ["gap", "padding", "space", "tile"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 32
        stepSize: 1
        value: Omarchy.hyprGapsIn
        valueText: Omarchy.hyprGapsIn + " px"
        enabled: !Omarchy.hyprNoGaps
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprGapsIn)
            Omarchy.setHyprGapsIn(next)
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Outer gaps"
      description: "Space between windows and the edge of the screen."
      hint: "~/.config/hypr/looknfeel.lua · general.gaps_out"
      query: root.query
      keywords: ["gap", "margin", "edge"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 48
        stepSize: 1
        value: Omarchy.hyprGapsOut
        valueText: Omarchy.hyprGapsOut + " px"
        enabled: !Omarchy.hyprNoGaps
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprGapsOut)
            Omarchy.setHyprGapsOut(next)
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Border"
      description: "Thickness of the window outline."
      hint: "~/.config/hypr/looknfeel.lua · general.border_size"
      query: root.query
      keywords: ["border", "outline", "frame"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 8
        stepSize: 1
        value: Omarchy.hyprBorderSize
        valueText: Omarchy.hyprBorderSize + " px"
        enabled: !Omarchy.hyprNoGaps
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprBorderSize)
            Omarchy.setHyprBorderSize(next)
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Corners"
      description: "How round the window corners are."
      hint: "~/.config/hypr/looknfeel.lua · decoration.rounding"
      query: root.query
      keywords: ["rounding", "radius", "corners"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 24
        stepSize: 1
        value: Omarchy.hyprRounding
        valueText: Omarchy.hyprRounding + " px"
        enabled: !Omarchy.hyprNoGaps
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprRounding)
            Omarchy.setHyprRounding(next)
        }
      }
    }

    SettingRow {
      label: "Blur"
      description: "Blur what sits behind a transparent window."
      hint: "~/.config/hypr/looknfeel.lua · decoration.blur"
      query: root.query
      keywords: ["blur", "glass", "transparent"]

      PrefsToggle {
        checked: Omarchy.hyprBlur
        onToggled: Omarchy.setHyprBlur(!Omarchy.hyprBlur)
      }
    }

    SettingRow {
      label: "Shadow"
      description: "A shadow under each window."
      hint: "~/.config/hypr/looknfeel.lua · decoration.shadow"
      query: root.query
      keywords: ["shadow", "drop"]

      PrefsToggle {
        checked: Omarchy.hyprShadow
        onToggled: Omarchy.setHyprShadow(!Omarchy.hyprShadow)
      }
    }

    SettingRow {
      label: "Tiling"
      description: "Dwindle splits the screen in two. Scrolling walks columns sideways, like niri."
      hint: "~/.config/hypr/looknfeel.lua · general.layout"
      query: root.query
      keywords: ["dwindle", "scrolling", "niri", "layout", "tile"]

      PrefsSelect {
        value: Omarchy.hyprLayout
        options: [
          { value: "dwindle", label: "Dwindle" },
          { value: "scrolling", label: "Scrolling" }
        ]
        onChanged: function(value) {
          if (value !== Omarchy.hyprLayout) Omarchy.setHyprLayout(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.hyprLayout === "scrolling"
      stretchControl: true
      label: "Column width"
      description: "How wide each scrolling column is. Near 1 shows one column. Near 0.5 shows two."
      hint: "~/.config/hypr/looknfeel.lua · scrolling.column_width"
      query: root.query
      keywords: ["column", "scrolling", "width"]

      PrefsSlider {
        width: parent.width
        from: 0.3
        to: 1
        stepSize: 0.01
        value: Omarchy.hyprColumnWidth
        valueText: Math.round(Omarchy.hyprColumnWidth * 100) + "%"
        formatTick: function(v) { return Math.round(v * 100) + "%" }
        enabled: Omarchy.hyprLayout === "scrolling"
        onChanged: function(value) {
          var next = Math.round(value * 100) / 100
          if (next !== Omarchy.hyprColumnWidth)
            Omarchy.setHyprColumnWidth(next)
        }
      }
    }

    SettingRow {
      available: Omarchy.hyprLookManaged
      label: "Reset look"
      description: "Remove the block Atmos wrote. Hyprland goes back to the rest of looknfeel.lua and the Omarchy defaults."
      hint: "~/.config/hypr/looknfeel.lua"
      query: root.query
      keywords: ["reset", "default", "looknfeel"]

      PrefsButton {
        text: "Reset"
        danger: true
        enabled: Omarchy.hyprLookManaged
        onClicked: Omarchy.resetHyprLook()
      }
    }
  }

  PrefsGroup {
    title: "Behavior"
    query: root.query
    detail: "Tight windows and square aspect are permanent Hyprland flags. Transparency and tiled fullscreen apply to the window that is focused right now."

    SettingRow {
      label: "Tight windows"
      description: "No gaps, borders, or rounding. The sliders above stay disabled while this is on."
      hint: "omarchy hyprland toggle window-no-gaps"
      query: root.query
      keywords: ["gaps", "borderless", "tight", "no gaps"]

      PrefsToggle {
        checked: Omarchy.hyprNoGaps
        onToggled: Omarchy.setHyprNoGaps(!Omarchy.hyprNoGaps)
      }
    }

    SettingRow {
      label: "Square single window"
      description: "When one window is on the screen, keep it from stretching across an ultrawide."
      hint: "omarchy hyprland toggle single-window-aspect-ratio"
      query: root.query
      keywords: ["aspect", "square", "ultrawide", "single"]

      PrefsToggle {
        checked: Omarchy.hyprSquareAspect
        onToggled: Omarchy.setHyprSquareAspect(!Omarchy.hyprSquareAspect)
      }
    }

    SettingRow {
      label: "This workspace"
      description: Omarchy.hyprWorkspaceLayout === "scrolling"
        ? "This workspace is scrolling. Switch layout turns it to dwindle."
        : "This workspace is dwindle. Switch layout turns it to scrolling."
      hint: "omarchy hyprland workspace layout toggle"
      query: root.query
      keywords: ["workspace", "layout", "dwindle", "scrolling"]

      PrefsButton {
        text: "Switch layout"
        onClicked: Omarchy.toggleWorkspaceLayout()
      }
    }

    SettingRow {
      label: "This window"
      description: "Switch transparency or tiled fullscreen on the focused window. That change lasts for this window only."
      hint: "omarchy hyprland window transparency toggle"
      query: root.query
      keywords: ["opacity", "transparent", "fullscreen", "tiled"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Switch transparency"
          onClicked: Omarchy.toggleWindowTransparency()
        }
        PrefsButton {
          text: "Switch tiled full"
          onClicked: Omarchy.toggleTiledFullscreen()
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Dim, animations, cursor, and tearing. These go in the same looknfeel.lua block as the sliders above."

    SettingRow {
      label: "Dim others"
      description: "Darken windows that are not focused."
      hint: "~/.config/hypr/looknfeel.lua · decoration.dim_inactive"
      query: root.query
      keywords: ["dim", "inactive", "focus"]

      PrefsToggle {
        checked: Omarchy.hyprDimInactive
        onToggled: Omarchy.setHyprDimInactive(!Omarchy.hyprDimInactive)
      }
    }

    SettingRow {
      available: Omarchy.hyprDimInactive
      stretchControl: true
      label: "Dim strength"
      description: "How far unfocused windows go toward black."
      hint: "~/.config/hypr/looknfeel.lua · decoration.dim_strength"
      query: root.query
      keywords: ["dim", "strength", "inactive"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 1
        stepSize: 0.05
        value: Omarchy.hyprDimStrength
        valueText: Math.round(Omarchy.hyprDimStrength * 100) + "%"
        formatTick: function(v) { return Math.round(v * 100) + "%" }
        enabled: Omarchy.hyprDimInactive
        onChanged: function(value) {
          var next = Math.round(value * 100) / 100
          if (next !== Omarchy.hyprDimStrength)
            Omarchy.setHyprDimStrength(next)
        }
      }
    }

    AnimationsRow { query: root.query }

    HideCursorRow { query: root.query }

    SettingRow {
      label: "Warp cursor on workspace"
      description: "The pointer jumps when you change workspaces."
      hint: "~/.config/hypr/looknfeel.lua · cursor.warp_on_change_workspace"
      query: root.query
      keywords: ["cursor", "warp", "workspace"]

      PrefsToggle {
        checked: Omarchy.hyprCursorWarp
        onToggled: Omarchy.setHyprCursorWarp(!Omarchy.hyprCursorWarp)
      }
    }

    SettingRow {
      label: "Resize on border"
      description: "Drag a window edge to resize it, without the modifier key."
      hint: "~/.config/hypr/looknfeel.lua · general.resize_on_border"
      query: root.query
      keywords: ["resize", "border", "drag"]

      PrefsToggle {
        checked: Omarchy.hyprResizeOnBorder
        onToggled: Omarchy.setHyprResizeOnBorder(!Omarchy.hyprResizeOnBorder)
      }
    }

    SettingRow {
      label: "Allow tearing"
      description: "Games and other windows may tear if they ask. That can cut input lag."
      hint: "~/.config/hypr/looknfeel.lua · general.allow_tearing"
      query: root.query
      keywords: ["tearing", "vrr", "latency", "game"]

      PrefsToggle {
        checked: Omarchy.hyprAllowTearing
        onToggled: Omarchy.setHyprAllowTearing(!Omarchy.hyprAllowTearing)
      }
    }

    CursorSizeRow { query: root.query }

    SettingRow {
      stretchControl: true
      label: "Active opacity"
      description: "How solid a focused window is."
      hint: "~/.config/hypr/looknfeel.lua · decoration.active_opacity"
      query: root.query
      keywords: ["opacity", "transparency", "alpha"]

      PrefsSlider {
        width: parent.width
        from: 0.2
        to: 1
        stepSize: 0.05
        value: Omarchy.hyprActiveOpacity
        valueText: Math.round(Omarchy.hyprActiveOpacity * 100) + "%"
        formatTick: function(v) { return Math.round(v * 100) + "%" }
        onChanged: function(value) {
          var next = Math.round(value * 100) / 100
          if (next !== Omarchy.hyprActiveOpacity)
            Omarchy.setHyprActiveOpacity(next)
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Inactive opacity"
      description: "How solid unfocused windows are."
      hint: "~/.config/hypr/looknfeel.lua · decoration.inactive_opacity"
      query: root.query
      keywords: ["opacity", "transparency", "alpha", "inactive"]

      PrefsSlider {
        width: parent.width
        from: 0.2
        to: 1
        stepSize: 0.05
        value: Omarchy.hyprInactiveOpacity
        valueText: Math.round(Omarchy.hyprInactiveOpacity * 100) + "%"
        formatTick: function(v) { return Math.round(v * 100) + "%" }
        onChanged: function(value) {
          var next = Math.round(value * 100) / 100
          if (next !== Omarchy.hyprInactiveOpacity)
            Omarchy.setHyprInactiveOpacity(next)
        }
      }
    }

    SettingRow {
      label: "Preserve split"
      description: "Keep the dwindle split after the last window in a branch closes."
      hint: "~/.config/hypr/looknfeel.lua · dwindle.preserve_split"
      query: root.query
      keywords: ["dwindle", "split", "tile"]

      PrefsToggle {
        checked: Omarchy.hyprPreserveSplit
        onToggled: Omarchy.setHyprPreserveSplit(!Omarchy.hyprPreserveSplit)
      }
    }

    SettingRow {
      label: "Focus on activate"
      description: "Focus a window when another client asks Hyprland to activate it."
      hint: "~/.config/hypr/looknfeel.lua · misc.focus_on_activate"
      query: root.query
      keywords: ["focus", "activate", "urgent"]

      PrefsToggle {
        checked: Omarchy.hyprFocusOnActivate
        onToggled: Omarchy.setHyprFocusOnActivate(!Omarchy.hyprFocusOnActivate)
      }
    }
  }
}
