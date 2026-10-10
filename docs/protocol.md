# Protocol

The window and `ratmos`, the Rust backend, talk in JSON. QML reaches `ratmos` only through `services/Backend.qml`, with request bodies built by `services/Requests.js`; `services/BackendProtocol.js` encodes and decodes the lines. This page is the wire format. The replies in `tests/golden/cases/` are recordings of it, replayed over both transports by `tests/golden.test.js`.

## Running ratmos

```
ratmos [--backend omarchy|plain] [--root DIR] [--sampler FILE] <command>
```

| Flag | Meaning |
| --- | --- |
| `--backend` | `omarchy` (default) talks to the real machine through `omarchy`, `hyprctl` and the Hyprland sentinels. `plain` keeps the same keys in its own config tree and answers host reads with stand-in data. |
| `--root DIR` | A fixture root. Every path, absolute ones too, is re-rooted under `DIR`, and the commands a write would run are appended to `DIR/commands.log` instead. Tests use it. |
| `--sampler FILE` | A script to run in place of the bundled collector for `display.get` `live`. |

| Command | What it does |
| --- | --- |
| `request` | Read one JSON request from stdin, print one reply, exit. |
| `serve` | Read requests line by line from stdin until it closes. One process per window. |
| `apply -- CMD ARGS...` | Run a host command with its streams attached; used for long jobs. |
| `snapshot`, `display KIND`, `display-snapshot`, `version` | Whole documents for scripts and tests, without a request body. |

## Replies

Every reply is an envelope:

```json
{"ok": true, "version": "0.1.0", "platform": {"id": "plain", "compositor": "none", "family": "portable"}, "result": "0.1.0"}
```

A failure keeps the same shape, with `result` null and an `error` in place of the answer:

```json
{"ok": false, "version": "0.1.0", "platform": {"...": "..."}, "result": null,
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

`watch.set` is the one op `serve` answers itself. It is not accepted over `request`.

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
< {"id":2,"ok":true,"result":{"watching":true},"version":"0.1.0"}
< {"event":"stamp","result":{"items":[{"path":"/note.txt","sig":"3:1700000000","text":"hi\n"}]}}
< {"id":1,"ok":true,"result":"0.1.0","version":"0.1.0"}
< {"event":"stamp","result":{"items":[{"path":"/note.txt","sig":"8:1700000000","text":"changed\n"}]}}
> not json
< {"ok":false,"error":{"code":"bad_request","context":null,"message":"request: expected ident at line 1 column 2"},"result":null,"version":"0.1.0"}
> {"id":3,"op":"nope"}
< {"id":3,"ok":false,"error":{"code":"bad_request","context":null,"message":"unknown variant `nope`, expected one of ..."},"result":null,"version":"0.1.0"}
```

Reply 2 came before reply 1. That is allowed.

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
| `display.get` | `kind`: `live`, `hardware`, `disks`, `services`, `software` or `diagnostics` | One inventory. An unknown kind is a `failed` reply. |
| `display.snapshot` | | An object holding every kind above, each as `display.get` returns it. One failing inventory stays on its own key. |

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
