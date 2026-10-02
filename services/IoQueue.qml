pragma Singleton
import QtQuick
import Quickshell.Io
import "Failure.js" as FailureJs
import "WorkQueue.js" as WorkQueue

// The queue that serialises Atmos's writes and reads, and the processes that
// run the ones that are not plain backend requests. It knows how to start a
// job and how it ended; it does not know what any job means. Omarchy listens
// to the signals below and does the domain work, then calls finished().
//
// A job is { kind: "read" | "mut" | "job", argv, stdin, key, apply, refresh,
// sudo, jobKind, onStdoutLine, onFinished, ... }.
//   read              -> readRequested
//   mut with a value  -> setRequested   (a settings.set request)
//   mut with argv     -> `ratmos apply` on mutProc, then mutExited
//   job               -> `ratmos apply` on jobProc, streaming, then jobExited
QtObject {
  id: root

  property var queue: WorkQueue.createWorkQueue()
  // The job on the machine right now.
  property var current: null

  // Streaming job.
  property bool jobBusy: false
  property string jobKind: ""
  property string jobLog: ""
  property var jobResult: null
  property string jobStdin: ""
  property string jobStdoutBuf: ""
  property var jobStdoutLineCb: null
  property var jobFinishedCb: null

  // A blocking picker or region tool, which stays off the queue.
  property var interactiveApply: null
  property string interactiveRefresh: "none"
  property string interactiveKind: ""
  readonly property bool interactiveRunning: interactiveProc.running

  signal readRequested(var job)
  signal setRequested(var job)
  // A write is about to start, before anything runs.
  signal writeStarting(var job)
  signal jobStarting(var job)
  signal mutExited(var job, int exitCode, string out, string err)
  signal jobExited(var job, int exitCode, string out, string err, var summary)
  signal interactiveExited(int exitCode, var apply, string refresh, string out, string err)

  function enqueueWrite(job) {
    WorkQueue.enqueueWrite(root.queue, job)
    root.kick()
  }

  function enqueueRead(group) {
    WorkQueue.enqueueRead(root.queue, group)
    root.kick()
  }

  function kick() {
    if (root.queue.running) return
    var job = WorkQueue.takeNext(root.queue)
    if (!job) return
    root.current = job
    root.start(job)
  }

  function finished() {
    root.current = null
    WorkQueue.release(root.queue)
    root.kick()
  }

  function start(job) {
    if (job.kind === "read") {
      root.readRequested(job)
      return
    }
    if (job.kind === "job") {
      root.jobLog = ""
      root.jobKind = String(job.jobKind || "")
      root.jobStdin = String(job.stdin || "")
      root.jobStdoutBuf = ""
      root.jobResult = null
      root.jobStdoutLineCb = typeof job.onStdoutLine === "function" ? job.onStdoutLine : null
      root.jobFinishedCb = typeof job.onFinished === "function" ? job.onFinished : null
      root.jobBusy = true
      root.jobStarting(job)
      jobProc.stdinEnabled = root.jobStdin.length > 0
      jobProc.command = Backend.applyCommand(job.argv)
      jobProc.running = true
      return
    }
    root.writeStarting(job)
    if (job.hasValue === true) {
      root.setRequested(job)
      return
    }
    mutProc.stdinEnabled = false
    mutProc.command = Backend.applyCommand(job.argv)
    mutProc.running = true
  }

  function cancelJob() {
    if (!jobProc.running) return
    jobProc.running = false
  }

  // The wrapper is the Process so a SIGTERM can kill file-select / slurp
  // children instead of orphaning them.
  function startInteractive(argv, kind, apply, refresh) {
    root.interactiveKind = String(kind || "")
    root.interactiveApply = apply
    root.interactiveRefresh = refresh
    var wrapped = ["bash", "-c", "trap 'trap - INT TERM; kill 0 2>/dev/null; exit 143' INT TERM; \"$@\"", "prefs-interactive"]
    var i
    for (i = 0; i < argv.length; i++) wrapped.push(argv[i])
    interactiveProc.running = false
    interactiveProc.command = Backend.applyCommand(wrapped)
    interactiveProc.running = true
  }

  function stopInteractive() {
    if (interactiveProc.running) interactiveProc.running = false
  }

  property Process interactiveProc: Process {
    command: ["true"]
    stdout: StdioCollector {
      id: interactiveOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: interactiveErr
      waitForEnd: true
    }
    onExited: function(exitCode) {
      var apply = root.interactiveApply
      var refresh = root.interactiveRefresh
      root.interactiveKind = ""
      root.interactiveApply = null
      root.interactiveRefresh = "none"
      root.interactiveExited(exitCode, apply, refresh, String(interactiveOut.text || ""), String(interactiveErr.text || ""))
    }
  }

  property Process mutProc: Process {
    command: ["true"]
    stdinEnabled: false
    stdout: StdioCollector {
      id: mutOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: mutErr
      waitForEnd: true
    }
    onExited: function(exitCode) {
      root.mutExited(root.current, exitCode, String(mutOut.text || ""), String(mutErr.text || ""))
    }
  }

  property Process jobProc: Process {
    command: ["true"]
    stdinEnabled: false
    stdout: SplitParser {
      onRead: function(line) {
        // The summary line is for us, not for the job's own output.
        var summary = FailureJs.applyLine(line)
        if (summary) {
          root.jobResult = summary
          return
        }
        root.jobStdoutBuf += String(line) + "\n"
        var cb = root.jobStdoutLineCb
        if (typeof cb === "function") cb(line)
      }
    }
    stderr: StdioCollector {
      id: jobErr
      waitForEnd: true
    }
    onStarted: {
      if (root.jobStdin.length > 0) {
        write(root.jobStdin)
        root.jobStdin = ""
        // Closed after writing, or a job that reads its input to EOF never
        // gets one. The next job re-arms this in start().
        stdinEnabled = false
      }
    }
    onExited: function(exitCode) {
      root.jobBusy = false
      var out = String(root.jobStdoutBuf || "").replace(/^\s+|\s+$/g, "")
      var err = String(jobErr.text || "").replace(/^\s+|\s+$/g, "")
      var summary = root.jobResult
      root.jobResult = null
      root.jobLog = out
      var finished = root.jobFinishedCb
      root.jobFinishedCb = null
      root.jobStdoutLineCb = null
      if (typeof finished === "function") finished(exitCode, out, err)
      root.jobExited(root.current, exitCode, out, err, summary)
    }
  }
}
