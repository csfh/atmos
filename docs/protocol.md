# Protocol

The window and `ratmos`, the Rust backend, talk in JSON. QML reaches `ratmos` only through `services/Backend.qml`, with request bodies built by `services/Requests.js`; `services/BackendProtocol.js` encodes and decodes the lines. This page is the wire format. The replies in `tests/golden/cases/` are recordings of it, replayed over both transports by `tests/golden.test.js`.

## Running ratmos

```
ratmos [--backend omarchy|plain] [--root DIR] <command>
```

| Flag | Meaning |
| --- | --- |
| `--backend` | `omarchy` (default) talks to the real machine through `omarchy`, `hyprctl` and the Hyprland sentinels. `plain` keeps the same keys in its own config tree and answers host reads with stand-in data. |
| `--root DIR` | A fixture root. Every path, absolute ones too, is re-rooted under `DIR`, and the commands a write would run are appended to `DIR/commands.log` instead. Tests use it. |

| Command | What it does |
| --- | --- |
| `request` | Read one JSON request from stdin, print one reply, exit. |
| `serve` | Read requests line by line from stdin until it closes. One process per window. |
| `apply -- CMD ARGS...` | Run a host command with its streams attached. A terminal tool: the window starts every job as `job.start`. |
| `job-check NAME [JSON]` | Say whether the job registry accepts a job or interactive tool with these arguments, and the command it would run. Nothing runs; the window's tests use it. |
| `snapshot`, `display KIND`, `display-snapshot`, `version` | Whole documents for tests, without a request body. |
| `task NAME ARGS...` | One of the tasks the job registry runs (`job.start`), also usable from a terminal: accounts, avatars, Wi-Fi, hooks, updates, imports (`settings-apply`) and the rest. `ratmos task` with no name lists them. Failures print the reason on stderr and exit non-zero. |
| `root TASK ARGS...` | The privileged half of a few settings. `task` starts it through polkit or sudo; it refuses to run unless it is root. |
| `mcp` | Atmos for agents: newline-delimited JSON-RPC on stdio (`atmos mcp`). |
| `install DIR` | Stage Atmos from a source tree into the XDG directories. `install.sh` runs it once the binary is built. |

## Replies

Every reply is an envelope:

```json
{"ok": true, "version": "0.3.0", "platform": {"id": "plain", "compositor": "none", "family": "portable"}, "result": "0.3.0"}
```

A failure keeps the same shape, with `result` null and an `error` in place of the answer:

```json
{"ok": false, "version": "0.3.0", "platform": {"...": "..."}, "result": null,
 "error": {"code": "denied", "message": "path may not contain ..", "context": "/../escape"}}
```

`error.code` says what kind of failure it was, so a caller can branch on it. `message` is for a person. `context` is the file, command or field the failure is about, or null.

| Code | Meaning |
| --- | --- |
| `bad_request` | The request itself is wrong: it is not JSON, names an unknown `op`, or has a missing or mistyped field. |
| `not_found` | The thing asked for is not there. |
| `denied` | The OS or the path rules refused the access, such as a `..` step or a path outside `$HOME` (or the root). |
| `io` | An I/O failure. |
| `command` | A child process failed or could not start. |
| `timeout` | A child process ran past its limit. |
| `failed` | Anything that does not fit a more specific code, including a well-formed request the handler turns down: an unknown domain, display kind or speedtest phase, or a `settings.set` value of the wrong type for its domain. |
| `unavailable` | The daemon is not there: a window's `serve` refuses a write, a job or a read it cannot do without one (docs/adr/0010). `BackendProtocol.js` makes the same envelope when the server itself cannot answer. |

Unknown extra fields in a request are ignored, so a newer window can talk to an older `ratmos`.

## One-shot: `request`

The body goes in on stdin, the envelope comes out on stdout. The exit code is 0 for `ok: true` and 1 for anything else, bad JSON included. Calling `ratmos` with no command, or an unknown one, exits 2 and prints usage on stderr.

```
$ echo '{"op":"host.read","paths":["/note.txt","/missing.txt"]}' | ratmos --backend plain --root DIR request
```

```json
{"ok": true, "result": {"files": [
  {"path": "/note.txt", "exists": true, "text": "one\ntwo\n"},
  {"path": "/missing.txt", "exists": false, "text": ""}]}}
```

## Long-lived: `serve`

`ratmos serve` reads one JSON request per line and writes one JSON line per reply. Add an `id` to a request and the reply carries the same `id`. Each request runs on its own thread, so a slow one never holds up a fast one, and replies can arrive in any order: match them by `id`. A line that is not JSON gets a `bad_request` reply with no `id`. The server stops when stdin closes, which is how a window going away looks.

`watch.set` and `sub.set` are the two ops `serve` answers itself. They are not accepted over `request`.

