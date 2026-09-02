import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Displays"
  description: "Each monitor keeps its own resolution. Scale and brightness apply to the one you are looking at. On a laptop you also get the built-in panel and its input devices."

  PrefsConfirm {
    id: hybridGpuConfirm
    title: "Switch GPU mode"
    message: Omarchy.hybridGpuMode === "Integrated"
      ? "Turn the dedicated GPU on (hybrid) and reboot."
      : "Use only the integrated GPU and reboot."
    confirmText: "Switch and reboot"
    onConfirmed: Omarchy.toggleHybridGpu()
  }

  Component.onCompleted: {
    hybridGpuConfirm.parent = root.prefsOverlay
  }

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
    var size = monitor.width + "×" + monitor.height
    var hz = monitor.refresh > 0 ? (" @ " + monitor.refresh + " Hz") : ""
    var extra = []
    if (monitor.focused) extra.push("focused")
    if (!monitor.enabled) extra.push("disabled")
    if (monitor.mirrorOf) extra.push("mirroring " + monitor.mirrorOf)
    var tail = extra.length ? ". " + extra.join(", ") + "." : "."
    return size + hz + tail
  }

  Repeater {
    model: Omarchy.monitors

    PrefsGroup {
      required property var modelData
      title: root.monitorTitle(modelData)
      query: root.query
      detail: "This output's current mode. Scale only shows when the monitor is focused. Brightness works on the built-in panel and on some external monitors."

      PrefsRow {
        label: "Resolution"
        description: root.monitorSummary(modelData)
        hint: "hyprctl monitors"
        query: root.query
        keywords: ["monitor", "display", "hdmi", "dp", "edp", "resolution", "refresh"]
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
          enabled: !Omarchy.busy && modelData && modelData.focused === true
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
          value: modelData && modelData.brightness ? modelData.brightness : 1
          valueText: (modelData && modelData.brightness ? modelData.brightness : 0) + "%"
          enabled: !Omarchy.busy && modelData && modelData.brightnessAvailable === true
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
        enabled: !Omarchy.busy && Omarchy.internalPresent && Omarchy.externalPresent
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
        enabled: !Omarchy.busy && Omarchy.internalPresent && Omarchy.externalPresent
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
        enabled: !Omarchy.busy && Omarchy.touchpadPresent
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
        enabled: !Omarchy.busy && Omarchy.touchscreenPresent
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
          enabled: !Omarchy.busy && Omarchy.keyboardBacklightPresent
          onClicked: Omarchy.adjustKeyboardBacklight("down")
        }
        PrefsButton {
          text: "Brighter"
          enabled: !Omarchy.busy && Omarchy.keyboardBacklightPresent
          onClicked: Omarchy.adjustKeyboardBacklight("up")
        }
        PrefsButton {
          text: "Off"
          enabled: !Omarchy.busy && Omarchy.keyboardBacklightPresent && Omarchy.keyboardBrightness > 0
          onClicked: Omarchy.adjustKeyboardBacklight("off")
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: Omarchy.hybridGpuAvailable ? root.query : "."
    detail: "Hybrid GPU is for machines with both an integrated and a dedicated GPU. Switching reboots."

    PrefsRow {
      available: Omarchy.hybridGpuAvailable
      label: "Hybrid GPU"
      description: Omarchy.hybridGpuMode === "Integrated"
        ? "Only the integrated GPU is on. Switch to hybrid if you want the dedicated GPU."
        : (Omarchy.hybridGpuMode === "Hybrid"
          ? "Hybrid mode. The dedicated GPU can wake for a game or CUDA."
          : "This machine can switch between integrated-only and hybrid.")
      hint: "omarchy toggle hybrid gpu"
      query: root.query
      keywords: ["gpu", "hybrid", "nvidia", "supergfx", "igpu"]

      PrefsButton {
        text: "Switch…"
        enabled: !Omarchy.busy && !Omarchy.jobBusy && Omarchy.hybridGpuAvailable
        onClicked: hybridGpuConfirm.ask()
      }
    }
  }
}
