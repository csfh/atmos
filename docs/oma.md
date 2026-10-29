# Packing the window as an OMA Bundle

The Rust window (`frontend/`, `docs/adr/0013`) is an `.oma` Bundle with `atmosd` in it (`docs/adr/0014`). `oma` and the Bundle format are ODK's, in `../odk`.

## Develop it

```
odk oma dev frontend
```

`frontend/oma.toml` (ODK's ADR 0073; it was `dev.toml`) says how: it builds `atmosd` and the window with cargo, makes the Bundle with `atmosd` in it, installs it over the installed copy (the one place `oma` replaces an installed application, ODK's ADR 0051) and runs it. It shows the consent dialog only when the Bundle asks for more than the record holds, and stops the daemon of the copy it replaces before it installs, so the window never talks to a `atmosd` from an earlier build. It says so if something else, such as the user unit of an installed `atmosd`, is still serving `$XDG_RUNTIME_DIR/atmos/atmosd.sock`. It replaces `~/Applications/Atmos.oma`, and runs the build commands in `oma.toml` as you.

The Bundle is one signed file, signed with a dev key `odk oma dev` makes for this computer the first time (ODK's ADR 0085). The manifest's `publisher` is display text there. The first `dev` after an older `odk` installed Atmos as a directory asks for its permissions once, and moves Atmos's data to the dev build's own directory.

## Make it by hand

What `odk oma dev` does, one step at a time, for a build that is not the project's own:

```
cargo build --release --manifest-path backend/Cargo.toml
cargo build --release --manifest-path frontend/Cargo.toml
../odk/tools/third-party-licenses backend/Cargo.toml frontend/target/release/THIRD-PARTY-LICENSES-atmosd atmosd
../odk/tools/third-party-licenses frontend/Cargo.toml frontend/target/release/THIRD-PARTY-LICENSES-atmos-ui atmos-ui
odk oma bundle frontend/target/release/atmos-ui \
  --manifest frontend/manifest.toml \
  --include backend/target/release/atmosd=bin/atmosd \
  --include LICENSE=share/licenses/LICENSE \
  --include frontend/target/release/THIRD-PARTY-LICENSES-atmosd=share/licenses/THIRD-PARTY-LICENSES-atmosd \
  --include frontend/target/release/THIRD-PARTY-LICENSES-atmos-ui=share/licenses/THIRD-PARTY-LICENSES-atmos-ui \
  -o dist
odk oma inspect dist/Atmos.oma
```

`inspect` prints the permissions the Bundle asks for. There are two: a daemon that runs outside the sandbox, and the graphics card.

## Install and run it

```
odk oma install dist/Atmos.oma
odk oma run dev.csfh.atmos
```

`odk oma install` refuses an Identifier that is installed already; use `odk oma dev`, or delete `~/Applications/Atmos.oma`, to replace it. It shows the consent dialog and records the answer. `odk oma run` starts the Bundle's daemon (`bin/atmosd daemon --with-sampler`, which also starts its sampler) as a user unit when nothing is listening on `$XDG_RUNTIME_DIR/atmos/atmosd.sock`, and then the window, which finds `bin/atmosd` beside itself and runs `atmosd serve` from it.

## Trying it without your session

Never run the window on your desktop from a test or an agent. ODK's `tools/headless-shot` runs a program under a private headless sway and takes a screenshot of that display only:

- Give `oma` a home of its own, with every `XDG_*_HOME` in it, or `odk oma install` writes the consent record into your real `~/.local/share/oma/grants/`.
- Put a `systemd-run` of your own first on `PATH`, one that runs the program after `--` in the background and records its arguments, so no unit is started in your user manager.
- `ATMOS_NO_AUTOSTART=1`, and stop the daemon afterwards: it is detached.

`frontend/tests/bundle.rs` makes the Bundle with the real `oma` and inspects it, and ODK's `crates/oma/tests/daemon.rs` runs a Bundle with a daemon in the real sandbox.
