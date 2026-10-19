# Architecture

Atmos is a QML window over one Rust backend. This page is the target design and the order to get there. `docs/protocol.md` is the wire contract as it stands today; where this page and that one differ, this page is the direction and that one is the truth until a phase lands.

## Rules

1. **Every read and every write goes through `ratmos`.** QML never touches the host. It reaches the backend only through `services/Backend.qml`.
2. **One authority owns state.** `ratmos daemon` is mandatory. Windows are clients.
3. **A failure is never an empty value.** A collector that fails leaves the last good value in place and says it is stale. `monitors: []` must mean Hyprland reported no monitors, not that `hyprctl` broke.
4. **Background work never competes with the foreground.** It runs in its own process under its own cgroup, behind a governor, and stops under load.
5. **A write is ordered with the reads it affects.** After a write, a read sees it.

## Processes

| Process | Unit | Priority | Job |
| --- | --- | --- | --- |
| `ratmos daemon` | `ratmos.service`, wanted by `graphical-session.target` | normal (`Nice=15`, `CPUWeight=20`) | Owns topics, runs the command bus, answers windows, runs foreground collections |
| `ratmos sampler` | `ratmos-sampler.service` | `CPUSchedulingPolicy=idle`, `IOSchedulingClass=idle`, `CPUWeight=1`, `CPUQuota` | Executes background collections the daemon asks for. No state, no scheduling decisions |
| `ratmos serve` | none, one per window | normal | Bridge between the window's stdio and the daemon socket. Keeps today's framing |
| `ratmos task`, `root`, `apply` | none | normal | Jobs that keep their own process tree: privileged work, streaming output, interactive shells |

`serve` makes sure a daemon exists when a window holds a topic and the socket is missing (`sampler/launch.rs`): the installed units first, then a transient user unit (`ratmos-adhoc.service`, with the daemon's own `Nice` and `CPUWeight`, outside the window's cgroup so closing that window does not end it), then a bare detached process. `ATMOS_NO_AUTOSTART=1` forbids it, and a `--root` fixture never starts one. It starts only the daemon, so with no sampler unit running the daemon does no background sampling. Topics fill on demand, which is slower and never wrong. A daemon leaves when its runtime dir is gone or the binary it runs has been replaced (a rebuild or an update; `VERSION` does not change between dev builds), and the next window starts a current one. `ratmos install` and `setup` stop `ratmos-adhoc.service` before they start the units, which could not take the socket from it.

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

`command::plan` is the registry: a request's lane, key and topics. `display.brightness` and `display.scale` are the typed ops so far. Brightness runs `omarchy brightness display`, waits until the panel reports the level it was given (a DDC/CI panel answers a read right after a write with the old one), and then reads the monitors back, so the reply carries the state the machine is in (`result.snapshot.monitors`) and the window shows that, never a patch it guessed. The daemon does not run the tool itself. It starts `ratmos request` as a child with the caller's environment and a `RATMOS_DIRECT` marker, so a daemon that outlives a Hyprland session never runs `hyprctl` with a stale `HYPRLAND_INSTANCE_SIGNATURE`. With no daemon, or an older one, the command runs in the caller's process, unordered across windows. A daemon that took a command and then did not answer is a failure for the caller, and the command is never run a second time.

**Privileged and interactive jobs keep their own process tree.** `--sudo-ask`, `ratmos root` and `runInteractive` need the caller's session and terminal, so the daemon does not run them. `ratmos task` and `ratmos apply` take a **lease** while they run: a one-shot connection that holds a lane until it closes, so a crashed holder releases it. A task leases a lane of its own (`job:<pid>`). `apply --`, the passthrough QML uses for writes the daemon has no typed op for, leases the shared `generic` lane, waiting at most 300 ms. While a lease is held, reads of `disks`, `services`, `software` and `diagnostics` wait, and releasing drops them.

## Wire

One Unix socket, `$XDG_RUNTIME_DIR/atmos/ratmos.sock`, NDJSON. A one-shot client sends one line and reads one line (`sample`, `ping`, `status`). A **session** is `{"op":"session"}`, answered by a `ready` frame, and then typed frames with ids multiplexed:

| Frame | Direction | Meaning | State |
| --- | --- | --- | --- |
| `sub`, `unsub` | client to daemon | Replace or drop the held topics, each with its interval | P2 |
| `event` | daemon to client | `{topic, sampledAt, tookMs, value}`, or `{topic, stale, error}` | P2 |
| `req` | client to daemon | Any op the one-shot socket answers, with an `id` | P2/P3: `serve` sends its `command` and `barrier` ops this way |
| `res` | daemon to client | The reply for that `id`, written directly so a backed-up event queue can never drop it | P2/P3 |
| `command`, `barrier` | one-shot ops | Run a typed command on its lane; wait for pending writes to a topic | P3, on the one-shot socket |
| `lease` | one-shot op, connection held | Hold a lane around a job that runs outside the daemon; closing the connection releases it | P3 |
| `stream` | daemon to client | Output of a long job, for jobs the daemon does run | not planned yet |

A session's writer has a bounded queue, and a window that falls behind is dropped, so one stalled window never holds up publishing. `ratmos serve` keeps the framing QML speaks (`docs/protocol.md`) and maps it onto the session: `sub.set` becomes `sub`, `event` frames become `{"event":"sample"}` lines. `serve` opens its session when the window starts, whether or not the window holds a topic, and keeps it for the window's life; `client::Bridge` carries the window's `command` and `barrier` ops over it as `req` and `res` frames, and falls back to the one-shot socket when the session is down. Reads of the cache and the other ops still run through `handle()` in `serve`'s own process, which asks the daemon on a miss. Where no daemon can be reached, or one predates sessions, `serve` samples the held topics itself on the same clock, and a held topic that goes quiet gets an error event after six intervals. `ratmos request` stays the one-shot fallback, and the golden replay and the `request.rs`, `Requests.js` and `protocol.md` checks stay as they are.

