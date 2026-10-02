pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "BackendProtocol.js" as Protocol

// The one connection to ratmos. A single `ratmos serve` process answers
// requests and pushes what it watches, so the app does not fork a process for
// every poll. If the server cannot run, requests fall back to one process each
// and the watches fall back to a once-a-second poll, so the UI keeps working.
//
// request(body, callback): body comes from Requests.js; callback gets the
// reply envelope { ok, result | error } and is always called exactly once.
// watch(owner, spec): { paths, chrome, accounts } to be pushed as stamp / chrome /
// accounts signals whenever they change.
QtObject {
  id: root

  readonly property string bin: {
    var fromEnv = String(Quickshell.env("ATMOS_BACKEND") || "")
    if (fromEnv.length > 0) return fromEnv
    return String(Quickshell.shellDir || "") + "/bin/ratmos"
  }
  readonly property string platformId: {
    var fromEnv = String(Quickshell.env("ATMOS_BACKEND_ID") || "")
    return fromEnv.length > 0 ? fromEnv : "omarchy"
  }

  // starting -> up -> down -> starting ... A request made while starting waits;
  // one made while down runs on its own process.
  property string link: "starting"
  readonly property bool up: link === "up"
  property int failures: 0
  property double startedAt: 0
  property int lastId: 0
  property var pending: ({})
  property var waiting: []
  property var watchSpecs: ({})

  signal stamp(var doc)
  signal chrome(var doc)
  signal accounts(var doc)
  // Raised when the server stops answering, so the UI can say so.
  signal lostConnection()

  function command(args) {
    var cmd = [root.bin, "--backend", root.platformId]
    var i
    for (i = 0; i < args.length; i++) cmd.push(String(args[i]))
    return cmd
  }

  // Streaming and interactive jobs run `ratmos apply -- <argv>` on a process
  // of their own, where output and exit code are the point.
  function applyCommand(argv) {
    var cmd = root.command(["apply", "--"])
    var list = argv instanceof Array ? argv : []
    var i
    for (i = 0; i < list.length; i++) cmd.push(String(list[i]))
    return cmd
  }

  function request(body, callback) {
    var cb = typeof callback === "function" ? callback : function() {}
    if (root.link === "starting") {
      root.waiting = root.waiting.concat([{ body: body, cb: cb }])
      return
    }
    if (root.link !== "up") {
      root.runOneShot(body, cb)
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

  function runOneShot(body, cb) {
    oneShot.createObject(root, {
      command: root.command(["request"]),
      body: body,
      done: cb
    })
  }

  function failPending(message) {
    var map = root.pending
    var waiting = root.waiting
    root.pending = ({})
    root.waiting = []
    var key
    for (key in map) map[key](Protocol.unavailable(message))
    // Waiting requests never reached the server; run them on their own.
    var i
    for (i = 0; i < waiting.length; i++) root.runOneShot(waiting[i].body, waiting[i].cb)
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
    }
  }

  function onServeStarted() {
    root.startedAt = Date.now()
    root.link = "up"
    var waiting = root.waiting
    root.waiting = []
    root.sendWatch()
    var i
    for (i = 0; i < waiting.length; i++) root.request(waiting[i].body, waiting[i].cb)
  }

  function onServeExited(code) {
    var wasUp = root.link === "up"
    var lived = Date.now() - root.startedAt
    root.failures = wasUp && lived >= Protocol.STABLE_MS ? 1 : root.failures + 1
    root.link = "down"
    root.failPending("The backend stopped")
    if (wasUp) root.lostConnection()
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
    onStarted: root.onServeStarted()
    onExited: function(code) { root.onServeExited(code) }
  }

  property Timer restartTimer: Timer {
    repeat: false
    onTriggered: root.startServe()
  }

  // While the server is down, watching falls back to asking once a second.
  property bool polling: false
  property Timer fallbackTimer: Timer {
    interval: 1000
    repeat: true
    running: root.link === "down"
    onTriggered: root.pollOnce()
  }

  function pollOnce() {
    if (root.polling) return
    var spec = root.mergedWatch()
    var calls = []
    if (spec.paths && spec.paths.length > 0)
      calls.push({ op: "host.stamp", paths: spec.paths, signal: "stamp" })
    if (spec.chrome === true) calls.push({ op: "host.chrome", signal: "chrome" })
    if (spec.accounts)
      calls.push({ op: "host.accounts", user: spec.accounts.user, home: spec.accounts.home, signal: "accounts" })
    if (calls.length === 0) return
    root.polling = true
    var left = calls.length
    var i
    for (i = 0; i < calls.length; i++) {
      (function(call) {
        var body = {}
        var key
        for (key in call) if (key !== "signal") body[key] = call[key]
        root.runOneShot(body, function(env) {
          if (env && env.ok === true && env.result) {
            if (call.signal === "stamp") root.stamp(env.result)
            else if (call.signal === "chrome") root.chrome(env.result)
            else root.accounts(env.result)
          }
          left -= 1
          if (left === 0) root.polling = false
        })
      })(calls[i])
    }
  }

  property Component oneShot: Component {
    Process {
      id: proc
      property var body: ({})
      property var done: null
      running: true
      stdinEnabled: true
      stdout: StdioCollector {
        id: out
        waitForEnd: true
      }
      onStarted: {
        write(JSON.stringify(body))
        stdinEnabled = false
      }
      onExited: function(code) {
        var cb = proc.done
        var env = Protocol.parseEnvelope(out.text, code)
        Qt.callLater(function() { proc.destroy() })
        if (cb) cb(env)
      }
    }
  }

  Component.onCompleted: root.startServe()
}
