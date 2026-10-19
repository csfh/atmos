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
| `apply -- CMD ARGS...` | Run a host command with its streams attached; used for long jobs. |
| `snapshot`, `display KIND`, `display-snapshot`, `version` | Whole documents for tests, without a request body. |
| `task NAME ARGS...` | One of the jobs the window runs through `apply`: accounts, avatars, Wi-Fi, hooks, updates, imports (`settings-apply`) and the rest. `ratmos task` with no name lists them. Failures print the reason on stderr and exit non-zero. |
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
| `unavailable` | Never sent by `ratmos`. `BackendProtocol.js` makes this envelope when the server cannot answer. |

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

A `watch.set` replaces the previous subscription; every field is optional, and an empty one turns that watch off. The reply is `{"watching": true}`. Then, about once a second, the server checks what it watches and pushes an event only when it changed. The first check after a `watch.set` always reports. An event has no `id`:

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

### Held display kinds: `sub.set`

`sub.set` says which display kinds the window holds, and the server pushes each new sample so the window polls nothing. It replaces the previous set; `{"topics": {}}` releases everything. An unknown kind is `bad_request`.

```json
{"id": 5, "op": "sub.set", "topics": {"live": {"intervalMs": 2000}}}
```

The reply is `{"holding": ["live"]}`. Then, on the holder's clock, each event is one sample, with `sampledAt` the Unix milliseconds the collection began (a window computes rates from it, never from its own clock) and `tookMs` its duration:

```json
{"event": "sample", "kind": "live", "sampledAt": 1760000000123, "tookMs": 9, "result": {"cpuIdle": 1, "cpuTotal": 2}}
```

When sampling fails, or a held kind goes quiet for six intervals (never less than 15 seconds), the event is `{"event": "sample", "kind": "live", "stale": true, "error": "..."}`. The last good sample stays where it is. `live` is sampled by the daemon only while some window holds it, no faster than the quickest holder asks and never faster than every 250 ms, and each window receives it at its own `intervalMs`. Other kinds are pushed when they are sampled, and once on `sub.set` if the daemon holds one.

The events come from the daemon over a session on `ratmos.sock` (`{"op": "session"}`, then `sub`, `unsub`, `req` frames one way and `event` and `res` frames the other; see `backend/src/sampler/session.rs`). With no daemon, or one that does not speak sessions, `serve` starts one when it can (`ratmos.service`, then a transient user unit, then a bare process; `ATMOS_NO_AUTOSTART=1` forbids it, and a `--root` fixture never does) and otherwise samples the held kinds itself on the same clock, so the events are the same.

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
| `settings.set` | `domain`, `value` | `{domain, value, file, encoding}` with the value as read back. `value` must match the domain's `type`. |
| `settings.snapshot` | `group` (default `all`), `keys` (optional) | An object of every set value; an unset value is absent, not null. `keys` limits it to the named top-level domains. |

### Displays and hardware

`display.get` runs one inventory and returns its document with a `collector` and `platform` added.

| Op | Fields | Result |
| --- | --- | --- |
| `display.get` | `kind`: `live`, `hardware`, `disks`, `services`, `software` or `diagnostics` | One inventory. An unknown kind is a `failed` reply. `live` also carries `uptimeSec`, a `startTicks` per process, and `tcpSockets` rows with `addr`, `port`, `uid` and `pid` (`null` when the owner's `/proc/<pid>/fd` is unreadable), listeners first. |
| `display.snapshot` | | An object holding every kind above, each as `display.get` returns it. One failing inventory stays on its own key. |
| `display.brightness` | `monitor`, `percent` (0 to 100) | `{monitor, percent, coalesced, snapshot}`. Runs `omarchy brightness display` on the monitor's own lane in the daemon, then reads the monitors back: `snapshot.monitors` is the list as the machine reports it now (absent when `hyprctl` failed, so a window keeps the list it has). A panel on DDC/CI answers a read right after a write with the old level, so the daemon re-reads that monitor for up to about a second until it reports the new one, give or take a point, and the reply shows that level. Writes queued on one monitor collapse to the newest, and a replaced caller gets that command's answer with `coalesced: true`. A bad `monitor` (letters, digits, `.`, `_`, `-` only) or `percent` is `bad_request` and nothing runs. Under a fixture root it only answers `{monitor, percent}`. |

| `display.scale` | `scale` (a number above 0 and at most 8, as text) | `{scale, coalesced, snapshot}`. Runs `omarchy hyprland monitor scaling` on the `hypr:monitors` lane, then returns the monitors as the machine reports them, like `display.brightness`. Under a fixture root it only answers `{scale}`. |

#### Writes and reads in order

`display.brightness` is a typed command: the daemon runs it on a lane (`monitor:<name>`), in order with other writes to that lane, and holds back reads of what it changes until it is done. A `settings.snapshot` of group `look`, `rest` or `all`, and any `display.get`, wait up to 3 seconds for the commands and leased jobs ahead of them, so a read after a write sees the write. `ratmos task` (a lane of its own) and `ratmos apply` (the shared `generic` lane, waiting at most 300 ms) lease a lane while they run, which holds back reads of `disks`, `services`, `software` and `diagnostics` the same way, and drops them when the job ends. The command runs in a child `ratmos request` that carries the caller's environment. The window's `serve` sends the command and the barrier over its session to the daemon as `req` frames. With no daemon, or an older one, the command runs in the caller's own process with no ordering across windows. A daemon that took a command and then did not answer is reported as a failure and the command is not run a second time.

#### Sample cache

On a live system `display.get` and `display.snapshot` answer from the samples `ratmos daemon` keeps (collected in the background by `ratmos sampler`, never on the reader's behalf) in `$XDG_RUNTIME_DIR/atmos/<kind>.json`, and the reply is the same document either way. A sample is served while it is younger than its kind allows: `live` one second (counters become rates, so a reader must never see one sample twice), every other kind three sample intervals. Past that, or with nothing cached, the request goes to the daemon over `$XDG_RUNTIME_DIR/atmos/ratmos.sock` with interactive priority and waits for the fresh sample, up to 2 seconds for `live` and 20 for the rest; simultaneous requests for one kind share one collection. With no daemon, a missing runtime dir, a daemon that does not answer in time, or any daemon failure, the request collects the inventory itself, as it did before the daemon existed.

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
| `sub.set` | `topics`: display kind to `{intervalMs}` | `{holding: [...]}`, then `sample` events. See Held display kinds above. |