```json
{"op": "watch.set", "paths": ["/note.txt"], "chrome": true, "accounts": {"user": "me", "home": "/home/me"}}
```

| Field | Watches |
| --- | --- |
| `paths` | Files by size and mtime. Reported as a `stamp` event. |
| `chrome` | The theme colors and shell config. Reported as a `chrome` event. |
| `accounts` | The passwd, group and hostname view for a user. Reported as an `accounts` event. |

A `watch.set` replaces the previous subscription; every field is optional, and an empty one turns that watch off. The reply is `{"watching": true}`. Then, about once a second, the daemon (a `--root` fixture has none, so this server itself) checks what is watched and the server pushes an event only when it changed. The first check after a `watch.set` always reports. An event has no `id`:

```json
{"event": "stamp", "result": {"items": [{"path": "/note.txt", "sig": "3:1700000000", "text": "hi\n"}]}}
```

This is a real run with `--backend plain`, a fixture root holding `/note.txt`, and the `platform` field and the list of ops in the last message trimmed. The file changed after the first event, then two bad lines were sent:

```
> {"id":1,"op":"version"}
> {"id":2,"op":"watch.set","paths":["/note.txt"]}
< {"id":2,"ok":true,"result":{"watching":true},"version":"0.3.0"}
< {"event":"stamp","result":{"items":[{"path":"/note.txt","sig":"3:1700000000","text":"hi\n"}]}}
< {"id":1,"ok":true,"result":"0.3.0","version":"0.3.0"}
< {"event":"stamp","result":{"items":[{"path":"/note.txt","sig":"8:1700000000","text":"changed\n"}]}}
> not json
< {"ok":false,"error":{"code":"bad_request","context":null,"message":"request: expected ident at line 1 column 2"},"result":null,"version":"0.3.0"}
> {"id":3,"op":"nope"}
< {"id":3,"ok":false,"error":{"code":"bad_request","context":null,"message":"unknown variant `nope`, expected one of ..."},"result":null,"version":"0.3.0"}
```

Reply 2 came before reply 1. That is allowed.

### Held topics: `sub.set`

`sub.set` says which topics the window holds, and the server pushes each new sample so the window polls nothing. A topic is a display kind (`live`, `hardware`, `disks`, `services`, `software`, `diagnostics`) or a settings topic (`settings.look`, `settings.network`, `settings.accounts`, `settings.system`, `settings.host`; docs/adr/0011). It replaces the previous set; `{"topics": {}}` releases everything. An unknown topic is `bad_request`.

```json
{"id": 5, "op": "sub.set", "topics": {"live": {"intervalMs": 2000}}}
```

The reply is `{"holding": ["live"]}`. Then, on the holder's clock, each event is one sample, with `sampledAt` the Unix milliseconds the collection began (a window computes rates from it, never from its own clock) and `tookMs` its duration:

```json
{"event": "sample", "kind": "live", "sampledAt": 1760000000123, "tookMs": 9, "result": {"cpuIdle": 1, "cpuTotal": 2}}
```

When sampling fails, or a held kind goes quiet for six intervals (never less than 15 seconds), the event is `{"event": "sample", "kind": "live", "stale": true, "error": "..."}`. The last good sample stays where it is. `live` is sampled by the daemon only while some window holds it, no faster than the quickest holder asks and never faster than every 250 ms, and each window receives it at its own `intervalMs`. Other kinds are pushed when they are sampled, and on `sub.set` the daemon sends what it holds and collects a kind it holds nothing recent of, so a page does not wait for the schedule. A settings topic is pushed when it is first held, when something invalidates it (a command or job that names it, a change to a file it is read from, an unsettled write, `snapshot.invalidate`) and when a write patches it with what it read back. Its `result` is a flat document of snapshot keys, plus `_failed`, a map from key to reason for the keys whose tool did not answer: those keys are not in the document, so the window keeps the values it had. `sampledAt` is when the collection began; a window ignores a key it wrote after that time.

The settings topics are collected with the window's environment (`hyprctl` and `omarchy` need the session's): the window's session sends it to the daemon as its first frame, `{"t": "env", "env": {...}}`, and the daemon uses the newest.

The events come from the daemon over a session on `ratmos.sock` (`{"op": "session"}`, then `sub`, `unsub`, `req` frames one way and `event` and `res` frames the other; see `backend/src/sampler/session.rs`). With no daemon, or one that does not speak sessions, `serve` starts one when it can (`ratmos.service`, then a transient user unit, then a bare process; `ATMOS_NO_AUTOSTART=1` forbids it, and a `--root` fixture never does) and otherwise samples nothing: a window does nothing without the daemon (docs/adr/0010). It looks again after one, two, four and then five seconds, and says so with a `backend` event:

```json
{"event": "backend", "state": "up"}
{"event": "backend", "state": "down", "why": "no daemon (…)", "attempt": 2, "retryMs": 2000}
```

