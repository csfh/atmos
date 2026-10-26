pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "BackendProtocol.js" as Protocol

// The one connection to atmosd. A single `atmosd serve` process answers
// requests and pushes what it watches, so the app does not fork a process for
// every poll. Nothing works without the daemon behind it (docs/adr/0010): when
// the server or the daemon is not there, a request is refused as unavailable,
// `available` goes false for the "Atmos backend unavailable" screen, and the
// connection is looked for again until it answers.
//
// request(body, callback): body comes from Requests.js; callback gets the
// reply envelope { ok, result | error } and is always called exactly once.
// watch(owner, spec): { paths, chrome, accounts } to be pushed as stamp / chrome /
// accounts signals whenever they change.
// subscribe(owner, topics): { live: { intervalMs } } display kinds the daemon
// pushes as sample(kind, frame) signals for as long as they are held.
QtObject {
  id: root

  readonly property string bin: {
    var fromEnv = String(Quickshell.env("ATMOS_BACKEND") || "")
    if (fromEnv.length > 0) return fromEnv
    return String(Quickshell.shellDir || "") + "/bin/atmosd"
  }
  readonly property string platformId: {
    var fromEnv = String(Quickshell.env("ATMOS_BACKEND_ID") || "")
    return fromEnv.length > 0 ? fromEnv : "omarchy"
  }

  // starting -> up -> down -> starting ... A request made while starting waits;
  // one made while down is refused.
  property string link: "starting"
  readonly property bool up: link === "up"
  // Whether the daemon answers the server: "unknown" until it says, then
  // "up" or "down" with why, which look this is and when the next one is.
  property string daemon: "unknown"
  property var daemonDown: ({ why: "", attempt: 0, retryMs: 0 })
  // False shows the "Atmos backend unavailable" screen.
  readonly property bool available: link !== "down" && daemon !== "down"
  property bool lapsed: false
  // The backend answers again after having been gone: what was read while it
  // was away was refused, so whoever reads at startup reads again.
  signal reconnected()
  onAvailableChanged: {
    if (!available) lapsed = true
    else if (lapsed) {
      lapsed = false
      reconnected()
    }
  }
  property int failures: 0
  property double startedAt: 0
  property int lastId: 0
  property var pending: ({})
  property var waiting: []
  property var watchSpecs: ({})
  property var subSpecs: ({})

  signal stamp(var doc)
  signal chrome(var doc)
  signal accounts(var doc)
  // A held display kind has a new sample: { kind, sampledAt, tookMs, result },
  // or { kind, stale, error } when sampling failed or went quiet.
  signal sample(string kind, var frame)

  // Jobs this window started, by its own id for each: { id (the daemon's,
  // 0 until it says), next (the first line not yet seen), handlers,
  // cancelled, input }. A job outlives this window and the link to the
  // daemon: when the link comes back, each one is attached again from `next`.
  property int lastJob: 0
  property var jobs: ({})
  // Ids of this window's jobs carry its own prefix, so one it adopts from an
  // earlier window can never collide with one it starts.
  readonly property string jobPrefix: "w" + Math.floor(Math.random() * 1679616).toString(36)

  function command(args) {
    var cmd = [root.bin, "--backend", root.platformId]
    var i
    for (i = 0; i < args.length; i++) cmd.push(String(args[i]))
    return cmd
  }

  function request(body, callback) {
    var cb = typeof callback === "function" ? callback : function() {}
    if (root.link === "starting") {
      root.waiting = root.waiting.concat([{ body: body, cb: cb }])
      return
    }
    if (root.link !== "up") {
      Qt.callLater(function() { cb(Protocol.unavailable("The Atmos backend is not running")) })
      return
    }
    root.lastId += 1
    var id = root.lastId
    var map = root.pending
    map[id] = cb
    root.pending = map
    serve.write(Protocol.encodeRequest(id, body))
  }

  // Each part of the app watches under its own name; the server gets the merge.
  function watch(owner, spec) {
    var map = root.watchSpecs
    map[String(owner)] = spec || ({})
    root.watchSpecs = map
    if (root.up) root.sendWatch()
  }

  // Merged when sent, not bound: watch() edits the map in place, which a
  // binding would not notice.
  function mergedWatch() {
    return Protocol.mergeWatch(root.watchSpecs)
  }

  function sendWatch() {
    root.request(Protocol.watchBody(root.mergedWatch()), function() {})
  }

  // Each part of the app holds topics under its own name, as with watch().
  function subscribe(owner, topics) {
    var map = root.subSpecs
    map[String(owner)] = topics || ({})
    root.subSpecs = map
    if (root.up) root.sendSubs()
  }

  function mergedSubs() {
    return Protocol.mergeSubs(root.subSpecs)
  }

  function sendSubs() {
    root.request(Protocol.subBody(root.mergedSubs()), function() {})
  }

  // Run a registry job or interactive tool (Requests.job) in the daemon.
  // handlers: onStream(fd, line), onExit(code, summary). Returns this window's
  // id for the job, for jobInput and jobCancel.
  function startJob(op, label, handlers) {
    root.lastJob += 1
    var cid = root.jobPrefix + "-" + root.lastJob
    root.jobs[cid] = { id: 0, next: 0, handlers: handlers || ({}), cancelled: false, input: [] }
    // No daemon to run it: a job that ended at once.
    if (root.link === "down" || root.daemon === "down") {
      Qt.callLater(function() { root.endJob(cid, 127, Protocol.jobRefused(null)) })
      return cid
    }
    root.request(Protocol.jobStartBody(cid, op.name, op.args, label), function(env) {
      if (env && env.ok === true) return
      var summary = Protocol.jobRefused(env)
      root.endJob(cid, summary.code, summary)
    })
    return cid
  }

  // The jobs the daemon is running, whichever window started them.
  function listJobs(callback) {
    root.request({ op: "job.list" }, function(env) {
      var ok = env && env.ok === true && env.result && Array.isArray(env.result.jobs)
      callback(ok ? env.result.jobs : [])
    })
  }

  // Follow a running job an earlier window started, from its first line.
  // Returns the id to cancel it by, or "" when this window already follows it.
  function adoptJob(row, handlers) {
    var cid = String(row.cid || "")
    if (!cid || root.jobs[cid]) return ""
    root.jobs[cid] = { id: Number(row.job), next: 0, handlers: handlers || ({}), cancelled: false, input: [] }
    root.request(Protocol.jobAttachBody(row.job, 0), function() {})
    return cid
  }

  function jobInput(cid, data, close) {
    var rec = root.jobs[cid]
    if (!rec) return
    if (rec.id === 0) {
      rec.input.push({ data: String(data || ""), close: close === true })
      return
    }
    root.request(Protocol.jobInputBody(rec.id, data, close), function() {})
  }

  function jobCancel(cid) {
    var rec = root.jobs[cid]
    if (!rec) return
    rec.cancelled = true
    if (rec.id !== 0) root.request(Protocol.jobCancelBody(rec.id), function() {})
  }

  function endJob(cid, code, summary) {
    var rec = root.jobs[cid]
    if (!rec) return
    delete root.jobs[cid]
    if (typeof rec.handlers.onExit === "function") rec.handlers.onExit(code, summary || null)
  }

  function onJobEvent(frame) {
    var cid = String(frame.cid || "")
    var rec = root.jobs[cid]
    if (!rec) return
    if (frame.kind === "started") {
      rec.id = Number(frame.job)
      var i
      for (i = 0; i < rec.input.length; i++) root.jobInput(cid, rec.input[i].data, rec.input[i].close)
      rec.input = []
      if (rec.cancelled) root.jobCancel(cid)
    } else if (frame.kind === "stream") {
      var seq = Number(frame.seq)
      if (seq < rec.next) return
      rec.next = seq + 1
      if (typeof rec.handlers.onStream === "function") rec.handlers.onStream(String(frame.fd), String(frame.line))
    } else if (frame.kind === "exit") {
      root.endJob(cid, Number(frame.code), frame.summary)
    }
  }

  // The daemon forgot a job this window was following (it restarted and the
  // job did not survive): say so as a failure.
  function onJobGone(frame) {
    var cid
    for (cid in root.jobs) {
      if (root.jobs[cid].id !== Number(frame.job)) continue
      root.endJob(cid, 1, { ok: false, code: 1, message: "The job is no longer running", warnings: [] })
      return
    }
  }

  function attachJobs() {
    var cid
    for (cid in root.jobs) {
      var rec = root.jobs[cid]
      if (rec.id !== 0) root.request(Protocol.jobAttachBody(rec.id, rec.next), function() {})
    }
    // A job whose start was sent just before the link dropped has no number
    // yet: the daemon's list says whether it took it.
    var unnumbered = []
    for (cid in root.jobs) if (root.jobs[cid].id === 0) unnumbered.push(cid)
    if (unnumbered.length === 0) return
    root.listJobs(function(rows) {
      var i, j
      for (i = 0; i < unnumbered.length; i++) {
        var rec2 = root.jobs[unnumbered[i]]
        if (!rec2 || rec2.id !== 0) continue
        var found = null
        for (j = 0; j < rows.length; j++) if (String(rows[j].cid) === unnumbered[i]) found = rows[j]
        if (found) {
          rec2.id = Number(found.job)
          root.request(Protocol.jobAttachBody(rec2.id, 0), function() {})
        } else {
          root.endJob(unnumbered[i], 1, { ok: false, code: 1, message: "The job did not start", warnings: [] })
        }
      }
    })
  }

  function failPending(message) {
    var map = root.pending
    var waiting = root.waiting
    root.pending = ({})
    root.waiting = []
    var key
    for (key in map) map[key](Protocol.unavailable(message))
    // Waiting requests never reached the server.
    var i
    for (i = 0; i < waiting.length; i++) waiting[i].cb(Protocol.unavailable(message))
  }

  function onLine(line) {
    var msg = Protocol.decodeLine(line)
    if (msg.type === "reply") {
      var cb = root.pending[msg.id]
      if (!cb) return
      var map = root.pending
      delete map[msg.id]
      root.pending = map
      cb(msg.envelope)
    } else if (msg.type === "event") {
      if (msg.name === "stamp") root.stamp(msg.result)
      else if (msg.name === "chrome") root.chrome(msg.result)
      else if (msg.name === "accounts") root.accounts(msg.result)
      else if (msg.name === "sample") root.sample(String(msg.frame.kind || ""), msg.frame)
      else if (msg.name === "backend") root.onBackend(msg.frame)
      else if (msg.name === "job") {
        if (msg.frame.kind === "gone") root.onJobGone(msg.frame)
        else root.onJobEvent(msg.frame)
      }
    }
  }

  // The server's word on the daemon. When it comes back after being gone, the
  // jobs this window follows are attached again; the server has already put
  // the watches and held topics to it.
  function onBackend(frame) {
    var was = root.daemon
    root.daemon = frame.state === "down" ? "down" : "up"
    root.daemonDown = frame.state === "down"
      ? { why: String(frame.why || ""), attempt: Number(frame.attempt) || 0, retryMs: Number(frame.retryMs) || 0 }
      : ({ why: "", attempt: 0, retryMs: 0 })
    if (root.daemon === "up" && was === "down") root.attachJobs()
  }

  function onServeStarted() {
    root.startedAt = Date.now()
    root.link = "up"
    root.daemon = "unknown"
    var waiting = root.waiting
    root.waiting = []
    root.sendWatch()
    root.sendSubs()
    root.attachJobs()
    var i
    for (i = 0; i < waiting.length; i++) root.request(waiting[i].body, waiting[i].cb)
  }

  function onServeExited(code) {
    var wasUp = root.link === "up"
    var lived = Date.now() - root.startedAt
    root.failures = wasUp && lived >= Protocol.STABLE_MS ? 1 : root.failures + 1
    root.link = "down"
    root.failPending("The backend stopped")
    restartTimer.interval = Protocol.restartDelay(root.failures)
    restartTimer.restart()
  }

  function startServe() {
    root.link = "starting"
    serve.running = true
  }

  property Process serve: Process {
    command: root.command(["serve"])
    stdinEnabled: true
    stdout: SplitParser {
      onRead: function(line) { root.onLine(line) }
    }
    // Without a parser Quickshell drops the backend's stderr, and with it the reason for a failed read.
    stderr: SplitParser {
      onRead: function(line) { console.warn("atmosd: " + line) }
    }
    onStarted: root.onServeStarted()
    onExited: function(code) { root.onServeExited(code) }
  }

  property Timer restartTimer: Timer {
    repeat: false
    onTriggered: root.startServe()
  }

  Component.onCompleted: root.startServe()
}
