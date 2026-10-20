# Glossary

**Snapshot group** — What a caller of `settings.snapshot` or a Refresh button calls a part of the snapshot: `look`, `rest`, `all`, `network`, `disks`, `accounts`, `system`. The daemon holds the snapshot as five **settings topics** (`settings.look`, `settings.network`, `settings.accounts`, `settings.system`, `settings.host`), each key in exactly one; a group is only a request for some of them. Not a UI cluster and not an export bucket.

**Nav group** — Sidebar cluster in the prefs window: `look`, `input`, `device`, `apps`, `general`, `admin`.

**Export tier** — Settings.js plan/export bucket: `look`, `behavior`, `identity`, `system`.

**Omafile** — Markdown document of an Omarchy system's settings. Written and applied from the Omafile hub. Fenced `atmos:` blocks are the payload. Security settings are reported as prose and never imported.

**Row text** — Characters a list setting may write: a binding chord or label, a command, a window match, a window workspace, or a window size. `services/RowText.js`, included by the Hypr writers and the Omafile row check.

**Adopt** — `Snapshot.adopt(current, patch, adapters)` merges a patch into the last record, clamps only patched keys, and keeps unpatched object/array references.

**Emit** — Producing a settings topic or a group's document (`snapshot.topic`, `settings.snapshot`): the producers in `snapshot/keys.rs` with the settings files laid over them.

**Apply** — Copying an adopted record onto Omarchy properties (`copyRecord`). There is no optimistic patch any more: after a write the window applies the keys the backend read back (ADR 0005).

**Omarchy bag** — The `Omarchy.qml` singleton: live property bag plus argv. Not a prefs store. `hyprLook` and `hyprInput` stay nested; pages bind those objects.

**copyRecord** — Mechanical assign of adopted keys onto Omarchy properties via `adoptValue` / `adoptArray`. Walks `SnapshotGroups.copyableBagKeys()`, the keys the window has a property for. Does not clamp, sanitize, or normalize. Account keys stay on AccountsStore.

**Adapter** — A seam that talks to QML, disk, or a process. Two adapters make a real seam.

**Module** — Node-evalable `services/*.js` (or a QML owner such as Theme.qml) that holds the interface.

**Interface** — The functions tests eval. The test surface is the interface.

**Sentinel** — `-- atmos:look|input|autostart|bindings|windows|workspaces|monitors begin/end` blocks in Hypr Lua. Not the Sentinel hub.

**Sentinel hub** — The hub that flags processes over a CPU or memory limit and listening ports held by long-running processes. Unrelated to the Hypr Lua blocks above and to `ratmos sentinel`.

**hypr.atmos** — `~/.config/hypr/atmos.lua`, required as `hypr.atmos` next to `hypr.omafetch`, before `default.hypr.toggles`.

**ATMOS_SKIP_HYPR** — Shell env that skips Hypr writes. Read by `ratmos` (`tasks::user`, `settings-apply`), not by JS.

**Writer lock** — Concurrent Atmos windows serialize on disk. `backend/src/hyprland` takes a per-destination flock sidecar (`store::with_lock`), merges `_patch` onto the on-disk block, and publishes with atomic rename. The `shell.json` writers (idle, bar widgets) share `shell.json.atmos.lock`. The remaining writers (env, hyprsunset, nightlight, presentation, tweaks, avatar) lock and publish atomically. Watchers on `gtkSettingsFile` / `swappinessFile` and a second `envFile`→rest watch keep open windows in sync.

**Look payload** — A snapshot (or `job.apply`) whose group is `look`. Not the Appearance hub.

**job.apply** — Optional object on an enqueueIo job (`kind` `mut` or `job`). `mutProc` and `jobProc` adopt it on success as an object (no JSON round-trip through the Emit parser), then enqueue the job's refresh group.

**SnapshotGroups.KEYS** — The snapshot keys the window mirrors. Which topic holds a key is the backend's (`backend/src/snapshot/topics.rs`).

**Backend (QML)** — `services/Backend.qml`, the one connection to `ratmos`. It owns a single `ratmos serve` process, `request(body, callback)`, and `watch(owner, spec)`. If the server cannot run it falls back to one process per request and a once-a-second poll. Builders for every request live in `services/Requests.js`; the line protocol is `services/BackendProtocol.js`.

**ratmos serve** — Long-lived NDJSON server: `{id, op, ...}` in, `{id, ok, result|error}` out. Each request runs on its own thread. `watch.set` subscribes to file stamps, theme chrome, and accounts; the server pushes `{event, result}` only when they change. One process per window, so concurrent windows stay independent.

**Envelope** — Every ratmos answer: `{ok, version, platform, result}` or `{ok:false, error:{code, message, context}}`. `code` is one of `bad_request`, `not_found`, `denied`, `io`, `command`, `timeout`, `failed`. `services/Failure.js` turns either shape into banner text.

**Confine** — `host.read|write|open` accept only paths under `$HOME` (or the fixture root), with no `..` and no symlink out.

**IoQueue** — `services/IoQueue.qml`. The queue that serialises writes and reads, and the processes for jobs that are not plain backend requests (`ratmos apply` for argv writes and streaming jobs, plus blocking interactive tools). It starts a job and reports how it ended through signals; Omarchy does the domain work and calls `finished()`.

**SnapshotStore** — `services/SnapshotStore.qml`. Batches the window's asks to read topics again (`snapshot.invalidate`). **SnapshotFeed** (`SnapshotFeed.qml`) holds the five settings topics and applies what the daemon pushes; **DisplayFeed** does the same for the five inventories.

**Paths** — `services/Paths.qml`. Every file, directory, and script path Atmos touches.

**apply summary** — `ratmos apply` ends with one stdout line `@@ratmos-apply@@ {ok, code, message, warnings}`. `message` is stderr that is not known noise. `Failure.applyBanner` turns it into banner text.

**SettingTable** — `services/SettingTable.js`. Settings that need only "skip a no-op, then send it", written with `Omarchy.set(key, value)`. Settings with a rule of their own keep a named setter.

