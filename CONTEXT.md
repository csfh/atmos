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
