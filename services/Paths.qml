pragma Singleton
import QtQuick
import Quickshell

// Every file, directory, and script path Atmos touches, in one place. The
// watch list, the snapshot groups, and the setters all read these.
QtObject {
  id: root

  readonly property string shellDir: Quickshell.shellDir

  readonly property string envFile: Quickshell.env("HOME") + "/.config/environment.d/10-atmos.conf"
  readonly property string presentationFile: Quickshell.env("HOME") + "/.local/state/omarchy/atmos-presentation.json"
  readonly property string favoritesFile: Quickshell.env("HOME") + "/.local/state/omarchy/atmos-favorites.json"
  readonly property string looknfeelLuaFile: Quickshell.env("HOME") + "/.config/hypr/looknfeel.lua"
  readonly property string inputLuaFile: Quickshell.env("HOME") + "/.config/hypr/input.lua"
  readonly property string autostartLuaFile: Quickshell.env("HOME") + "/.config/hypr/autostart.lua"
  readonly property string bindingsLuaFile: Quickshell.env("HOME") + "/.config/hypr/bindings.lua"
  readonly property string windowsLuaFile: Quickshell.env("HOME") + "/.config/hypr/atmos.lua"
  readonly property string hyprsunsetConfFile: Quickshell.env("HOME") + "/.config/hypr/hyprsunset.conf"
  readonly property string pacmanConfFile: "/etc/pacman.conf"
  readonly property string localtimeFile: "/etc/localtime"
  readonly property string hostnameFile: "/etc/hostname"
  readonly property string passwdFile: "/etc/passwd"
  readonly property string groupFile: "/etc/group"
  readonly property string vconsoleFile: "/etc/vconsole.conf"
  readonly property string localeConfFile: "/etc/locale.conf"
  readonly property string gtkSettingsFile: Quickshell.env("HOME") + "/.config/gtk-4.0/settings.ini"
  readonly property string swappinessFile: "/etc/sysctl.d/99-atmos-swappiness.conf"
  readonly property string userShellJson: Quickshell.env("HOME") + "/.config/omarchy/shell.json"
  readonly property string defaultShellJson: "/usr/share/omarchy/config/omarchy/shell.json"
  readonly property string userShellToml: Quickshell.env("HOME") + "/.config/omarchy/shell.toml"
  readonly property string weatherJson: Quickshell.env("HOME") + "/.local/state/omarchy/settings/weather.json"
  readonly property string notificationsJson: Quickshell.env("HOME") + "/.local/state/omarchy/notifications.json"
  readonly property string currentThemeNameFile: Quickshell.env("HOME") + "/.local/state/omarchy/current/theme.name"
  readonly property string currentBackgroundFile: Quickshell.env("HOME") + "/.local/state/omarchy/current/background"
  readonly property string screensaverBrandFile: Quickshell.env("HOME") + "/.config/omarchy/branding/screensaver.txt"
  readonly property string defaultScreensaverBrandFile: "/usr/share/omarchy/logo.txt"
  readonly property string aboutBrandFile: Quickshell.env("HOME") + "/.config/omarchy/branding/about.txt"
  readonly property string defaultAboutBrandFile: "/usr/share/omarchy/icon.txt"
  readonly property string powerProfileAcFile: Quickshell.env("HOME") + "/.local/state/omarchy/powerprofiles/ac"
  readonly property string powerProfileBatteryFile: Quickshell.env("HOME") + "/.local/state/omarchy/powerprofiles/battery"
  readonly property string togglesDir: Quickshell.env("HOME") + "/.local/state/omarchy/toggles"
  readonly property string hyprTogglesDir: Quickshell.env("HOME") + "/.local/state/omarchy/toggles/hypr"
  readonly property string touchpadDisabledFile: hyprTogglesDir + "/touchpad-disabled-name"
  readonly property string touchscreenDisabledFile: hyprTogglesDir + "/touchscreen-disabled-name"
  readonly property string indicatorsDir: Quickshell.env("HOME") + "/.local/state/omarchy/indicators"
  readonly property string extraThemesDir: Quickshell.env("HOME") + "/.config/omarchy/themes"
  readonly property string applicationsDir: Quickshell.env("HOME") + "/.local/share/applications"
  readonly property string packagedThemesDir: "/usr/share/omarchy/themes"
  readonly property string defaultEditorFile: Quickshell.env("HOME") + "/.local/state/omarchy/defaults/editor"
  readonly property string defaultAgentFile: Quickshell.env("HOME") + "/.config/omarchy/defaults/agent"
  readonly property string defaultTerminalFile: Quickshell.env("HOME") + "/.config/xdg-terminals.list"
  readonly property string defaultBrowserFile: Quickshell.env("HOME") + "/.config/mimeapps.list"
  readonly property string fontconfigFile: Quickshell.env("HOME") + "/.config/fontconfig/fonts.conf"
  readonly property string reminderDir: (Quickshell.env("XDG_RUNTIME_DIR") || "/tmp") + "/omarchy-reminders"
  readonly property string dnsConfFile: "/etc/NetworkManager/conf.d/20-omarchy-dns.conf"
  readonly property string bluetoothRfkillDir: "/var/lib/systemd/rfkill"
  readonly property string plymouthLogoFile: "/usr/share/plymouth/themes/omarchy/logo.png"
  readonly property string defaultPlymouthLogoFile: "/usr/share/omarchy/default/plymouth/logo.png"
  readonly property string powerProfilesStateFile: "/var/lib/power-profiles-daemon/state.ini"
  readonly property string networkManagerDevicesDir: "/run/NetworkManager/devices"
  readonly property string monitorsLuaFile: Quickshell.env("HOME") + "/.config/hypr/monitors.lua"
}
