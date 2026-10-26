# Architecture

Atmos is a QML window over one Rust backend. This page is the design and the order it was built in; the reasons for each rule are the records in [`docs/adr/`](adr/README.md). `docs/protocol.md` is the wire contract as it stands today; where this page and that one differ, this page is the direction and that one is the truth until a phase lands.

## Rules

1. **Every read and every write goes through `ratmos`.** QML never touches the host. It reaches the backend only through `services/Backend.qml`.
2. **One authority owns state.** `ratmos daemon` is mandatory. Windows are clients.
3. **A failure is never an empty value.** A collector that fails leaves the last good value in place and says it is stale. `monitors: []` must mean Hyprland reported no monitors, not that `hyprctl` broke.
4. **Background work never competes with the foreground.** It runs in its own process under its own cgroup, behind a governor, and stops under load.
5. **A write is ordered with the reads it affects.** After a write, a read sees it.

## Processes

| Process | Unit | Priority | Job |
| --- | --- | --- | --- |
| `ratmos daemon` | `ratmos.service`, wanted by `graphical-session.target`, started on a connection by `ratmos.socket` | normal (`Nice=15`, `CPUWeight=20`) | Owns topics, runs the command bus, answers windows, runs foreground collections |
| `ratmos sampler` | `ratmos-sampler.service` | `CPUSchedulingPolicy=idle`, `IOSchedulingClass=idle`, `CPUWeight=1`, `CPUQuota` | Executes background collections the daemon asks for. No state, no scheduling decisions |
| `ratmos serve` | none, one per window | normal | Bridge between the window's stdio and the daemon socket. Keeps today's framing |
| `ratmos task`, `root`, `apply` | none | normal | Terminal tools. Jobs the window starts run as transient units of the daemon (ADR 0009), at normal priority |

A socket unit (`ratmos.socket`, `packaging/ratmos.socket`) listens on the daemon's socket from login, and systemd starts the daemon on the first connection and hands it the open socket (`LISTEN_FDS`), so on an installed system a window never finds the authority missing and a daemon that left (binary replaced) is started again by the next connection. Where the units are not installed, `serve` makes sure a daemon exists when the socket is missing (`sampler/launch.rs`): the installed units first, then a transient user unit (`ratmos-adhoc.service`, with the daemon's own `Nice` and `CPUWeight`, outside the window's cgroup so closing that window does not end it), then a bare detached process. `ATMOS_NO_AUTOSTART=1` forbids it, and a `--root` fixture never starts one. It starts only the daemon, so with no sampler unit running the daemon does no background sampling. Topics fill on demand, which is slower and never wrong. A daemon leaves when its runtime dir is gone or the binary it runs has been replaced (a rebuild or an update; `VERSION` does not change between dev builds), and the next window starts a current one. `ratmos install` and `setup` stop `ratmos-adhoc.service` before they start the units, which could not take the socket from it.

## The sampling bus

A **topic** is one inventory: the six display kinds today, and the settings snapshot groups (`look`, `network`, ...) later. Each holds:

- the last good value, a `seq`, and `sampledAt`;
- `stale` and the last error, when the latest collection failed;
- a policy: interval, how old a reader may be served, and a cost class.

Windows `sub` to topics and receive `event` frames when a topic changes, so nothing in QML polls. `live` is pushed while a page holds it. The daemon samples it itself, in the foreground lane because a window is waiting on it, no faster than the quickest holder asks, and hands each window a sample only once that window's own interval has passed. Its events carry `sampledAt`, and the live store computes rates from that rather than `Date.now()`, so a late or bunched event is still correct, and drops a sample it already has.

A read (`get`) is answered from the topic when its value is fresh enough. A cold or too-old topic is collected in the foreground at once, in the daemon, and an interactive request never joins a background run.

### Cost classes

