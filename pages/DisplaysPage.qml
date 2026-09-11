import QtQuick
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi
import "../services/Monitors.js" as MonJs
import "rows"

PrefsPage {
  id: root
  hubId: "display"
  title: "Displays"
  description: "Each monitor keeps its own resolution and refresh rate. Scale and brightness apply to the one you are looking at. On a laptop you also get the built-in panel and its input devices. GPU switching is on Drivers."

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
    var n = RichUi.monitorResolutions(monitor).length
    var modes = n > 1 ? (n + " resolutions on this output. ") : ""
    return summary + " " + modes + "Picking one writes a monitor rule in ~/.config/hypr/monitors.lua. The refresh rate behind it stays."
  }

  function resolutionValue(monitor) {
    return RichUi.currentMonitorResolutionValue(monitor)
  }

  function refreshValue(monitor) {
    return RichUi.currentMonitorRefreshValue(monitor, root.resolutionValue(monitor))
  }

  function writeMonitorMode(monitor, resolution, refresh) {
    var name = monitor && monitor.name ? String(monitor.name) : ""
    if (!name) return
    var mode = MonJs.sanitizeMode(String(resolution || "") + "@" + String(refresh || ""))
    if (mode) Omarchy.patchMonitorRule(name, { mode: mode, disabled: false })
  }

  function ruleFor(monitor) {
    var name = monitor && monitor.name ? String(monitor.name) : ""
    var list = Omarchy.monitorRules || []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && String(list[i].output || "") === name) return list[i]
    }
    return null
  }

  function ruleTransform(monitor) {
    var rule = root.ruleFor(monitor)
    if (rule && rule.transform != null) return String(rule.transform)
    return String(Math.round(Number(monitor && monitor.transform)) || 0)
  }

  function ruleDisabled(monitor) {
    var rule = root.ruleFor(monitor)
    if (rule) return rule.disabled === true
    return monitor && monitor.enabled === false
  }

  function ruleVrr(monitor) {
    var rule = root.ruleFor(monitor)
    if (rule && rule.vrr != null) return String(rule.vrr)
    return String(Math.round(Number(monitor && monitor.vrr)) || 0)
  }

  function ruleBitdepth(monitor) {
    var rule = root.ruleFor(monitor)
    if (rule && rule.bitdepth === 10) return "10"
    return "8"
  }

  function ruleCm(monitor) {
    var rule = root.ruleFor(monitor)
    return rule && rule.cm ? String(rule.cm) : ""
  }

  function vrrAvailable(monitor) {
    if (!monitor) return false
    if (monitor.vrr === true || Number(monitor.vrr) > 0) return true
    if (monitor.availableVrr === true) return true
    var modes = Array.isArray(monitor.availableModes) ? monitor.availableModes : []
    return modes.length > 0
  }

  function hdrAvailable(monitor) {
    if (!monitor) return false
    var cm = String(monitor.currentFormat || monitor.cm || "")
    return /hdr|bt2020|10/i.test(cm) || monitor.hdr === true || root.ruleCm(monitor) === "hdr"
  }

  PrefsGroup {
    title: "Displays"
    query: Omarchy.monitors.length === 0 ? root.query : "."
    detail: "Hyprland did not report any outputs. Refresh after a display is connected. Modes live in ~/.config/hypr/monitors.lua."
    hint: "hyprctl monitors all"

    SettingRow {
      available: Omarchy.monitors.length === 0
      label: "Monitors"
      description: "No monitors reported."
      hint: "hyprctl monitors all"
      query: root.query
      keywords: ["monitor", "display", "hdmi", "dp", "edp", "resolution", "empty"]

      Row {
        spacing: Theme.space
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
    model: Omarchy.monitors.length

    PrefsGroup {
    framed: true
      required property int index
      readonly property var modelData: Omarchy.monitors[index] || ({})
      title: root.monitorTitle(modelData)
      query: root.query
      detail: "Mode, scale, rotation, and disable write a monitor rule in ~/.config/hypr/monitors.lua. Brightness works on the built-in panel and on some external monitors."

      SettingRow {
        label: "Resolution"
        description: root.resolutionDescription(modelData)
        hint: "~/.config/hypr/monitors.lua"
        detail: "Every resolution the output reports, plus standard lower modes, largest first. Atmos writes the pick together with the refresh rate behind it as hl.monitor mode."
        query: root.query
        keywords: ["monitor", "display", "hdmi", "dp", "edp", "resolution", "refresh"]

        Row {
          spacing: Theme.space
          PrefsSelect {
            value: root.resolutionValue(modelData)
            options: RichUi.monitorResolutions(modelData)
            enabled: !!(modelData && modelData.name)
            onChanged: function(value) {
              if (value !== root.resolutionValue(modelData))
                root.writeMonitorMode(modelData, value, RichUi.currentMonitorRefreshValue(modelData, value))
            }
          }
          PrefsButton {
            text: "Copy"
            enabled: RichUi.monitorModeCopyText(modelData).length > 0
            onClicked: Omarchy.copyText(RichUi.monitorModeCopyText(modelData))
          }
        }
      }

      SettingRow {
        label: "Refresh rate"
        description: "How many frames this panel draws per second. Only the rates the resolution above supports are listed."
        hint: "~/.config/hypr/monitors.lua"
        detail: "Atmos writes the pick together with the resolution above as hl.monitor mode."
        query: root.query
        keywords: ["monitor", "display", "refresh", "hertz", "hz", "fps", "highrr"]

        PrefsSelect {
          value: root.refreshValue(modelData)
          options: RichUi.monitorRefreshRates(modelData, root.resolutionValue(modelData))
          enabled: !!(modelData && modelData.name)
          onChanged: function(value) {
            if (value !== root.refreshValue(modelData))
              root.writeMonitorMode(modelData, root.resolutionValue(modelData), value)
          }
        }
      }

      SettingRow {
        available: !!(modelData && modelData.name)
        label: "Rotation"
        description: "How this panel is turned."
        hint: "hl.monitor transform"
        query: root.query
        keywords: ["rotate", "transform", "portrait"]

        PrefsSelect {
          value: root.ruleTransform(modelData)
          options: [
            { value: "0", label: "Normal" },
            { value: "1", label: "90°" },
            { value: "2", label: "180°" },
            { value: "3", label: "270°" }
          ]
          onChanged: function(value) {
            Omarchy.patchMonitorRule(modelData.name, { transform: Math.round(Number(value)) || 0 })
          }
        }
      }

      SettingRow {
        available: !!(modelData && modelData.name)
        label: "Disable this display"
        description: "Keeps the rule so you can turn it back on. Does not delete the output from the file."
        hint: "hl.monitor disabled"
        query: root.query
        keywords: ["disable", "off", "lid"]

        PrefsToggle {
          checked: root.ruleDisabled(modelData)
          onToggled: Omarchy.patchMonitorRule(modelData.name, { disabled: !root.ruleDisabled(modelData) })
        }
      }

      SettingRow {
        available: !!(modelData && modelData.name && root.vrrAvailable(modelData))
        label: "Variable refresh"
        description: "VRR when this output supports it. Fullscreen-only is the safer game setting."
        hint: "hl.monitor vrr"
        query: root.query
        keywords: ["vrr", "freesync", "g-sync"]

        PrefsSelect {
          value: root.ruleVrr(modelData)
          options: [
            { value: "0", label: "Off" },
            { value: "1", label: "On" },
            { value: "2", label: "Fullscreen" },
            { value: "3", label: "Fullscreen games" }
          ]
          onChanged: function(value) {
            Omarchy.patchMonitorRule(modelData.name, { vrr: Math.round(Number(value)) || 0 })
          }
        }
      }

      SettingRow {
        available: !!(modelData && modelData.name)
        label: "Bit depth"
        description: "10-bit when the panel and cable can do it."
        hint: "hl.monitor bitdepth"
        query: root.query
        keywords: ["bitdepth", "10-bit", "hdr"]

        PrefsSelect {
          value: root.ruleBitdepth(modelData)
          options: [
            { value: "8", label: "8-bit" },
            { value: "10", label: "10-bit" }
          ]
          onChanged: function(value) {
            Omarchy.patchMonitorRule(modelData.name, { bitdepth: value === "10" ? 10 : 8 })
          }
        }
      }

      SettingRow {
        available: !!(modelData && modelData.name)
        label: "Color"
        description: root.hdrAvailable(modelData)
          ? "This output can take HDR. Auto leaves Hyprland's default."
          : "Color profile for this output. HDR stays off unless the panel reports it."
        hint: "hl.monitor cm"
        query: root.query
        keywords: ["hdr", "color", "srgb", "wide"]

        PrefsSelect {
          value: root.ruleCm(modelData)
          options: [
            { value: "", label: "Auto" },
            { value: "srgb", label: "sRGB" },
            { value: "wide", label: "Wide" },
            { value: "dcip3", label: "DCI-P3" },
            { value: "hdr", label: "HDR" },
            { value: "hdredid", label: "HDR EDID" }
          ]
          onChanged: function(value) {
            Omarchy.patchMonitorRule(modelData.name, { cm: value })
          }
        }
      }

      SettingRow {
        available: !!(modelData && modelData.name)
        label: "Scale"
        description: "How large the interface looks on this monitor. The focused output uses omarchy hyprland monitor scaling. Other outputs write a monitor rule."
        hint: "omarchy hyprland monitor scaling"
        detail: "Scale is Hyprland's factor of UI pixels over physical pixels. 200% on a 4K panel makes chrome and text about the size they would be at 1080p. Hyprland snaps to a factor it can render cleanly, so 125% or 160% can land a little off the number you pick. This control only applies to the focused output. Other monitors keep their own scale."
        query: root.query
        keywords: ["scale", "hidpi", "fractional", "dpi"]

        PrefsSelect {
          value: root.scaleValue(modelData)
          options: root.scaleOptions(modelData)
          enabled: !!(modelData && modelData.name)
          onChanged: function(value) {
            if (value !== root.scaleValue(modelData)) {
              if (modelData.focused === true) Omarchy.setMonitorScale(value)
              else {
                var rules = []
                var list = Omarchy.monitorRules || []
                var found = false
                for (var i = 0; i < list.length; i++) {
                  var row = list[i] || {}
                  if (row.output === modelData.name) {
                    var next = {}
                    for (var k in row) next[k] = row[k]
                    next.scale = Number(value)
                    rules.push(next)
                    found = true
                  } else rules.push(row)
                }
                if (!found) {
                  rules.push({
                    output: modelData.name,
                    mode: "preferred",
                    position: "auto",
                    scale: Number(value),
                    transform: 0,
                    disabled: modelData.enabled === false,
                    vrr: 0,
                    bitdepth: 8,
                    cm: ""
                  })
                }
                Omarchy.writeMonitorRules(rules)
              }
            }
          }
        }
      }

      SettingRow {
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
          showTicks: false
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
    title: "Layouts"
    query: Omarchy.monitors.length ? root.query : "."
    detail: "Desk keeps every output on. Laptop keeps the built-in panel. Docked turns the built-in panel off. Each write is a monitor rule in ~/.config/hypr/monitors.lua."
    hint: "~/.config/hypr/monitors.lua"

    SettingRow {
      label: "Apply a layout"
      description: "Uses the outputs Hyprland sees right now."
      hint: "hl.monitor"
      query: root.query
      keywords: ["desk", "laptop", "docked", "layout", "profile"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Desk"
          onClicked: Omarchy.applyMonitorLayout("desk")
        }
        PrefsButton {
          text: "Laptop"
          onClicked: Omarchy.applyMonitorLayout("laptop")
        }
        PrefsButton {
          text: "Docked"
          onClicked: Omarchy.applyMonitorLayout("docked")
        }
      }
    }
  }

  PrefsGroup {
    title: "Laptop"
    query: (Omarchy.internalPresent || Omarchy.keyboardBacklightPresent || Omarchy.touchpadPresent || Omarchy.touchscreenPresent) ? root.query : "."
    detail: "These only show up on a laptop. The built-in screen can turn off while an external monitor is plugged in. Touchpad and touchscreen stay off across a Hyprland reload."

    SettingRow {
      available: Omarchy.internalPresent
      label: "Laptop screen"
      description: Omarchy.externalPresent
        ? "Keep the built-in panel on."
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

    SettingRow {
      available: Omarchy.internalPresent && Omarchy.externalPresent
      label: "Mirror to the first external"
      description: "The same picture on the laptop screen and the first external monitor."
      hint: "omarchy hyprland monitor internal mirror"
      query: root.query
      keywords: ["mirror", "clone", "duplicate"]

      PrefsToggle {
        checked: Omarchy.mirroring
        enabled: Omarchy.internalPresent && Omarchy.externalPresent
        onToggled: Omarchy.setInternalMirror(!Omarchy.mirroring)
      }
    }

    SettingRow {
      available: Omarchy.touchpadPresent
      label: "Touchpad"
      description: "Finger and pointer input on the trackpad. The choice survives a Hyprland reload."
      hint: "omarchy toggle touchpad"
      query: root.query
      keywords: ["trackpad", "touchpad", "pointer", "mouse", "input"]

      PrefsToggle {
        checked: Omarchy.touchpadEnabled
        enabled: Omarchy.touchpadPresent
        onToggled: Omarchy.setTouchpad(!Omarchy.touchpadEnabled)
      }
    }

    TouchscreenRow {
      query: root.query
      requirePresent: true
    }

    SettingRow {
      available: Omarchy.keyboardBacklightPresent
      label: "Keyboard backlight"
      description: Omarchy.keyboardBrightness > 0
        ? ("The keys are at " + Omarchy.keyboardBrightness + "%.")
        : "Step the keyboard LEDs up or down, or turn them off."
      hint: "omarchy brightness keyboard"
      query: root.query
      keywords: ["kbd", "backlight", "leds"]

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Dim"
          enabled: Omarchy.keyboardBacklightPresent
          onClicked: Omarchy.adjustKeyboardBacklight("down")
        }
        PrefsButton {
          text: "Brighten"
          enabled: Omarchy.keyboardBacklightPresent
          onClicked: Omarchy.adjustKeyboardBacklight("up")
        }
        PrefsButton {
          text: "Turn off"
          enabled: Omarchy.keyboardBacklightPresent && Omarchy.keyboardBrightness > 0
          onClicked: Omarchy.adjustKeyboardBacklight("off")
        }
      }
    }
  }
}
