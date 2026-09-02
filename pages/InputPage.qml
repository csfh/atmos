import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Input"
  description: "How the mouse, touchpad, and keyboard feel. Turning the laptop trackpad off is on Displays. The system layout picker is on System."

  PrefsGroup {
    title: "Pointer"
    query: root.query
    detail: "Sensitivity and acceleration for the mouse and trackpad. These write a managed block in ~/.config/hypr/input.lua."

    PrefsRow {
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
        enabled: !Omarchy.busy
        onChanged: function(value) {
          var next = Math.round(value * 100) / 100
          if (next !== Omarchy.hyprSensitivity)
            Omarchy.setHyprSensitivity(next)
        }
      }
    }

    PrefsRow {
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
        enabled: !Omarchy.busy
        onChanged: function(value) {
          if (value !== Omarchy.hyprAccelProfile)
            Omarchy.setHyprAccelProfile(value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Touchpad"
    query: root.query
    detail: "Feel for the trackpad. The on/off switch for the device itself is on Displays."

    PrefsRow {
      label: "Natural scroll"
      description: "Content moves with your fingers, the way a phone does."
      hint: "~/.config/hypr/input.lua · input.touchpad.natural_scroll"
      query: root.query
      keywords: ["natural", "invert", "scroll", "direction"]

      PrefsToggle {
        checked: Omarchy.hyprNaturalScroll
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprNaturalScroll(!Omarchy.hyprNaturalScroll)
      }
    }

    PrefsRow {
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
        enabled: !Omarchy.busy
        onChanged: function(value) {
          var next = Math.round(value * 10) / 10
          if (next !== Omarchy.hyprScrollFactor)
            Omarchy.setHyprScrollFactor(next)
        }
      }
    }

    PrefsRow {
      label: "Two-finger click"
      description: "A two-finger tap is a right click."
      hint: "~/.config/hypr/input.lua · input.touchpad.clickfinger_behavior"
      query: root.query
      keywords: ["clickfinger", "right click", "tap"]

      PrefsToggle {
        checked: Omarchy.hyprClickfinger
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprClickfinger(!Omarchy.hyprClickfinger)
      }
    }

    PrefsRow {
      label: "Ignore while typing"
      description: "The trackpad rests while you type, so a palm does not move the pointer."
      hint: "~/.config/hypr/input.lua · input.touchpad.disable_while_typing"
      query: root.query
      keywords: ["disable while typing", "palm", "reject"]

      PrefsToggle {
        checked: Omarchy.hyprDisableWhileTyping
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprDisableWhileTyping(!Omarchy.hyprDisableWhileTyping)
      }
    }

    PrefsRow {
      label: "Three-finger drag"
      description: "Three fingers down and moving drags, like a click-and-hold."
      hint: "~/.config/hypr/input.lua · input.touchpad.drag_3fg"
      query: root.query
      keywords: ["three finger", "drag"]

      PrefsToggle {
        checked: Omarchy.hyprDrag3fg === 1
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprDrag3fg(!(Omarchy.hyprDrag3fg === 1))
      }
    }
  }

  PrefsGroup {
    title: "Keyboard"
    query: root.query
    detail: "Repeat and numlock for Hyprland. The console and login layout stay on System."

    PrefsRow {
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
        enabled: !Omarchy.busy
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprRepeatRate)
            Omarchy.setHyprRepeatRate(next)
        }
      }
    }

    PrefsRow {
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
        enabled: !Omarchy.busy
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprRepeatDelay)
            Omarchy.setHyprRepeatDelay(next)
        }
      }
    }

    PrefsRow {
      label: "Numlock on boot"
      description: "Turn the number pad on when Hyprland starts."
      hint: "~/.config/hypr/input.lua · input.numlock_by_default"
      query: root.query
      keywords: ["numlock", "keypad"]

      PrefsToggle {
        checked: Omarchy.hyprNumlock
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprNumlock(!Omarchy.hyprNumlock)
      }
    }

    PrefsRow {
      available: Omarchy.hyprInputManaged
      label: "Reset input"
      description: "Remove the block Preferences wrote. Hyprland goes back to the rest of input.lua and the Omarchy defaults."
      hint: "~/.config/hypr/input.lua"
      query: root.query
      keywords: ["reset", "default", "input"]

      PrefsButton {
        text: "Reset"
        danger: true
        enabled: !Omarchy.busy && Omarchy.hyprInputManaged
        onClicked: Omarchy.resetHyprInput()
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Follow mouse, waking the screen, an optional Hyprland layout list, and a three-finger workspace swipe."

    PrefsRow {
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
        enabled: !Omarchy.busy
        onChanged: function(value) {
          var next = Math.round(Number(value))
          if (next !== Omarchy.hyprFollowMouse)
            Omarchy.setHyprFollowMouse(next)
        }
      }
    }

    PrefsRow {
      label: "Wake on key"
      description: "A key press turns the screen back on after DPMS off."
      hint: "~/.config/hypr/input.lua · misc.key_press_enables_dpms"
      query: root.query
      keywords: ["dpms", "wake", "key"]

      PrefsToggle {
        checked: Omarchy.hyprKeyPressDpms
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprKeyPressDpms(!Omarchy.hyprKeyPressDpms)
      }
    }

    PrefsRow {
      label: "Wake on mouse"
      description: "Moving the pointer turns the screen back on after DPMS off."
      hint: "~/.config/hypr/input.lua · misc.mouse_move_enables_dpms"
      query: root.query
      keywords: ["dpms", "wake", "mouse"]

      PrefsToggle {
        checked: Omarchy.hyprMouseMoveDpms
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprMouseMoveDpms(!Omarchy.hyprMouseMoveDpms)
      }
    }

    PrefsRow {
      stretchControl: true
      label: "Hyprland layouts"
      description: "Optional. A comma list such as us,dk for Hyprland only. Leave it blank to keep the System layout."
      hint: "~/.config/hypr/input.lua · input.kb_layout"
      query: root.query
      keywords: ["layout", "xkb", "multi", "us,dk"]

      Row {
        width: parent.width
        spacing: Theme.space

        PrefsField {
          id: layoutField
          width: parent.width - layoutSetBtn.width - parent.spacing
          value: Omarchy.hyprKbLayout
          placeholder: "us,dk"
          enabled: !Omarchy.busy
        }

        PrefsButton {
          id: layoutSetBtn
          text: "Set"
          primary: true
          enabled: !Omarchy.busy
          onClicked: Omarchy.setHyprKbOverride(layoutField.currentText(), variantField.currentText(), Omarchy.hyprKbGroupToggle)
        }
      }
    }

    PrefsRow {
      stretchControl: true
      label: "Variants"
      description: "Optional variants matching the layout list, such as ,nodeadkeys."
      hint: "~/.config/hypr/input.lua · input.kb_variant"
      query: root.query
      keywords: ["variant", "intl", "nodeadkeys"]

      PrefsField {
        id: variantField
        width: parent.width
        value: Omarchy.hyprKbVariant
        placeholder: ",nodeadkeys"
        enabled: !Omarchy.busy
      }
    }

    PrefsRow {
      label: "Alt+Alt layout switch"
      description: "Left Alt and Right Alt together cycle the Hyprland layouts above."
      hint: "~/.config/hypr/input.lua · input.kb_options"
      query: root.query
      keywords: ["grp", "alts", "switch", "layout"]

      PrefsToggle {
        checked: Omarchy.hyprKbGroupToggle
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprKbOverride(layoutField.currentText() || Omarchy.hyprKbLayout, variantField.currentText() || Omarchy.hyprKbVariant, !Omarchy.hyprKbGroupToggle)
      }
    }

    PrefsRow {
      label: "Three-finger swipe"
      description: "Swipe sideways with three fingers to change workspace."
      hint: "~/.config/hypr/input.lua · hl.gesture"
      query: root.query
      keywords: ["gesture", "swipe", "workspace", "three"]

      PrefsToggle {
        checked: Omarchy.hyprWorkspaceGesture
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprWorkspaceGesture(!Omarchy.hyprWorkspaceGesture)
      }
    }
  }
}
