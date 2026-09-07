#!/usr/bin/env python3
"""Emit a no-sudo health snapshot for the Atmos Diagnostics page."""

from __future__ import annotations

import json
import os
import re
import socket
import subprocess
import sys
from datetime import datetime, timezone


SYS = os.environ.get("ATMOS_SYS_ROOT") or "/"
SKIP_CMDS = os.environ.get("ATMOS_DIAG_SKIP_CMDS", "") in ("1", "true", "yes")
HOME = os.environ.get("ATMOS_HOME") or os.environ.get("HOME") or ""
OMARCHY_PATH = os.environ.get("OMARCHY_PATH") or "/usr/share/omarchy"


def root(*parts: str) -> str:
    return os.path.join(SYS, *[p.lstrip("/") for p in parts])


def read_text(path: str, limit: int = 8192) -> str:
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return fh.read(limit)
    except OSError:
        return ""


def read_strip(path: str, limit: int = 512) -> str:
    return read_text(path, limit).strip()


def run(argv: list[str], timeout: float = 2.0) -> str:
    if SKIP_CMDS:
        return ""
    try:
        return subprocess.check_output(
            argv,
            text=True,
            stderr=subprocess.DEVNULL,
            timeout=timeout,
        )
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return ""


def looks_secret(text: str) -> bool:
    lower = text.lower()
    return (
        "password" in lower
        or "passwd" in lower
        or "secret" in lower
        or "token" in lower
        or "bearer " in lower
    )


def clean_line(text: str, limit: int = 240) -> str:
    s = " ".join(str(text or "").split())
    if not s or looks_secret(s):
        return ""
    if len(s) > limit:
        s = s[:limit]
    return s


UNIT_RE = re.compile(r"^[A-Za-z0-9:_.@\\-]+$")


def hostname() -> str:
    raw = read_strip(root("etc/hostname"), 256)
    raw = raw.split()[0] if raw else ""
    if not raw:
        try:
            raw = socket.gethostname()
        except OSError:
            raw = ""
    raw = raw.strip()
    if "\n" in raw or "\r" in raw:
        return ""
    return raw[:253]


def kernel() -> dict:
    try:
        info = os.uname()
        return {
            "sysname": info.sysname[:32],
            "release": info.release[:64],
            "machine": info.machine[:32],
        }
    except OSError:
        return {"sysname": "", "release": "", "machine": ""}


def kb_to_bytes(raw: str) -> int:
    try:
        n = int(raw.split()[0])
    except (TypeError, ValueError, IndexError):
        return 0
    if n < 0:
        return 0
    return n * 1024


def memory() -> dict:
    total = used = available = swap_total = swap_used = 0
    for line in read_text(root("proc/meminfo"), 4096).splitlines():
        if ":" not in line:
            continue
        key, rest = line.split(":", 1)
        value = kb_to_bytes(rest.strip())
        if key == "MemTotal":
            total = value
        elif key == "MemAvailable":
            available = value
        elif key == "SwapTotal":
            swap_total = value
        elif key == "SwapFree":
            swap_used = max(0, swap_total - value) if swap_total else 0
    if total and available > total:
        available = total
    if total:
        used = max(0, total - available)
    return {
        "total": total,
        "used": used,
        "available": available,
        "swapTotal": swap_total,
        "swapUsed": swap_used,
    }


def disk() -> dict:
    path = root("")
    try:
        st = os.statvfs(path)
    except OSError:
        return {"path": "/", "total": 0, "used": 0, "available": 0, "percent": 0}
    total = st.f_frsize * st.f_blocks
    available = st.f_frsize * st.f_bavail
    used = total - st.f_frsize * st.f_bfree if total else 0
    percent = int(round((used / total) * 100)) if total else 0
    if percent < 0:
        percent = 0
    if percent > 100:
        percent = 100
    return {
        "path": "/",
        "total": total,
        "used": used,
        "available": available,
        "percent": percent,
    }


def gpu() -> dict:
    drm = root("sys/class/drm")
    driver = ""
    try:
        names = sorted(os.listdir(drm))
    except OSError:
        names = []
    for name in names:
        if not name.startswith("card") or "-" in name:
            continue
        link = os.path.join(drm, name, "device", "driver")
        try:
            target = os.readlink(link)
        except OSError:
            continue
        driver = os.path.basename(target)
        if driver:
            break
    identity = ""
    uevent = read_text(root("sys/class/drm/card0/device/uevent"), 2048)
    for line in uevent.splitlines():
        if line.startswith("DRIVER="):
            if not driver:
                driver = line.split("=", 1)[1].strip()
        if line.startswith("PCI_ID="):
            identity = line.split("=", 1)[1].strip()
    return {"driver": driver[:64], "identity": identity[:160]}


def default_route() -> bool:
    for line in read_text(root("proc/net/route"), 8192).splitlines()[1:]:
        cols = line.split()
        if len(cols) >= 2 and cols[1] == "00000000":
            return True
    return False


def network_kind() -> str:
    net = root("sys/class/net")
    try:
        names = os.listdir(net)
    except OSError:
        return ""
    wired = False
    wifi = False
    for name in names:
        if name == "lo":
            continue
        oper = read_strip(os.path.join(net, name, "operstate"), 32)
        if oper != "up":
            continue
        if os.path.isdir(os.path.join(net, name, "wireless")):
            wifi = True
        else:
            wired = True
    if wifi:
        return "wifi"
    if wired:
        return "ethernet"
    return "offline"


