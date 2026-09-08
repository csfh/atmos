import QtQuick
import QtQuick.Dialogs
import Quickshell
import "../../components"
import "../../services"
import "../../services/Diagnostics.js" as DiagJs
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "system/diagnostics"
  title: "Diagnostics"
  description: "What this machine is doing, in one place. Copy report is the text to paste on Discord instead of twelve shell commands."

  readonly property var diag: DiagJs.normalize(Omarchy.diagnostics)
  readonly property bool reportBusy: Omarchy.jobKind === "diag-report" && Omarchy.jobBusy
  property string reportStatus: ""
  property string savedPath: ""

  function healthLabel(ok) {
    return DiagJs.statusLabel(ok)
  }

  function openSave() {
    saveDialog.currentFile = root.folderUrl(root.savedPath) + "/" + DiagJs.reportFileName(Omarchy.hostname, new Date())
    saveDialog.open()
  }

  function folderUrl(path) {
    var text = String(path || "")
    var slash = text.lastIndexOf("/")
    var dir = slash > 0 ? text.substring(0, slash) : (Omarchy.picturesDir || "")
    if (!dir) dir = Quickshell.env("HOME") || ""
    return "file://" + dir
  }

  PrefsGroup {
    title: "Report"
    query: root.query
    detail: "Copy puts an Atmos summary plus omarchy debug --no-sudo --print on the clipboard. Save writes the same file. Ask my Agent copies it, then opens the default coding agent with the path."

    SettingRow {
      label: "Copy report"
      description: root.reportBusy
        ? "Building the report…"
        : (root.reportStatus
          ? root.reportStatus
          : "A text dump of Omarchy, Atmos, Hyprland, systemd, disk, memory, and recent errors.")
      hint: "omarchy debug --no-sudo --print"
      query: root.query
      keywords: ["copy", "report", "discord", "debug", "paste", "clipboard"]

      PrefsButton {
        text: root.reportBusy ? "Building…" : "Copy"
        primary: true
        enabled: !root.reportBusy
        onClicked: {
          root.reportStatus = ""
          Omarchy.copyDiagnosticReport()
        }
      }
    }

    SettingRow {
      label: "Save report"
      description: root.savedPath.length
        ? ("Last file: " + root.savedPath + ".")
        : "Write the same report to a text file."
      hint: "~/.cache/atmos/diagnostic-report.txt"
      query: root.query
      keywords: ["save", "file", "export", "txt"]

      PrefsButton {
        text: "Save…"
        enabled: !root.reportBusy
        onClicked: root.openSave()
      }
    }

    SettingRow {
      label: "Ask my Agent"
      description: "Copy the report, then open the default coding agent with that file."
      hint: "omarchy agent prompt"
      query: root.query
      keywords: ["agent", "diagnose", "help", "grok", "claude"]

      PrefsButton {
        text: "Ask…"
        enabled: !root.reportBusy
        onClicked: {
          root.reportStatus = ""
          Omarchy.askAgentAboutDiagnostics()
        }
      }
    }

    SettingRow {
      label: "Refresh"
      description: "Read the machine again. Failed units and Hyprland errors change while it runs."
      hint: "snapshot"
      query: root.query
      keywords: ["reload", "rescan"]

      PrefsButton {
        text: "Refresh"
        onClicked: Omarchy.refresh()
      }
    }
  }

  PrefsGroup {
    title: "Omarchy"
    query: root.query
    detail: "Package version and channel. Copy on System still copies the version string alone."

    SettingRow {
      label: Omarchy.omarchyVersion.length ? ("Omarchy " + Omarchy.omarchyVersion) : "Omarchy"
      description: DiagJs.omarchySummary(root.diag)
      hint: "omarchy version"
      query: root.query
      keywords: ["omarchy", "version", "channel", "health"]
      valueText: Omarchy.omarchyVersion.length ? "OK" : "Needs attention"
    }
  }

  PrefsGroup {
    title: "Atmos"
    query: root.query
    detail: "The drop-in require in hyprland.lua is what keeps this window floating. Sentinels are the blocks Atmos wrote."

    SettingRow {
      label: Omarchy.atmosRevision.length ? Omarchy.atmosRevision : "Atmos"
      description: DiagJs.atmosSummary(root.diag)
      hint: "~/.config/hypr/hyprland.lua"
      query: root.query
      keywords: ["atmos", "revision", "sentinel", "hypr.atmos", "drop-in"]
      valueText: root.healthLabel(DiagJs.atmosOk(root.diag))
    }
  }

  PrefsGroup {
    title: "Hyprland"
    query: root.query
    detail: "hyprctl version and configerrors. A dirty config still runs; errors here are why a bind or monitor rule did not apply."

    SettingRow {
      label: root.diag.hyprland.version ? ("Hyprland " + root.diag.hyprland.version) : "Hyprland"
      description: DiagJs.hyprSummary(root.diag)
      hint: "hyprctl configerrors"
      query: root.query
      keywords: ["hyprland", "compositor", "configerrors", "reload"]
      valueText: root.healthLabel(DiagJs.hyprOk(root.diag))
    }

    Repeater {
      model: root.diag.hyprland.configErrors

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: "Config error"
        description: String(modelData || "")
        hint: "hyprctl configerrors"
        query: root.query
        keywords: ["error", "hypr", "lua"]
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Failed units"
    query: root.query
    detail: "systemctl --failed and the same list for the user session. Start and stop stay on a later Services page. This list is read-only."

    SettingRow {
      available: DiagJs.failedCount(root.diag) === 0
      sectionHelp: false
      label: "Failed units"
      description: "No failed units."
      hint: "systemctl --failed"
      query: root.query
      keywords: ["systemd", "failed", "empty"]
      valueText: "OK"
    }

    Repeater {
      model: root.diag.failedUnits

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: modelData && modelData.unit ? modelData.unit : "unit"
        description: (modelData && modelData.scope ? modelData.scope + ". " : "") + (modelData && modelData.description ? modelData.description : "Failed.")
        hint: "systemctl --failed"
        query: root.query
        keywords: ["systemd", "failed", "unit"]
        valueText: "Failed"
      }
    }
  }

  PrefsGroup {
    title: "Disk"
    query: root.diag.disk.total > 0 ? root.query : "."
    detail: "Use on the root filesystem, from statvfs. The Disks hub has per-drive detail."

    SettingRow {
      available: root.diag.disk.total > 0
      stretchControl: true
      label: "Root"
      description: root.diag.disk.percent >= 90 ? "This filesystem is almost full." : ""
      hint: "/"
      query: root.query
      keywords: ["disk", "df", "space", "pressure"]
      valueText: root.healthLabel(!DiagJs.diskPressure(root.diag))

      PrefsUsageBar {
        width: parent.width
        used: root.diag.disk.used
        size: root.diag.disk.total
        avail: root.diag.disk.available
      }
    }
  }

  PrefsGroup {
    title: "Memory"
    query: root.diag.memory.total > 0 ? root.query : "."
    detail: "RAM and swap from /proc/meminfo. Hardware has the DIMM list."

    SettingRow {
      available: root.diag.memory.total > 0
      stretchControl: true
      label: "RAM"
      description: ""
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["memory", "ram", "pressure"]
      valueText: root.healthLabel(!DiagJs.memoryPressure(root.diag))

      PrefsUsageBar {
        width: parent.width
        used: root.diag.memory.used
        size: root.diag.memory.total
        avail: root.diag.memory.available
      }
    }

    SettingRow {
      available: root.diag.memory.swapTotal > 0
      stretchControl: true
      label: "Swap"
      description: ""
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["swap", "zram"]

      PrefsUsageBar {
        width: parent.width
        used: root.diag.memory.swapUsed
        size: root.diag.memory.swapTotal
        avail: Math.max(0, root.diag.memory.swapTotal - root.diag.memory.swapUsed)
      }
    }
  }

  PrefsGroup {
    title: "Kernel"
    query: root.query
    detail: "uname. The Hardware hub has firmware and DMI."

    SettingRow {
      label: "Kernel"
      description: DiagJs.kernelSummary(root.diag) || "Kernel version was not readable."
      hint: "uname -sr"
      query: root.query
      keywords: ["kernel", "uname", "linux", "arch"]

      PrefsButton {
        text: "Copy"
        enabled: DiagJs.kernelSummary(root.diag).length > 0
        onClicked: Omarchy.copyText(DiagJs.kernelSummary(root.diag))
      }
    }
  }

  PrefsGroup {
    title: "GPU"
    query: root.query
    detail: "The DRM driver name. Drivers has the PCI device list and hybrid GPU switching."

    SettingRow {
      label: "Driver"
      description: DiagJs.gpuSummary(root.diag)
      hint: "/sys/class/drm"
      query: root.query
      keywords: ["gpu", "drm", "amdgpu", "nvidia", "i915", "xe"]

      PrefsButton {
        text: "Copy"
        enabled: DiagJs.gpuSummary(root.diag).length > 0
        onClicked: Omarchy.copyText(DiagJs.gpuSummary(root.diag))
      }
    }
  }

  PrefsGroup {
    title: "Portals"
    query: root.query
    detail: "xdg-desktop-portal and the Hyprland and GTK backends. Screen sharing and file pickers need these."

    SettingRow {
      label: "Portals"
      description: DiagJs.serviceMapSummary(root.diag.portals)
      hint: "systemctl --user status xdg-desktop-portal"
      query: root.query
      keywords: ["portal", "xdg", "screencast", "pipewire"]
      valueText: root.healthLabel(DiagJs.servicesOk(root.diag.portals))
    }
  }

  PrefsGroup {
    title: "PipeWire"
    query: root.query
    detail: "The session audio graph. Restart is on Sound."

    SettingRow {
      label: "PipeWire"
      description: DiagJs.serviceMapSummary(root.diag.pipewire)
      hint: "systemctl --user status pipewire"
      query: root.query
      keywords: ["pipewire", "pulse", "wireplumber", "audio"]
      valueText: root.healthLabel(DiagJs.servicesOk(root.diag.pipewire))
    }
  }

  PrefsGroup {
    title: "Network"
    query: root.query
    detail: "Whether this machine has a default route. Join a network on the Network hub."

    SettingRow {
      label: "Connectivity"
      description: DiagJs.networkSummary(root.diag)
      hint: "/proc/net/route"
      query: root.query
      keywords: ["network", "online", "route", "offline"]
      valueText: root.healthLabel(root.diag.network.online === true)
    }
  }

  PrefsGroup {
    title: "Packages"
    query: root.query
    detail: "pacman sync databases under /var/lib/pacman/sync. Updates stay on System."

    SettingRow {
      label: "Package database"
      description: DiagJs.pacmanSummary(root.diag)
      hint: "/var/lib/pacman/sync"
      query: root.query
      keywords: ["pacman", "sync", "mirrors", "database"]
      valueText: root.healthLabel(root.diag.pacman.syncOk === true)
    }
  }

  PrefsGroup {
    framed: true
    title: "Recent errors"
    query: root.query
    detail: "journalctl -b -p 3, last fifteen lines. Lines that look like passwords are dropped."

    SettingRow {
      available: root.diag.recentErrors.length === 0
      sectionHelp: false
      label: "Errors"
      description: "No recent error-priority journal lines."
      hint: "journalctl -b -p 3"
      query: root.query
      keywords: ["journal", "empty"]
      valueText: "OK"
    }

    Repeater {
      model: root.diag.recentErrors

      SettingRow {
        required property var modelData
        sectionHelp: false
        label: "Journal"
        description: String(modelData || "")
        hint: "journalctl -b -p 3"
        query: root.query
        keywords: ["journal", "error", "log"]
      }
    }
  }

  PrefsGroup {
    title: "Crash capture"
    query: root.query
    detail: "When a process dumps core, Omarchy can notify you so a coding agent can look at the crash."

    SettingRow {
      label: "Crash capture"
      description: "A notification when a process crashes, so a coding agent can look at the dump."
      hint: "omarchy toggle crash capture"
      query: root.query
      keywords: ["crash", "coredump", "agent", "watch"]

      PrefsToggle {
        checked: Omarchy.crashCapture
        onToggled: Omarchy.setCrashCapture(!Omarchy.crashCapture)
      }
    }
  }

  FileDialog {
    id: saveDialog
    title: "Save diagnostic report"
    fileMode: FileDialog.SaveFile
    nameFilters: ["Text files (*.txt)", "All files (*)"]
    onAccepted: {
      var path = RichUi.pathFromUrl(selectedFile)
      if (!path) return
      if (path.length < 5 || path.substring(path.length - 4) !== ".txt")
        path = path + ".txt"
      root.savedPath = path
      root.reportStatus = ""
      Omarchy.saveDiagnosticReport(path)
    }
  }

  Connections {
    target: Omarchy
    function onJobBusyChanged() {
      if (Omarchy.jobBusy) return
      if (Omarchy.jobKind !== "diag-report") return
      if (Omarchy.lastError)
        root.reportStatus = "Could not build the report."
      else if (root.savedPath.length)
        root.reportStatus = "Wrote " + root.savedPath + "."
      else
        root.reportStatus = "Copied the report."
    }
  }
}