## Phases

| Phase | Content | State |
| --- | --- | --- |
| baseline | `ratmos daemon` as a user unit, cache-first `display.*` reads, interactive-first queue, drops after tasks | done |
| P0 | A failed `hyprctl` leaves the monitor list alone and logs why | done |
| P1 | Engine: topics with last-good and stale metadata, the governor, the sampler split into its own unit | done, except the two gaps below |
| P2 | Frames and subscriptions on the socket, `serve` as a bridge, `live` and stamps pushed, the QML polling Timers removed | done. `live` is pushed from the daemon with `sampledAt` rates and the live poll and stall timers are gone. Stamps were already pushed to QML by `serve` before P2 and still are; that loop is not on the daemon's bus (gap below) |
| P3 | Command bus: lanes, coalescing, read-your-writes, leases. Brightness first. This also retires the window's optimistic `apply:` patches: a patch crosses a QML signal, which hands its arrays back wrapped, and that once emptied the monitor list on the second brightness change (fixed in `Lists.js`, but the cause is the window writing state itself) | done. No window-built patch is left: brightness and monitor scale are their own typed ops, thirty-six other writes are named `write` entries in `command/writes.rs`, and a setting written through `settings.set` is read back by the keys it names. The window shows what the backend reports, never a value it worked out |
| P4 | The settings snapshot groups become topics; `apply --` passthroughs replaced by typed ops where they matter | |

### Known gaps after P1

- **The idle IO class is not a guarantee.** It only acts under an IO scheduler that honors priorities. On `none`, the NVMe default, it does nothing, so the CPU weight, the quota and the governor carry the promise. Checked under real systemd on one machine: the sampler gets `SCHED_IDLE`, `cpu.weight 1` and `cpu.max 10%`, and the quota does throttle its bursts.
- **A background job that times out is not cancelled.** The worker keeps running it, and the daemon sends it nothing new until it answers, so a starved sampler cannot pile up a burst of queued jobs.
- **A failed tool is still an empty string inside most collectors.** `capture()` turns a failed `systemctl`, `df` or `omarchy` call into empty output, so a collector can return a document with blank fields and succeed. The topics catch a collector that errors, panics or times out, not one that quietly returns less. Fixing that is an audit of every collector, and `hypr::monitors()` (P0) is the only one done.
- **Duty accounting is per run.** The governor charges each background collection the CPU the sampler process and its reaped children used while it ran, which is exact only because one background run goes at a time.

### Known gaps after P2

- **No socket activation.** `serve` starts the daemon (P2), but only when a window holds a topic, and `ratmos request` and a window's one-shot fallback never do. The authority exists whenever a window is open and a topic is held.
- **Stamps, chrome and accounts are still looked at once a second by each `serve`.** That is a stat loop in Rust, in the window's own process, with no forks, and it is not on the daemon's bus. Moving it there means inotify in the daemon and is not in P2.
- **Other QML timers poll.** The Wi-Fi and Bluetooth pages (900 ms) and the guard bar (200 ms) tick on their own timers. Each only rebuilds from state already in memory, Quickshell's network and Bluetooth models and the revert countdown, and asks the host nothing, so they were left alone.
- **A local fallback has no stall watchdog for a wedged collection,** since the sampler runs on the thread that would raise it. Only the session path reports a silent `live`.
- **Display kinds other than `live` are pushed but nothing in the window holds them yet.** They are still read with `display.get`.

### Known gaps after P3

- **Toggles change when the answer arrives, not when pressed.** Without the optimistic patch a control shows the new state after the write and its read-back, usually 0.2 to 1 s (a DDC panel or a Wi-Fi change takes longer). That is the cost of never showing a guess.
- **A write the machine finishes late is waited for, not guessed.** Each entry says what the read-back must show (`settle`), and the backend re-reads for up to about 1.5 s. If the machine never shows it, the reply says `settled: false` and carries what it does show.
- **A snapshot read that began before a write is not trusted for the keys the write set.** `Ops.dropStale` drops them from it. The same window race exists for a read that begins at the same instant as a write, which the barrier on `settings` covers.
- **`runCommand` keeps its `argv` for the history and the Preview list.** It is a label; the backend builds the command it runs.
- **Writes the window still sends as `apply -- <argv>` change nothing it shows.** Forty-six `runCommand` calls and thirty-six `runJob` calls remain (launchers, tests, notifications, installs that refresh a group afterwards). None sets or patches state in the window. They are not on a lane; `apply` leases the shared `generic` lane for them.
- **The window still sets two derived flags itself.** `passwordlessSudo` after the sudo-mode job exits cleanly, because the pending write proceeds at once, and the favorites list, which comes back from the file's stamp.
- **A local write (sudo or polkit) runs in the window's `serve`,** holding its own lane. It is ordered against reads, not against another window's write to the same lane.
- **The daemon runs a command's child at its own `Nice=15`,** so a write is slower on a saturated machine than the same write inline.
- **`hook.remove` is done in the daemon or the window, not by `rm`,** and only inside `~/.config/omarchy/hooks/`.

## Not in scope

- Running privileged or interactive jobs inside the daemon.
- A history store. Topics keep the latest value only.
- Any private preferences store. Settings still live in the files they live in today.