| Class | Examples | Background polling |
| --- | --- | --- |
| `cheap` | `/proc` and `/sys` reads | yes, budgeted |
| `spawn` | `systemctl`, `df`, `snapper` | yes, budgeted, longer intervals |
| `device` | DDC/CI brightness over i2c | **never**; read on demand, and on the same lane as the write |

### The governor

The daemon decides whether a background collection may start. All of these must hold:

- CPU pressure is low: `/proc/pressure/cpu` `some avg10` under the threshold, or load average per core when PSI is absent;
- the duty budget allows it, about 1% of one core averaged over a window, counting the sampler's children (`cutime`, `cstime`);
- no foreground collection is running;
- no other background collection is running.

Pressure backs a topic off exponentially, and a quiet machine recovers it. The sampler unit's `CPUQuota` is the hard backstop. "Never contends" means this bound, not a promise about every instant: under sustained load, background sampling stops and foreground reads carry the page.

## The command bus

Every write, and every read of a device, is a command with a **lane**, such as `monitor:DP-3`. The bus is `sampler/commands.rs`; what a command is lives in `command.rs`, beside `request.rs`.

- Commands on one lane run in order. Different lanes run in parallel.
- A lane may coalesce: a command queued behind only commands of its own key is replaced by the newest, so a dragged slider costs one write per settle. A replaced caller still gets an answer: the newest command's, flagged `coalesced`.
- A command declares the topics it affects. When it finishes the daemon runs the invalidation hook (it drops the display kinds among them) and only then lets readers go.
- A read of an affected topic waits for what is pending ahead of it, for at most three seconds (`barrier`). `settings.snapshot` of `look`, `rest` and `all` waits on `monitors`, and a `display.get` on its kind.

`command::plan` is the registry: a request's lane, key and topics. `display.brightness` and `display.scale` are the typed ops so far. Brightness runs `omarchy brightness display`, waits until the panel reports the level it was given (a DDC/CI panel answers a read right after a write with the old one), and then reads the monitors back, so the reply carries the state the machine is in (`result.snapshot.monitors`) and the window shows that, never a patch it guessed. The daemon does not run the tool itself. It starts `ratmos request` as a child with the caller's environment and a `RATMOS_DIRECT` marker, so a daemon that outlives a Hyprland session never runs `hyprctl` with a stale `HYPRLAND_INSTANCE_SIGNATURE`. A window's `serve` has no other way to run it, so with no daemon, or an older one, the write fails with `unavailable` (ADR 0010); only a terminal `ratmos request` or a `--root` fixture runs it in the caller's process, unordered across windows. A daemon that took a command and then did not answer is a failure for the caller, and the command is never run a second time.

**Jobs run in the daemon, in units of their own.** A job or interactive tool the window starts (`job.start`) runs as a transient user unit (`systemd-run --user --pipe --wait`), at the priority of an ordinary program and not the daemon's, on a lane, with the window's environment. It outlives the window; a window that opens later lists the jobs and attaches. `ratmos task` and `ratmos apply` take a **lease** while they run from a terminal: a one-shot connection that holds a lane until it closes. While a lane is held, reads of `disks`, `services`, `software` and `diagnostics` wait, and releasing drops them. See ADR 0009.

## Wire

One Unix socket, `$XDG_RUNTIME_DIR/atmos/ratmos.sock`, NDJSON. A one-shot client sends one line and reads one line (`sample`, `ping`, `status`). A **session** is `{"op":"session"}`, answered by a `ready` frame, and then typed frames with ids multiplexed:

