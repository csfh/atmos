import QtQuick
import "../components"
import "../services"
import "../services/HyprPrefs.js" as HyprPrefs

PrefsPage {
  id: root
  hubId: "input"
  title: "Input"
  description: "How the mouse, touchpad, and keyboard feel. Turning the laptop trackpad off is on Displays. The system layout picker is on System."

  property string kbLayoutDraft: Omarchy.hyprInput.kbLayoutOverride || ""
  property string kbVariantDraft: Omarchy.hyprInput.kbVariantOverride || ""
  readonly property string kbLayoutParsed: HyprPrefs.sanitizeLayoutList(root.kbLayoutDraft)
  readonly property bool kbLayoutValid: String(root.kbLayoutDraft || "").replace(/^\s+|\s+$/g, "").length === 0 || root.kbLayoutParsed.length > 0
  readonly property string kbVariantParsed: root.kbLayoutParsed
    ? HyprPrefs.sanitizeVariantList(root.kbVariantDraft, root.kbLayoutParsed.split(",").length)
    : ""
  readonly property bool kbVariantValid: String(root.kbVariantDraft || "").replace(/^\s+|\s+$/g, "").length === 0 || root.kbVariantParsed.length > 0
  readonly property bool kbOverrideValid: root.kbLayoutValid && root.kbVariantValid
  readonly property bool kbOverrideDirty: root.kbLayoutParsed !== (Omarchy.hyprInput.kbLayoutOverride || "") || root.kbVariantParsed !== (Omarchy.hyprInput.kbVariantOverride || "")

  function applyKbOverride() {
    if (!root.kbOverrideValid) return
    Omarchy.setHyprKbOverride(root.kbLayoutParsed, root.kbVariantParsed, Omarchy.hyprInput.kbGroupToggle === true)
  }

  Connections {
    target: Omarchy
    function onHyprInputChanged() {
      root.kbLayoutDraft = Omarchy.hyprInput.kbLayoutOverride || ""
      root.kbVariantDraft = Omarchy.hyprInput.kbVariantOverride || ""
    }
  }

  PrefsGroup {
    title: "Pointer"
    query: root.query
    writesFile: "~/.config/hypr/input.lua"
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
        value: Omarchy.hyprInput.sensitivity
        valueText: Omarchy.hyprInput.sensitivity.toFixed(2)
        onChanged: function(value) {
          var next = Math.round(value * 100) / 100
          if (next !== Omarchy.hyprInput.sensitivity)
            Omarchy.writeHyprInput({ sensitivity: next })
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
        value: Omarchy.hyprInput.accelProfile === "flat" ? "flat" : (Omarchy.hyprInput.accelProfile === "adaptive" ? "adaptive" : "")
        options: [
          { value: "", label: "Default" },
          { value: "adaptive", label: "Adaptive" },
          { value: "flat", label: "Flat" }
        ]
        onChanged: function(value) {
          if (value !== Omarchy.hyprInput.accelProfile)
            Omarchy.writeHyprInput({ accelProfile: value })
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
        value: String(Omarchy.hyprInput.emulateDiscreteScroll)
        options: [
          { value: "0", label: "Smooth" },
          { value: "1", label: "Default" },
          { value: "2", label: "Stepped" }
        ]
        onChanged: function(value) {
          var next = Math.round(Number(value))
          if (next !== Omarchy.hyprInput.emulateDiscreteScroll)
            Omarchy.writeHyprInput({ emulateDiscreteScroll: next })
        }
      }
    }
  }

  PrefsGroup {
    title: "Touchpad"
    query: root.query
    writesFile: "~/.config/hypr/input.lua"
    detail: "Feel for the trackpad. The on/off switch for the device itself is on Displays."

    SettingRow {
      label: "Natural scroll"
      description: "Content moves with your fingers, the way a phone does."
      hint: "~/.config/hypr/input.lua · input.touchpad.natural_scroll"
      query: root.query
      keywords: ["natural", "invert", "scroll", "direction"]

      PrefsToggle {
        checked: Omarchy.hyprInput.naturalScroll
        onToggled: Omarchy.writeHyprInput({ naturalScroll: !Omarchy.hyprInput.naturalScroll })
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
        value: Omarchy.hyprInput.scrollFactor
        valueText: Omarchy.hyprInput.scrollFactor.toFixed(1)
        onChanged: function(value) {
          var next = Math.round(value * 10) / 10
          if (next !== Omarchy.hyprInput.scrollFactor)
            Omarchy.writeHyprInput({ scrollFactor: next })
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
        checked: Omarchy.hyprInput.clickfinger
        onToggled: Omarchy.writeHyprInput({ clickfinger: !Omarchy.hyprInput.clickfinger })
      }
    }

    SettingRow {
      label: "Ignore while typing"
      description: "The trackpad rests while you type, so a palm does not move the pointer."
      hint: "~/.config/hypr/input.lua · input.touchpad.disable_while_typing"
      query: root.query
      keywords: ["disable while typing", "palm", "reject"]

      PrefsToggle {
        checked: Omarchy.hyprInput.disableWhileTyping
        onToggled: Omarchy.writeHyprInput({ disableWhileTyping: !Omarchy.hyprInput.disableWhileTyping })
      }
    }

    SettingRow {
      label: "Three-finger drag"
      description: "Three fingers down and moving drags, like a click-and-hold."
      hint: "~/.config/hypr/input.lua · input.touchpad.drag_3fg"
      query: root.query
      keywords: ["three finger", "drag"]

      PrefsToggle {
        checked: Omarchy.hyprInput.drag3fg === 1
        onToggled: Omarchy.writeHyprInput({ drag3fg: Omarchy.hyprInput.drag3fg === 1 ? 0 : 1 })
      }
    }
  }

  PrefsGroup {
    title: "Keyboard"
    query: root.query
    writesFile: "~/.config/hypr/input.lua"
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
        value: Omarchy.hyprInput.repeatRate
        valueText: Omarchy.hyprInput.repeatRate + "/s"
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprInput.repeatRate)
            Omarchy.writeHyprInput({ repeatRate: next })
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
        value: Omarchy.hyprInput.repeatDelay
        valueText: Omarchy.hyprInput.repeatDelay + " ms"
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprInput.repeatDelay)
            Omarchy.writeHyprInput({ repeatDelay: next })
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
        checked: Omarchy.hyprInput.numlock
        onToggled: Omarchy.writeHyprInput({ numlock: !Omarchy.hyprInput.numlock })
      }
    }

    SettingRow {
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
    writesFile: "~/.config/hypr/input.lua"
    writesNote: Omarchy.hyprWorkspaceGestureUnmanaged ? "your gesture stays" : ""
    detail: "Follow mouse, waking the screen, an optional Hyprland layout list, and a three-finger workspace swipe."

    SettingRow {
      label: "Follow mouse"
      description: "How the pointer picks the focused window. 1 is the usual Omarchy setting."
      hint: "~/.config/hypr/input.lua · input.follow_mouse"
      query: root.query
      keywords: ["follow", "focus", "mouse"]

      PrefsSelect {
        value: String(Omarchy.hyprInput.followMouse)
        options: [
          { value: "0", label: "Click to focus" },
          { value: "1", label: "Follow" },
          { value: "2", label: "Follow, detached" },
          { value: "3", label: "Follow, loose" }
        ]
        onChanged: function(value) {
          var next = Math.round(Number(value))
          if (next !== Omarchy.hyprInput.followMouse)
            Omarchy.writeHyprInput({ followMouse: next })
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
        checked: Omarchy.hyprInput.keyPressDpms
        onToggled: Omarchy.writeHyprInput({ keyPressDpms: !Omarchy.hyprInput.keyPressDpms })
      }
    }

    SettingRow {
      label: "Wake on mouse"
      description: "Moving the pointer turns the screen back on after DPMS off."
      hint: "~/.config/hypr/input.lua · misc.mouse_move_enables_dpms"
      query: root.query
      keywords: ["dpms", "wake", "mouse"]

      PrefsToggle {
        checked: Omarchy.hyprInput.mouseMoveDpms
        onToggled: Omarchy.writeHyprInput({ mouseMoveDpms: !Omarchy.hyprInput.mouseMoveDpms })
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
        checked: Omarchy.hyprInput.kbGroupToggle
        onToggled: {
          if (root.kbOverrideValid)
            Omarchy.setHyprKbOverride(root.kbLayoutParsed, root.kbVariantParsed, Omarchy.hyprInput.kbGroupToggle !== true)
          else
            Omarchy.setHyprKbOverride(Omarchy.hyprInput.kbLayoutOverride, Omarchy.hyprInput.kbVariantOverride, Omarchy.hyprInput.kbGroupToggle !== true)
        }
      }
    }

    SettingRow {
      label: "Three-finger swipe"
      description: Omarchy.hyprWorkspaceGestureUnmanaged
        ? "This gesture is already in ~/.config/hypr/input.lua outside the Atmos block, so the switch stays on and Atmos will not change it. Comment that line out to let Atmos own it."
        : "Swipe sideways with three fingers to change workspace."
      hint: "~/.config/hypr/input.lua · hl.gesture"
      query: root.query
      keywords: ["gesture", "swipe", "workspace", "three"]

      PrefsToggle {
        checked: Omarchy.hyprWorkspaceGesture
        enabled: !Omarchy.hyprWorkspaceGestureUnmanaged
        onToggled: Omarchy.setHyprWorkspaceGesture(!Omarchy.hyprWorkspaceGesture)
      }
    }
  }
}
