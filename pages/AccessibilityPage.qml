import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Accessibility"
  description: "Motion, type size, the pointer, and touch. These use the same writers as Windows, Appearance, and Displays."

  PrefsGroup {
    title: "Motion and type"
    query: root.query
    detail: "Animations write the look sentinel. Text size is the same slider as Appearance."

    PrefsRow {
      label: "Animations"
      description: "Window open, close, and fade motion."
      hint: "~/.config/hypr/looknfeel.lua · animations.enabled"
      query: root.query
      keywords: ["animation", "motion", "reduce", "a11y"]

      PrefsToggle {
        checked: Omarchy.hyprAnimations
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprAnimations(!Omarchy.hyprAnimations)
      }
    }

    PrefsRow {
      label: "Text size"
      description: "How large type is in the shell, GTK apps, and terminals."
      hint: "omarchy display text size"
      query: root.query
      keywords: ["scale", "size", "type", "font", "a11y"]

      Item {
        implicitWidth: a11ySizeSlider.width + 10 + a11ySizeSpin.width
        implicitHeight: Math.max(a11ySizeSlider.implicitHeight, a11ySizeSpin.implicitHeight)

        PrefsSlider {
          id: a11ySizeSlider
          width: 162
          anchors.verticalCenter: parent.verticalCenter
          from: 9
          to: 20
          stepSize: 1
          value: Omarchy.textSize
          valueText: Omarchy.textSize + " px"
          showValue: false
          showTicks: false
          enabled: !Omarchy.busy
          onChanged: function(value) {
            var next = Math.round(value)
            if (next !== Omarchy.textSize) Omarchy.setTextSize(next)
          }
        }

        PrefsSpinBox {
          id: a11ySizeSpin
          anchors.left: a11ySizeSlider.right
          anchors.leftMargin: 10
          anchors.verticalCenter: parent.verticalCenter
          from: 9
          to: 20
          value: Omarchy.textSize
          enabled: !Omarchy.busy
          onChanged: function(value) {
            if (value !== Omarchy.textSize) Omarchy.setTextSize(value)
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Pointer"
    query: root.query
    detail: "Cursor hide is the same look key as Windows. Size is a new look-sentinel field."

    PrefsRow {
      label: "Hide cursor while typing"
      description: "The pointer disappears when you start typing."
      hint: "~/.config/hypr/looknfeel.lua · cursor.hide_on_key_press"
      query: root.query
      keywords: ["cursor", "pointer", "hide", "type"]

      PrefsToggle {
        checked: Omarchy.hyprCursorHideOnKey
        enabled: !Omarchy.busy
        onToggled: Omarchy.setHyprCursorHideOnKey(!Omarchy.hyprCursorHideOnKey)
      }
    }

    PrefsRow {
      stretchControl: true
      label: "Cursor size"
      description: "How large the pointer is."
      hint: "~/.config/hypr/looknfeel.lua · HYPRCURSOR_SIZE"
      query: root.query
      keywords: ["cursor", "pointer", "size", "a11y"]

      PrefsSlider {
        width: parent.width
        from: 8
        to: 64
        stepSize: 2
        value: Omarchy.hyprCursorSize
        valueText: Omarchy.hyprCursorSize + " px"
        enabled: !Omarchy.busy
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.hyprCursorSize)
            Omarchy.setHyprCursorSize(next)
        }
      }
    }
  }

  PrefsGroup {
    title: "Touch"
    query: root.query
    detail: "This is the same touchscreen switch as Displays. It only shows when Hyprland reports a touch device."

    PrefsRow {
      available: Omarchy.touchscreenPresent
      label: "Touchscreen"
      description: "Finger input on the display. Turning it off survives a Hyprland reload."
      hint: "omarchy toggle touchscreen"
      query: root.query
      keywords: ["touch", "touchscreen", "tablet", "digitizer"]

      PrefsToggle {
        checked: Omarchy.touchscreenEnabled
        enabled: !Omarchy.busy && Omarchy.touchscreenPresent
        onToggled: Omarchy.setTouchscreen(!Omarchy.touchscreenEnabled)
      }
    }

    PrefsRow {
      available: !Omarchy.touchscreenPresent
      sectionHelp: false
      label: "No touchscreen"
      description: "Hyprland has not reported a touch device on this machine."
      query: root.query
      keywords: ["touch", "touchscreen"]
    }
  }

  PrefsGroup {
    title: "Tools"
    query: root.query
    detail: "Herdr is a screen reader Omarchy can launch in a terminal when the package is present."

    PrefsRow {
      available: Omarchy.extras && Omarchy.extras.herdr === true
      label: "Herdr"
      description: "Open the Herdr screen reader in a terminal."
      hint: "omarchy launch terminal herdr"
      query: root.query
      keywords: ["herdr", "screen reader", "a11y", "tts"]

      PrefsButton {
        text: "Open"
        enabled: !Omarchy.busy && Omarchy.extras && Omarchy.extras.herdr === true
        onClicked: Omarchy.launchHerdr()
      }
    }

    PrefsRow {
      available: !(Omarchy.extras && Omarchy.extras.herdr === true)
      sectionHelp: false
      label: "Herdr"
      description: "Herdr is not installed, so Preferences has nothing to open."
      query: root.query
      keywords: ["herdr", "screen reader"]
    }
  }
}