| Frame | Direction | Meaning | State |
| --- | --- | --- | --- |
| `sub`, `unsub` | client to daemon | Replace or drop the held topics, each with its interval | P2 |
| `watch` | client to daemon | Replace the files, theme and accounts the window watches; the daemon answers with `event` frames for `stamp`, `chrome` and `accounts` | P2 |
| `event` | daemon to client | `{topic, sampledAt, tookMs, value}`, or `{topic, stale, error}` | P2 |
| `req` | client to daemon | Any op the one-shot socket answers, with an `id` | P2/P3: `serve` sends its `command` and `barrier` ops this way |
| `res` | daemon to client | The reply for that `id`, written directly so a backed-up event queue can never drop it | P2/P3 |
| `command`, `barrier` | one-shot ops | Run a typed command on its lane; wait for pending writes to a topic | P3, on the one-shot socket |
| `lease` | one-shot op, connection held | Hold a lane around a job that runs outside the daemon; closing the connection releases it | P3 |
| `stream` | daemon to client | Output of a long job, for jobs the daemon does run | not planned yet |

A session's writer has a bounded queue, and a window that falls behind is dropped, so one stalled window never holds up publishing. `ratmos serve` keeps the framing QML speaks (`docs/protocol.md`) and maps it onto the session: `sub.set` becomes `sub`, `event` frames become `{"event":"sample"}` lines. `serve` opens its session when the window starts, whether or not the window holds a topic, and keeps it for the window's life; `client::Bridge` carries the window's `command` and `barrier` ops over it as `req` and `res` frames, and falls back to the one-shot socket when the session is down. Every other op is handed to the daemon, which runs it in a child `ratmos request` with the window's environment, on a lane of its own for a read (ADR 0012); `display.get`, `display.snapshot` and `snapshot.invalidate` only ask the daemon for what it holds. `serve` fails with `unavailable` when none answers. `serve` looks for the daemon for as long as the window lives (starting it, then again after one, two, four and five seconds) and says whether it is there with a `{"event":"backend"}` line; nothing is sampled or watched in its place, and a held topic that goes quiet gets an error event after six intervals. A `--root` fixture has no daemon and samples its own files. `ratmos request` stays the terminal one-shot, and the golden replay and the `request.rs`, `Requests.js` and `protocol.md` checks stay as they are.

## Phases

| Phase | Content | State |
| --- | --- | --- |
| baseline | `ratmos daemon` as a user unit, cache-first `display.*` reads, interactive-first queue, drops after tasks | done |
| P0 | A failed `hyprctl` leaves the monitor list alone and logs why | done |
| P1 | Engine: topics with last-good and stale metadata, the governor, the sampler split into its own unit | done. What it cannot guarantee on every machine is listed under its limits below |
| P2 | Frames and subscriptions on the socket, `serve` as a bridge, `live` and stamps pushed, the QML polling Timers removed | done. `live`, the watched files (stamp, chrome, accounts) and the window's commands all ride the one session to the daemon, which looks at files once for every window; the live poll and stall timers are gone from QML; the daemon starts on a connection through `ratmos.socket` |
| P3 | Command bus: lanes, coalescing, read-your-writes, leases. Brightness first. This also retires the window's optimistic `apply:` patches: a patch crosses a QML signal, which hands its arrays back wrapped, and that once emptied the monitor list on the second brightness change (fixed in `Lists.js`, but the cause is the window writing state itself) | done. No window-built patch is left: brightness and monitor scale are their own typed ops, thirty-six other writes are named `write` entries in `command/writes.rs`, and a setting written through `settings.set` is read back by the keys it names. The window shows what the backend reports, never a value it worked out |
| P4 | The settings snapshot groups become topics; `apply --` passthroughs replaced by typed ops where they matter; jobs and the remaining window-side fallbacks move onto the bus | done. Every command, job and interactive tool is a typed registry entry and jobs run in the daemon (ADR 0009); nothing in the window runs without the daemon (ADR 0010); the window holds the five inventories and the five settings topics and reads neither (ADR 0011), and a tool that fails is a missing key with a reason (ADR 0006); `settings.set`, `host.write`, `host.open` and `agents.mcp.set` are commands on lanes. See *Limits after P4*. The window is to be replaced by a new frontend, so this is done against the protocol and the registry, not against QML call sites |
| P5 | `serve` becomes a pure bridge: every read it still answered in its own process (`settings.get`, `settings.list`, `settings.snapshot`, `snapshot.keys`, `host.*`, `search.query`, `agents.mcp.*`, `speedtest.*`, `unit.output`, `platform`, `version`) is run by the daemon, in a child with the window's environment, on a lane of its own | done. See ADR 0012; `backend/tests/serve.rs` sends every op `request.rs` accepts over a window's `serve` and checks the daemon ran it |
| P6 | Held state as accurate as it can be without a poll: a failed tool takes its own key and not its producer's; the watch gaps are closed (`/etc/sudoers.d` for `passwordlessSudo`, Hyprland's event socket for monitors, which `std` can read); and the file watcher decision is settled, since ADR 0007 rules out inotify without a foreign call | planned. The GPUI frontend is outside both |

