import QtQuick
import "../components"
import "../services"
import "windows" as Win

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
    if (n === 1) return "One Atmos override in bindings.lua. Open to add or unbind."
    if (n > 1) return n + " Atmos overrides in bindings.lua. Open to add or unbind."
    var total = Omarchy.keybindings.length
    if (total === 1) return "One chord Hyprland is running. Open to add an override."
    if (total > 1) return total + " chords Hyprland is running. Open to add an override."
    return "No chords listed yet. Open to add one."
  }

  function ruleCountText() {
    var n = 0
    var list = Omarchy.windowRules || []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].managed === true) n++
    }
    if (n === 1) return "One Atmos rule. Open to float, tile, or pin a class."
    if (n > 1) return n + " Atmos rules. Open to float, tile, or pin a class."
    return "No Atmos rules yet. Open to float, tile, or pin a class."
  }

  Component { id: bindingsPage; Win.BindingsPage {} }
  Component { id: rulesPage; Win.RulesPage {} }

  PrefsGroup {
    title: "Shortcuts and rules"
    query: root.query
    detail: "Keybindings lists what Hyprland is running and lets you add a personal override. Window rules float, tile, or pin a class without rewriting hyprland.lua."

    PrefsRow {
      label: "Keybindings"
      description: root.overrideCountText()
      hint: "~/.config/hypr/bindings.lua"
      query: root.query
      keywords: ["keybinding", "hotkey", "shortcut", "bind", "unbind", "chord"]

      PrefsButton {
        text: "Open"
        enabled: true
        onClicked: root.openSubpage("bindings")
      }
    }

    PrefsRow {
      label: "Window rules"
      description: root.ruleCountText()
      hint: "~/.config/hypr/atmos.lua"
      query: root.query
      keywords: ["window", "rule", "float", "tile", "class", "regex"]

      PrefsButton {
        text: "Open"
        enabled: true
        onClicked: root.openSubpage("rules")
      }
    }
  }

  PrefsGroup {
    title: "Look"
    query: root.query
    detail: "These write a managed block at the end of ~/.config/hypr/looknfeel.lua. Tight windows overrides the sliders until you turn it off."

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
      label: "Shadow"
      description: "Drop a shadow under each window."
      hint: "~/.config/hypr/looknfeel.lua · decoration.shadow"
      query: root.query
      keywords: ["shadow", "drop"]

      PrefsToggle {
        checked: Omarchy.hyprShadow
        onToggled: Omarchy.setHyprShadow(!Omarchy.hyprShadow)
      }
    }

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
      label: "Tight windows"
      description: "Drop gaps, borders, and rounding. The sliders above stay disabled while this is on."
      hint: "omarchy hyprland toggle window-no-gaps"
      query: root.query
      keywords: ["gaps", "borderless", "tight", "no gaps"]

      PrefsToggle {
        checked: Omarchy.hyprNoGaps
        onToggled: Omarchy.setHyprNoGaps(!Omarchy.hyprNoGaps)
      }
    }

    PrefsRow {
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

    PrefsRow {
      label: "This workspace"
      description: Omarchy.hyprWorkspaceLayout === "scrolling"
        ? "This workspace is scrolling. Toggle switches it to dwindle."
        : "This workspace is dwindle. Toggle switches it to scrolling."
      hint: "omarchy hyprland workspace layout toggle"
      query: root.query
      keywords: ["workspace", "layout", "dwindle", "scrolling"]

      PrefsButton {
        text: "Toggle"
        onClicked: Omarchy.toggleWorkspaceLayout()
      }
    }

    PrefsRow {
      label: "This window"
      description: "Flip transparency or tiled fullscreen on the focused window. That change lasts for this window only."
      hint: "omarchy hyprland window transparency toggle"
      query: root.query
      keywords: ["opacity", "transparent", "fullscreen", "tiled"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Transparency"
          onClicked: Omarchy.toggleWindowTransparency()
        }
        PrefsButton {
          text: "Tiled full"
          onClicked: Omarchy.toggleTiledFullscreen()
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Dim, animations, cursor, and tearing. These go in the same looknfeel.lua block as the sliders above."

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
      label: "Animations"
      description: "Window open, close, and fade motion."
      hint: "~/.config/hypr/looknfeel.lua · animations.enabled"
      query: root.query
      keywords: ["animation", "motion", "reduce"]

      PrefsToggle {
        checked: Omarchy.hyprAnimations
        onToggled: Omarchy.setHyprAnimations(!Omarchy.hyprAnimations)
      }
    }

    PrefsRow {
      label: "Hide cursor while typing"
      description: "The pointer disappears when you start typing."
      hint: "~/.config/hypr/looknfeel.lua · cursor.hide_on_key_press"
      query: root.query
      keywords: ["cursor", "pointer", "hide", "type"]

      PrefsToggle {
        checked: Omarchy.hyprCursorHideOnKey
        onToggled: Omarchy.setHyprCursorHideOnKey(!Omarchy.hyprCursorHideOnKey)
      }
    }

    PrefsRow {
      label: "Warp cursor on workspace"
      description: "Jump the pointer when you change workspaces."
      hint: "~/.config/hypr/looknfeel.lua · cursor.warp_on_change_workspace"
      query: root.query
      keywords: ["cursor", "warp", "workspace"]

      PrefsToggle {
        checked: Omarchy.hyprCursorWarp
        onToggled: Omarchy.setHyprCursorWarp(!Omarchy.hyprCursorWarp)
      }
    }

    PrefsRow {
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

    PrefsRow {
      label: "Allow tearing"
      description: "Let a game or other window tear if it asks. That can cut input lag."
      hint: "~/.config/hypr/looknfeel.lua · general.allow_tearing"
      query: root.query
      keywords: ["tearing", "vrr", "latency", "game"]

      PrefsToggle {
        checked: Omarchy.hyprAllowTearing
        onToggled: Omarchy.setHyprAllowTearing(!Omarchy.hyprAllowTearing)
      }
    }

    PrefsRow {
      stretchControl: true
      label: "Cursor size"
      description: "How large the pointer is. Accessibility has the same slider."
      hint: "~/.config/hypr/looknfeel.lua · HYPRCURSOR_SIZE"
      query: root.query
      keywords: ["cursor", "pointer", "size"]

      PrefsSlider {
        width: parent.width
        from: 8
        to: 64
        stepSize: 2
        value: Omarchy.hyprCursorSize
        valueText: Omarchy.hyprCursorSize + " px"
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprCursorSize)
            Omarchy.setHyprCursorSize(next)
        }
      }
    }

    PrefsRow {
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

    PrefsRow {
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

    PrefsRow {
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
