import QtQuick
import QtQuick.Dialogs
import Quickshell
import Quickshell.Io
import "../components"
import "../services"
import "../services/Settings.js" as SettingsJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Import and export"
  description: "Write this machine's settings to a Markdown file you can read, keep, or hand to someone else. Importing shows you every change before anything happens."

  readonly property string home: Quickshell.env("HOME")
  readonly property string applyScript: Omarchy.shellDir + "/scripts/apply-settings.sh"
  // Regenerated on use rather than held, so the stamp is the moment you
  // exported and not the moment the page happened to load.
  function suggestedName() {
    return SettingsJs.exportFileName(Omarchy.hostname, new Date())
  }

  // Section id -> included. Seeded from the catalog, so a section added
  // there appears here without this page being told about it.
  property var sections: ({})

  property string exportPath: ""
  property string importPath: ""
  property string exportStatus: ""
  property string importStatus: ""
  property string writtenPath: ""
  property var plan: null
  property var lastDoc: null
  // Set once an import has run, so the way back is offered where you are
  // looking rather than buried in a row further down.
  property int appliedCount: 0
  // Bound to the backup path the apply JSON just returned, not the newest
  // directory on disk.
  property string lastBackupDir: ""
  property bool lastUndoNeedsRoot: false
  property bool pendingApply: false
  property bool undoing: false
  // Progress, streamed from runJob as each change is reached.
  property int stepNow: 0
  property int stepTotal: 0
  property string stepKey: ""

  readonly property bool applyingJob: Omarchy.jobKind === "settings-import" || Omarchy.jobKind === "settings-undo"
  // Derived rather than assigned, so it cannot be left stuck on by a path
  // that forgot to clear it.
  readonly property bool working: writeProc.running || readProc.running || root.applyingJob || root.pendingApply

  readonly property var sectionList: SettingsJs.selectableSections()

  // Reads root.sections so the binding re-runs when a switch moves. A
  // function call on its own would not be tracked.
  readonly property var chosenKeys: {
    var chosen = root.sections
    var ids = []
    for (var i = 0; i < root.sectionList.length; i++) {
      var id = root.sectionList[i].id
      if (chosen[id]) ids.push(id)
    }
    return SettingsJs.keysForSections(ids)
  }

  readonly property int chosenSectionCount: {
    var chosen = root.sections
    var n = 0
    for (var i = 0; i < root.sectionList.length; i++)
      if (chosen[root.sectionList[i].id]) n++
    return n
  }

  readonly property string planSummary: root.plan ? root.plan.summary : ""
  readonly property bool planHasChanges: !!root.plan && root.plan.changes.length > 0

  // ---- paths -------------------------------------------------------------

  // A path typed with a leading tilde never reaches a shell that would
  // expand it, so it would arrive at the writer as a literal directory.
  function realPath(path) {
    var text = String(path || "")
    if (text === "~") return root.home
    if (text.indexOf("~/") === 0) return root.home + text.slice(1)
    return text
  }

  function folderUrl(path) {
    var text = root.realPath(path)
    var cut = text.lastIndexOf("/")
    return "file://" + (cut > 0 ? text.slice(0, cut) : root.home)
  }

  // The save dialog has no default suffix, so a name typed without one
  // would produce a file nothing recognises as a settings file.
  function withMarkdownSuffix(path) {
    var text = String(path || "")
    if (text.length === 0) return root.home + "/" + root.suggestedName()
    return /\.md$/i.test(text) ? text : text + ".md"
  }

  // Opens on the folder last used, with a fresh name already filled in.
  function chooseExportFile() {
    exportFileDialog.currentFolder = root.folderUrl(root.exportPath)
    exportFileDialog.currentFile = root.folderUrl(root.exportPath) + "/" + root.suggestedName()
    exportFileDialog.open()
  }

  // ---- state -------------------------------------------------------------

  function setSection(id, on) {
    var next = {}
    for (var key in root.sections) next[key] = root.sections[key]
    next[id] = on
    root.sections = next
    root.forgetExport()
  }

  function setAllSections(on) {
    var next = {}
    for (var i = 0; i < root.sectionList.length; i++) next[root.sectionList[i].id] = on
    root.sections = next
    root.forgetExport()
  }

  function resetSections() {
    var next = {}
    for (var i = 0; i < root.sectionList.length; i++)
      next[root.sectionList[i].id] = root.sectionList[i].byDefault
    root.sections = next
  }

  // What was written is only worth offering to open while it still matches
  // what the page would write now.
  function forgetExport() {
    root.writtenPath = ""
    root.exportStatus = ""
  }

  // A plan belongs to one file. Point at another and the old plan is a lie.
  function forgetPlan() {
    root.plan = null
    root.importStatus = ""
  }

  onExportPathChanged: root.forgetExport()
  onImportPathChanged: root.forgetPlan()

  // ---- actions -----------------------------------------------------------

  function doExport() {
    if (root.chosenKeys.length === 0) return
    var text = SettingsJs.exportMarkdown(Omarchy.snapshotData, root.chosenKeys, {
      exported: new Date().toISOString(),
      hardware: Omarchy.dmiProduct
    })
    root.forgetExport()
    // Re-armed every time: onStarted clears it to give cat its EOF, and that
    // assignment does not restore itself, so a second export would write
    // nothing and still report success.
    writeProc.stdinEnabled = true
    writeProc.target = root.realPath(root.exportPath)
    writeProc.text = text
    writeProc.command = ["sh", "-c", "cat > \"$1\"", "sh", writeProc.target]
    writeProc.running = true
  }

  function openFile(path) {
    openProc.command = ["xdg-open", root.realPath(path)]
    openProc.running = true
  }

  function doReview() {
    root.forgetPlan()
    readProc.command = ["cat", root.realPath(root.importPath)]
    readProc.running = true
  }

  // One queued job. enqueueIo prompts for sudo when the plan needs root,
  // then apply-settings.sh --progress names each change on stdout.
  function doApply() {
    if (!root.planHasChanges) return
    root.pendingApply = true
    root.undoing = false
    root.importStatus = ""
    root.stepNow = 0
    root.stepTotal = root.plan.changes.length
    root.stepKey = ""
    root.lastUndoNeedsRoot = SettingsJs.passwordCount(root.plan) > 0
    Omarchy.runJob(["bash", root.applyScript, "--progress"], SettingsJs.planToJson(root.plan), "settings-import", {
      sudo: root.lastUndoNeedsRoot,
      onStdoutLine: root.onApplyStdout,
      onFinished: root.onApplyFinished
    })
  }

  function doUndo() {
    if (root.lastBackupDir.length === 0) return
    root.pendingApply = true
    root.undoing = true
    root.importStatus = ""
    root.stepNow = 0
    root.stepTotal = root.appliedCount
    root.stepKey = ""
    // --no-backup, or undoing would leave a way back of its own and a
    // second undo would put the import straight back.
    Omarchy.runJob(["bash", root.applyScript, "--no-backup", "--plan", root.lastBackupDir + "/undo.json"], "", "settings-undo", {
      sudo: root.lastUndoNeedsRoot,
      onStdoutLine: root.onApplyStdout,
      onFinished: root.onApplyFinished
    })
  }

  function onApplyStdout(line) {
    var parts = String(line).split("\t")
    if (parts[0] !== "progress") return
    root.stepNow = Number(parts[1]) || 0
    root.stepTotal = Number(parts[2]) || root.stepTotal
    root.stepKey = String(parts[3] || "")
  }

  function onApplyFinished(exitCode, out, err) {
    root.pendingApply = false
    root.stepNow = 0
    root.stepKey = ""
    var result = SettingsJs.parseApplyResult(out)
    var applied = SettingsJs.appliedCountFromResult(result)
    var backup = SettingsJs.backupDirFromResult(result)
    var reason = String(err || "").replace(/^\s+|\s+$/g, "")
    if (root.undoing) {
      root.undoing = false
      if (exitCode === 0) {
        root.appliedCount = 0
        root.lastBackupDir = ""
        root.importStatus = "Put back."
      } else {
        root.importStatus = reason.length > 0 ? reason : "Could not put everything back."
      }
      return
    }
    root.plan = null
    if (backup.length > 0) root.lastBackupDir = backup
    root.appliedCount = applied
    if (exitCode === 0) {
      root.importStatus = "Applied " + applied + ". Try it, and put it back below if you do not like it."
    } else if (applied > 0) {
      root.importStatus = (reason.length > 0 ? reason : "Some changes did not apply.") +
        " Put it back restores what did."
    } else {
      root.importStatus = reason.length > 0 ? reason : "Some changes did not apply."
    }
  }

  // ---- processes ---------------------------------------------------------

  Process {
    id: writeProc
    property string text: ""
    property string target: ""
    command: ["true"]
    stdinEnabled: true
    stderr: StdioCollector { id: writeErr; waitForEnd: true }
    onStarted: {
      write(text)
      // cat only finishes when its input ends.
      stdinEnabled = false
    }
    onExited: function(exitCode) {
      if (exitCode === 0) {
        root.writtenPath = writeProc.target
        root.exportStatus = "Wrote " + root.chosenKeys.length + " settings to " + writeProc.target
        return
      }
      var err = String(writeErr.text || "").replace(/^\s+|\s+$/g, "")
      root.exportStatus = err.length > 0 ? err : "Could not write " + writeProc.target
    }
  }

  Process {
    id: openProc
    command: ["true"]
    stderr: StdioCollector { id: openErr; waitForEnd: true }
    onExited: function(exitCode) {
      if (exitCode === 0) return
      var err = String(openErr.text || "").replace(/^\s+|\s+$/g, "")
      root.exportStatus = err.length > 0 ? err : "Nothing on this machine opens that file."
    }
  }

  Process {
    id: readProc
    command: ["true"]
    stdout: StdioCollector { id: readOut; waitForEnd: true }
    stderr: StdioCollector { id: readErr; waitForEnd: true }
    onExited: function(exitCode) {
      if (exitCode !== 0) {
        var err = String(readErr.text || "").replace(/^\s+|\s+$/g, "")
        root.importStatus = err.length > 0 ? err : "Could not read " + root.importPath
        return
      }
      var doc = SettingsJs.parseSettingsMarkdown(String(readOut.text || ""))
      root.lastDoc = doc
      root.plan = SettingsJs.planImport(doc, Omarchy.snapshotData, null, {
        hardware: Omarchy.dmiProduct
      })
      root.importStatus = root.plan.changes.length === 0
        ? "Nothing to change. " + root.plan.summary
        : ""
    }
  }

  // ---- export ------------------------------------------------------------

  PrefsGroup {
    title: "Export"
    query: root.query
    detail: "Atmos writes a Markdown file. The settings live in fenced blocks, so you can read the file, edit it, and hand it to someone without it being able to do anything you cannot see. Security settings are written down for you to read but Atmos will never import them."

    PrefsRow {
      label: "Sections"
      description: root.chosenKeys.length + " settings across " + root.chosenSectionCount + " sections."
      query: root.query
      keywords: ["all", "none", "select", "sections", "choose"]

      Row {
        spacing: 8

        PrefsButton {
          text: "All"
          enabled: !root.working
          onClicked: root.setAllSections(true)
        }

        PrefsButton {
          text: "None"
          enabled: !root.working
          onClicked: root.setAllSections(false)
        }
      }
    }

    Repeater {
      model: root.sectionList

      PrefsRow {
        required property var modelData
        label: modelData.title
        description: modelData.note + "  (" + modelData.count + " settings)"
        query: root.query
        keywords: [modelData.id, "section", "include", "export"]

        PrefsToggle {
          checked: !!root.sections[modelData.id]
          enabled: !root.working
          onToggled: root.setSection(modelData.id, !root.sections[modelData.id])
        }
      }
    }

    PrefsRow {
      label: "Where to write it"
      description: root.exportPath
      query: root.query
      keywords: ["path", "file", "folder", "directory", "markdown", "md", "save", "choose"]

      PrefsButton {
        text: "Choose…"
        enabled: !root.working
        onClicked: root.chooseExportFile()
      }
    }

    PrefsRow {
      label: "Write the file"
      description: root.exportStatus.length > 0
        ? root.exportStatus
        : "Writes the sections switched on above."
      query: root.query
      keywords: ["export", "save", "write", "backup", "open"]

      Row {
        spacing: 8

        PrefsButton {
          text: "Export"
          primary: true
          enabled: !root.working && root.chosenKeys.length > 0
          onClicked: root.doExport()
        }

        PrefsButton {
          text: "Open"
          enabled: !root.working && root.writtenPath.length > 0
          onClicked: root.openFile(root.writtenPath)
        }
      }
    }
  }

  // ---- import ------------------------------------------------------------

  PrefsGroup {
    title: "Import"
    query: root.query
    detail: "Nothing is applied until you have read the plan. Atmos compares the file against this machine, shows every change with what it will do, and writes a way back before it touches anything."

    PrefsRow {
      label: "File to read"
      // Left empty rather than guessed: a suggested name here would point
      // at a file that does not exist.
      description: root.importPath.length > 0 ? root.importPath : "No file chosen yet."
      query: root.query
      keywords: ["path", "file", "import", "load", "browse", "open"]

      PrefsButton {
        text: "Browse…"
        enabled: !root.working
        onClicked: importFileDialog.open()
      }
    }

    PrefsRow {
      label: "See what it would do"
      description: root.importStatus.length > 0
        ? root.importStatus
        : "Reads the file and compares it against this machine. Changes nothing."
      query: root.query
      keywords: ["review", "dry run", "preview", "plan", "diff"]

      PrefsButton {
        text: "Review"
        enabled: !root.working && root.importPath.length > 0
        onClicked: root.doReview()
      }
    }

    PrefsRow {
      label: "This file"
      description: "Where it came from."
      query: root.query
      available: !!root.lastDoc && SettingsJs.fileSummary(root.lastDoc, root.plan).length > 0
      stretchControl: true
      keywords: ["origin", "host", "when", "sections", "provenance"]

      PrefsText {
        width: parent.width
        text: root.lastDoc ? SettingsJs.fileSummary(root.lastDoc, root.plan) : ""
        color: Theme.muted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        wrapMode: Text.Wrap
      }
    }

    PrefsRow {
      label: "Changes"
      description: root.planSummary
      query: root.query
      available: root.planHasChanges
      stretchControl: true
      keywords: ["changes", "plan"]

      PrefsText {
        width: parent.width
        text: root.plan ? SettingsJs.changeLines(root.plan) : ""
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        wrapMode: Text.Wrap
      }
    }

    PrefsRow {
      label: "Worth knowing"
      description: "These still happen."
      query: root.query
      available: !!root.plan && root.plan.warnings.length > 0
      stretchControl: true
      keywords: ["warning", "hardware", "skipped", "shadow"]

      PrefsText {
        width: parent.width
        text: root.plan ? SettingsJs.warningLines(root.plan) : ""
        color: Theme.foreground
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        wrapMode: Text.Wrap
      }
    }

    PrefsRow {
      label: "Blocked"
      description: "Atmos will not do these."
      query: root.query
      available: !!root.plan && root.plan.blocked.length > 0
      stretchControl: true
      keywords: ["blocked", "refused", "security"]

      PrefsText {
        width: parent.width
        text: root.plan ? SettingsJs.blockedLines(root.plan) : ""
        color: Theme.urgent
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        wrapMode: Text.Wrap
      }
    }

    PrefsRow {
      label: "Apply the plan you just read"
      description: root.plan ? SettingsJs.applyForecast(root.plan) : "Runs exactly the changes listed above."
      query: root.query
      available: root.planHasChanges
      keywords: ["apply", "import", "run", "password"]

      PrefsButton {
        text: "Apply"
        danger: true
        enabled: !root.working
        onClicked: applyConfirm.ask()
      }
    }

    PrefsRow {
      label: "Applying"
      description: root.stepKey.length > 0
        ? root.stepKey + "  (" + root.stepNow + " of " + root.stepTotal + ")"
        : (Omarchy.sudoPromptOpen ? "Waiting for sudo mode…" : "Working…")
      query: root.query
      available: root.applyingJob || root.pendingApply
      stretchControl: true
      keywords: ["progress", "applying"]

      PrefsProgress {
        width: parent.width
        from: 0
        to: Math.max(1, root.stepTotal)
        value: root.stepNow
        indeterminate: root.stepTotal === 0
        valueText: root.stepTotal > 0 ? root.stepNow + " / " + root.stepTotal : ""
      }
    }

    PrefsRow {
      label: "Put it back"
      description: root.appliedCount > 0
        ? "Restores the " + root.appliedCount + " values that import replaced. Try the machine first."
        : "Restores the values the last import replaced."
      query: root.query
      available: root.lastBackupDir.length > 0
      keywords: ["undo", "revert", "restore", "back", "put back"]

      PrefsButton {
        text: "Put it back"
        primary: root.appliedCount > 0
        enabled: !root.working && root.lastBackupDir.length > 0
        onClicked: undoConfirm.ask()
      }
    }
  }

  // ---- dialogs -----------------------------------------------------------

  FileDialog {
    id: exportFileDialog
    title: "Write the settings file"
    fileMode: FileDialog.SaveFile
    nameFilters: ["Settings files (*.md)", "All files (*)"]
    onAccepted: root.exportPath = root.withMarkdownSuffix(RichUi.pathFromUrl(selectedFile))
  }

  FileDialog {
    id: importFileDialog
    title: "Open a settings file"
    fileMode: FileDialog.OpenFile
    nameFilters: ["Settings files (*.md)", "All files (*)"]
    currentFolder: root.folderUrl(root.importPath)
    onAccepted: root.importPath = RichUi.pathFromUrl(selectedFile)
  }

  PrefsConfirm {
    id: applyConfirm
    title: "Apply these settings?"
    message: root.plan ? SettingsJs.applyConfirmMessage(root.plan) : ""
    confirmText: "Apply"
    onConfirmed: {
      if (SettingsJs.hasCommandImport(root.plan)) commandConfirm.ask()
      else root.doApply()
    }
  }

  PrefsConfirm {
    id: commandConfirm
    title: "These commands will run"
    message: root.plan ? SettingsJs.commandConfirmMessage(root.plan) : ""
    confirmText: "Apply commands"
    onConfirmed: root.doApply()
  }

  PrefsConfirm {
    id: undoConfirm
    title: "Undo the last import?"
    message: "Puts back the values the last import replaced."
    confirmText: "Undo"
    onConfirmed: root.doUndo()
  }

  Connections {
    target: Omarchy

    // The prompt closing with no sudo mode means it was refused, and an
    // import that needs root would only fail partway through.
    function onSudoPromptOpenChanged() {
      if (!root.pendingApply || Omarchy.sudoPromptOpen || Omarchy.passwordlessSudo) return
      if (root.applyingJob || Omarchy.sudoEnabling || Omarchy.jobBusy) return
      root.pendingApply = false
      root.undoing = false
      root.importStatus = "Nothing applied: these changes need sudo mode."
    }
  }

  Component.onCompleted: {
    root.resetSections()
    // Seeded here rather than in the property, so the name carries this
    // machine's hostname once Omarchy has read it.
    root.exportPath = root.home + "/" + root.suggestedName()
    applyConfirm.parent = root.prefsOverlay
    commandConfirm.parent = root.prefsOverlay
    undoConfirm.parent = root.prefsOverlay
  }
}
