#!/usr/bin/env python3
"""Emit one live sample of CPU, memory, network, and user processes. No sudo."""

from __future__ import annotations

import json
import os
import re
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
GPU_CLASSES = ("[0300]", "[0302]", "[0380]")
PCI_TAIL = re.compile(r"\[\s*([^\]]+?)\s*\]\s*\[[0-9a-fA-F]+\]\s*$")
PCI_VENDOR = re.compile(r"\[([0-9a-fA-F]{4})\]")


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


def mc_to_c(raw: str) -> int | None:
    if not raw.lstrip("-").isdigit():
        return None
    return (int(raw) + 500) // 1000


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
        if read_line(os.path.join(base, "name")) in CPU_HWMON_NAMES:
            temp = mc_to_c(read_line(os.path.join(base, "temp1_input")))
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
        if ttype in CPU_TZ_TYPES or ttype.startswith("cpu"):
            temp = mc_to_c(read_line(os.path.join(base, "temp")))
            if temp is not None:
                return temp
    return None


def driver_of(dev: str) -> str:
    try:
        return os.path.basename(os.readlink(os.path.join(dev, "driver")))
    except (OSError, ValueError):
        return ""


def read_drm_gpu_temp(dev: str) -> int | None:
    base = os.path.join(dev, "hwmon")
    try:
        hwmons = sorted(os.listdir(base))
    except OSError:
        return None
    for hw in hwmons:
        if not hw.startswith("hwmon"):
            continue
        temp = mc_to_c(read_line(os.path.join(base, hw, "temp1_input")))
        if temp is not None:
            return temp
    return None


def nvidia_smi_rows() -> list[dict]:
    rows: list[dict] = []
    try:
        import shutil

        binary = shutil.which("nvidia-smi")
        if not binary:
            return rows
        out = subprocess.run(
            [binary, "--query-gpu=uuid,name,temperature.gpu", "--format=csv,noheader,nounits"],
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout or ""
    except (OSError, subprocess.TimeoutExpired):
        return rows
    for line in out.splitlines():
        parts = [part.strip() for part in line.split(",", 2)]
        if len(parts) == 3 and parts[2].isdigit():
            rows.append({"uuid": parts[0], "name": parts[1], "temp": int(parts[2])})
    return rows


def pci_gpus() -> list[dict]:
    if SYS != "/":
        return []
    rows: list[dict] = []
    try:
        import shutil

        binary = shutil.which("lspci")
        if not binary:
            return rows
        out = subprocess.run(
            [binary, "-mmnn"],
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout or ""
    except (OSError, subprocess.TimeoutExpired):
        return rows
    for line in out.splitlines():
        try:
            import shlex

            parts = shlex.split(line)
        except ValueError:
            continue
        if len(parts) < 4 or "[" not in parts[1]:
            continue
        cls = parts[1]
        if not any(code in cls for code in GPU_CLASSES):
            continue
        vendor_text = parts[2].strip('"')
        vendor_m = PCI_VENDOR.search(vendor_text)
        vendor = "0x" + vendor_m.group(1).lower() if vendor_m else ""
        rows.append(
            {
                "slot": parts[0],
                "cls": cls.strip('"'),
                "vendor": vendor,
                "model": pci_model(parts[3].strip('"')),
            }
        )
    return rows


def pci_model(device: str) -> str:
    m = PCI_TAIL.search(device)
    if m:
        return m.group(1).strip()
    m = re.search(r"\[([^\]]+)\]\s*$", device)
    if m:
        return m.group(1).strip()
    text = device.strip()
    if " / " in text:
        text = text.split(" / ", 1)[1]
    return text.strip()


def read_gpus(nvidia_rows: list[dict]) -> list[dict]:
    try:
        names = sorted(os.listdir(root("sys/class/drm")))
    except OSError:
        names = []
    smi_by_uuid = {row["uuid"]: row for row in nvidia_rows if row.get("uuid")}
    pci_rows = pci_gpus()
    out: list[dict] = []
    claimed = set()
    for name in names:
        if not name.startswith("card") or "-" in name:
            continue
        dev = root("sys/class/drm", name, "device")
        if not os.path.isdir(dev):
            continue
        driver = driver_of(dev)
        vendor = read_line(os.path.join(dev, "vendor")).strip().lower()
        brand = GPU_VENDOR_NAMES.get(vendor, vendor or "")
        slot = ""
        for line in read_text(os.path.join(dev, "uevent")).splitlines():
            if line.startswith("PCI_SLOT_NAME="):
                slot = line.split("=", 1)[1].strip()
                break
        claimed.add(slot)
        bus = slot.split(":")[1] if slot.count(":") >= 2 else ""
        # i915 only ever drives integrated parts; xe on bus 00 is the on-die
        # Tile, Arc discrete cards sit on higher buses.
        integrated = driver == "i915" or (driver == "xe" and bus == "00")
        gpu = {
            "card": name,
            "name": brand or driver or name,
            "vendor": brand or vendor,
            "driver": driver,
            "integrated": integrated,
            "temp": read_drm_gpu_temp(dev),
        }
        uuid = ""
        if driver == "nvidia":
            gpu["temp"] = None
            info = read_text(root("proc/driver/nvidia/gpus", slot, "information"))
            for line in info.splitlines():
                if line.startswith("Model:"):
                    model = line.split(":", 1)[1].strip()
                    if model:
                        gpu["name"] = model
                elif line.startswith("GPU UUID:"):
                    uuid = line.split(":", 1)[1].strip()
            smi = smi_by_uuid.get(uuid) if uuid else None
            temp = smi["temp"] if smi else None
            if temp is None and len(nvidia_rows) == 1:
                temp = nvidia_rows[0]["temp"]
                smi = nvidia_rows[0]
            if smi and smi["name"].strip():
                gpu["name"] = smi["name"]
            gpu["temp"] = temp
        else:
            for pci in pci_rows:
                if slot and slot.endswith(pci["slot"]):
                    if pci["model"]:
                        gpu["name"] = pci["model"]
                    if pci["vendor"]:
                        gpu["vendor"] = GPU_VENDOR_NAMES.get(pci["vendor"], pci["vendor"])
                    break
        out.append(gpu)
    for pci in pci_rows:
        if any(slot and slot.endswith(pci["slot"]) for slot in claimed):
            continue
        vendor_brand = GPU_VENDOR_NAMES.get(pci["vendor"], pci["vendor"] or "")
        out.append(
            {
                "card": "pci:" + pci["slot"],
                "name": pci["model"] or vendor_brand or "GPU",
                "vendor": vendor_brand,
                "driver": "",
                "integrated": False,
                "temp": None,
            }
        )
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
    nvidia_rows = nvidia_smi_rows()
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
        "gpus": read_gpus(nvidia_rows),
        "processes": read_processes(uid_wanted()),
    }
    json.dump(payload, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
