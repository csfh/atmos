# Packing the window as an OMA Bundle

The Rust window (`frontend/`, `docs/adr/0013`) is an `.oma` Bundle with `ratmos` in it (`docs/adr/0014`). `oma` and the Bundle format are ODK's, in `../odk`.

## Make it

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

`oma install` shows the consent dialog and records the answer. `oma run` starts the Bundle's daemon (`bin/ratmos daemon --with-sampler`, which also starts its sampler) as a user unit when nothing is listening on `$XDG_RUNTIME_DIR/atmos/ratmos.sock`, and then the window, which finds `bin/ratmos` beside itself and runs `ratmos serve` from it.

## Trying it without your session

Never run the window on your desktop from a test or an agent. ODK's `tools/headless-shot` runs a program under a private headless sway and takes a screenshot of that display only:

- Give `oma` a home of its own, with every `XDG_*_HOME` in it, or `oma install` writes the consent record into your real `~/.local/share/oma/grants/`.
- Put a `systemd-run` of your own first on `PATH`, one that runs the program after `--` in the background and records its arguments, so no unit is started in your user manager.
- `ATMOS_NO_AUTOSTART=1`, and stop the daemon afterwards: it is detached.

`frontend/tests/bundle.rs` makes the Bundle with the real `oma` and inspects it, and ODK's `crates/oma/tests/daemon.rs` runs a Bundle with a daemon in the real sandbox.
