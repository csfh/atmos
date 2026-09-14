#!/usr/bin/env python3
"""Emit one live sample of CPU, memory, network, temps, and user processes. No sudo."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys

SYS = os.environ.get("ATMOS_SYS_ROOT") or "/"
PF_KTHREAD = 0x00200000
CPU_HWMON_NAMES = {"coretemp", "k10temp", "k8temp", "zenpower"}
CPU_TZ_TYPES = {
    "x86_pkg_temp",
    "cpu-thermal",
    "cpu_thermal",
    "k10temp",
    "soc-thermal",
    "tjmax",
    "pkg-temp",
}
GPU_VENDOR_NAMES = {
    "0x1002": "AMD",
    "0x10de": "NVIDIA",
    "0x8086": "Intel",
    "0x1a03": "ASPEED",
}


def root(*parts: str) -> str:
    return os.path.join(SYS, *[p.lstrip("/") for p in parts])


def read_text(path: str, limit: int = 65536) -> str:
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return fh.read(limit)
    except OSError:
        return ""


def read_line(path: str) -> str:
    text = read_text(path, 4096)
    return text.splitlines()[0].strip() if text else ""


def milli_to_c(raw: str) -> int | None:
    """hwmon/thermal millidegrees. Missing or non-positive stays unknown."""
    if not raw.lstrip("-").isdigit():
        return None
    milli = int(raw)
    celsius = (milli + (500 if milli >= 0 else -500)) // 1000
    if celsius <= 0:
        return None
    return celsius


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


def read_cpu_temp() -> int | None:
    try:
        hwmons = sorted(os.listdir(root("sys/class/hwmon")))
    except OSError:
        hwmons = []
    for hw in hwmons:
        if not hw.startswith("hwmon"):
            continue
        base = root("sys/class/hwmon", hw)
        if read_line(os.path.join(base, "name")) not in CPU_HWMON_NAMES:
            continue
        temp = milli_to_c(read_line(os.path.join(base, "temp1_input")))
        if temp is not None:
            return temp
    try:
        zones = sorted(os.listdir(root("sys/class/thermal")))
    except OSError:
        zones = []
    for zone in zones:
        if not zone.startswith("thermal_zone"):
            continue
        base = root("sys/class/thermal", zone)
        ttype = read_line(os.path.join(base, "type")).strip().lower()
        if ttype not in CPU_TZ_TYPES and not ttype.startswith("cpu"):
            continue
        temp = milli_to_c(read_line(os.path.join(base, "temp")))
        if temp is not None:
            return temp
    return None


def driver_of(dev: str) -> str:
    try:
        return os.path.basename(os.readlink(os.path.join(dev, "driver")))
    except (OSError, ValueError):
        return ""


def pci_id_of(dev: str) -> str:
    vendor = read_line(os.path.join(dev, "vendor")).strip().lower().replace("0x", "")
    device = read_line(os.path.join(dev, "device")).strip().lower().replace("0x", "")
    if not vendor or not device:
        return ""
    return f"{vendor}:{device}"


def slot_of(dev: str) -> str:
    for line in read_text(os.path.join(dev, "uevent")).splitlines():
        if line.startswith("PCI_SLOT_NAME="):
            return line.split("=", 1)[1].strip()
    return ""


def slots_match(left: str, right: str) -> bool:
    a = left.lower()
    b = right.lower()
    if not a or not b:
        return False
    return a == b or a.endswith(b) or b.endswith(a)


def read_drm_gpu_temp(dev: str) -> int | None:
    base = os.path.join(dev, "hwmon")
    try:
        hwmons = sorted(os.listdir(base))
    except OSError:
        return None
    for hw in hwmons:
        if not hw.startswith("hwmon"):
            continue
        temp = milli_to_c(read_line(os.path.join(base, hw, "temp1_input")))
        if temp is not None:
            return temp
    return None


def nvidia_smi_rows() -> list[dict]:
    rows: list[dict] = []
    binary = shutil.which("nvidia-smi")
    if not binary:
        return rows
    try:
        out = (
            subprocess.run(
                [
                    binary,
                    "--query-gpu=pci.bus_id,temperature.gpu,name",
                    "--format=csv,noheader,nounits",
                ],
                capture_output=True,
                text=True,
                timeout=2,
            ).stdout
            or ""
        )
    except (OSError, subprocess.TimeoutExpired):
        return rows
    for line in out.splitlines():
        parts = [part.strip() for part in line.split(",", 2)]
        if len(parts) < 2 or not parts[1].isdigit():
            continue
        temp = int(parts[1])
        if temp <= 0:
            continue
        rows.append(
            {
                "slot": parts[0],
                "temp": temp,
                "name": parts[2] if len(parts) > 2 else "",
            }
        )
    return rows


def nvidia_temp_for(slot: str, rows: list[dict]) -> int | None:
    for row in rows:
        if slots_match(slot, row["slot"]):
            return row["temp"]
    if len(rows) == 1:
        return rows[0]["temp"]
    return None


def nvidia_name_for(slot: str, rows: list[dict]) -> str:
    for row in rows:
        if slots_match(slot, row["slot"]) and row["name"].strip():
            return row["name"].strip()
    if len(rows) == 1:
        return rows[0]["name"].strip()
    return ""


def read_gpus() -> list[dict]:
    try:
        names = sorted(os.listdir(root("sys/class/drm")))
    except OSError:
        names = []
    cards: list[dict] = []
    want_nvidia = False
    for name in names:
        if not name.startswith("card") or "-" in name:
            continue
        dev = root("sys/class/drm", name, "device")
        if not os.path.isdir(dev):
            continue
        driver = driver_of(dev)
        vendor_id = read_line(os.path.join(dev, "vendor")).strip().lower()
        brand = GPU_VENDOR_NAMES.get(vendor_id, "")
        slot = slot_of(dev)
        bus = slot.split(":")[1] if slot.count(":") >= 2 else ""
        integrated = driver == "i915" or (driver == "xe" and bus == "00")
        if driver == "nvidia":
            want_nvidia = True
        cards.append(
            {
                "card": name,
                "pciId": pci_id_of(dev),
                "name": brand or driver or name,
                "vendor": brand or vendor_id,
                "driver": driver,
                "integrated": integrated,
                "slot": slot,
                "temp": None if driver == "nvidia" else read_drm_gpu_temp(dev),
            }
        )
    nvidia_rows = nvidia_smi_rows() if want_nvidia else []
    out: list[dict] = []
    for gpu in cards:
        if gpu["driver"] == "nvidia":
            gpu["temp"] = nvidia_temp_for(gpu["slot"], nvidia_rows)
            smi_name = nvidia_name_for(gpu["slot"], nvidia_rows)
            if smi_name:
                gpu["name"] = smi_name
        del gpu["slot"]
        out.append(gpu)
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
        "cpuTemp": read_cpu_temp(),
        "gpus": read_gpus(),
        "processes": read_processes(uid_wanted()),
    }
    json.dump(payload, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
