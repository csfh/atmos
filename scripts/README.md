# Placeholder

Atmos has no helper scripts. This directory exists only because the updater
in installs older than 0.2.0 copies `scripts/` from the new tree before it
stages the app, and fails when it is missing. It holds no code except
`hypr-sentinel.py`, a shim the same updater calls once the app is staged; it hands
straight to `ratmos sentinel`. Drop both once those installs are gone.
