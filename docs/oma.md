# Packing the window as an OMA Bundle

The Rust window (`frontend/`, `docs/adr/0013`) is an `.oma` Bundle with `ratmos` in it (`docs/adr/0014`). `oma` and the Bundle format are ODK's, in `../odk`.

## Develop it

```
oma dev frontend
```

`frontend/dev.toml` says how: it builds `ratmos` and the window with cargo, makes the Bundle with `ratmos` in it, installs it over the installed copy (the one place `oma` replaces an installed application, ODK's ADR 0051) and runs it. It shows the consent dialog only when the Bundle asks for more than the record holds, and stops the daemon of the copy it replaces before it installs, so the window never talks to a `ratmos` from an earlier build. It says so if something else, such as the user unit of an installed `ratmos`, is still serving `$XDG_RUNTIME_DIR/atmos/ratmos.sock`. It replaces `~/Applications/Atmos.oma`, and runs the build commands in `dev.toml` as you.

## Make it by hand

What `oma dev` does, one step at a time, for a build that is not the project's own:

```
cargo build --release --manifest-path backend/Cargo.toml
cargo build --release --manifest-path frontend/Cargo.toml
oma bundle frontend/target/release/atmos-ui \
  --manifest frontend/manifest.toml \
  --include backend/target/release/ratmos=bin/ratmos \
  -o dist
oma inspect dist/Atmos.oma
```

`inspect` prints the permissions the Bundle asks for. There are two: a daemon that runs outside the sandbox, and the graphics card.

## Install and run it

```
oma install dist/Atmos.oma
oma run dev.csfh.atmos
```

`oma install` refuses an Identifier that is installed already; use `oma dev`, or delete `~/Applications/Atmos.oma`, to replace it. It shows the consent dialog and records the answer. `oma run` starts the Bundle's daemon (`bin/ratmos daemon --with-sampler`, which also starts its sampler) as a user unit when nothing is listening on `$XDG_RUNTIME_DIR/atmos/ratmos.sock`, and then the window, which finds `bin/ratmos` beside itself and runs `ratmos serve` from it.

## Trying it without your session

Never run the window on your desktop from a test or an agent. ODK's `tools/headless-shot` runs a program under a private headless sway and takes a screenshot of that display only:

- Give `oma` a home of its own, with every `XDG_*_HOME` in it, or `oma install` writes the consent record into your real `~/.local/share/oma/grants/`.
- Put a `systemd-run` of your own first on `PATH`, one that runs the program after `--` in the background and records its arguments, so no unit is started in your user manager.
- `ATMOS_NO_AUTOSTART=1`, and stop the daemon afterwards: it is detached.

`frontend/tests/bundle.rs` makes the Bundle with the real `oma` and inspects it, and ODK's `crates/oma/tests/daemon.rs` runs a Bundle with a daemon in the real sandbox.
