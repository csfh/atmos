#!/usr/bin/env python3
"""List user services, failed system units, and the Atmos allowlist."""

from __future__ import annotations

import json
import subprocess
import sys

# Keep in sync with services/Systemd.js allowlist().
ALLOWLIST = [
    ("pipewire.service", "user"),
    ("pipewire-pulse.service", "user"),
    ("wireplumber.service", "user"),
    ("xdg-desktop-portal.service", "user"),
    ("xdg-desktop-portal-hyprland.service", "user"),
    ("bluetooth.service", "system"),
    ("NetworkManager.service", "system"),
    ("cups.service", "system"),
    ("docker.service", "system"),
    ("tailscaled.service", "system"),
    ("fstrim.timer", "system"),
]
ALLOW_SET = set(ALLOWLIST)
ROW_CAP = 80


def run(args: list[str]) -> str:
    try:
        return subprocess.check_output(args, text=True, stderr=subprocess.DEVNULL)
    except (OSError, subprocess.CalledProcessError):
        return ""


def parse_units(text: str, scope: str) -> list[dict]:
    rows = []
    for raw in text.splitlines():
        cols = raw.split()
        if not cols:
            continue
        rows.append(
            {
                "unit": cols[0],
                "scope": scope,
                "load": cols[1] if len(cols) > 1 else "",
                "active": cols[2] if len(cols) > 2 else "",
                "sub": cols[3] if len(cols) > 3 else "",
                "description": " ".join(cols[4:]),
                "unitFileState": "",
            }
        )
    return rows


def parse_files(text: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for raw in text.splitlines():
        cols = raw.split()
        if len(cols) >= 2:
            out[cols[0]] = cols[1]
    return out


def rank(row: dict) -> tuple:
    failed = row.get("active") == "failed" or row.get("sub") == "failed"
    allowed = (row.get("unit"), row.get("scope")) in ALLOW_SET
    return (0 if failed else 1, 0 if allowed else 1, str(row.get("unit") or ""))


def main() -> int:
    user_units = parse_units(
        run(
            [
                "systemctl",
                "--user",
                "--no-legend",
                "--no-pager",
                "list-units",
                "--type=service",
                "--all",
            ]
        ),
        "user",
    )
    sys_failed = parse_units(
        run(
            [
                "systemctl",
                "--no-legend",
                "--no-pager",
                "list-units",
                "--type=service",
                "--state=failed",
            ]
        ),
        "system",
    )
    allow_sys = [unit for unit, scope in ALLOWLIST if scope == "system"]
    extra = []
    if allow_sys:
        extra = parse_units(
            run(["systemctl", "--no-legend", "--no-pager", "list-units", "--all", *allow_sys]),
            "system",
        )

    user_files = parse_files(run(["systemctl", "--user", "--no-legend", "--no-pager", "list-unit-files"]))
    sys_files = parse_files(run(["systemctl", "--no-legend", "--no-pager", "list-unit-files"]))

    seen: set[str] = set()
    rows: list[dict] = []

    def add(row: dict) -> None:
        key = f"{row['scope']}\t{row['unit']}"
        if key in seen:
            return
        files = user_files if row["scope"] == "user" else sys_files
        row["unitFileState"] = files.get(row["unit"], "")
        seen.add(key)
        rows.append(row)

    for row in sys_failed:
        add(row)
    for row in extra:
        add(row)
    for row in user_units:
        add(row)

    rows.sort(key=rank)
    json.dump(rows[:ROW_CAP], sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
