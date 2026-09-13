#!/usr/bin/env python3
"""Emit one live sample of CPU, memory, network, and user processes. No sudo."""

from __future__ import annotations

import json
import os
import sys

SYS = os.environ.get("ATMOS_SYS_ROOT") or "/"
PF_KTHREAD = 0x00200000


def root(*parts: str) -> str:
    return os.path.join(SYS, *[p.lstrip("/") for p in parts])


def read_text(path: str, limit: int = 65536) -> str:
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return fh.read(limit)
    except OSError:
        return ""


def uid_wanted() -> int:
    raw = os.environ.get("ATMOS_UID")
    if raw and raw.isdigit():
        return int(raw)
    try:
        return os.getuid()
    except OSError:
        return -1


def clk_tck() -> int:
    try:
        n = os.sysconf("SC_CLK_TCK")
        if isinstance(n, int) and n > 0:
            return n
    except (OSError, ValueError):
        pass
    return 100


def read_cpu() -> tuple[int | None, int | None]:
    for line in read_text(root("proc/stat")).splitlines():
        if not line.startswith("cpu "):
            continue
        nums = []
        for part in line.split()[1:]:
            try:
                nums.append(int(part))
            except ValueError:
                return None, None
        if len(nums) < 4:
            return None, None
        idle = nums[3] + (nums[4] if len(nums) > 4 else 0)
        total = sum(nums[:8] if len(nums) >= 8 else nums)
        if total <= 0:
            return None, None
        return idle, total
    return None, None


def read_mem() -> tuple[int | None, int | None, int | None]:
    fields: dict[str, int] = {}
    for line in read_text(root("proc/meminfo")).splitlines():
        parts = line.split()
        if len(parts) < 2:
            continue
        key = parts[0].rstrip(":")
        try:
            fields[key] = int(parts[1])
        except ValueError:
            continue
    total = fields.get("MemTotal")
    avail = fields.get("MemAvailable")
    if total is None or total <= 0:
        return None, None, None
    if avail is None or avail < 0:
        return None, total, None
    used = total - avail
    if used < 0:
        used = 0
    return used, total, avail


def read_net() -> tuple[int | None, int | None]:
    text = read_text(root("proc/net/dev"))
    if not text:
        return None, None
    rx = 0
    tx = 0
    found = False
    for line in text.splitlines():
        if ":" not in line:
            continue
        name, rest = line.split(":", 1)
        iface = name.strip()
        if not iface or iface == "lo":
            continue
        cols = rest.split()
        if len(cols) < 9:
            continue
        try:
            rx += int(cols[0])
            tx += int(cols[8])
        except ValueError:
            continue
        found = True
    if not found:
        return None, None
    return rx, tx


def parse_stat(text: str) -> tuple[str, int, int] | None:
    start = text.find("(")
    end = text.rfind(")")
    if start < 0 or end <= start:
        return None
    comm = text[start + 1 : end]
    rest = text[end + 1 :].split()
    # flags, utime, stime are fields 9, 14, 15 in /proc/pid/stat (1-based).
    if len(rest) < 13:
        return None
    try:
        flags = int(rest[6])
        utime = int(rest[11])
        stime = int(rest[12])
    except ValueError:
        return None
    return comm, flags, utime + stime


def parse_status(text: str) -> tuple[int | None, int | None]:
    uid = None
    rss = None
    for line in text.splitlines():
        if line.startswith("Uid:"):
            cols = line.split()
            if len(cols) >= 3:
                try:
                    uid = int(cols[2])
                except ValueError:
                    uid = None
        elif line.startswith("VmRSS:"):
            cols = line.split()
            if len(cols) >= 2:
                try:
                    rss = int(cols[1])
                except ValueError:
                    rss = None
    return uid, rss


def read_cmdline(path: str) -> str:
    raw = read_text(path, 4096)
    if not raw:
        return ""
    return " ".join(part for part in raw.split("\0") if part)


def iter_pids() -> list[str]:
    try:
        names = os.listdir(root("proc"))
    except OSError:
        return []
    out = []
    for name in names:
        if name.isdigit():
            out.append(name)
    return out


def read_processes(want_uid: int) -> list[dict]:
    rows = []
    for pid_s in iter_pids():
        stat = parse_stat(read_text(root("proc", pid_s, "stat")))
        if not stat:
            continue
        comm, flags, ticks = stat
        if flags & PF_KTHREAD:
            continue
        cmdline = read_cmdline(root("proc", pid_s, "cmdline"))
        if not cmdline:
            continue
        uid, rss = parse_status(read_text(root("proc", pid_s, "status")))
        if uid is None or uid != want_uid:
            continue
        try:
            pid = int(pid_s)
        except ValueError:
            continue
        rows.append(
            {
                "pid": pid,
                "comm": comm,
                "cmdline": cmdline,
                "uid": uid,
                "rssKb": rss,
                "ticks": ticks,
            }
        )
    return rows


def main() -> int:
    idle, total = read_cpu()
    used, mem_total, avail = read_mem()
    rx, tx = read_net()
    payload = {
        "cpuIdle": idle,
        "cpuTotal": total,
        "memUsed": used,
        "memTotal": mem_total,
        "memAvail": avail,
        "netRx": rx,
        "netTx": tx,
        "clkTck": clk_tck(),
        "processes": read_processes(uid_wanted()),
    }
    json.dump(payload, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