### Limits after P1

- **The idle IO class is not a guarantee.** It only acts under an IO scheduler that honors priorities. On `none`, the NVMe default, it does nothing, so the CPU weight, the quota and the governor carry the promise. Checked under real systemd on one machine: the sampler gets `SCHED_IDLE`, `cpu.weight 1` and `cpu.max 10%`, and the quota does throttle its bursts.
- **A background job that times out is not cancelled.** The worker keeps running it, and the daemon sends it nothing new until it answers, so a starved sampler cannot pile up a burst of queued jobs.
- **A failed tool is an error in the display collectors that depend on one.** `services` (`systemctl`) and `disks` (`lsblk`, `df`) answer with the failure when their tool fails, so the topic goes stale and keeps its last good sample (`inventory::require`, `tests/inventory_failure.rs`). The others were audited and need nothing: `hardware` reads sysfs and falls back to it when `lspci` fails, `diagnostics` reports each check's own failure as its result, `live` reads `/proc` and `software` reads desktop files, and a tool they call that is absent leaves a field absent by design. The settings snapshot is not covered: its producers still turn a failed `omarchy` call into an empty field, apart from `hyprctl` (P0). That belongs with moving the snapshot groups onto the bus.
- **Duty accounting is per run.** The governor charges each background collection the CPU the sampler process and its reaped children used while it ran, which is exact only because one background run goes at a time.

### Limits after P2

- **File watching is a poll.** The daemon looks at the watched paths once a second for every window together, one `stat` per distinct path, and reads a file's text only when its signature moved. Without a foreign call there is no inotify; a window with no daemon looks at nothing (ADR 0010).
- **Other QML timers poll.** The Wi-Fi and Bluetooth pages (900 ms) and the guard bar (200 ms) tick on their own timers. Each only rebuilds from state already in memory, Quickshell's network and Bluetooth models and the revert countdown, and asks the host nothing, so they were left alone.

### Limits after P3

- **Toggles change when the answer arrives, not when pressed.** Without the optimistic patch a control shows the new state after the write and its read-back, usually 0.2 to 1 s (a DDC panel or a Wi-Fi change takes longer). That is the cost of never showing a guess.
- **A write the machine finishes late is waited for, not guessed.** Each entry says what the read-back must show (`settle`), and the backend re-reads for up to about 1.5 s. If the machine never shows it, the reply says `settled: false` and carries what it does show.
- **A snapshot read that began before a write is not trusted for the keys the write set.** `Ops.dropStale` drops them from it. The same window race exists for a read that begins at the same instant as a write, which the barrier on `settings` covers.
- **`runCommand` keeps its `argv` for the history and the Preview list.** It is a label; the backend builds the command it runs.
- **The daemon runs a command's child at its own `Nice=15`,** so a write is slower on a saturated machine than the same write inline.
- **`hook.remove` is done in the daemon or the window, not by `rm`,** and only inside `~/.config/omarchy/hooks/`.

### Limits after jobs (ADR 0009)

