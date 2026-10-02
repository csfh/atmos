# Glossary

**Snapshot group** — Wire identity of a snapshot JSON payload: `look`, `rest`, `all`, `network`, `disks`, `accounts`, `system`. Not a UI cluster and not an export bucket.

**Nav group** — Sidebar cluster in the prefs window: `look`, `input`, `device`, `apps`, `general`, `admin`.

**Export tier** — Settings.js plan/export bucket: `look`, `behavior`, `identity`, `system`.

**Omafile** — Markdown document of an Omarchy system's settings. Written and applied from the Omafile hub. Fenced `atmos:` blocks are the payload. Security settings are reported as prose and never imported.

**Adopt** — `Snapshot.adopt(current, patch, adapters)` merges a patch into the last record, clamps only patched keys, and keeps unpatched object/array references.

**Emit** — Producing snapshot JSON for a group (`scripts/snapshot.sh`, later SnapshotGroups).

**Apply** — Copying an adopted record onto Omarchy properties (`copyRecord`), or an optimistic `job.apply` patch on an enqueueIo job.

**Omarchy bag** — The `Omarchy.qml` singleton: live property bag plus argv. Not a prefs store. `hyprLook` and `hyprInput` stay nested; pages bind those objects.

**copyRecord** — Mechanical assign of adopted keys onto Omarchy properties via `adoptValue` / `adoptArray`. Walks `SnapshotGroups.copyableBagKeys()`. Does not clamp, sanitize, or normalize. Account keys stay on AccountsStore.

**Adapter** — A seam that talks to QML, disk, or a process. Two adapters make a real seam.

**Module** — Node-evalable `services/*.js` (or a QML owner such as Theme.qml) that holds the interface.

**Interface** — The functions tests eval. The test surface is the interface.

**Sentinel** — `-- atmos:look|input|autostart|bindings|windows begin/end` blocks in Hypr Lua.

**hypr.atmos** — `~/.config/hypr/atmos.lua`, required as `hypr.atmos` next to `hypr.omafetch`, before `default.hypr.toggles`.

**ATMOS_SKIP_HYPR** — Shell env that skips Hypr writes. Lives in `atmos-env.sh` / `apply-settings.sh`, not in JS.

**Writer lock** — Concurrent Atmos windows serialize on disk. `hypr-sentinel.py` takes a per-destination flock sidecar, merges `_patch` onto the on-disk block, and publishes with atomic rename. `set-idle.sh` / `set-bar-widget.sh` / `set-workspace-bar.sh` share `shell.json.atmos.lock`. Remaining writers (`set-env.sh`, `set-hyprsunset.sh`, `set-nightlight-temp.sh`, `set-presentation.sh`, `set-tweaks.sh`, `set-avatar.sh`) lock and publish atomically. Watchers on `gtkSettingsFile` / `swappinessFile` and a second `envFile`→rest watch keep open windows in sync.

**Look payload** — A snapshot (or `job.apply`) whose group is `look`. Not the Appearance hub.

**job.apply** — Optional object on an enqueueIo job (`kind` `mut` or `job`). `mutProc` and `jobProc` adopt it on success as an object (no JSON round-trip through the Emit parser), then enqueue the job's refresh group.

**emitKeys** — Snapshot key identity in SnapshotGroups. `allowedKey`, `copyableBagKeys`, `groupForKey`, and `tagApply` read it. snapshot.sh remains the live Emit adapter that fills values.

**Backend (QML)** — `services/Backend.qml`, the one connection to `ratmos`. It owns a single `ratmos serve` process, `request(body, callback)`, and `watch(owner, spec)`. If the server cannot run it falls back to one process per request and a once-a-second poll. Builders for every request live in `services/Requests.js`; the line protocol is `services/BackendProtocol.js`.

**ratmos serve** — Long-lived NDJSON server: `{id, op, ...}` in, `{id, ok, result|error}` out. Each request runs on its own thread. `watch.set` subscribes to file stamps, theme chrome, and accounts; the server pushes `{event, result}` only when they change. One process per window, so concurrent windows stay independent.

**Envelope** — Every ratmos answer: `{ok, version, platform, result}` or `{ok:false, error:{code, message, context}}`. `code` is one of `bad_request`, `not_found`, `denied`, `io`, `command`, `timeout`, `failed`. `services/Failure.js` turns either shape into banner text.

**Confine** — `host.read|write|open` accept only paths under `$HOME` (or the fixture root), with no `..` and no symlink out.

**IoQueue** — `services/IoQueue.qml`. The queue that serialises writes and reads, and the processes for jobs that are not plain backend requests (`ratmos apply` for argv writes and streaming jobs, plus blocking interactive tools). It starts a job and reports how it ended through signals; Omarchy does the domain work and calls `finished()`.

**SnapshotStore** — `services/SnapshotStore.qml`. Which snapshot groups to read at the start of a session, batched refreshes, and the file watch list. A pushed stamp for a changed file re-reads that file's group; the two files Omarchy reads itself arrive on `stamped`.

**Paths** — `services/Paths.qml`. Every file, directory, and script path Atmos touches.

**apply summary** — `ratmos apply` ends with one stdout line `@@ratmos-apply@@ {ok, code, message, warnings}`. `message` is stderr that is not known noise. `Failure.applyBanner` turns it into banner text.

**SettingTable** — `services/SettingTable.js`. Settings that need only "skip a no-op, then send it", written with `Omarchy.set(key, value)`. Settings with a rule of their own keep a named setter.

