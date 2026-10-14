# Compatibility shim, not a helper script. The updater in installs older than
# 0.2.0 runs `python3 scripts/hypr-sentinel.py require enable FILE` from the
# freshly copied tree to switch the Hyprland drop-in on, and aborts the update
# when it is missing. Hand the call to ratmos. Delete this file with
# scripts/README.md once those installs are gone.
import os
import sys

backend = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "bin", "ratmos")
os.execv(backend, [backend, "sentinel"] + sys.argv[1:])
