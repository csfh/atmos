import QtQuick
import "../components"
import "../services"
import "../services/HyprPrefs.js" as HyprPrefs

PrefsPage {
  id: root
  title: "Input"
  description: "How the mouse, touchpad, and keyboard feel. Turning the laptop trackpad off is on Displays. The system layout picker is on System."

  property string kbLayoutDraft: Omarchy.hyprKbLayout
  property string kbVariantDraft: Omarchy.hyprKbVariant
  readonly property string kbLayoutParsed: HyprPrefs.sanitizeLayoutList(root.kbLayoutDraft)
  readonly property bool kbLayoutValid: String(root.kbLayoutDraft || "").replace(/^\s+|\s+$/g, "").length === 0 || root.kbLayoutParsed.length > 0
  readonly property string kbVariantParsed: root.kbLayoutParsed
    ? HyprPrefs.sanitizeVariantList(root.kbVariantDraft, root.kbLayoutParsed.split(",").length)
    : ""
  readonly property bool kbVariantValid: String(root.kbVariantDraft || "").replace(/^\s+|\s+$/g, "").length === 0 || root.kbVariantParsed.length > 0
  readonly property bool kbOverrideValid: root.kbLayoutValid && root.kbVariantValid
  readonly property bool kbOverrideDirty: root.kbLayoutParsed !== Omarchy.hyprKbLayout || root.kbVariantParsed !== Omarchy.hyprKbVariant

  function applyKbOverride() {
    if (!root.kbOverrideValid) return
    Omarchy.setHyprKbOverride(root.kbLayoutParsed, root.kbVariantParsed, Omarchy.hyprKbGroupToggle)
  }

  Connections {
    target: Omarchy
    function onHyprKbLayoutChanged() { root.kbLayoutDraft = Omarchy.hyprKbLayout }
    function onHyprKbVariantChanged() { root.kbVariantDraft = Omarchy.hyprKbVariant }
  }

  PrefsGroup {
    title: "Pointer"
    query: root.query
    detail: "Sensitivity and acceleration for the mouse and trackpad. These write a managed block in ~/.config/hypr/input.lua."

    SettingRow {
      stretchControl: true
      label: "Sensitivity"
      description: "Pointer speed. Zero is the Hyprland default. Negative is slower."
      hint: "~/.config/hypr/input.lua · input.sensitivity"
      query: root.query
      keywords: ["mouse", "pointer", "speed", "sensitivity"]

      PrefsSlider {
        width: parent.width
        from: -1
        to: 1
        stepSize: 0.02
        value: Omarchy.hyprSensitivity
        valueText: Omarchy.hyprSensitivity.toFixed(2)
        onChanged: function(value) {
          var next = Math.round(value * 100) / 100
          if (next !== Omarchy.hyprSensitivity)
            Omarchy.setHyprSensitivity(next)
        }
      }
    }

    SettingRow {
      label: "Acceleration"
      description: "Adaptive speeds up as you move. Flat keeps a steady ratio."
      hint: "~/.config/hypr/input.lua · input.accel_profile"
      query: root.query
      keywords: ["accel", "acceleration", "flat", "adaptive"]

      PrefsSelect {
        value: Omarchy.hyprAccelProfile === "flat" ? "flat" : (Omarchy.hyprAccelProfile === "adaptive" ? "adaptive" : "")
        options: [
          { value: "", label: "Default" },
          { value: "adaptive", label: "Adaptive" },
          { value: "flat", label: "Flat" }
        ]
        onChanged: function(value) {
          if (value !== Omarchy.hyprAccelProfile)
            Omarchy.setHyprAccelProfile(value)
        }
      }
    }

    SettingRow {
      label: "Scroll inertia"
      description: "How a high-resolution or free-spin mouse wheel is turned into scroll events. Smooth keeps the fine motion. Stepped turns it into clicks."
      hint: "~/.config/hypr/input.lua · input.emulate_discrete_scroll"
      query: root.query
      keywords: ["inertia", "wheel", "high-res", "discrete", "smooth", "scroll"]

      PrefsSelect {
        value: String(Omarchy.hyprEmulateDiscreteScroll)
        options: [
          { value: "0", label: "Smooth" },
          { value: "1", label: "Default" },
          { value: "2", label: "Stepped" }
        ]
        onChanged: function(value) {
          var next = Math.round(Number(value))
          if (next !== Omarchy.hyprEmulateDiscreteScroll)
            Omarchy.setHyprEmulateDiscreteScroll(next)
        }
      }
    }
  }

  PrefsGroup {
    title: "Touchpad"
    query: root.query
    detail: "Feel for the trackpad. The on/off switch for the device itself is on Displays."

    SettingRow {
      label: "Natural scroll"
      description: "Content moves with your fingers, the way a phone does."
      hint: "~/.config/hypr/input.lua · input.touchpad.natural_scroll"
      query: root.query
      keywords: ["natural", "invert", "scroll", "direction"]

      PrefsToggle {
        checked: Omarchy.hyprNaturalScroll
        onToggled: Omarchy.setHyprNaturalScroll(!Omarchy.hyprNaturalScroll)
      }
    }

    SettingRow {
      stretchControl: true
      label: "Scroll speed"
      description: "How far two-finger scroll moves."
      hint: "~/.config/hypr/input.lua · input.touchpad.scroll_factor"
      query: root.query
      keywords: ["scroll", "factor", "speed"]

      PrefsSlider {
        width: parent.width
        from: 0.1
        to: 2
        stepSize: 0.1
        value: Omarchy.hyprScrollFactor
        valueText: Omarchy.hyprScrollFactor.toFixed(1)
        onChanged: function(value) {
          var next = Math.round(value * 10) / 10
          if (next !== Omarchy.hyprScrollFactor)
            Omarchy.setHyprScrollFactor(next)
        }
      }
    }

    SettingRow {
      label: "Two-finger click"
      description: "A two-finger tap is a right click."
      hint: "~/.config/hypr/input.lua · input.touchpad.clickfinger_behavior"
      query: root.query
      keywords: ["clickfinger", "right click", "tap"]

      PrefsToggle {
        checked: Omarchy.hyprClickfinger
        onToggled: Omarchy.setHyprClickfinger(!Omarchy.hyprClickfinger)
      }
    }

    SettingRow {
      label: "Ignore while typing"
      description: "The trackpad rests while you type, so a palm does not move the pointer."
      hint: "~/.config/hypr/input.lua · input.touchpad.disable_while_typing"
      query: root.query
      keywords: ["disable while typing", "palm", "reject"]

      PrefsToggle {
        checked: Omarchy.hyprDisableWhileTyping
        onToggled: Omarchy.setHyprDisableWhileTyping(!Omarchy.hyprDisableWhileTyping)
      }
    }

    SettingRow {
      label: "Three-finger drag"
      description: "Three fingers down and moving drags, like a click-and-hold."
      hint: "~/.config/hypr/input.lua · input.touchpad.drag_3fg"
      query: root.query
      keywords: ["three finger", "drag"]

      PrefsToggle {
        checked: Omarchy.hyprDrag3fg === 1
        onToggled: Omarchy.setHyprDrag3fg(!(Omarchy.hyprDrag3fg === 1))
      }
    }
  }

  PrefsGroup {
    title: "Keyboard"
    query: root.query
    detail: "Repeat and numlock for Hyprland. The console and login layout stay on System."

    SettingRow {
      stretchControl: true
      label: "Repeat rate"
      description: "How many times a held key repeats each second."
      hint: "~/.config/hypr/input.lua · input.repeat_rate"
      query: root.query
      keywords: ["repeat", "rate", "hold"]

      PrefsSlider {
        width: parent.width
        from: 10
        to: 80
        stepSize: 1
        value: Omarchy.hyprRepeatRate
        valueText: Omarchy.hyprRepeatRate + "/s"
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprRepeatRate)
            Omarchy.setHyprRepeatRate(next)
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Repeat delay"
      description: "How long you hold a key before it starts repeating."
      hint: "~/.config/hypr/input.lua · input.repeat_delay"
      query: root.query
      keywords: ["repeat", "delay", "hold"]

      PrefsSlider {
        width: parent.width
        from: 150
        to: 600
        stepSize: 10
        value: Omarchy.hyprRepeatDelay
        valueText: Omarchy.hyprRepeatDelay + " ms"
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprRepeatDelay)
            Omarchy.setHyprRepeatDelay(next)
        }
      }
    }

    SettingRow {
      label: "Numlock on boot"
      description: "The number pad is on when Hyprland starts."
      hint: "~/.config/hypr/input.lua · input.numlock_by_default"
      query: root.query
      keywords: ["numlock", "keypad"]

      PrefsToggle {
        checked: Omarchy.hyprNumlock
        onToggled: Omarchy.setHyprNumlock(!Omarchy.hyprNumlock)
      }
    }

    SettingRow {
      available: Omarchy.hyprInputManaged
      label: "Reset input"
      description: "Remove the block Atmos wrote. Hyprland goes back to the rest of input.lua and the Omarchy defaults."
      hint: "~/.config/hypr/input.lua"
      query: root.query
      keywords: ["reset", "default", "input"]

      PrefsButton {
        text: "Reset"
        danger: true
        enabled: Omarchy.hyprInputManaged
        onClicked: Omarchy.resetHyprInput()
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Follow mouse, waking the screen, an optional Hyprland layout list, and a three-finger workspace swipe."

    SettingRow {
      label: "Follow mouse"
      description: "How the pointer picks the focused window. 1 is the usual Omarchy setting."
      hint: "~/.config/hypr/input.lua · input.follow_mouse"
      query: root.query
      keywords: ["follow", "focus", "mouse"]

      PrefsSelect {
        value: String(Omarchy.hyprFollowMouse)
        options: [
          { value: "0", label: "Click to focus" },
          { value: "1", label: "Follow" },
          { value: "2", label: "Follow, detached" },
          { value: "3", label: "Follow, loose" }
        ]
        onChanged: function(value) {
          var next = Math.round(Number(value))
          if (next !== Omarchy.hyprFollowMouse)
            Omarchy.setHyprFollowMouse(next)
        }
      }
    }

    SettingRow {
      label: "Wake on key"
      description: "A key press turns the screen back on after DPMS off."
      hint: "~/.config/hypr/input.lua · misc.key_press_enables_dpms"
      query: root.query
      keywords: ["dpms", "wake", "key"]

      PrefsToggle {
        checked: Omarchy.hyprKeyPressDpms
        onToggled: Omarchy.setHyprKeyPressDpms(!Omarchy.hyprKeyPressDpms)
      }
    }

    SettingRow {
      label: "Wake on mouse"
      description: "Moving the pointer turns the screen back on after DPMS off."
      hint: "~/.config/hypr/input.lua · misc.mouse_move_enables_dpms"
      query: root.query
      keywords: ["dpms", "wake", "mouse"]

      PrefsToggle {
        checked: Omarchy.hyprMouseMoveDpms
        onToggled: Omarchy.setHyprMouseMoveDpms(!Omarchy.hyprMouseMoveDpms)
      }
    }

    SettingRow {
      stretchControl: true
      label: "Hyprland layouts"
      description: root.kbLayoutValid
        ? "Optional. A comma list such as us,dk for Hyprland only. Leave it blank to keep the System layout."
        : "Layout ids are 1–8 letters or digits, separated by commas. Spaces around commas are fine. Set stays off until the list is valid."
      hint: "~/.config/hypr/input.lua · input.kb_layout"
      query: root.query
      keywords: ["layout", "xkb", "multi", "us,dk"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: layoutField
          width: parent.width - layoutSetBtn.width - parent.spacing
          value: root.kbLayoutDraft
          placeholder: "us,dk"
          invalid: !root.kbLayoutValid
          onEdited: function(value) { root.kbLayoutDraft = value }
          onSubmitted: function(value) {
            root.kbLayoutDraft = value
            root.applyKbOverride()
          }
        }

        PrefsButton {
          id: layoutSetBtn
          text: "Set"
          primary: true
          enabled: root.kbOverrideValid && root.kbOverrideDirty
          onClicked: root.applyKbOverride()
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Variants"
      description: root.kbVariantValid
        ? "Optional variants matching the layout list, such as ,nodeadkeys."
        : "One variant per layout, same comma count. Letters, digits, underscore, or hyphen. Set stays off until the list matches."
      hint: "~/.config/hypr/input.lua · input.kb_variant"
      query: root.query
      keywords: ["variant", "intl", "nodeadkeys"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: variantField
          width: parent.width - variantSetBtn.width - parent.spacing
          value: root.kbVariantDraft
          placeholder: ",nodeadkeys"
          invalid: !root.kbVariantValid
          onEdited: function(value) { root.kbVariantDraft = value }
          onSubmitted: function(value) {
            root.kbVariantDraft = value
            root.applyKbOverride()
          }
        }

        PrefsButton {
          id: variantSetBtn
          text: "Set"
          primary: true
          enabled: root.kbOverrideValid && root.kbOverrideDirty
          onClicked: root.applyKbOverride()
        }
      }
    }

    SettingRow {
      label: "Alt+Alt layout switch"
      description: "Left Alt and Right Alt together cycle the Hyprland layouts above."
      hint: "~/.config/hypr/input.lua · input.kb_options"
      query: root.query
      keywords: ["grp", "alts", "switch", "layout"]

      PrefsToggle {
        checked: Omarchy.hyprKbGroupToggle
        onToggled: {
          if (root.kbOverrideValid)
            Omarchy.setHyprKbOverride(root.kbLayoutParsed, root.kbVariantParsed, !Omarchy.hyprKbGroupToggle)
          else
            Omarchy.setHyprKbOverride(Omarchy.hyprKbLayout, Omarchy.hyprKbVariant, !Omarchy.hyprKbGroupToggle)
        }
      }
    }

    SettingRow {
      label: "Three-finger swipe"
      description: "Swipe sideways with three fingers to change workspace."
      hint: "~/.config/hypr/input.lua · hl.gesture"
      query: root.query
      keywords: ["gesture", "swipe", "workspace", "three"]

      PrefsToggle {
        checked: Omarchy.hyprWorkspaceGesture
        onToggled: Omarchy.setHyprWorkspaceGesture(!Omarchy.hyprWorkspaceGesture)
      }
    }
  }
}
