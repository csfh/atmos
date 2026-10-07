import QtQuick
import QtQuick.Dialogs
import Quickshell
import "../components"
import "../services"
import "../services/Failure.js" as FailureJs
import "../services/Requests.js" as Requests
import "../services/Settings.js" as SettingsJs
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "export"
  title: I18n.tr("Omafile")
  description: I18n.tr("An Omafile is this Omarchy system's configuration as a Markdown file. Write one to share the desktop, or apply one from another machine. Review every change before anything happens.")

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
  readonly property bool working: root.ioBusy || root.applyingJob || root.pendingApply

  readonly property var sectionList: SettingsJs.selectableSections()

  // Reads root.sections so the binding re-runs when a box is checked. A
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
  // would produce a file nothing recognises as an Omafile.
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
    var path = root.realPath(root.exportPath)
    root.startIo("write", Requests.hostWrite(path, text))
  }

  function openFile(path) {
    root.startIo("open", Requests.hostOpen(root.realPath(path)))
  }

  function doReview() {
    root.forgetPlan()
    root.startIo("read", Requests.hostRead([root.realPath(root.importPath)]))
  }

  function startIo(kind, body) {
    if (root.ioBusy) return
    root.ioKind = kind
    root.ioPath = body.path || (body.paths && body.paths[0]) || ""
    root.ioText = body.text || ""
    root.ioBusy = true
    Backend.request(body, function(env) {
      root.ioBusy = false
      root.ioAnswered(env)
    })
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

  property string ioKind: ""
  property string ioPath: ""
  property string ioText: ""
  property bool ioBusy: false

  function ioAnswered(env) {
    var ok = !!(env && env.ok === true)
    var err = ok ? "" : FailureJs.errorText(env && env.error)
    if (root.ioKind === "write") {
      if (ok) {
        root.writtenPath = root.ioPath
        root.exportStatus = "Wrote " + root.chosenKeys.length + " settings to " + root.ioPath
      } else {
        root.exportStatus = err.length > 0 ? err : "Could not write " + root.ioPath
      }
      return
    }
    if (root.ioKind === "open") {
      if (!ok)
        root.exportStatus = err.length > 0 ? err : "Nothing on this machine opens that file."
      return
    }
    if (!ok) {
      root.importStatus = err.length > 0 ? err : "Could not read " + root.importPath
      return
    }
    var files = env.result && env.result.files ? env.result.files : []
    var raw = files.length ? String(files[0].text || "") : ""
    var doc = SettingsJs.parseSettingsMarkdown(raw)
    root.lastDoc = doc
    root.plan = SettingsJs.planImport(doc, Omarchy.snapshotData, null, {
      hardware: Omarchy.dmiProduct,
      workspaceGestureUnmanaged: Omarchy.liveWorkspaceGestureUnmanaged(),
    })
    root.importStatus = root.plan.changes.length === 0
      ? "Nothing to change. " + root.plan.summary
      : ""
  }

  // ---- export ------------------------------------------------------------

  PrefsGroup {
    framed: true
    title: I18n.tr("Export")
    query: root.query
    detail: "Writes an Omafile of this Omarchy system. Settings live in fenced blocks you can read and edit. Security settings are written as prose and never applied."

    SettingRow {
      label: "Sections"
      description: I18n.tr("{settings} settings across {sections} sections.", { settings: root.chosenKeys.length, sections: root.chosenSectionCount })
      query: root.query
      keywords: ["all", "none", "select", "sections", "choose"]

      Row {
        spacing: Theme.space

        PrefsButton {
          text: I18n.tr("All")
          enabled: !root.working
          onClicked: root.setAllSections(true)
        }

        PrefsButton {
          text: I18n.tr("None")
          enabled: !root.working
          onClicked: root.setAllSections(false)
        }
      }
    }

    Repeater {
      model: root.sectionList

      SettingRow {
        required property var modelData
        interactive: !root.working
        label: modelData.title
        description: modelData.note
        caption: I18n.tr("{count} settings", { count: modelData.count })
        query: root.query
        keywords: [modelData.id, "section", "include", "export"]
        onActivated: root.setSection(modelData.id, !root.sections[modelData.id])

        leading: PrefsCheck {
          checked: !!root.sections[modelData.id]
          enabled: !root.working
          onToggled: root.setSection(modelData.id, !root.sections[modelData.id])
        }
      }
    }

    SettingRow {
      label: "Where to write it"
      description: root.exportPath
      query: root.query
      keywords: ["path", "file", "folder", "directory", "markdown", "md", "save", "choose"]

      PrefsButton {
        text: I18n.tr("Choose…")
        enabled: !root.working
        onClicked: root.chooseExportFile()
      }
    }

    SettingRow {
      label: "Write the file"
      description: root.exportStatus.length > 0
        ? root.exportStatus
        : "Writes the sections selected above."
      query: root.query
      keywords: ["export", "save", "write", "backup", "open"]

      Row {
        spacing: Theme.space

        PrefsButton {
          text: I18n.tr("Export")
          primary: true
          enabled: !root.working && root.chosenKeys.length > 0
          onClicked: root.doExport()
        }

        PrefsButton {
          text: I18n.tr("Open file")
          enabled: !root.working && root.writtenPath.length > 0
          onClicked: root.openFile(root.writtenPath)
        }
      }
    }
  }

  // ---- import ------------------------------------------------------------

  PrefsGroup {
    framed: true
    title: I18n.tr("Import")
    query: root.query
    detail: "Apply an Omafile from another Omarchy machine. Nothing is written until you have read the plan. Atmos shows every change and keeps a way back."

    SettingRow {
      label: "File to read"
      // Left empty rather than guessed: a suggested name here would point
      // at a file that does not exist.
      description: root.importPath.length > 0 ? root.importPath : "No file chosen yet."
      query: root.query
      keywords: ["path", "file", "import", "load", "browse", "open"]

      PrefsButton {
        text: I18n.tr("Choose…")
        enabled: !root.working
        onClicked: importFileDialog.open()
      }
    }

    SettingRow {
      label: "See what it would do"
      description: root.importStatus.length > 0
        ? root.importStatus
        : "Reads the file and compares it against this machine. Changes nothing."
      query: root.query
      keywords: ["review", "dry run", "preview", "plan", "diff"]

      PrefsButton {
        text: I18n.tr("Review")
        enabled: !root.working && root.importPath.length > 0
        onClicked: root.doReview()
      }
    }

    SettingRow {
      label: "This file"
      description: I18n.tr("Where it came from.")
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

    SettingRow {
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

    SettingRow {
      label: "Worth knowing"
      description: I18n.tr("These still happen.")
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

    SettingRow {
      label: "Blocked"
      description: I18n.tr("Atmos will not do these.")
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

    SettingRow {
      label: "Apply the plan you just read"
      description: root.plan ? SettingsJs.applyForecast(root.plan) : "Runs exactly the changes listed above."
      query: root.query
      available: root.planHasChanges
      keywords: ["apply", "import", "run", "password"]

      PrefsButton {
        text: I18n.tr("Apply…")
        danger: true
        enabled: !root.working
        onClicked: applyConfirm.ask()
      }
    }

    SettingRow {
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

    SettingRow {
      label: "Put it back"
      description: root.appliedCount > 0
        ? "Restores the " + root.appliedCount + " values that import replaced. Try the machine first."
        : "Restores the values the last import replaced."
      query: root.query
      available: root.lastBackupDir.length > 0
      keywords: ["undo", "revert", "restore", "back", "put back"]

      PrefsButton {
        text: I18n.tr("Put it back…")
        primary: root.appliedCount > 0
        enabled: !root.working && root.lastBackupDir.length > 0
        onClicked: undoConfirm.ask()
      }
    }
  }

  // ---- dialogs -----------------------------------------------------------

  FileDialog {
    id: exportFileDialog
    title: I18n.tr("Write an Omafile")
    fileMode: FileDialog.SaveFile
    nameFilters: ["Omafile (*.md)", "All files (*)"]
    onAccepted: root.exportPath = root.withMarkdownSuffix(RichUi.pathFromUrl(selectedFile))
  }

  FileDialog {
    id: importFileDialog
    title: I18n.tr("Open an Omafile")
    fileMode: FileDialog.OpenFile
    nameFilters: ["Omafile (*.md)", "All files (*)"]
    currentFolder: root.folderUrl(root.importPath)
    onAccepted: root.importPath = RichUi.pathFromUrl(selectedFile)
  }

  PrefsConfirm {
    id: applyConfirm
    title: I18n.tr("Apply this Omafile?")
    message: root.plan ? SettingsJs.applyConfirmMessage(root.plan) : ""
    confirmText: "Apply"
    onConfirmed: {
      if (SettingsJs.hasCommandImport(root.plan)) commandConfirm.ask()
      else root.doApply()
    }
  }

  PrefsConfirm {
    id: commandConfirm
    title: I18n.tr("These commands will run")
    message: root.plan ? SettingsJs.commandConfirmMessage(root.plan) : ""
    confirmText: "Apply commands"
    onConfirmed: root.doApply()
  }

  PrefsConfirm {
    id: undoConfirm
    title: I18n.tr("Undo the last import?")
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
