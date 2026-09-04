pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "Theme.js" as ThemeJs

QtObject {
  id: root

  readonly property string home: Quickshell.env("HOME")
  readonly property string currentThemePath: home + "/.local/state/omarchy/current/theme"
  readonly property string currentThemeNameFile: home + "/.local/state/omarchy/current/theme.name"
  readonly property string userShellPath: home + "/.config/omarchy/shell.toml"

  property color foreground: "#cacccc"
  property color background: "#101315"
  property color accent: "#cacccc"
  property color muted: "#707880"
  property color urgent: "#a55555"

  property var themeShellValues: ({})
  property var userShellValues: ({})
  property var shellValues: ({})

  readonly property int fontSize: Math.max(9, Math.round(ThemeJs.numberToken(root.shellValues, "font.base-size", 12)))
  readonly property string fontFamily: "monospace"
  // Remix Icon names under icons/<name>.svg.
  readonly property string iconChevronLeft: "arrow-left-s-line"
  readonly property string iconChevronRight: "arrow-right-s-line"
  readonly property string iconInfo: "information-line"
  readonly property int radius: 0
  readonly property real normalFill: ThemeJs.numberToken(root.shellValues, "controls.normal-fill-alpha", 0.04)
  readonly property real hoverFill: ThemeJs.numberToken(root.shellValues, "controls.hover-cursor-fill-alpha", 0.08)
  readonly property real selectedFill: ThemeJs.numberToken(root.shellValues, "controls.selected-fill-alpha", 0.18)
  readonly property real borderAlpha: ThemeJs.numberToken(root.shellValues, "controls.normal-border-alpha", 0.4)
  readonly property int pad: Math.max(8, Math.round(fontSize * 0.9))
  readonly property int gap: Math.max(6, Math.round(fontSize * 0.6))
  readonly property int space: 8
  readonly property int spaceMd: 12
  readonly property int spaceLg: 20
  readonly property int rowPad: 10
  readonly property int rowHeight: Math.max(36, fontSize + spaceMd * 2)
  readonly property int controlHeight: Math.max(28, rowHeight - 8)
  readonly property int sidebarWidth: 220
  readonly property int contentMaxWidth: 1000
  readonly property int controlColumnWidth: 280
  readonly property int titleSize: Math.max(fontSize + 6, Math.round(fontSize * 1.4))
  readonly property int captionSize: Math.max(11, fontSize - 2)

  function contentColumnWidth(avail) {
    var w = Number(avail)
    if (!isFinite(w)) w = 0
    return Math.max(240, Math.min(contentMaxWidth, w - spaceLg * 2))
  }

  function contentColumnX(avail, colWidth) {
    var outer = Number(avail)
    var inner = Number(colWidth)
    if (!isFinite(outer)) outer = 0
    if (!isFinite(inner)) inner = 0
    return Math.max(spaceLg, Math.round((outer - inner) / 2))
  }

  function fill(alpha) {
    return Qt.rgba(foreground.r, foreground.g, foreground.b, alpha)
  }

  function accentFill(alpha) {
    return Qt.rgba(accent.r, accent.g, accent.b, alpha)
  }

  function borderColor() {
    return Qt.rgba(foreground.r, foreground.g, foreground.b, borderAlpha)
  }

  function splitColor() {
    return Qt.rgba(foreground.r, foreground.g, foreground.b, Math.max(0.08, borderAlpha * 0.28))
  }

  function applyColors(raw) {
    var parsed = ThemeJs.parseColors(raw)
    foreground = parsed.foreground
    background = parsed.background
    accent = parsed.accent
    muted = parsed.muted
    urgent = parsed.urgent
  }

  function mergeShell() {
    shellValues = ThemeJs.mergeShell(themeShellValues, userShellValues)
  }

  function reload() {
    colorsFile.reload()
    shellFile.reload()
    userShellFile.reload()
  }

  function readPath(path) {
    peekFile.path = path
    peekFile.reload()
    peekFile.waitForJob()
    return peekFile.text() || ""
  }

  function applyNamedTheme(name) {
    var paths = ThemeJs.themeFileCandidates(name, "colors.toml", root.home)
    var i
    var raw = ""
    for (i = 0; i < paths.length; i++) {
      raw = root.readPath(paths[i])
      if (raw) {
        root.applyColors(raw)
        break
      }
    }
    paths = ThemeJs.themeFileCandidates(name, "shell.toml", root.home)
    raw = ""
    for (i = 0; i < paths.length; i++) {
      raw = root.readPath(paths[i])
      if (raw) {
        root.themeShellValues = ThemeJs.parseShell(raw)
        root.mergeShell()
        break
      }
    }
  }

  // omarchy-theme-set rm -rf's current/theme then mv's a new directory in.
  // FileView watches the old inode, so bounce the path onto the new files.
  function reopenThemeFiles() {
    var colors = root.currentThemePath + "/colors.toml"
    var shell = root.currentThemePath + "/shell.toml"
    colorsFile.path = ""
    shellFile.path = ""
    colorsFile.path = colors
    shellFile.path = shell
    userShellFile.reload()
  }

  property FileView peekFile: FileView {
    printErrors: false
    blockLoading: true
  }

  property FileView themeNameFile: FileView {
    path: root.currentThemeNameFile
    watchChanges: true
    preload: true
    printErrors: false
    onFileChanged: root.reopenThemeFiles()
  }

  property FileView colorsFile: FileView {
    path: root.currentThemePath + "/colors.toml"
    watchChanges: true
    printErrors: false
    onLoaded: {
      var raw = text()
      if (!raw) return
      root.applyColors(raw)
    }
    onFileChanged: reload()
  }

  property FileView shellFile: FileView {
    path: root.currentThemePath + "/shell.toml"
    watchChanges: true
    printErrors: false
    onLoaded: {
      var raw = text()
      if (!raw) return
      root.themeShellValues = ThemeJs.parseShell(raw)
      root.mergeShell()
    }
    onLoadFailed: {
      root.themeShellValues = ({})
      root.mergeShell()
    }
    onFileChanged: reload()
  }

  property FileView userShellFile: FileView {
    path: root.userShellPath
    watchChanges: true
    printErrors: false
    onLoaded: {
      root.userShellValues = ThemeJs.parseShell(text())
      root.mergeShell()
    }
    onLoadFailed: {
      root.userShellValues = ({})
      root.mergeShell()
    }
    onFileChanged: reload()
  }
}