def pacman() -> dict:
    sync = root("var/lib/pacman/sync")
    count = 0
    last = 0.0
    try:
        for name in os.listdir(sync):
            if not name.endswith(".db"):
                continue
            path = os.path.join(sync, name)
            try:
                mtime = os.stat(path).st_mtime
            except OSError:
                continue
            count += 1
            if mtime > last:
                last = mtime
    except OSError:
        return {"syncOk": False, "lastSync": "", "dbCount": 0}
    stamp = ""
    if last:
        stamp = datetime.fromtimestamp(last, tz=timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    return {"syncOk": count > 0, "lastSync": stamp, "dbCount": count}


def omarchy_info() -> dict:
    version = clean_line(run(["omarchy", "version"], 2.0), 64)
    channel = clean_line(run(["omarchy", "channel", "current"], 2.0), 32)
    path = OMARCHY_PATH if os.path.isdir(OMARCHY_PATH) else ""
    return {"version": version, "channel": channel, "path": path[:240]}


def file_has(path: str, needle: str) -> bool:
    text = read_text(path, 65536)
    return needle in text


def atmos_info() -> dict:
    data = os.path.join(
        os.environ.get("XDG_DATA_HOME") or os.path.join(HOME, ".local/share"),
        "atmos",
    )
    revision = read_strip(os.path.join(data, "REVISION"), 80)
    hypr = os.path.join(HOME, ".config/hypr") if HOME else ""
    hyprland = os.path.join(hypr, "hyprland.lua") if hypr else ""
    sentinels = {
        "look": file_has(os.path.join(hypr, "looknfeel.lua"), "-- atmos:look begin") if hypr else False,
        "input": file_has(os.path.join(hypr, "input.lua"), "-- atmos:input begin") if hypr else False,
        "autostart": file_has(os.path.join(hypr, "autostart.lua"), "-- atmos:autostart begin")
        if hypr
        else False,
        "bindings": file_has(os.path.join(hypr, "bindings.lua"), "-- atmos:bindings begin")
        if hypr
        else False,
        "windows": file_has(os.path.join(hypr, "atmos.lua"), "-- atmos:windows begin") if hypr else False,
    }
    return {
        "revision": revision[:80],
        "installed": bool(revision) or os.path.isdir(data),
        "hyprAtmos": file_has(hyprland, 'require("hypr.atmos")') if hyprland else False,
        "hyprAtmosLayout": file_has(hyprland, 'require("hypr.atmos_layout")') if hyprland else False,
        "sentinels": sentinels,
    }


def hyprland_info() -> dict:
    version_raw = run(["hyprctl", "version"], 2.0)
    version = ""
    if version_raw:
        first = version_raw.splitlines()[0]
        m = re.search(r"Hyprland\s+(\S+)", first)
        version = m.group(1) if m else clean_line(first, 80)
    errors = []
    err_raw = run(["hyprctl", "configerrors"], 2.0)
    for line in err_raw.splitlines():
        text = clean_line(line, 240)
        if text:
            errors.append(text)
        if len(errors) >= 20:
            break
    return {"version": version[:80], "configErrors": errors}


def parse_failed(raw: str, scope: str) -> list[dict]:
    out = []
    for line in raw.splitlines():
        cols = line.split(None, 4)
        if not cols:
            continue
        unit = cols[0]
        if not UNIT_RE.match(unit):
            continue
        sub = cols[3] if len(cols) > 3 else ""
        desc = cols[4] if len(cols) > 4 else ""
        out.append(
            {
                "unit": unit,
                "scope": scope,
                "sub": clean_line(sub, 32),
                "description": clean_line(desc, 160),
            }
        )
        if len(out) >= 12:
            break
    return out


def failed_units() -> list[dict]:
    system = parse_failed(
        run(["systemctl", "--failed", "--plain", "--no-legend", "--no-pager"], 2.0),
        "system",
    )
    user = parse_failed(
        run(
            ["systemctl", "--user", "--failed", "--plain", "--no-legend", "--no-pager"],
            2.0,
        ),
        "user",
    )
    return system + user


def user_state(unit: str) -> str:
    raw = run(["systemctl", "--user", "is-active", unit], 2.0).strip()
    if raw in ("active", "inactive", "failed", "activating", "deactivating"):
        return raw
    return "unknown" if raw else ""


def recent_errors() -> list[str]:
    raw = run(
        ["journalctl", "-b", "-p", "3", "-n", "15", "--no-pager", "-o", "cat"],
        3.0,
    )
    out = []
    for line in raw.splitlines():
        text = clean_line(line, 240)
        if text:
            out.append(text)
        if len(out) >= 15:
            break
    return out


def generated_at() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")


def main() -> int:
    online = default_route()
    kind = network_kind()
    if not online:
        kind = "offline"
    payload = {
        "generatedAt": generated_at(),
        "hostname": hostname(),
        "kernel": kernel(),
        "omarchy": omarchy_info(),
        "atmos": atmos_info(),
        "hyprland": hyprland_info(),
        "failedUnits": failed_units(),
        "disk": disk(),
        "memory": memory(),
        "gpu": gpu(),
        "portals": {
            "xdg-desktop-portal": user_state("xdg-desktop-portal.service"),
            "xdg-desktop-portal-hyprland": user_state("xdg-desktop-portal-hyprland.service"),
            "xdg-desktop-portal-gtk": user_state("xdg-desktop-portal-gtk.service"),
        },
        "pipewire": {
            "pipewire": user_state("pipewire.service"),
            "pipewire-pulse": user_state("pipewire-pulse.service"),
            "wireplumber": user_state("wireplumber.service"),
        },
        "network": {"online": online, "kind": kind},
        "pacman": pacman(),
        "recentErrors": recent_errors(),
    }
    json.dump(payload, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception:
        json.dump({}, sys.stdout, separators=(",", ":"))
        sys.stdout.write("\n")
        raise SystemExit(0)
