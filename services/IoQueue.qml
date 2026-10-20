pragma Singleton
import QtQuick
import "WorkQueue.js" as WorkQueue

// The queue that serialises Atmos's writes and reads, and the daemon jobs that
// run the ones that are not plain backend requests. It knows how to start a
// job and how it ended; it does not know what any job means. Omarchy listens
// to the signals below and does the domain work, then calls finished().
//
// A job is { kind: "mut" | "job", op, key, refresh, sudo, jobKind,
// onStdoutLine, onFinished, ... }.
//   mut with a value  -> setRequested   (a settings.set request)
//   job               -> a daemon job (Requests.job), streaming, then jobExited
// Every job and interactive tool runs in the daemon, so it outlives this
// window (docs/adr/0009). Backend.startJob is the only way in.
QtObject {
  id: root

  property var queue: WorkQueue.createWorkQueue()
  // The job on the machine right now.
  property var current: null

  // Streaming job.
  property bool jobBusy: false
  property string jobKind: ""
  property string jobLog: ""
  property string jobCid: ""
  property string jobStdoutBuf: ""
  property string jobStderrBuf: ""
  property var jobStdoutLineCb: null
  property var jobFinishedCb: null

  // A blocking picker or region tool, which stays off the queue.
  property string interactiveKind: ""
  property string interactiveCid: ""
  readonly property bool interactiveRunning: interactiveCid.length > 0

  signal setRequested(var job)
  // A write is about to start, before anything runs.
  signal writeStarting(var job)
  signal jobStarting(var job)
  signal jobExited(var job, int exitCode, string out, string err, var summary)
  signal interactiveExited(int exitCode, var apply, string refresh, string out, string err, var summary)

  function enqueueWrite(job) {
    WorkQueue.enqueueWrite(root.queue, job)
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
    // A job an earlier window left running waits for the queue to go idle.
    if (root.resumeRows.length > 0 && !root.jobBusy) {
      root.followJob(root.resumeRows.shift())
      return
    }
    root.kick()
  }

  function start(job) {
    if (job.kind === "job") {
      root.jobLog = ""
      root.jobKind = String(job.jobKind || "")
      root.jobStdoutBuf = ""
      root.jobStderrBuf = ""
      root.jobStdoutLineCb = typeof job.onStdoutLine === "function" ? job.onStdoutLine : null
      root.jobFinishedCb = typeof job.onFinished === "function" ? job.onFinished : null
      root.jobBusy = true
      root.jobStarting(job)
      root.jobCid = Backend.startJob(job.op, root.jobKind, {
        onStream: root.onJobStream,
        onExit: function(code, summary) { root.onJobExit(job, code, summary) }
      })
      return
    }
    root.writeStarting(job)
    root.setRequested(job)
  }

  function onJobStream(fd, line) {
    if (fd === "err") {
      root.jobStderrBuf += line + "\n"
      return
    }
    root.jobStdoutBuf += line + "\n"
    var cb = root.jobStdoutLineCb
    if (typeof cb === "function") cb(line)
  }

  function onJobExit(job, exitCode, summary) {
    root.jobBusy = false
    root.jobCid = ""
    var out = String(root.jobStdoutBuf || "").replace(/^\s+|\s+$/g, "")
    var err = String(root.jobStderrBuf || "").replace(/^\s+|\s+$/g, "")
    root.jobLog = out
    var finished = root.jobFinishedCb
    root.jobFinishedCb = null
    root.jobStdoutLineCb = null
    if (typeof finished === "function") finished(exitCode, out, err)
    root.jobExited(job, exitCode, out, err, summary)
  }

  function cancelJob() {
    if (root.jobCid) Backend.jobCancel(root.jobCid)
  }

  // A job or a tool an earlier window left running: follow it, so the page
  // shows it as running and says how it ended. Only when nothing else is.
  property bool resumed: false
  property var resumeRows: []
  function resumeFromDaemon() {
    if (root.resumed) return
    root.resumed = true
    Backend.listJobs(function(rows) { root.resume(rows) })
  }

  function resume(rows) {
    var i
    for (i = 0; i < rows.length; i++) {
      var row = rows[i]
      if (row.state !== "running" || !row.label) continue
      if (/^(theme\.(switcher|bg-switcher|bg-file)|branding\.|capture\.|tailscale\.(send|receive))/.test(String(row.name))) {
        if (root.interactiveCid) continue
        root.followInteractive(row)
      } else {
        if (root.queue.running || root.jobBusy) root.resumeRows.push(row)
        else root.followJob(row)
      }
    }
  }

  function followJob(row) {
    var job = { kind: "job", jobKind: String(row.label), refresh: "all", sudo: false }
    root.queue.running = true
    root.current = job
    root.jobStdoutBuf = ""
    root.jobStderrBuf = ""
    root.jobStdoutLineCb = null
    root.jobFinishedCb = null
    root.jobKind = job.jobKind
    root.jobBusy = true
    root.jobCid = Backend.adoptJob(row, {
      onStream: root.onJobStream,
      onExit: function(code, summary) { root.onJobExit(job, code, summary) }
    })
    if (!root.jobCid) {
      root.jobBusy = false
      root.jobKind = ""
      root.finished()
    }
  }

  function followInteractive(row) {
    var out = ""
    var err = ""
    var cid = Backend.adoptJob(row, {
      onStream: function(fd, line) {
        if (fd === "err") err += line + "\n"
        else out += line + "\n"
      },
      onExit: function(code, summary) {
        if (root.interactiveCid === cid) {
          root.interactiveCid = ""
          root.interactiveKind = ""
        }
        root.interactiveExited(code, null, "none", out, err, summary)
      }
    })
    if (!cid) return
    root.interactiveKind = String(row.label)
    root.interactiveCid = cid
  }

  // A picker, a region tool or a recorder. The daemon runs it in the user's
  // session and stopping it stops its whole unit, so no child is orphaned.
  function startInteractive(op, kind, apply, refresh) {
    root.stopInteractive()
    var out = ""
    var err = ""
    root.interactiveKind = String(kind || "")
    var cid = Backend.startJob(op, root.interactiveKind, {
      onStream: function(fd, line) {
        if (fd === "err") err += line + "\n"
        else out += line + "\n"
      },
      onExit: function(code, summary) {
        if (root.interactiveCid === cid) {
          root.interactiveCid = ""
          root.interactiveKind = ""
        }
        root.interactiveExited(code, apply, refresh, out, err, summary)
      }
    })
    root.interactiveCid = cid
  }

  function stopInteractive() {
    if (root.interactiveCid) Backend.jobCancel(root.interactiveCid)
  }
}
