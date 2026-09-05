import QtQuick
import "../../components"
import "../../services"

SettingRow {
  property bool requirePresent: false

  available: !requirePresent || Omarchy.touchscreenPresent
  label: "Touchscreen"
  description: Omarchy.touchscreenPresent
    ? "Finger input on the display. The choice survives a Hyprland reload."
    : "Hyprland has not reported a touch device."
  hint: "omarchy toggle touchscreen"
  keywords: ["touch", "touchscreen", "tablet", "digitizer"]

  PrefsToggle {
    checked: Omarchy.touchscreenPresent && Omarchy.touchscreenEnabled
    enabled: Omarchy.touchscreenPresent
    onToggled: Omarchy.setTouchscreen(!Omarchy.touchscreenEnabled)
  }
}
