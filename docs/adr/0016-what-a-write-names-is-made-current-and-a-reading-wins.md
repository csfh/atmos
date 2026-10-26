# 0016. What a write names is made current, and a reading wins over a stored value

Status: Accepted. Date: 2026-10-25.

## Context

ADR 0011 made the settings snapshot five topics the daemon holds, and ADR 0015 made sure a cached topic is never sent when the files it is read from moved. An audit of what else could leave a topic stale found three causes that have nothing to do with files.

- **Writes.** A typed `write` names `settings` as what it affects, a barrier name that no topic has, so a write invalidated no topic. A write that settled patched the cached topic with what it read back. A write that failed, or whose read-back tool failed, or that read back nothing for a key it names, changed nothing: the topic kept the value from before the write. An interactive tool (a theme picker, a recorder) invalidated nothing either; only a `Job` read the topics again when it ended.
- **Stored values.** A topic is collected by its producers, and then every settings file's value was laid over the producer's reading, except for a short list of live switches. A value Atmos once wrote therefore covered what the machine says now: a hand edit of `gtk-4.0/settings.ini` still showed `tweaks.json`'s value, an `audio-level` file covered PipeWire's volume, a `bluetooth-power` file covered BlueZ. `settings.set` read its keys back through the producers alone, so its reply and the next pushed sample could disagree.
- **Files no table named.** Some producers read files the daemon did not watch (`/etc/passwd`, `/etc/group`, the avatar, the hooks folder, `user-dirs.dirs`, the resume hook).

The user's rule is that the daemon always has data that is up to date.

## Decision

**After a write, the topics that hold the keys it names are made current.** The keys a write names are what it reads back (`command::named_keys`: a typed write's `readback`, `settings.set`'s `readback`, the monitor list for brightness and scale). When the write ends, whether or not a window still waits for it:

- a key it read back once the machine had settled is patched into the cached topic, with no second collection;
- every other topic that holds a key it names is invalidated, so it is read again: the write failed, the key's tool failed, the machine had not settled, the key is missing from what came back, or nothing was cached to patch.

Settling and patching work as before for a typed write. `settings.set` names its own topic as what it affects, as before, so it is read again after a settled patch too. The bare `settings` affect stays: it is the barrier that orders a read after a write, not a topic. A table command (`wifi.ssid`, the restarts, `bluetooth.trust`) names no keys and is not covered.

**An interactive tool names the topics it can change,** in its registry entry, and they are read again when it ends: the theme and branding tools the look topic, the recorder the host topic, the rest none. A `Job` still reads every topic again.

**A stored value never covers a reading.** Each settings domain's stored value has one of three rules (`snapshot::merge::stored`):

- **Fallback**, the default. Where the producer reads the tool or file that is in effect, its reading is shown, and the stored value only where it has none: its tool failed (the key is in `_failed`), or it reads nothing for the key or the field.
- **Never**, for the live command switches (night light, the mutes, the bar, the screensaver, stay awake, the touchpad and touchscreen, do not disturb): the tool's answer is the value, and a file never stands in for it.
- **Wins**, for `atmosChannel`, whose producer reports a constant: the channel file is the setting.

A topic is collected, and every write and `snapshot.keys` reads back, through one function (`snapshot::topics::read_keys`), so a reply and the next pushed sample show the same value for a key unless the machine changed in between.

**The watched files include what these producers read:** `/etc/passwd`, `/etc/group`, `/var/lib/AccountsService/icons`, `~/.face.icon` and `~/.face` (accounts), `~/.config/omarchy/hooks` and `/etc/mkinitcpio.conf.d/omarchy_resume.conf` (host), `~/.config/user-dirs.dirs` (system).

This amends ADR 0011's "The backend owns the triggers" (a typed write patches the cached topic, and nothing else) and its document "with the settings files laid over".

## Consequences

- A failed or unread write costs a collection of the topics it names, which the window waits on, instead of leaving them as they were.
- A value Atmos wrote is shown only while the machine has no reading of its own. A writer that does not reach the machine (one that writes its record and spawns a tool without waiting) now shows the machine's answer when the topic is next collected, not the record.
- A producer that turns a tool's failure into a value (`bluetooth` and `audioTuningOn` read an exit status or unchecked output) never lets the stored value stand in: a failure shows as off.
- What is not a file is not covered: Hyprland's runtime options, NetworkManager, BlueZ, PipeWire, systemd units, the power profile, the monitors plugged in, and anything under `/sys`, whose size and modification time do not move when its value does. Keeping those current is a decision of its own.
- A table command, `ratmos task` and `ratmos apply` run from a terminal still invalidate no settings topic.

## Alternatives considered

- **Invalidate every topic a write names, settled or not.** One rule, and a second full collection after every settled write, which the read-back patch exists to avoid.
- **Keep the overlay and add domains to its exemption list one at a time.** Every new setting would start out covering the machine until someone noticed.
- **Make the stored value win wherever Atmos wrote one.** It shows what Atmos meant and not what the machine does, which is what the window is for (ADR 0005).
- **Give every interactive tool every topic, as a job has.** A screenshot would read all five topics again.

## Enforced by

- `backend/src/sampler/settings.rs` (`after_write`: patch what settled, invalidate the rest)
- `backend/src/sampler/ops.rs` (it runs when the write ends, also for a window that stopped waiting)
- `backend/src/command/mod.rs` (`named_keys`)
- `backend/src/command/jobs.rs` and `backend/src/command/entries.rs` (an interactive tool's topics)
- `backend/src/command/entries_tests.rs` (every interactive tool is classified; every name an entry affects is a topic)
- `backend/src/sampler/daemon_tests.rs` (a settled write collects nothing more; a failed write and an unread key are read again)
- `backend/src/snapshot/merge.rs` (the three rules, and a reading wins for every domain a producer reads)
- `backend/src/snapshot/topics.rs` (`read_keys`; the files the accounts, hooks, defaults and hibernation producers read are watched)
- `backend/tests/serve.rs` (a `settings.set` reply and the next pushed sample agree, and a stored tweak does not cover the GTK file)
