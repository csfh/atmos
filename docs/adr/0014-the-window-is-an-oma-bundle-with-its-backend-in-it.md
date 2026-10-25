# 0014. The window is an OMA Bundle with its backend in it

Status: Accepted. Date: 2026-10-24.

## Context

ADR 0013 made the window a Rust program on ODK and left packing it as an `.oma` Bundle blocked: a sandboxed application can run nothing outside `/app` and reach nothing of the user's runtime directory, and the window needs `ratmos serve` and the daemon's `ratmos.sock`. ODK's ADR 0050 removes the block. A Bundle may declare one daemon in its manifest, which `oma run` starts outside the sandbox with the user's consent, and the application is given that daemon's socket and nothing else of the runtime directory. The user asked that the daemon not run twice and that the manifest declare it.

## Decision

`frontend/manifest.toml` makes the Bundle: `dev.csfh.atmos`, the window `atmos-ui` as its executable, `gpu = true`, and

```toml
[daemon]
executable = "bin/ratmos"
args = ["daemon", "--with-sampler"]
socket = "atmos/ratmos.sock"
```

`ratmos` is put in the Bundle with `oma bundle --include <ratmos>=bin/ratmos` (`docs/oma.md`). That one file is the daemon `oma run` starts and the `ratmos serve` the window runs from `/app/bin/ratmos`, which needs no permission, since the baseline lets `/app` execute.

- **One daemon, one socket.** `socket` is the file `ratmos.socket` listens on in an install of Atmos, so whichever of the two came first serves it, `oma run` starts nothing when it is served, and the window does not care which. `oma` starts the daemon as a transient, socket-activated user unit (ODK's ADR 0050), so that systemd owns the socket and the sandbox's bind of its file is meant to stay good when the daemon restarts. That hand-off (`LISTEN_FDS` to `ratmos daemon`, which `ratmos.service` already relies on) has not been run through a transient unit: the end-to-end run used a stand-in for the user manager that started the daemon directly, and it bound its own socket (ODK's `docs/checks.md`, Open).
- **The window finds its own `ratmos`.** `ServeProcess` takes `ATMOS_BACKEND`, then `bin/ratmos` beside its own executable, then the app root's and `PATH`. A copy found beside the executable is the Bundle's, and runs with `ATMOS_NO_AUTOSTART=1`, since a sandbox cannot start a daemon, and `ATMOS_SESSION=daemon`.
- **A window in a sandbox has no session to tell the daemon of.** It sees `HOME`, `PATH=/usr/local/bin:/usr/bin`, `WAYLAND_DISPLAY` and a few more, and no `HYPRLAND_INSTANCE_SIGNATURE` or D-Bus address. Before this record every window's `serve` sent its whole environment as the one the daemon runs commands, jobs and settings topics in (ADR 0004, 0011, 0012), and a bundled window would have made `hyprctl` and `omarchy` fail for every window. With `ATMOS_SESSION=daemon` `serve` sends no environment frame and no per-request environment, and the daemon runs what it is asked in the newest window's environment when there is one, and in **its own** when there is none (`sampler::client::or_own_env`). A child is never given an environment of nothing, because it runs with a cleared one. The daemon's own environment is the real session's: `oma run` gave it the allowlisted part of its own.
- **Consent is the protection.** The Bundle's consent line says it runs a program outside the sandbox with full access to the account. A sandboxed window that reaches the daemon's socket can ask for what the daemon's registry answers, and the daemon trusts its environment frame, but a Bundle that is allowed the daemon already holds that power, so the socket adds none.

## Consequences

- `oma run` for this Bundle needs `systemd-run --user` and a running user manager. Without them it stops before the window starts, and says why.
- A Bundle with a daemon cannot be run once, and only the installed copy of it runs: `oma run <path>` of any other copy is refused (ODK's ADR 0050).
- There is no upgrade path yet: `oma install` refuses an Identifier that is installed, and upgrades are not built. When they are, a running daemon is the old Bundle's until it leaves, which Atmos's does when the binary it runs has been replaced (ADR 0001).
- **The daemon starts its own sampler.** An install of Atmos starts `ratmos sampler` from `ratmos-sampler.service`, and `oma` starts only the one program the manifest declares, so without more the daemon of this Bundle said "background sampling is off" and answered inventories on demand only. The manifest passes `--with-sampler`: `ratmos daemon --with-sampler` starts `ratmos sampler --owned` as its own child, behind `nice -n 19` and `ionice -c 3` when there is one, and starts another, not more often than every ten seconds, if it ends. Sampling is still a process of its own at the lowest priorities behind the governor, not threads of the daemon, which answers windows that are waiting for a page (ADR 0002), so the user declares one daemon and there is one unit. `--owned` makes the sampler leave when its daemon does, by the connection ending or its parent changing; without it the sampler waits for the next daemon for ever, and a replaced daemon would find the old sampler still there and start a second beside it. A daemon without the flag starts nothing, so an install's units and a daemon in a Bundle never sample twice. The flag is an argument in the manifest, so the consent record and a changed daemon's re-prompt cover it.
- A daemon that outlives a login keeps the session environment it was started with, which for Hyprland includes an instance signature that ends with the session. The user manager ends with the login unless lingering is on, so the daemon does too. A user who lingers sees stale `hyprctl` until the daemon is restarted.
- The daemon cannot be given the search index of the QML app (`search.query` reads page files under the app root). The Rust window has its own search (ADR 0013).
- `ratmos` in the Bundle and the `ratmos` of an install of Atmos may be different versions behind one socket. The window shows the daemon's version, which is the one that answered.
- The Bundle is the size of both programs, about 650 MB for debug builds. A release build is far smaller.

## Alternatives considered

- **A socket permission and a daemon installed some other way.** The sandbox's boundary would then be the daemon's distrust of its clients' environments, which it does not have, and installing the daemon would be a second step the user did not ask for.
- **A second socket for sandboxed clients.** The user ruled out a second listener.
- **The daemon inside the sandbox.** It would need the home directory, `sudo` and Hyprland's command socket, which ODK's vocabulary never grants.
- **The window speaks `ratmos.sock` itself and drops `serve`.** It would remove a process. The window would carry the session framing, and the backend would have two kinds of client.

## Enforced by

- `frontend/tests/bundle.rs`
- `frontend/manifest.toml`
- `frontend/src/client.rs`
- `backend/src/sampler/client.rs`
- `backend/src/sampler/settings.rs`
- `backend/src/sampler/owned.rs`
- `backend/tests/serve.rs`
- `docs/oma.md`
