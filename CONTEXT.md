# Glossary

**Snapshot group** — Wire identity of a snapshot JSON payload: `look`, `rest`, `all`, `network`, `disks`, `accounts`, `system`. Not a UI cluster and not an export bucket.

**Nav group** — Sidebar cluster in the prefs window: `look`, `input`, `device`, `apps`, `general`, `admin`.

**Export tier** — Settings.js plan/export bucket: `look`, `behavior`, `identity`, `system`.

**Omafile** — Markdown document of an Omarchy system's settings. Written and applied from the Omafile hub. Fenced `atmos:` blocks are the payload. Security settings are reported as prose and never imported.

**Adopt** — `Snapshot.adopt(current, patch, adapters)` merges a patch into the last record, clamps only patched keys, and keeps unpatched object/array references.

**Emit** — Producing snapshot JSON for a group (`scripts/snapshot.sh`, later SnapshotGroups).

**Apply** — Copying an adopted record onto Omarchy properties (`copyRecord`), or an optimistic `job.apply` patch on an enqueueIo job.

**Omarchy bag** — The `Omarchy.qml` singleton: live property bag plus argv. Not a prefs store.

**copyRecord** — Mechanical assign of adopted keys onto Omarchy properties via `adoptValue` / `adoptArray`. Does not clamp, sanitize, or normalize.

**Adapter** — A seam that talks to QML, disk, or a process. Two adapters make a real seam.

**Module** — Node-evalable `services/*.js` (or a QML owner such as Theme.qml) that holds the interface.

**Interface** — The functions tests eval. The test surface is the interface.

**Sentinel** — `-- atmos:look|input|autostart|bindings|windows begin/end` blocks in Hypr Lua.

**hypr.atmos** — `~/.config/hypr/atmos.lua`, required as `hypr.atmos` next to `hypr.omafetch`, before `default.hypr.toggles`.

**ATMOS_SKIP_HYPR** — Shell env that skips Hypr writes. Lives in `atmos-env.sh` / `apply-settings.sh`, not in JS.

**Look payload** — A snapshot (or `job.apply`) whose group is `look`. Not the Appearance hub.

**job.apply** — Optional object on an enqueueIo job (`kind` `mut` or `job`). `mutProc` and `jobProc` adopt it on success, then enqueue the job's refresh group.