`down` follows a failed look or a session that ended; `up` follows a session that opened. A window shows its "Atmos backend unavailable" screen between the two. A `--root` fixture has no daemon, sends no `backend` event and samples its own files on the same clock.

## Streaming jobs: `apply`

`ratmos apply -- CMD ARGS...` runs the command with stdin and stdout connected, copies stderr through, and when the command ends prints one last stdout line:

```
@@ratmos-apply@@ {"ok":false,"code":3,"message":"...","warnings":[...]}
```

`ok` means the exit code was zero, `message` is the stderr that is not known noise, and `warnings` is the noise that was filtered out. The exit code is the command's own. Long jobs that stream output use this; everything else uses a request.

## Ops

Every body is `{"op": "<name>", ...fields}`. A new op needs its `Request` variant in `backend/src/request.rs`, its builder in `Requests.js`, a golden case, and a row here; `tests/protocol-doc.test.js` checks the names against this page.

### Identity

| Op | Fields | Result |
| --- | --- | --- |
| `version` | | The ratmos version, a string. |
| `platform` | | `{id, compositor, family}` for the backend. |

### Settings

A domain is one named setting, such as `theme` or `hyprLook.gapsIn`.

| Op | Fields | Result |
| --- | --- | --- |
| `settings.list` | | Every domain: `[{domain, type, file, encoding, prefix}]`. |
| `settings.get` | `domain` | The domain's value, or null when it is unset. |
| `settings.set` | `domain`, `value`, `readback` (optional list of snapshot keys) | `{domain, value, file, encoding}` with the value as read back. `value` must match the domain's `type`. |
| `settings.snapshot` | `group` (default `all`), `keys` (optional) | An object of every set value; an unset value is absent, not null. `keys` limits it to the named top-level domains. |

### Displays and hardware

`display.get` runs one inventory and returns its document with a `collector` and `platform` added.

