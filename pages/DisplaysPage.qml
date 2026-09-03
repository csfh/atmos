import QtQuick
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Displays"
  description: "Each monitor keeps its own resolution. Scale and brightness apply to the one you are looking at. On a laptop you also get the built-in panel and its input devices. GPU switching is on Hardware."

  readonly property var scalePresets: [
    { value: "1", label: "100%" },
    { value: "1.25", label: "125%" },
    { value: "1.6", label: "160%" },
    { value: "2", label: "200%" },
    { value: "3", label: "300%" },
    { value: "4", label: "400%" }
  ]

  function scaleValue(monitor) {
    var n = Number(monitor && monitor.scale)
    if (!isFinite(n) || n <= 0) return "1"
    return String(n)
  }

  function scaleOptions(monitor) {
    var current = root.scaleValue(monitor)
    var list = root.scalePresets.slice()
    for (var i = 0; i < list.length; i++) {
      if (list[i].value === current) return list
    }
    return list.concat([{ value: current, label: current }])
  }

  function monitorTitle(monitor) {
    var name = String((monitor && monitor.name) || "Display")
    var desc = String((monitor && monitor.description) || "")
    if (desc.length === 0) return name
    var shortName = desc
    var brand = desc.indexOf(" ")
    if (brand !== -1 && desc.length > 28) shortName = desc.substring(0, 28)
    return name + "  " + shortName
  }

  function monitorSummary(monitor) {
    if (!monitor) return "No signal."
    var size = RichUi.formatMonitorMode(RichUi.currentMonitorModeValue(monitor))
    if (!size) size = monitor.width + "×" + monitor.height
    var extra = []
    if (monitor.focused) extra.push("focused")
    if (!monitor.enabled) extra.push("disabled")
    if (monitor.mirrorOf) extra.push("mirroring " + monitor.mirrorOf)
    var tail = extra.length ? ". " + extra.join(", ") + "." : "."
    return size + tail
  }

  function resolutionDescription(monitor) {
    var summary = root.monitorSummary(monitor)
    var n = monitor && Array.isArray(monitor.availableModes) ? monitor.availableModes.length : 0
    var modes = n > 1 ? (n + " modes on this output. ") : ""
    return summary + " " + modes + "Atmos cannot change the panel mode here. Edit ~/.config/hypr/monitors.lua, then reload Hyprland."
  }

  PrefsGroup {
    title: "Displays"
    query: Omarchy.monitors.length === 0 ? root.query : "."
    detail: "Hyprland did not report any outputs. Refresh after a display is connected. Modes live in ~/.config/hypr/monitors.lua."
    hint: "hyprctl monitors all"

    PrefsRow {
      available: Omarchy.monitors.length === 0
      label: "No outputs"
      description: "Hyprland did not report any monitors. Refresh after a display is connected. Atmos cannot change the panel mode here — edit ~/.config/hypr/monitors.lua."
      hint: "hyprctl monitors all"
      query: root.query
      keywords: ["monitor", "display", "hdmi", "dp", "edp", "resolution", "empty"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Refresh"
          onClicked: Omarchy.refresh()
        }
        PrefsButton {
          text: "Edit"
          onClicked: Omarchy.editMonitorsLua()
        }
      }
    }
  }

  Repeater {
    model: Omarchy.monitors

    PrefsGroup {
      required property var modelData
      title: root.monitorTitle(modelData)
      query: root.query
      detail: "This output's current mode. Scale only shows when the monitor is focused. Brightness works on the built-in panel and on some external monitors. Resolution is set in ~/.config/hypr/monitors.lua."

      PrefsRow {
        label: "Resolution"
        description: root.resolutionDescription(modelData)
        hint: "~/.config/hypr/monitors.lua"
        detail: "Hyprland picks a mode from this output's EDID list. Atmos cannot change it here because there is no omarchy monitor-mode command. Edit ~/.config/hypr/monitors.lua, then reload Hyprland. Copy puts the current mode on the clipboard."
        query: root.query
        keywords: ["monitor", "display", "hdmi", "dp", "edp", "resolution", "refresh"]

        Row {
          spacing: 8
          PrefsSelect {
            value: RichUi.currentMonitorModeValue(modelData)
            options: RichUi.monitorModeOptions(modelData)
            enabled: false
          }
          PrefsButton {
            text: "Copy"
            enabled: RichUi.monitorModeCopyText(modelData).length > 0
            onClicked: Omarchy.copyText(RichUi.monitorModeCopyText(modelData))
          }
          PrefsButton {
            text: "Edit"
            onClicked: Omarchy.editMonitorsLua()
          }
        }
      }

      PrefsRow {
        available: modelData && modelData.focused === true
        label: "Scale"
        description: "How large the interface looks on the focused monitor. Hyprland snaps to a factor it can draw cleanly."
        hint: "omarchy hyprland monitor scaling"
        detail: "Scale is Hyprland's factor of UI pixels over physical pixels. 200% on a 4K panel makes chrome and text about the size they would be at 1080p. Hyprland snaps to a factor it can render cleanly, so 125% or 160% can land a little off the number you pick. This control only applies to the focused output. Other monitors keep their own scale."
        query: root.query
        keywords: ["scale", "hidpi", "fractional", "dpi"]

        PrefsSelect {
          value: root.scaleValue(modelData)
          options: root.scaleOptions(modelData)
          enabled: modelData && modelData.focused === true
          onChanged: function(value) {
            if (value !== root.scaleValue(modelData)) Omarchy.setMonitorScale(value)
          }
        }
      }

      PrefsRow {
        available: modelData && modelData.brightnessAvailable === true
        stretchControl: true
        label: "Brightness"
        description: "How bright this panel is. Works on the built-in display and on some external monitors."
        hint: "omarchy brightness display"
        query: root.query
        keywords: ["backlight", "ddc", "luminance"]

        PrefsSlider {
          width: parent.width
          from: 1
          to: 100
          stepSize: 1
          live: true
          value: modelData && modelData.brightness ? modelData.brightness : 1
          valueText: (modelData && modelData.brightness ? modelData.brightness : 0) + "%"
          enabled: modelData && modelData.brightnessAvailable === true
          onChanged: function(value) {
            var next = Math.round(value)
            if (!modelData || next === modelData.brightness) return
            Omarchy.setDisplayBrightness(modelData.name, next)
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Laptop"
    query: (Omarchy.internalPresent || Omarchy.keyboardBacklightPresent || Omarchy.touchpadPresent || Omarchy.touchscreenPresent) ? root.query : "."
    detail: "These only show up on a laptop. The built-in screen can turn off while an external monitor is plugged in. Touchpad and touchscreen stay off across a Hyprland reload."

    PrefsRow {
      available: Omarchy.internalPresent
      label: "Laptop screen"
      description: Omarchy.externalPresent
        ? "Leave the built-in panel on, or turn it off while an external monitor is plugged in."
        : "This is the only display, so it has to stay on."
      hint: "omarchy hyprland monitor internal"
      query: root.query
      keywords: ["lid", "edp", "laptop", "clamshell"]

      PrefsToggle {
        checked: Omarchy.internalEnabled
        enabled: Omarchy.internalPresent && Omarchy.externalPresent
        onToggled: Omarchy.setInternalDisplay(!Omarchy.internalEnabled)
      }
    }

    PrefsRow {
      available: Omarchy.internalPresent && Omarchy.externalPresent
      label: "Mirror to the first external"
      description: "Show the same picture on the laptop screen and the first external monitor."
      hint: "omarchy hyprland monitor internal mirror"
      query: root.query
      keywords: ["mirror", "clone", "duplicate"]

      PrefsToggle {
        checked: Omarchy.mirroring
        enabled: Omarchy.internalPresent && Omarchy.externalPresent
        onToggled: Omarchy.setInternalMirror(!Omarchy.mirroring)
      }
    }

    PrefsRow {
      available: Omarchy.touchpadPresent
      label: "Touchpad"
      description: "The laptop trackpad. Turning it off survives a Hyprland reload."
      hint: "omarchy toggle touchpad"
      query: root.query
      keywords: ["trackpad", "touchpad", "pointer", "mouse", "input"]

      PrefsToggle {
        checked: Omarchy.touchpadEnabled
        enabled: Omarchy.touchpadPresent
        onToggled: Omarchy.setTouchpad(!Omarchy.touchpadEnabled)
      }
    }

    PrefsRow {
      available: Omarchy.touchscreenPresent
      label: "Touchscreen"
      description: "Finger input on the display. Turning it off survives a Hyprland reload."
      hint: "omarchy toggle touchscreen"
      query: root.query
      keywords: ["touch", "touchscreen", "tablet", "digitizer"]

      PrefsToggle {
        checked: Omarchy.touchscreenEnabled
        enabled: Omarchy.touchscreenPresent
        onToggled: Omarchy.setTouchscreen(!Omarchy.touchscreenEnabled)
      }
    }

    PrefsRow {
      available: Omarchy.keyboardBacklightPresent
      label: "Keyboard backlight"
      description: Omarchy.keyboardBrightness > 0
        ? ("The keys are at " + Omarchy.keyboardBrightness + "%.")
        : "Step the keyboard LEDs up or down, or turn them off."
      hint: "omarchy brightness keyboard"
      query: root.query
      keywords: ["kbd", "backlight", "leds"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Dim"
          enabled: Omarchy.keyboardBacklightPresent
          onClicked: Omarchy.adjustKeyboardBacklight("down")
        }
        PrefsButton {
          text: "Brighter"
          enabled: Omarchy.keyboardBacklightPresent
          onClicked: Omarchy.adjustKeyboardBacklight("up")
        }
        PrefsButton {
          text: "Off"
          enabled: Omarchy.keyboardBacklightPresent && Omarchy.keyboardBrightness > 0
          onClicked: Omarchy.adjustKeyboardBacklight("off")
        }
      }
    }
  }
}
