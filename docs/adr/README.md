# Architecture decision records

An ADR records one decision that shaped Atmos: what was decided, why, and what it costs. They are short and they stay. A decision that changes gets a new ADR that supersedes the old one, and the old one keeps its text with its status changed.

`docs/architecture.md` is the design as a whole and the order it was built in. These records are the reasons behind its rules.

| # | Decision | Status |
| --- | --- | --- |
| [0001](0001-one-daemon-is-the-authority.md) | One daemon is the authority | Accepted |
| [0002](0002-sampling-bus-behind-a-governor.md) | A sampling bus behind a CPU governor, with background work in its own idle process | Accepted |
| [0003](0003-one-session-per-window.md) | One multiplexed session per window | Accepted |
| [0004](0004-command-bus-typed-writes-on-lanes.md) | A command bus: typed writes on lanes, with barriers and leases | Accepted |
| [0005](0005-the-window-shows-what-the-backend-reports.md) | The window shows what the backend reports and never a guess | Accepted |
| [0006](0006-a-failure-is-never-an-empty-value.md) | A failure is never an empty value | Accepted |
| [0007](0007-system-integration-uses-std-only.md) | System integration uses std only | Accepted |
| [0008](0008-every-read-and-write-goes-through-the-backend.md) | Every read and every write goes through the backend | Accepted |
| [0009](0009-every-job-runs-in-the-daemon.md) | Every job and interactive tool runs in the daemon | Accepted |
| [0010](0010-nothing-works-without-the-daemon.md) | Nothing works without the daemon | Accepted |
| [0011](0011-the-settings-snapshot-is-held-as-topics.md) | The settings snapshot is held as topics | Accepted, amended by 0015 and 0016 |
| [0012](0012-serve-is-a-bridge.md) | `ratmos serve` is a bridge | Accepted |
| [0013](0013-the-window-is-rust-on-odk.md) | The window is Rust, built on ODK's components alone | Accepted |
| [0014](0014-the-window-is-an-oma-bundle-with-its-backend-in-it.md) | The window is an OMA Bundle with its backend in it | Accepted |
| [0015](0015-a-cached-topic-is-sent-only-when-its-files-have-not-moved.md) | A cached settings topic is sent only when its files have not moved | Accepted |
| [0016](0016-what-a-write-names-is-made-current-and-a-reading-wins.md) | What a write names is made current, and a reading wins over a stored value | Accepted |
| [0017](0017-v1-does-not-update-installs-older-than-v1.md) | v1 does not update installs older than v1 | Accepted |
| [0018](0018-the-backend-is-atmosd.md) | The backend is `atmosd` | Accepted |
| [0019](0019-a-passwordless-sudo-grant-ends-at-boot-too.md) | A passwordless sudo grant ends at boot too | Accepted |
| [0020](0020-the-window-searches-the-rows-its-pages-declare.md) | The window searches the rows its pages declare | Accepted |

## Format

Each record has a status line, the date, and these sections in order: Context, Decision, Consequences, Alternatives considered, Enforced by. `tests/adr.test.js` checks the headings, that every record is in the table above, and that every file named under "Enforced by" exists.

A record is `Accepted`, `Superseded by NNNN`, or `Deprecated`. Number the next one after the last; never reuse a number.