| Op | Fields | Result |
| --- | --- | --- |
| `display.get` | `kind`: `live`, `hardware`, `disks`, `services`, `software` or `diagnostics` | One inventory. An unknown kind is a `failed` reply. `live` also carries `uptimeSec`, a `startTicks` per process, and `tcpSockets` rows with `addr`, `port`, `uid` and `pid` (`null` when the owner's `/proc/<pid>/fd` is unreadable), listeners first. |
| `display.snapshot` | | An object holding every kind above, each as `display.get` returns it. One failing inventory stays on its own key. |
| `snapshot.keys` | `keys`: snapshot key names | `{values, unknown, failed, settled}`. What the machine reports now for those keys only, each from the producer the full snapshot uses, so a value has the same shape in both. `unknown` lists keys nothing produces, and `failed` names, with the reason, each key whose tool did not answer: such a key is not in `values`, and a reader keeps the value it had (docs/adr/0006). Under a fixture root `values` is empty and `unknown` is still computed. |
| `snapshot.topic` | `topic`: one of `settings.look`, `settings.network`, `settings.accounts`, `settings.system`, `settings.host` | The topic's document, read from the machine now: its keys as the producers report them with the settings files laid over, and `_failed`, a map from key to reason, for the keys whose tool did not answer (they are left out). The daemon asks for this itself, in a child process that carries a window's environment, to fill a topic. Under a fixture root the document is empty. |
| `snapshot.invalidate` | `topics` (optional list of settings topics or inventories, never `live`; none means the five settings topics) | `{invalidated}`. The daemon reads those topics again and pushes them to the windows that hold them. What a Refresh button sends. It waits for the writes ahead of it. A window without a daemon is told `unavailable`. |
| `write` | `name`, `args` | A typed write, see Writes below. `{snapshot, settled, coalesced}`: `snapshot` holds the keys the write can change as the machine reports them once it has run, and `settled` says whether they showed the expected state before the wait ran out. An unknown `name`, a missing or malformed argument, or an argument that could pass as an option is `bad_request` and nothing runs. Under a fixture root nothing runs and the reply is `{argv, snapshot: {}, settled: true}`. |
| `display.brightness` | `monitor`, `percent` (0 to 100) | `{monitor, percent, coalesced, snapshot}`. Runs `omarchy brightness display` on the monitor's own lane in the daemon, then reads the monitors back: `snapshot.monitors` is the list as the machine reports it now (absent when `hyprctl` failed, so a window keeps the list it has). A panel on DDC/CI answers a read right after a write with the old level, so the daemon re-reads that monitor for up to about a second until it reports the new one, give or take a point, and the reply shows that level. Writes queued on one monitor collapse to the newest, and a replaced caller gets that command's answer with `coalesced: true`. A bad `monitor` (letters, digits, `.`, `_`, `-` only) or `percent` is `bad_request` and nothing runs. Under a fixture root it only answers `{monitor, percent}`. |

| `display.scale` | `scale` (a number above 0 and at most 8, as text) | `{scale, coalesced, snapshot}`. Runs `omarchy hyprland monitor scaling` on the `hypr:monitors` lane, then returns the monitors as the machine reports them, like `display.brightness`. Under a fixture root it only answers `{scale}`. |

#### Writes and reads in order

`display.brightness` is a typed command: the daemon runs it on a lane (`monitor:<name>`), in order with other writes to that lane, and holds back reads of what it changes until it is done. A `settings.snapshot` of group `look`, `rest` or `all`, and any `display.get`, wait up to 3 seconds for the commands and leased jobs ahead of them, so a read after a write sees the write. `ratmos task` (a lane of its own) and `ratmos apply` (the shared `generic` lane, waiting at most 300 ms) lease a lane while they run, which holds back reads of `disks`, `services`, `software` and `diagnostics` the same way, and drops them when the job ends. The command runs in a child `ratmos request` that carries the caller's environment. The window's `serve` sends the command and the barrier over its session to the daemon as `req` frames. With no daemon, or an older one, a window's `serve` fails the write with `unavailable`; a terminal `ratmos request` or a `--root` fixture runs it in the caller's own process with no ordering across windows. A daemon that took a command and then did not answer is reported as a failure and the command is not run a second time.

#### Sample cache

On a live system `display.get` and `display.snapshot` answer from the samples `ratmos daemon` keeps (collected in the background by `ratmos sampler`, never on the reader's behalf) in `$XDG_RUNTIME_DIR/atmos/<kind>.json`, and the reply is the same document either way. A sample is served while it is younger than its kind allows: `live` one second (counters become rates, so a reader must never see one sample twice), every other kind three sample intervals. Past that, or with nothing cached, the request goes to the daemon over `$XDG_RUNTIME_DIR/atmos/ratmos.sock` with interactive priority and waits for the fresh sample, up to 2 seconds for `live` and 20 for the rest; simultaneous requests for one kind share one collection. With no daemon, a missing runtime dir, a daemon that does not answer in time, or any daemon failure, a request from a window fails with `unavailable` (docs/adr/0010), and one from a terminal (`ratmos display`, `ratmos request`) collects the inventory itself.

A `--root` fixture never reads the cache. `ratmos task`, `ratmos apply` and `settings.set` drop the kinds they can change (`disks`, `services`, `software`, `diagnostics`; `settings.set` only `services`), so the next read is fresh, and the daemon resamples them in the background after 5 seconds, but not within 30 seconds of the kind's last sample. Settings, `host.*`, `search.query` and the other ops are always read live.

### Search

| Op | Fields | Result |
| --- | --- | --- |
| `search.query` | `query`; `hubs`: `[{id, title, description, keywords}]`; `files`: page file to hub id; `prefixes`: `[[folder, hub]]`; `titles`: hub or child id to title | `{query, hits}`. The catalog is the hub rows plus the labelled rows ratmos finds by reading the page sources under the app root; a hit is a row whose text contains the query, case-insensitive, ordered by hub then label. The window sends the hub fields it already holds from `Hubs.js`. |

### Host

`host.read`, `host.write` and `host.open` confine the path they are given: live it must sit under `$HOME`, and under a fixture root it is re-rooted. A `..` step is refused with `denied`.

| Op | Fields | Result |
| --- | --- | --- |
| `host.chrome` | | `{themeName, colors, themeShell, userShell}`: the current theme's name, its `colors.toml` and `shell.toml` text, and `~/.config/omarchy/shell.toml`. |
| `host.themePack` | `name`, `home`, `part` (optional) | `{colors, shell}`, the `colors.toml` and `shell.toml` text of a theme. `part` of `colors.toml` or `shell.toml` skips the other file, which comes back empty. |
| `host.accounts` | `user`, `home` | `{hostname, passwd, group, exists}`: the text of `/etc/hostname`, `/etc/passwd` and `/etc/group`, and `exists`, a map from each avatar path (`/var/lib/AccountsService/icons/<user>`, `<home>/.face.icon`, `<home>/.face`) to whether the file is there. |
| `host.stamp` | `paths` | `{items: [{path, sig, text}]}`. `sig` is size and mtime, or `missing`. |
| `host.read` | `paths` | `{files: [{path, exists, text}]}`. A missing file is `exists: false`, not an error. |
| `host.write` | `path`, `text` | `{bytes, path}`. The write replaces the file atomically. |
| `host.open` | `path` | `{opened}`. Opens a file or an `http(s)` URL with `xdg-open`. A missing file is `not_found`. Under a fixture root it only answers. |

### Speed and units

| Op | Fields | Result |
| --- | --- | --- |
| `speedtest.disk` | `dir` (optional) | `{exit, stdout, stderr}` from the disk test. |
| `speedtest.net` | `phase`: `down` (default) or `up` | `{exit, stdout, stderr}` from the network test. Any other phase is a `failed` reply. |
| `unit.output` | `kind`: `status` or `logs`, `scope`: `system` (default) or `user`, `unit` | `{exit, text}` for a systemd unit. |

### Agents

These read and edit the MCP entry in each coding agent's own config; Atmos keeps no list of its own. `agents.mcp.set` and `agents.mcp.check` need Atmos installed and reply `not_found` when it is not.

| Op | Fields | Result |
| --- | --- | --- |
| `agents.mcp.list` | | One row per known agent: `{id, name, installed, writer, path, state, on}`. |
| `agents.mcp.set` | `agent`, `on`, `replace` (optional) | `{agent, on}`. An agent Atmos cannot write for is `denied`. |
| `agents.mcp.check` | | `{tools: [...]}`, the tool names `atmos mcp` reports. |

### Serve only

| Op | Fields | Result |
| --- | --- | --- |
| `watch.set` | `paths`, `chrome`, `accounts` | `{watching: true}`, then `stamp`, `chrome` and `accounts` events. See Long-lived above. |
| `job.start` | `cid`, `name`, `args`, `label` | `{sent: true}`, then `job` events with that `cid`. See Commands, launchers, jobs and interactive tools. |
| `job.input` | `job`, `data`, `close` | `{sent: true}`. Writes to a running job's stdin. |
| `job.cancel` | `job` | `{sent: true}`. Stops the job's unit. |
| `job.attach` | `job`, `from` | `{sent: true}`, then the job's output from line `from` and its end. |
| `job.list` | | `{jobs: [{job, cid, name, label, state, code, startedAt, lines}]}` |
| `sub.set` | `topics`: topic name to `{intervalMs}` | `{holding: [...]}`, then `sample` events. See Held topics above. |

#### Writes

`write` runs one of these by `name`. `args` are checked by the entry, and the window never sends a command line. A write runs in order on its lane in the daemon, in a child with the caller's environment. A privileged write asks for sudo mode first (the window's in-app prompt), so `sudo -n` in the child needs no session. Every write is answered with the keys it can change as the machine reports them after the command, and a write the machine finishes a moment after the command returns is waited for (up to about a second and a half) until the read-back shows the expected state, so the window is never shown the old state and never a guess.

| Name | Args | Runs |
| --- | --- | --- |
| `audio.output.mute-toggle` | | `omarchy audio output volume mute-toggle`; settled when `audioOutputMuted` flipped |
| `audio.input.mute-toggle` | | `omarchy audio input mute`; settled when `audioInputMuted` flipped |
| `audio.output.default` | `id`, `name` | `omarchy audio output set default`; settled when that sink is the default in `audioSinks` |
| `audio.input.default` | `id`, `name` | `omarchy audio input set default`; settled when that source is the default in `audioSources` |
| `power.profile` | `name` | `omarchy powerprofiles set autodetect` |
| `wifi.band` | `band`: `auto`, `2.4`, `5`, `6` | `omarchy network band` |
| `wifi.connection` | `action`: `up`, `down`, `delete`; `uuid` | `ratmos task set-wifi-connection`; settled on that connection's `active` flag or its absence |
| `bluetooth.device` | `action`: `connect`, `disconnect`, `forget`; `address` | `omarchy bluetooth device`; settled on that device's `connected` flag or its absence |
| `plugin.set` | `id`, `on` | `omarchy plugin enable` or `disable` |
| `bar.spacer` | `on` | `omarchy bar put omarchy.spacer` or `omarchy plugin disable omarchy.spacer` |
| `monitor.internal` | `on` | `omarchy hyprland monitor internal on` or `off` |
| `monitor.mirror` | `on` | `omarchy hyprland monitor internal mirror on` or `off` |
| `keyboard.backlight` | `direction`: `up`, `down`, `off` | `omarchy brightness keyboard --no-osd` |
| `display.text-size-reset` | | `omarchy display text size reset` |
| `theme.remove` | `name` | `omarchy theme remove` |
| `branding.reset` | `which`: `screensaver`, `about` | `omarchy branding <which> reset` |
| `hypr.reset` | `group`: `look`, `input` | `ratmos task reset-hypr` |
| `hypr.workspace-layout-toggle` | | `omarchy hyprland workspace layout toggle`; settled when `hyprWorkspaceLayout` flipped |
| `hypr.apply` | `group`: `look`, `input`; `payload` (JSON text) | `ratmos task hypr-apply` |
| `atmos.channel` | `channel`: `stable` | `ratmos task set-atmos-channel` |
| `weather.coordinates` | `name`, `coords` | `omarchy weather location --set` |
| `reminder.set` | `minutes`, `message` (optional) | `omarchy reminder` |
| `reminder.clear` | | `omarchy reminder clear` |
| `recording.stop` | | `omarchy capture screenrecording --stop-recording` |
| `hook.install` | `type`, `path` | `omarchy hook install` |
| `hook.create` | `type`, `name`, `command` | `ratmos task create-hook` |
| `hook.remove` | `path`, inside `~/.config/omarchy/hooks/` | removes the file in-process; a `.sample` is refused |
| `hook.sample` | `path`, `enabled` | `ratmos task set-hook-sample` |
| `tui.remove` | `name` | `omarchy tui remove` |
| `webapp.remove` | `name` | `omarchy webapp remove` |
| `app.remove-desktop` | `id`, `name` | `omarchy remove launcher entry` |
| `avatar.set` | `user`, `path` | `ratmos task set-avatar set` |
| `avatar.clear` | `user` | `ratmos task set-avatar clear` |
| `snapper.number-limit` | `n`, 1 to 50 | `ratmos task set-snapper-policy number-limit` |
| `snapper.timeline` | `on` | `ratmos task set-snapper-policy timeline` |
| `fstrim` | `on` | `ratmos task set-fstrim` |

`settings.set` accepts `readback`, a list of snapshot keys to read once the value is written, for the keys a setting changes besides its own; they come back as `snapshot` in the reply.

#### Commands, launchers, jobs and interactive tools

Everything else the window starts is an entry in `backend/src/command/entries.rs`, named and checked the same way: each argument says what it may be, so a value that could pass as an option, hold a line break or climb out of a path is refused before anything runs. The window sends a name and arguments and never a command line, and a password or a plan is an argument that travels as the command's stdin and is never in a command line or a job list.

- **Commands and launchers** go through `write`, like the writes above, with an empty `snapshot` unless the entry reads keys back. A **launcher** starts something that stays open (an app, a window) as a transient user unit of its own and does not wait for it.
- **Jobs and interactive tools** go through `job.start`. A job streams output, may read input and may take a while; an interactive tool needs the user's session (a picker, a region tool, a recorder). Both run in the daemon, as a transient user unit with the priority of an ordinary program of yours and not the daemon's, with the window's environment, on a lane. The reply to `job.start` is `{sent: true}`, and the rest arrives as events carrying the window's own `cid`:

```json
{"op": "job.start", "cid": "j7", "name": "theme.install", "args": {"url": "https://example.org/theme.git"}, "label": "theme-install"}
{"event": "job", "kind": "started", "cid": "j7", "job": 41}
{"event": "job", "kind": "stream", "cid": "j7", "job": 41, "seq": 0, "fd": "out", "line": "Cloning..."}
{"event": "job", "kind": "exit", "cid": "j7", "job": 41, "code": 0, "summary": {"ok": true, "code": 0, "message": "", "warnings": []}}
```

`job.input` (`job`, `data`, `close`) writes to a running job, `job.cancel` (`job`) stops it, and `job.attach` (`job`, `from`) points a job at this window and replays its output from line `from`, so a window that was closed and reopened finds what it missed. `job.list` answers with the daemon's jobs and the `label` each window gave them. A job outlives its window. The daemon does not replace itself for a rebuilt binary while a job runs. With no daemon `job.start` fails with `unavailable`: there is nothing to run the job.

##### Commands (`write`)

| Name | Args | Runs |
| --- | --- | --- |
| `theme.refresh` |  | `omarchy theme refresh` |
| `theme.bg-next` |  | `omarchy theme bg next` |
| `theme.bg-cache` |  | `omarchy theme bg cache` |
| `keyboard.backlight-restore` |  | `omarchy brightness keyboard --no-osd restore` |
| `account.member` | group, user, on | `ratmos task manage-account set-member <group> <user> on\|off` (needs root) |
| `window.transparency-toggle` |  | `omarchy hyprland window transparency toggle` |
| `window.tiled-fullscreen-toggle` |  | `omarchy hyprland window tiled fullscreen toggle` |
| `wifi.password-copy` | iface | `bash -c omarchy network password "$1" \| wl-copy -n wifi-password <iface>` |
| `clipboard.copy` | text (stdin) | `wl-copy -n` |
| `wifi.metered` | uuid, value | `ratmos task set-wifi-connection metered <uuid> <value>` |
| `wifi.priority` | uuid, value | `ratmos task set-wifi-connection priority <uuid> <value>` |
| `wifi.mac` | uuid, value | `ratmos task set-wifi-connection mac <uuid> <value>` |
| `wifi.ipv4-auto` | uuid | `ratmos task set-wifi-connection ipv4 <uuid> auto` |
| `wifi.ipv4-manual` | uuid, address, prefix, gateway?, dns? | `ratmos task set-wifi-connection ipv4 <uuid> manual <address> <prefix> [<gateway>] [<dns>]` |
| `wifi.wireguard-import` | path | `ratmos task set-wifi-connection wireguard-import <path>` |
| `wifi.hotspot-on` | ssid, password (stdin) | `ratmos task set-wifi-connection hotspot on <ssid>` |
| `wifi.hotspot-off` |  | `ratmos task set-wifi-connection hotspot off` |
| `favorites.write` | json | `ratmos task set-favorites write <json>` |
| `process.signal` | pid, signal | `ratmos task signal-process <pid> <signal>` |
| `systemd.user` | action, unit | `systemctl --user <action> <unit>` |
| `systemd.system` | action, unit | `sudo -n systemctl <action> <unit>` (needs root) |
| `wifi.ssid` | action, ssid | `ratmos task set-wifi-connection <action> <ssid>` |
| `restart` | what | `omarchy restart <what>` |
| `bluetooth.pair` | address | `omarchy bluetooth device pair <address>` |
| `bluetooth.trust` | address | `bluetoothctl trust <address>` |
| `audio.output-switch` |  | `omarchy audio output switch` |
| `notification` | kind | `omarchy notification <kind>` |
| `notification.test` |  | `omarchy notification send Atmos This is a test toast.` |
| `reminder.show` |  | `omarchy reminder show` |
| `webcam.resize` | action | `omarchy capture webcam resize <action>` |
| `share.clipboard` |  | `omarchy share clipboard` |
| `share.file` | path | `omarchy share file <path>` |
| `share.folder` | path | `omarchy share folder <path>` |
| `hook.run` | name, arg? | `omarchy hook <name> [<arg>]` |

##### Launchers (`write`)

| Name | Args | Runs |
| --- | --- | --- |
| `theme.folder` | theme | `bash -c dir=$(omarchy theme dir "$1") && [[ -d "$dir" ]] && xdg-open "$dir" >/dev/null 2>&1 & theme-dir <theme>` |
| `agent.prompt` | text | `bash -c omarchy agent prompt "$1" >/dev/null 2>&1 & agent-prompt <text>` |
| `hook.folder` | type | `bash -c dir="$HOME/.config/omarchy/hooks/$1.d"; mkdir -p "$dir"; xdg-open "$dir" >/dev/null 2>&1 & prefs-hooks <type>` |
| `theme.bg-folder` |  | `omarchy theme bg install` |
| `aether` |  | `aether` |
| `open.path` | path | `xdg-open <path>` |
| `cups.admin` |  | `xdg-open http://127.0.0.1:631` |
| `printers.setup` |  | `system-config-printer` |
| `terminal.herdr` |  | `omarchy launch terminal herdr` |

##### Jobs (`job.start`)

| Name | Args | Runs |
| --- | --- | --- |
| `theme.install` | url | `omarchy theme install <url>` |
| `theme.update` |  | `omarchy theme update` |
| `app.desktop-install` | name, command, icon | `ratmos task add-desktop-launcher <name> <command> <icon>` |
| `app.tui-install` | name, command, style, icon | `omarchy tui install <name> <command> <style> <icon>` |
| `app.webapp-install` | name, url, icon? | `omarchy webapp install <name> <url> [<icon>]` |
| `dns.custom` | servers | `ratmos task set-dns-custom <servers>` |
| `account.add-user` | name, full, wheel, password (stdin) | `ratmos task manage-account add-user <name> <full> true\|false` (needs root) |
| `account.remove-user` | name | `ratmos task manage-account remove-user <name>` (needs root) |
| `account.set-password` | name, password (stdin) | `ratmos task manage-account set-password <name>` (needs root) |
| `account.add-group` | name | `ratmos task manage-account add-group <name>` (needs root) |
| `account.remove-group` | name | `ratmos task manage-account remove-group <name>` (needs root) |
| `sshd.disable` |  | `ratmos task set-sshd disable` (needs root) |
| `sudo.enable` | minutes, password (stdin) | `ratmos task set-passwordless-sudo on <minutes> --sudo-ask` |
| `sudo.disable` |  | `ratmos task set-passwordless-sudo off` (needs root) |
| `update.omarchy-check` |  | `omarchy update available` |
| `update.atmos-check` |  | `ratmos task update-atmos check` |
| `update.atmos-apply` |  | `ratmos task update-atmos apply` |
| `wifi.qr` | iface? | `omarchy network qr --meta [<iface>]` |
| `diag.report` | action, dest?, header (stdin) | `ratmos task diag-report <action> [<dest>]` |
| `wifi.enterprise` | ssid, identity, password (stdin) | `ratmos task enterprise-wifi-connect <ssid> <identity>` |
| `wifi.join` | ssid, password? (stdin) | `ratmos task set-wifi-connection join <ssid>` |
| `luks.change-key` | device (stdin), current (stdin), new (stdin) | `ratmos task luks-change-key` (needs root) |
| `snapshot.create` |  | `omarchy snapshot create` (needs root) |
| `snapshot.rollback` | config, id | `ratmos task rollback-snapshot <config> <id>` (needs root) |
| `hibernation.setup` |  | `omarchy hibernation setup --force` (needs root) |
| `hibernation.remove` |  | `omarchy hibernation remove` (with progress, needs root) |
| `refresh.hyprland` |  | `ratmos task refresh-hyprland` |
| `refresh.shell` |  | `omarchy refresh shell` |
| `reset.atmos` |  | `ratmos task reset-atmos` |
| `plymouth.reset` |  | `omarchy plymouth reset` (needs root) |
| `plymouth.set` | path | `bash -c bg=$(omarchy theme color background); text=$(omarchy theme color foreground); omarchy plymouth set "$bg" "$text" "$1" plymouth-set <path>` (needs root) |
| `plymouth.preview` | path | `bash -c bg=$(omarchy theme color background); text=$(omarchy theme color foreground); out=$(mktemp --suffix=.png); omarchy plymouth preview "$bg" "$text" "$1" "$out"; echo "$out" plymouth-preview <path>` |
| `settings.apply` | plan (stdin) | `ratmos task settings-apply --progress` |
| `settings.undo` | plan | `ratmos task settings-apply --no-backup --plan <plan>` |
| `security.fingerprint-setup` |  | `omarchy setup security fingerprint` (with progress, needs root) |
| `security.fingerprint-remove` |  | `omarchy remove security fingerprint` (with progress, needs root) |
| `security.fido2-setup` |  | `omarchy setup security fido2` (with progress, needs root) |
| `security.fido2-remove` |  | `omarchy remove security fido2` (with progress, needs root) |
| `security.sshd-setup` | key? | `omarchy setup security sshd [--key=<key>]` (with progress, needs root) |
| `security.docker-setup` |  | `omarchy setup security sudoless docker` (with progress, needs root) |
| `security.docker-remove` |  | `omarchy remove security sudoless docker` (with progress, needs root) |
| `channel.set` | name | `omarchy channel set <name>` (with progress, needs root) |
| `update.omarchy` |  | `omarchy update` (with progress, needs root) |
| `update.firmware` |  | `omarchy update firmware` (with progress, needs root) |
| `update.orphans` |  | `omarchy update orphan pkgs` (with progress, needs root) |
| `update.prune` |  | `omarchy update pkg prune` (with progress, needs root) |
| `voxtype.install` |  | `omarchy voxtype install` (with progress, needs root) |
| `voxtype.remove` |  | `omarchy voxtype remove` (with progress, needs root) |
| `hybrid-gpu.toggle` |  | `omarchy toggle hybrid gpu` (with progress, needs root) |
| `tailscale.install` |  | `omarchy install service tailscale` (with progress, needs root) |
| `tailscale.remove` |  | `omarchy remove service tailscale` (with progress, needs root) |
| `direct-boot.setup` |  | `omarchy setup direct boot` (with progress, needs root) |
| `software.run` | action, words | `omarchy <action> <words...>` (with progress, needs root) |
| `dev-env.install` | lang | `omarchy install dev env <lang>` (with progress, needs root) |
| `dev-env.remove` | lang | `omarchy remove dev env <lang>` (with progress, needs root) |
| `docker-db.install` | name | `omarchy install docker dbs <name>` (with progress, needs root) |

##### Interactive tools (`job.start`)

| Name | Args | Runs |
| --- | --- | --- |
| `theme.switcher` |  | `bash -c theme=$(omarchy theme switcher \|\| true); [[ -n $theme ]] && omarchy theme set "$theme" >/dev/null 2>&1 &` |
| `theme.bg-switcher` |  | `omarchy theme bg-switcher` |
| `theme.bg-file` |  | `bash -c path=$(omarchy file select --title "Set background" --extensions "jpg jpeg png gif webp bmp" \|\| true); [[ -n $path ]] && omarchy theme bg set "$path"` |
| `branding.screensaver` | action | `omarchy branding screensaver <action>` |
| `branding.about` | action | `omarchy branding about <action>` |
| `capture.screenshot` | mode, dest | `omarchy capture screenshot <mode> <dest>` |
| `capture.recording` | fullscreen?, desktopAudio?, microphone?, webcam?, webcamSize? | `omarchy capture screenrecording [--fullscreen] [--with-desktop-audio] [--with-microphone-audio] [--with-webcam] [--webcam-size=<webcamSize>]` |
| `capture.text` |  | `omarchy capture text` |
| `capture.qr` |  | `omarchy capture qr` |
| `tailscale.send` | machine, path? | `omarchy tailscale send <machine> [<path>]` |
| `tailscale.receive` |  | `omarchy tailscale receive --once` |
test command::entries::tests::print_the_protocol_rows ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 378 filtered out; finished in 0.00s

