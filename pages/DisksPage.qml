import QtQuick
import Quickshell
import Quickshell.Io
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Disks"
  description: "Space on each drive, plus encryption if the disk is locked. Snapper snapshots and hibernation are further down."

  property bool diskRunning: false
  property bool diskExpectedStop: false
  property string diskPhase: ""
  property string diskName: ""
  property string diskRead: ""
  property string diskWrite: ""
  property string diskError: ""
  property string diskTarget: ""
  property var diskResults: ({})
  property string pendingRollbackConfig: ""
  property int pendingRollbackId: 0
  property string pendingLuksDevice: ""

  function diskTitle(disk) {
    var model = String((disk && disk.model) || "")
    var name = String((disk && disk.name) || "disk")
    return model.length ? model : name
  }

  function speedtestDir(disk) {
    var mounts = disk && disk.mounts
    if (!(mounts instanceof Array)) return ""
    for (var i = 0; i < mounts.length; i++) {
      var t = String(mounts[i].target || "")
      if (!t || t === "/boot" || t.indexOf("/boot/") === 0) continue
      return t
    }
    return ""
  }

  function diskResult(dir) {
    return (dir && root.diskResults[dir]) || null
  }

  function setDiskResult(dir, patch) {
    if (!dir) return
    var copy = {}
    var keys = Object.keys(root.diskResults)
    for (var i = 0; i < keys.length; i++)
      copy[keys[i]] = root.diskResults[keys[i]]
    var cur = copy[dir] || { name: "", read: "", write: "", error: "" }
    copy[dir] = {
      name: patch.name !== undefined ? patch.name : cur.name,
      read: patch.read !== undefined ? patch.read : cur.read,
      write: patch.write !== undefined ? patch.write : cur.write,
      error: patch.error !== undefined ? patch.error : cur.error
    }
    root.diskResults = copy
  }

  function snapperSummary() {
    var list = Omarchy.snapperConfigs
    if (!(list instanceof Array) || list.length === 0)
      return "Snapper is installed, but no configs were readable."
    var parts = []
    for (var i = 0; i < list.length; i++) {
      var name = String(list[i].name || "")
      var sub = String(list[i].subvolume || "")
      if (name) parts.push(sub ? (name + " on " + sub) : name)
    }
    return "Configs: " + parts.join(", ") + "."
  }

  function snapshotLabel(item) {
    if (!item) return "Snapshot"
    var bits = ["#" + item.id]
    if (item.date) bits.push(item.date)
    if (item.description) bits.push(item.description)
    if (item.config) bits.push("(" + item.config + ")")
    return bits.join("  ")
  }

  function startDiskSpeedtest(dir) {
    if (diskProc.running) return
    diskError = ""
    diskName = ""
    diskRead = ""
    diskWrite = ""
    diskPhase = "read"
    diskRunning = true
    diskExpectedStop = false
    diskTarget = dir || ""
    if (diskTarget)
      setDiskResult(diskTarget, { name: "", read: "", write: "", error: "" })
    if (dir && Omarchy.validMountPath(dir))
      diskProc.command = ["omarchy", "disk", "speedtest", dir]
    else
      diskProc.command = ["omarchy", "disk", "speedtest"]
    diskProc.running = true
  }

  function stopDiskSpeedtest() {
    diskExpectedStop = true
    diskRunning = false
    diskPhase = ""
    if (diskProc.running) diskProc.running = false
  }

  Component.onDestruction: {
    if (diskProc.running) diskProc.running = false
  }

  Process {
    id: diskProc
    stdout: SplitParser {
      onRead: function(line) {
        var parsed = RichUi.parseDiskSpeedLine(line)
        if (!parsed) return
        if (parsed.kind === "disk") {
          root.diskName = String(parsed.value)
          root.setDiskResult(root.diskTarget, { name: root.diskName })
        } else if (parsed.kind === "read") {
          root.diskPhase = "read"
          root.diskRead = String(parsed.value)
          root.setDiskResult(root.diskTarget, { read: root.diskRead })
        } else if (parsed.kind === "write") {
          root.diskPhase = "write"
          root.diskWrite = String(parsed.value)
          root.setDiskResult(root.diskTarget, { write: root.diskWrite })
        }
      }
    }
    stderr: StdioCollector {
      id: diskErr
      waitForEnd: true
    }
    onExited: function(code) {
      if (root.diskExpectedStop) {
        root.diskExpectedStop = false
        root.diskRunning = false
        root.diskPhase = ""
        return
      }
      root.diskRunning = false
      root.diskPhase = ""
      if (code !== 0) {
        root.diskError = String(diskErr.text || "Disk speed test failed").replace(/^\s+|\s+$/g, "")
        root.setDiskResult(root.diskTarget, { error: root.diskError })
      } else {
        root.setDiskResult(root.diskTarget, { error: "" })
      }
    }
  }

  PrefsConfirm {
    id: createConfirm
    title: "Create snapshot"
    message: "Take a numbered Snapper snapshot of each config, then run cleanup. You can roll back to it later."
    confirmText: "Create"
    onConfirmed: Omarchy.createSnapshot()
  }

  PrefsConfirm {
    id: rollbackConfirm
    title: "Roll back"
    message: "Snapper will make this snapshot the new default. Reboot after it finishes so the machine boots into it."
    confirmText: "Roll back"
    onConfirmed: Omarchy.restoreSnapshot(root.pendingRollbackConfig, String(root.pendingRollbackId))
  }

  PrefsConfirm {
    id: hibernateSetupConfirm
    title: "Set up hibernation"
    message: "Write a RAM-sized swap file on the boot drive and set up resume. This uses sudo."
    confirmText: "Set up"
    onConfirmed: Omarchy.setupHibernation()
  }

  PrefsConfirm {
    id: hibernateRemoveConfirm
    title: "Remove hibernation"
    message: "Remove the hibernation swap file and the boot resume settings that go with it."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeHibernation()
  }

  PrefsDialog {
    id: luksDialog
    title: "Change encryption password"

    PrefsText {
      width: parent.width
      text: "New passphrase for " + root.pendingLuksDevice + ". You will need the current one."
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsPassword {
      id: luksCurrent
      width: parent.width
      placeholder: "Current passphrase"
    }
    PrefsPassword {
      id: luksNew
      width: parent.width
      placeholder: "New passphrase"
    }
    PrefsPassword {
      id: luksConfirm
      width: parent.width
      placeholder: "Confirm new passphrase"
    }

    Text {
      id: luksHint
      width: parent.width
      visible: text.length > 0
      text: ""
      color: Theme.urgent
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    Row {
      spacing: Theme.space
      PrefsButton {
        text: "Cancel"
        onClicked: luksDialog.close()
      }
      PrefsButton {
        text: "Change"
        primary: true
        enabled: !Omarchy.jobBusy
        onClicked: {
          var a = luksNew.currentText()
          var b = luksConfirm.currentText()
          if (!luksCurrent.currentText() || !a) {
            luksHint.text = "Passphrases cannot be empty."
            return
          }
          if (a !== b) {
            luksHint.text = "New passphrases do not match."
            return
          }
          luksHint.text = ""
          Omarchy.changeDrivePassword(root.pendingLuksDevice, luksCurrent.currentText(), a)
          luksCurrent.clear()
          luksNew.clear()
          luksConfirm.clear()
          luksDialog.close()
        }
      }
    }
  }

  Repeater {
    model: Omarchy.disks

    PrefsGroup {
    framed: true
      required property var modelData
      title: root.diskTitle(modelData)
      query: root.query
      detail: "Usage bars come from df on each mount. Speed test reads for about eight seconds, then writes for about eight, on this disk."
      hint: "df"

      SettingRow {
        label: "Drive"
        description: String((modelData && modelData.info) || "")
        hint: "omarchy drive info"
        query: root.query
        keywords: ["nvme", "ssd", "sata", "lsblk", "model"]

        PrefsButton {
          text: "Copy"
          enabled: !!(modelData && modelData.info)
          onClicked: Omarchy.copyText(String(modelData.info || ""))
        }
      }

      Repeater {
        model: modelData && modelData.mounts

        SettingRow {
          required property var modelData
          stretchControl: true
          sectionHelp: false
          label: modelData && modelData.target ? modelData.target : "Mount"
          description: ""
          hint: "df"
          query: root.query
          keywords: ["usage", "capacity", "btrfs", "ext4", "vfat", "home", "boot"]

          PrefsUsageBar {
            width: parent.width
            used: modelData && modelData.used ? modelData.used : 0
            size: modelData && modelData.size ? modelData.size : 0
            avail: modelData && modelData.avail ? modelData.avail : 0
          }
        }
      }

      SettingRow {
        stretchControl: true
        label: "Speed test"
        description: {
          var dir = root.speedtestDir(modelData)
          var active = root.diskRunning && root.diskTarget === dir
          if (active)
            return root.diskName ? ("Testing " + root.diskName + ".") : "Reading, then writing, for about eight seconds each."
          var last = root.diskResult(dir)
          if (last && (last.read || last.write)) {
            var bits = []
            if (last.name) bits.push(last.name)
            bits.push("last run")
            return bits.join(" · ") + "."
          }
          return "Read for about eight seconds, then write for about eight, on this disk."
        }
        hint: "omarchy disk speedtest"
        query: root.query
        keywords: ["benchmark", "mbps", "performance"]

        Column {
          width: parent.width
          spacing: Theme.space
          Row {
            spacing: Theme.space
            PrefsButton {
              text: root.diskRunning && root.diskTarget === root.speedtestDir(modelData) ? "Cancel" : "Run now"
              onClicked: {
                var dir = root.speedtestDir(modelData)
                if (root.diskRunning && root.diskTarget === dir) root.stopDiskSpeedtest()
                else if (!root.diskRunning) root.startDiskSpeedtest(dir)
              }
            }
          }
          PrefsProgress {
            width: parent.width
            visible: root.diskRunning && root.diskTarget === root.speedtestDir(modelData)
            indeterminate: true
            valueText: "Read " + (root.diskRead || "—") + " MB/s   Write " + (root.diskWrite || "—") + " MB/s"
          }
          PrefsText {
            width: parent.width
            visible: {
              var dir = root.speedtestDir(modelData)
              if (root.diskRunning && root.diskTarget === dir) return false
              var last = root.diskResult(dir)
              return !!(last && (last.read || last.write))
            }
            text: {
              var last = root.diskResult(root.speedtestDir(modelData))
              if (!last) return ""
              return "Read " + (last.read || "—") + " MB/s   Write " + (last.write || "—") + " MB/s"
            }
            wrapMode: Text.NoWrap
            font.family: Theme.fontFamily
            font.pixelSize: Theme.captionSize
            color: Theme.foreground
          }
          Text {
            visible: {
              var dir = root.speedtestDir(modelData)
              var last = root.diskResult(dir)
              var err = (root.diskRunning && root.diskTarget === dir) ? root.diskError : (last ? last.error : "")
              return String(err || "").length > 0
            }
            text: {
              var dir = root.speedtestDir(modelData)
              var last = root.diskResult(dir)
              if (root.diskTarget === dir && root.diskError.length) return root.diskError
              return last && last.error ? last.error : ""
            }
            color: Theme.urgent
            font.family: Theme.fontFamily
            font.pixelSize: Theme.captionSize
          }
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Encryption"
    query: Omarchy.luksDevices.length > 0 ? root.query : "."
    detail: "LUKS volumes on this machine. Change the passphrase here. You will need the current one."

    Repeater {
      model: Omarchy.luksDevices

      SettingRow {
        required property var modelData
        available: true
        label: String(modelData || "LUKS")
        description: "This volume is encrypted with LUKS. Change the passphrase here. You will need the current one."
        hint: "cryptsetup luksChangeKey"
        query: root.query
        keywords: ["luks", "cryptsetup", "encryption", "passphrase"]

        PrefsButton {
          text: "Change…"
          enabled: !Omarchy.jobBusy
          onClicked: {
            root.pendingLuksDevice = String(modelData || "")
            luksDialog.open()
          }
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Snapshots"
    query: Omarchy.snapperPresent ? root.query : "."
    detail: "Each listed snapshot can become the new Snapper default. Rollback asks first. Reboot after it finishes."
    hint: "snapper rollback"

    SettingRow {
      available: Omarchy.snapperPresent
      label: "Snapper"
      description: root.snapperSummary()
      hint: "omarchy snapshot"
      query: root.query
      keywords: ["btrfs", "limine", "restore", "rollback"]

      PrefsButton {
        text: "Copy"
        enabled: root.snapperSummary().length > 0
        onClicked: Omarchy.copyText(root.snapperSummary())
      }
    }

    SettingRow {
      available: Omarchy.snapperPresent
      label: "Create snapshot"
      description: Omarchy.jobKind === "snapshot-create" && Omarchy.jobBusy
        ? "Creating snapshots…"
        : "Take a numbered snapshot of each Snapper config, then run cleanup."
      hint: "omarchy snapshot create"
      query: root.query
      keywords: ["snapper", "create"]

      PrefsButton {
        text: "Create…"
        primary: true
        enabled: !Omarchy.jobBusy && Omarchy.snapperPresent
        onClicked: createConfirm.ask()
      }
    }

    SettingRow {
      available: Omarchy.snapperPresent && Omarchy.snapshots.length === 0
      sectionHelp: false
      label: "Snapshots"
      description: "No snapshots."
      query: root.query
      keywords: ["empty", "snapshot"]
    }

    Repeater {
      model: Omarchy.snapshots

      SettingRow {
        required property var modelData
        available: Omarchy.snapperPresent
        sectionHelp: false
        label: root.snapshotLabel(modelData)
        description: "Make this snapshot the new Snapper default, then reboot into it."
        hint: "snapper rollback"
        query: root.query
        keywords: ["rollback", "restore", "limine"]

        PrefsButton {
          text: "Roll back…"
          danger: true
          enabled: !Omarchy.jobBusy && modelData && modelData.id
          onClicked: {
            root.pendingRollbackConfig = String(modelData.config || "root")
            root.pendingRollbackId = modelData.id
            rollbackConfirm.ask()
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Hibernation"
    query: root.query
    detail: "Hibernation writes RAM to a swap file and resumes from it on the next boot. Set up needs sudo."

    SettingRow {
      label: "Hibernation"
      description: (Omarchy.jobKind === "hibernation-setup" || Omarchy.jobKind === "hibernation-remove") && Omarchy.jobBusy
        ? (Omarchy.jobKind === "hibernation-remove" ? "Removing hibernation…" : "Setting up hibernation…")
        : (Omarchy.hibernationConfigured
          ? "Set up. Remove deletes the swap file and the boot resume settings."
          : (Omarchy.hibernationSupported
            ? "This machine can hibernate. Set up writes a swap file and the boot resume settings."
            : "Not available on this machine. Set up stays disabled."))
      hint: "omarchy hibernation available"
      query: root.query
      keywords: ["sleep", "swap", "resume"]

      PrefsButton {
        text: Omarchy.hibernationConfigured ? "Remove…" : "Set up…"
        primary: !Omarchy.hibernationConfigured && Omarchy.hibernationSupported
        danger: Omarchy.hibernationConfigured
        enabled: !Omarchy.jobBusy && (Omarchy.hibernationConfigured || Omarchy.hibernationSupported)
        onClicked: {
          if (Omarchy.hibernationConfigured) hibernateRemoveConfirm.ask()
          else hibernateSetupConfirm.ask()
        }
      }
    }


  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "How many Snapper snapshots to keep, whether hourly snapshots run, and weekly SSD TRIM."

    SettingRow {
      available: Omarchy.snapperPresent
      label: "Keep snapshots"
      description: "How many numbered Snapper snapshots the root config keeps before cleanup."
      hint: "/etc/snapper/configs/root · NUMBER_LIMIT"
      query: root.query
      keywords: ["snapper", "retention", "limit", "number"]

      PrefsSliderStepper {
        from: 1
        to: 20
        stepSize: 1
        value: Omarchy.snapperNumberLimit
        valueText: String(Omarchy.snapperNumberLimit)
        enabled: Omarchy.snapperPresent
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.snapperNumberLimit)
            Omarchy.setSnapperNumberLimit(next)
        }
      }
    }

    SettingRow {
      available: Omarchy.snapperPresent
      label: "Timeline snapshots"
      description: "Hourly snapshots, cleaned on a timer. That uses disk quickly."
      hint: "/etc/snapper/configs/root · TIMELINE_CREATE"
      query: root.query
      keywords: ["snapper", "timeline", "hourly", "timer"]

      PrefsToggle {
        checked: Omarchy.snapperTimeline
        enabled: Omarchy.snapperPresent
        onToggled: Omarchy.setSnapperTimeline(!Omarchy.snapperTimeline)
      }
    }

    SettingRow {
      label: "Weekly TRIM"
      description: "The SSD reclaims unused blocks once a week."
      hint: "systemctl enable fstrim.timer"
      query: root.query
      keywords: ["fstrim", "trim", "ssd", "discard"]

      PrefsToggle {
        checked: Omarchy.fstrimEnabled
        onToggled: Omarchy.setFstrim(!Omarchy.fstrimEnabled)
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Swap"
    query: Omarchy.swapDevices.length > 0 ? root.query : "."
    detail: "Swap devices this machine is using, including zram when the kernel has it loaded."

    Repeater {
      model: Omarchy.swapDevices

      SettingRow {
        required property var modelData
        label: String((modelData && (modelData.label || modelData.name)) || "swap")
        description: modelData && modelData.path
          ? (modelData.path + ". " + RichUi.formatBytes(modelData.size) + ".")
          : RichUi.formatBytes(modelData && modelData.size)
        hint: "lsblk"
        query: root.query
        keywords: ["zram", "swap"]

        PrefsButton {
          text: "Copy"
          enabled: !!(modelData && (modelData.path || modelData.name || modelData.label))
          onClicked: Omarchy.copyText(String((modelData && (modelData.path || modelData.name || modelData.label)) || ""))
        }
      }
    }
  }
}