- **A cancel needs the user manager that started the unit.** `systemctl --user kill` fails from a daemon in another runtime directory, and the job runs on; nothing tells the window.
- **A window sees running jobs, not finished ones.** A window that opens while a job runs attaches and replays its output. One that finished while no window was open is gone from the next window's point of view; the daemon keeps it only for `job.list`.
- **`KillMode=process`.** What a tool starts and leaves behind (a recorder, a clipboard holder) is not the job's to end, so a cancel signals the whole unit but a job that has already exited leaves its children alone.
- **An interactive tool that backgrounds its work stays "running" while the background process holds the job's output open.** The recorder keeps the window's recording job running until the recording stops, as `ratmos apply` did.
- **The first job of a new window waits for the session** to the daemon, up to eight seconds, while the daemon starts.
- **Proven in a real window against a real daemon:** the file picker (start, cancel, reattach from a second window), the region picker (start, cancel), the recorder (start, 4.3 s recording, stop), the in-app sudo prompt (password on stdin, wrong password). Not run: text and QR capture, which need a selection made by hand, and the clipboard holder they leave behind.

## Limits after P4

- **The settings topics are collected in a child process,** one `ratmos request` per topic carrying the newest window's environment, so a collection costs a process and a daemon with no window cannot collect one. They are collected when a window first holds them and when something invalidates them, never on a clock. A cached topic is sent to a window only when the files it is read from have not moved since the daemon last looked, and the daemon looks before every hold (ADR 0015), so a change to a watched file made while no window held its topic shows at the next hold. A change made outside Atmos to a file the daemon does not watch (`snapshot/topics.rs`) or to a thing with no file (a link coming up, a monitor plugged in) shows at the next invalidation, as it did when the window read groups.
- **A producer's failure takes its whole key set with it.** A tool that fails leaves every key of that producer out (a Wi-Fi failure does not touch Bluetooth, which is its own producer, but a failed `hyprctl getoption` leaves all of `hyprLook` out), and the window shows the values it had. Only the calls whose answer is the value are checked for an exit code; a missing tool is a failure only for `omarchy`, `omarchy-shell` and `hyprctl`.
- **The file watcher is a poll,** once a second while a window holds a settings topic, and once more whenever a window holds one, one `stat` per distinct path (directories walk to a bounded depth). What it saw is kept while nothing is held. It invalidates every topic whose files moved, a write the daemon ran itself included, so such a write is followed within a second by a collection of its topic (ADR 0015).
- **A window sees a Refresh button's answer when the collection ends,** 0.3 to 1 s for most topics and a few seconds for `settings.host`, which runs the slowest producers. The window no longer says which group to read first.
- **`apply --` has no window caller.** It stays a terminal subcommand (ADR 0009); there is no passthrough left to type.
- **`favorites` and `passwordlessSudo` are snapshot keys,** read back after their writes. `passwordlessSudo` does not follow the sudoers file expiring on its own: nothing watches `/etc/sudoers.d`.

### Limits after P5

- **A read costs a process.** A few milliseconds, where it was a thread in `serve`. `search.query`, the one a window sends often, is sent once per change of the text.
- **At most four read children run at once,** so a grid that asks once per item queues; a speed test takes no slot.
- **A daemon older than the window cannot run its reads.** The window refuses it (`proto` in the session's `ready` frame) and shows its screen. Replacing the daemon is deferred while a job runs, so a window that outlives an update of Atmos shows the screen until the job ends and the daemon restarts.
- **`version` and `platform` answer for the daemon,** not the window.
- **`status` counts what the daemon ran for windows** (`commandsRun`), next to `requestFrames`.
- **A read runs at the daemon's priority** (`Nice=15` under the unit) and with the 120 s limit of every command, so it is slower on a saturated machine, and a disk speed test that needed longer would be cut off.
- **`display.get`, `display.snapshot` and `snapshot.invalidate` stay in `serve`.** They only ask the daemon for what it holds.

## Not in scope

- A history store. Topics keep the latest value only.
- Any private preferences store. Settings still live in the files they live in today.
