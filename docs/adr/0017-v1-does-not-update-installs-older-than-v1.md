# 0017. v1 does not update installs older than v1

Status: Accepted. Date: 2026-10-26.

## Context

The updater in installs older than 0.2.0 copies the `scripts` directory out of the new tree and fails when it is missing, then runs `hypr-sentinel.py` from it once the app is staged to switch the Hyprland drop-in on. So the tree kept a `scripts` directory holding a README and that shim, which hands straight to `ratmos sentinel`, and `tests/update-compat.test.js` and `tests/consistency.test.js` pinned both. Every other rule here says Atmos has no helper scripts.

The user decided on 2026-10-25 that Atmos v1 is not compatible with older versions.

## Decision

v1 carries nothing for an install older than v1.

- The `scripts` directory (the README and `hypr-sentinel.py`), `tests/update-compat.test.js` and the pin in `tests/consistency.test.js` are removed, and `tests/consistency.test.js` now fails if a `scripts` directory comes back.
- An older install that updates to v1 with its own updater fails at the copy of `scripts`. This is accepted. It is installed again with `install.sh`.
- Nothing is added to a 0.x release to warn those installs. It can be, as a later decision, if a user is hurt by it.

This supersedes the rule in `AGENTS.md` that `scripts` stays for installs older than 0.2.0.

## Consequences

- The only shell in the tree is `bin/atmos` and `install.sh`, and nothing is Python.
- The `ratmos` rename (a separate decision, its own ADR) carries no shim for the old name.

## Alternatives considered

- **Keep `scripts` and the shim until the last install older than 0.2.0 is gone.** It costs a directory and two tests, and v1 is not compatible with older versions anyway, so those installs cannot update to it.
- **Release a last 0.x that tells an older install to reinstall.** It would need work in a tree this decision otherwise empties. It is left until a user is hurt.

## Enforced by

- `tests/consistency.test.js` fails if a `scripts` directory exists.
