pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "Theme.js" as ThemeJs

QtObject {
  id: root

  readonly property string home: Quickshell.env("HOME")
  readonly property string currentDir: home + "/.local/state/omarchy/current"
  readonly property string currentThemePath: currentDir + "/theme"
  readonly property string currentThemeNameFile: currentDir + "/theme.name"
  readonly property string userShellPath: home + "/.config/omarchy/shell.toml"

  property color foreground: "#cacccc"
  property color background: "#101315"
  property color accent: "#cacccc"
  property color muted: "#707880"
  property color urgent: "#a55555"

  property var themeShellValues: ({})
  property var userShellValues: ({})
  property var shellValues: ({})

  // Visual language. Monospace, grayscale chrome, 1px hairlines, radius 0,
  // dense native controls, a capped content column, a persistent sidebar.
  // Tokens record the live look. Do not round cards, add shadows, or grow
  // the column because the window is wide.

  readonly property string fontFamily: "monospace"
  readonly property int fontSize: Math.max(9, Math.round(ThemeJs.numberToken(root.shellValues, "font.base-size", 12)))
  readonly property int titleSize: Math.max(fontSize + 6, Math.round(fontSize * 1.4))
  readonly property int captionSize: Math.max(11, fontSize - 2)
  readonly property int pageTitleSize: titleSize
  readonly property int pageDescriptionSize: fontSize
  readonly property int embedTitleSize: fontSize + 2
  readonly property int sectionSize: captionSize
  readonly property int labelSize: fontSize
  readonly property int descriptionSize: captionSize
  readonly property int metaSize: captionSize
  readonly property real sectionTracking: 1.2
  readonly property real metaOpacity: 0.7

  // Remix Icon names under icons/<name>.svg.
  readonly property string iconChevronLeft: "arrow-left-s-line"
  readonly property string iconChevronRight: "arrow-right-s-line"
  readonly property string iconInfo: "information-line"
  readonly property int radius: 0
  readonly property real normalFill: ThemeJs.numberToken(root.shellValues, "controls.normal-fill-alpha", 0.04)
  readonly property real hoverFill: ThemeJs.numberToken(root.shellValues, "controls.hover-cursor-fill-alpha", 0.08)
  readonly property real selectedFill: ThemeJs.numberToken(root.shellValues, "controls.selected-fill-alpha", 0.18)
  readonly property real borderAlpha: ThemeJs.numberToken(root.shellValues, "controls.normal-border-alpha", 0.4)
  readonly property real primaryFill: 0.22
  readonly property real toggleOffFill: 0.12
  readonly property real scrimAlpha: 0.55

  readonly property int pad: Math.max(8, Math.round(fontSize * 0.9))
  readonly property int gap: Math.max(6, Math.round(fontSize * 0.6))
  readonly property int space: 8
  readonly property int spaceMd: 12
  readonly property int spaceLg: 20
  readonly property int rowPad: 10
  // Shared left edge for page title, section heading, and setting label.
  readonly property int copyInset: rowPad
  readonly property int pageMargin: spaceLg
  readonly property int sectionSpacing: spaceLg * 2
  readonly property int headingGap: space
  readonly property int titleGap: 4
  readonly property int labelGap: 4
  readonly property int stackGap: 6
  readonly property int controlGap: space
  readonly property int sidebarItemSpacing: 2
  readonly property int sidebarGroupSpacing: spaceMd

  readonly property int rowHeight: Math.max(36, fontSize + spaceMd * 2)
  readonly property int controlHeight: Math.max(28, rowHeight - 8)
  readonly property int toggleWidth: 44
  readonly property int toggleHeight: 24
  readonly property int toggleThumb: 16
  readonly property int checkSize: 16
  readonly property int radioSize: 16
  readonly property int sliderTrack: 22
  readonly property int sliderBar: 4
  readonly property int sliderHandle: 14
  readonly property int sliderTickGap: 4
  readonly property int sliderCompactWidth: 162
  readonly property int fieldInset: 10
  readonly property int fieldWidth: 200
  readonly property int spinWidth: 108
  readonly property int helpHit: 22
  readonly property int helpIcon: 16
  readonly property int navIconSize: fontSize + 2
  readonly property int railWidth: 3
  readonly property int borderWidth: 1
  readonly property real disabledOpacity: 0.45
  readonly property int motionFast: 90
  readonly property int motionMed: 120
  readonly property int motionNav: 180

  readonly property int sidebarWidth: 220
  readonly property int contentMaxWidth: 1000
  readonly property int controlColumnWidth: 280
  readonly property int dialogWidth: 520
  readonly property int confirmWidth: 420
  readonly property int overlayInset: 48

  function controlOpacity(on) {
    return on ? 1 : disabledOpacity
  }

  function contentColumnWidth(avail) {
    var w = Number(avail)
    if (!isFinite(w)) w = 0
    return Math.max(240, Math.min(contentMaxWidth, w - pageMargin * 2))
  }

  function contentColumnX(avail, colWidth) {
    var outer = Number(avail)
    var inner = Number(colWidth)
    if (!isFinite(outer)) outer = 0
    if (!isFinite(inner)) inner = 0
    return Math.max(pageMargin, Math.round((outer - inner) / 2))
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

  function currentThemeSlug() {
    return String(root.readPath(root.currentThemeNameFile) || "").replace(/^\s+|\s+$/g, "")
  }

  // The bar watches ~/.local/state/omarchy/current (the directory), not
  // theme.name. omarchy-theme-set replaces the theme directory, then writes
  // theme.name, then retargets the background symlink. Debounce those events
  // so colors.toml exists before we read it.
  function handleCurrentChanged() {
    var slug = root.currentThemeSlug()
    var colors = root.readPath(root.currentThemePath + "/colors.toml")
    if (!colors && root.currentDirTries < 8) {
      root.currentDirTries++
      currentDirDebounce.interval = 40
      currentDirDebounce.restart()
      return
    }
    root.currentDirTries = 0
    currentDirDebounce.interval = 80
    if (slug) root.applyNamedTheme(slug)
    if (colors) root.applyColors(colors)
    var shellRaw = root.readPath(root.currentThemePath + "/shell.toml")
    if (shellRaw) {
      root.themeShellValues = ThemeJs.parseShell(shellRaw)
      root.mergeShell()
    }
    root.reopenThemeFiles()
    if (slug) root.currentThemeSwapped(slug)
  }

  signal currentThemeSwapped(string slug)

  property int currentDirTries: 0

  property Timer currentDirDebounce: Timer {
    interval: 80
    repeat: false
    onTriggered: root.handleCurrentChanged()
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
    onFileChanged: currentDirDebounce.restart()
  }

  property FileView colorsFile: FileView {
    path: root.currentThemePath + "/colors.toml"
    watchChanges: true
    preload: true
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
    preload: true
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
    preload: true
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
