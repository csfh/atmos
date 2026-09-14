#!/usr/bin/env python3
"""Emit one live sample of CPU, memory, network, temps, and user processes. No sudo."""

from __future__ import annotations

import json
import os
import re
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
DISK_SKIP = ("ram", "loop", "sr", "zram")
DISK_PARTITION = re.compile(
    r"^(sd[a-z]+|vd[a-z]+|hd[a-z]+|xvd[a-z]+)\d+$|nvme\d+n\d+p\d+$|mmcblk\d+p\d+$"
)


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


def parse_cpu_counts(line: str) -> tuple[int, int] | None:
    nums = []
    for part in line.split()[1:]:
        try:
            nums.append(int(part))
        except ValueError:
            return None
    if len(nums) < 4:
        return None
    idle = nums[3] + (nums[4] if len(nums) > 4 else 0)
    total = sum(nums[:8] if len(nums) >= 8 else nums)
    if total <= 0:
        return None
    return idle, total


def read_cpu() -> tuple[int | None, int | None]:
    for line in read_text(root("proc/stat")).splitlines():
        if not line.startswith("cpu "):
            continue
        parsed = parse_cpu_counts(line)
        if not parsed:
            return None, None
        return parsed
    return None, None


def read_cpu_freq(idx: int) -> tuple[int | None, str]:
    base = root("sys/devices/system/cpu", f"cpu{idx}", "cpufreq")
    raw = read_line(os.path.join(base, "scaling_cur_freq"))
    mhz = None
    if raw.isdigit():
        n = int(raw) // 1000
        if n > 0:
            mhz = n
    return mhz, read_line(os.path.join(base, "scaling_governor"))


def read_cpus() -> list[dict]:
    out: list[dict] = []
    for line in read_text(root("proc/stat")).splitlines():
        if not line.startswith("cpu") or line.startswith("cpu "):
            continue
        name = line.split()[0]
        if not name[3:].isdigit():
            continue
        parsed = parse_cpu_counts(line)
        if not parsed:
            continue
        idle, total = parsed
        idx = int(name[3:])
        mhz, governor = read_cpu_freq(idx)
        out.append(
            {
                "id": idx,
                "idle": idle,
                "total": total,
                "freqMhz": mhz,
                "governor": governor,
            }
        )
    return out


def read_load() -> tuple[float | None, float | None, float | None]:
    parts = read_line(root("proc/loadavg")).split()
    if len(parts) < 3:
        return None, None, None
    vals: list[float | None] = []
    for part in parts[:3]:
        try:
            n = float(part)
        except ValueError:
            n = None
        if n is None or n < 0:
            vals.append(None)
        else:
            vals.append(n)
    return vals[0], vals[1], vals[2]


def mem_field(fields: dict[str, int], key: str) -> int | None:
    n = fields.get(key)
    if n is None or n < 0:
        return None
    return n


def read_mem() -> dict:
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
    total = mem_field(fields, "MemTotal")
    avail = mem_field(fields, "MemAvailable")
    used = None
    if total is not None and total > 0 and avail is not None:
        used = total - avail
        if used < 0:
            used = 0
    swap_total = mem_field(fields, "SwapTotal")
    swap_free = mem_field(fields, "SwapFree")
    swap_used = None
    if swap_total is not None and swap_free is not None:
        swap_used = swap_total - swap_free
        if swap_used < 0:
            swap_used = 0
    file_pages = None
    active_file = mem_field(fields, "Active(file)")
    inactive_file = mem_field(fields, "Inactive(file)")
    if active_file is not None or inactive_file is not None:
        file_pages = (active_file or 0) + (inactive_file or 0)
    return {
        "used": used,
        "total": total,
        "avail": avail,
        "free": mem_field(fields, "MemFree"),
        "buffers": mem_field(fields, "Buffers"),
        "cached": mem_field(fields, "Cached"),
        "shared": mem_field(fields, "Shmem"),
        "sreclaimable": mem_field(fields, "SReclaimable"),
        "anon": mem_field(fields, "AnonPages"),
        "dirty": mem_field(fields, "Dirty"),
        "slab": mem_field(fields, "Slab"),
        "pagetables": mem_field(fields, "PageTables"),
        "kernelstack": mem_field(fields, "KernelStack"),
        "file": file_pages,
        "swapUsed": swap_used,
        "swapTotal": swap_total,
    }


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


def parse_stat(text: str) -> dict | None:
    start = text.find("(")
    end = text.rfind(")")
    if start < 0 or end <= start:
        return None
    comm = text[start + 1 : end]
    rest = text[end + 1 :].split()
    # state, ppid, flags, utime, stime, nice, threads in /proc/pid/stat.
    if len(rest) < 13:
        return None
    try:
        flags = int(rest[6])
        utime = int(rest[11])
        stime = int(rest[12])
        ppid = int(rest[1])
        nice = int(rest[16]) if len(rest) > 16 else 0
        threads = int(rest[17]) if len(rest) > 17 else 1
    except ValueError:
        return None
    state = rest[0] if rest else ""
    if threads < 0:
        threads = 0
    return {
        "comm": comm,
        "flags": flags,
        "ticks": utime + stime,
        "state": state[:1],
        "ppid": ppid,
        "nice": nice,
        "threads": threads,
    }


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


def smi_int(raw: str) -> int | None:
    s = raw.strip()
    if not s.isdigit():
        return None
    n = int(s)
    if n < 0:
        return None
    return n


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
                    "--query-gpu=pci.bus_id,temperature.gpu,utilization.gpu,memory.used,memory.total,name",
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
        parts = [part.strip() for part in line.split(",")]
        if len(parts) < 5:
            continue
        temp = smi_int(parts[1])
        if temp is not None and temp <= 0:
            temp = None
        busy = smi_int(parts[2])
        mem_used = smi_int(parts[3])
        mem_total = smi_int(parts[4])
        name = ",".join(parts[5:]).strip() if len(parts) > 5 else ""
        rows.append(
            {
                "slot": parts[0],
                "temp": temp,
                "busy": busy,
                "vramUsed": mem_used * 1024 if mem_used is not None else None,
                "vramTotal": mem_total * 1024 if mem_total is not None else None,
                "name": name,
            }
        )
    return rows


def nvidia_field_for(slot: str, rows: list[dict], key: str):
    for row in rows:
        if slots_match(slot, row["slot"]):
            return row.get(key)
    if len(rows) == 1:
        return rows[0].get(key)
    return None


def nvidia_temp_for(slot: str, rows: list[dict]) -> int | None:
    temp = nvidia_field_for(slot, rows, "temp")
    return temp if isinstance(temp, int) else None


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
        busy = None
        vram_used = None
        vram_total = None
        if driver != "nvidia":
            busy_raw = read_line(os.path.join(dev, "gpu_busy_percent"))
            if busy_raw.isdigit():
                busy_n = int(busy_raw)
                if 0 <= busy_n <= 100:
                    busy = busy_n
            used_raw = read_line(os.path.join(dev, "mem_info_vram_used"))
            total_raw = read_line(os.path.join(dev, "mem_info_vram_total"))
            if used_raw.isdigit():
                vram_used = int(used_raw) // 1024
            if total_raw.isdigit():
                vram_total = int(total_raw) // 1024
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
                "busy": busy,
                "vramUsed": vram_used,
                "vramTotal": vram_total,
            }
        )
    nvidia_rows = nvidia_smi_rows() if want_nvidia else []
    out: list[dict] = []
    for gpu in cards:
        if gpu["driver"] == "nvidia":
            gpu["temp"] = nvidia_temp_for(gpu["slot"], nvidia_rows)
            busy = nvidia_field_for(gpu["slot"], nvidia_rows, "busy")
            gpu["busy"] = busy if isinstance(busy, int) else None
            used = nvidia_field_for(gpu["slot"], nvidia_rows, "vramUsed")
            total = nvidia_field_for(gpu["slot"], nvidia_rows, "vramTotal")
            gpu["vramUsed"] = used if isinstance(used, int) else None
            gpu["vramTotal"] = total if isinstance(total, int) else None
            smi_name = nvidia_name_for(gpu["slot"], nvidia_rows)
            if smi_name:
                gpu["name"] = smi_name
        del gpu["slot"]
        out.append(gpu)
    return out


def read_io(pid_s: str) -> tuple[int | None, int | None]:
    read_b = None
    write_b = None
    for line in read_text(root("proc", pid_s, "io"), 4096).splitlines():
        if line.startswith("read_bytes:"):
            parts = line.split()
            if len(parts) >= 2 and parts[1].isdigit():
                read_b = int(parts[1])
        elif line.startswith("write_bytes:"):
            parts = line.split()
            if len(parts) >= 2 and parts[1].isdigit():
                write_b = int(parts[1])
    return read_b, write_b


def read_processes(want_uid: int) -> list[dict]:
    rows = []
    for pid_s in iter_pids():
        stat = parse_stat(read_text(root("proc", pid_s, "stat")))
        if not stat:
            continue
        kthread = bool(stat["flags"] & PF_KTHREAD)
        cmdline = read_cmdline(root("proc", pid_s, "cmdline"))
        if not cmdline and not kthread:
            cmdline = ""
        uid, rss = parse_status(read_text(root("proc", pid_s, "status")))
        if uid is None:
            continue
        try:
            pid = int(pid_s)
        except ValueError:
            continue
        read_b, write_b = read_io(pid_s)
        fds = read_fd_count(pid_s)
        rows.append(
            {
                "pid": pid,
                "ppid": stat["ppid"],
                "comm": stat["comm"],
                "cmdline": cmdline,
                "uid": uid,
                "rssKb": rss,
                "ticks": stat["ticks"],
                "state": stat["state"],
                "threads": stat["threads"],
                "nice": stat["nice"],
                "kthread": kthread,
                "readBytes": read_b,
                "writeBytes": write_b,
                "fds": fds,
                "mine": uid == want_uid,
            }
        )
    return rows


def read_fd_count(pid_s: str) -> int | None:
    path = root("proc", pid_s, "fd")
    try:
        return len(os.listdir(path))
    except OSError:
        return None


def read_ifaces() -> list[dict]:
    text = read_text(root("proc/net/dev"))
    if not text:
        return []
    out: list[dict] = []
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
            rx = int(cols[0])
            rx_packets = int(cols[1])
            rx_err = int(cols[2])
            rx_drop = int(cols[3])
            tx = int(cols[8])
            tx_packets = int(cols[9]) if len(cols) > 9 else 0
            tx_err = int(cols[10]) if len(cols) > 10 else 0
            tx_drop = int(cols[11]) if len(cols) > 11 else 0
        except ValueError:
            continue
        if rx < 0 or tx < 0:
            continue
        out.append(
            {
                "name": iface,
                "rx": rx,
                "tx": tx,
                "rxPackets": rx_packets,
                "txPackets": tx_packets,
                "rxErr": rx_err,
                "txErr": tx_err,
                "rxDrop": rx_drop,
                "txDrop": tx_drop,
            }
        )
    return out


def read_disks() -> list[dict]:
    text = read_text(root("proc/diskstats"))
    if not text:
        return []
    out: list[dict] = []
    for line in text.splitlines():
        cols = line.split()
        if len(cols) < 14:
            continue
        name = cols[2]
        if not name or name.startswith(DISK_SKIP) or DISK_PARTITION.search(name):
            continue
        try:
            read_sectors = int(cols[5])
            write_sectors = int(cols[9])
        except ValueError:
            continue
        if read_sectors < 0 or write_sectors < 0:
            continue
        out.append(
            {
                "name": name,
                "readSectors": read_sectors,
                "writeSectors": write_sectors,
            }
        )
    return out


def read_psi_file(kind: str) -> float | None:
    text = read_text(root("proc/pressure", kind), 4096)
    for line in text.splitlines():
        if not line.startswith("some "):
            continue
        for part in line.split():
            if part.startswith("avg10="):
                try:
                    n = float(part.split("=", 1)[1])
                except ValueError:
                    return None
                if n < 0:
                    return None
                return n
    return None


def read_psi() -> dict:
    return {
        "cpu": read_psi_file("cpu"),
        "memory": read_psi_file("memory"),
        "io": read_psi_file("io"),
    }


TCP_STATES = {
    "01": "established",
    "0A": "listen",
    "06": "timeWait",
    "08": "closeWait",
}

TCP_SOCKET_STATES = {
    "01": "ESTABLISHED",
    "02": "SYN_SENT",
    "03": "SYN_RECV",
    "04": "FIN_WAIT1",
    "05": "FIN_WAIT2",
    "06": "TIME_WAIT",
    "07": "CLOSE",
    "08": "CLOSE_WAIT",
    "09": "LAST_ACK",
    "0A": "LISTEN",
    "0B": "CLOSING",
}


def read_tcp() -> dict:
    counts = {
        "established": 0,
        "listen": 0,
        "timeWait": 0,
        "closeWait": 0,
        "total": 0,
    }
    for fname in ("proc/net/tcp", "proc/net/tcp6"):
        lines = read_text(root(fname)).splitlines()
        for line in lines[1:]:
            cols = line.split()
            if len(cols) < 4:
                continue
            st = cols[3].upper()
            counts["total"] += 1
            key = TCP_STATES.get(st)
            if key:
                counts[key] += 1
    return counts


def hwmon_label(base: str, prefix: str, idx: str) -> str:
    label = read_line(os.path.join(base, f"{prefix}{idx}_label"))
    if label:
        return label
    return f"{prefix}{idx}"


def read_tcp_sockets() -> list[dict]:
    out: list[dict] = []
    for fname in ("proc/net/tcp", "proc/net/tcp6"):
        lines = read_text(root(fname)).splitlines()
        for line in lines[1:]:
            cols = line.split()
            if len(cols) < 10:
                continue
            st = TCP_SOCKET_STATES.get(cols[3].upper())
            if not st:
                continue
            inode = cols[9]
            out.append({"inode": inode, "state": st})
            if len(out) >= 240:
                return out
    return out


def parse_irq_table(text: str) -> dict | None:
    lines = [ln for ln in text.splitlines() if ln.strip()]
    if len(lines) < 2:
        return None
    header = lines[0].split()
    cpus = 0
    for part in header:
        if part.startswith("CPU"):
            cpus += 1
    if cpus <= 0:
        return None
    rows: list[dict] = []
    for line in lines[1:]:
        cols = line.split()
        if len(cols) < cpus + 1:
            continue
        name = cols[0].rstrip(":")
        values: list[int] = []
        ok = True
        for part in cols[1 : 1 + cpus]:
            if not part.isdigit():
                ok = False
                break
            values.append(int(part))
        if not ok:
            continue
        rows.append({"id": name, "values": values})
        if len(rows) >= 48:
            break
    if not rows:
        return None
    return {"cpus": cpus, "rows": rows}


def read_interrupts() -> dict | None:
    return parse_irq_table(read_text(root("proc/interrupts")))


def read_softirqs() -> dict | None:
    return parse_irq_table(read_text(root("proc/softirqs")))


def read_buddy() -> list[int] | None:
    text = read_text(root("proc/buddyinfo"))
    if not text:
        return None
    totals: list[int] = []
    for line in text.splitlines():
        cols = line.split()
        if "zone" not in cols:
            continue
        zone_at = cols.index("zone")
        orders: list[int] = []
        for part in cols[zone_at + 2 :]:
            if not part.isdigit():
                continue
            orders.append(int(part))
        if len(orders) < 8:
            continue
        if not totals:
            totals = [0] * len(orders)
        for i, n in enumerate(totals):
            if i < len(orders):
                totals[i] = n + orders[i]
    if not totals:
        return None
    return totals[:11]


def read_cgroup_node(path: str, rel: str, depth: int) -> dict | None:
    if depth > 4:
        return None
    current_raw = read_line(os.path.join(path, "memory.current"))
    if not current_raw.isdigit():
        current = None
    else:
        current = int(current_raw)
        if current < 0:
            current = None
    children: list[dict] = []
    try:
        names = sorted(os.listdir(path))
    except OSError:
        names = []
    if depth < 4:
        for name in names:
            child_path = os.path.join(path, name)
            if not os.path.isdir(child_path):
                continue
            if name.startswith("."):
                continue
            child = read_cgroup_node(child_path, (rel + "/" + name).lstrip("/"), depth + 1)
            if child:
                children.append(child)
            if len(children) >= 24:
                break
    if current is None and not children:
        return None
    node: dict = {"id": rel or "/", "path": rel or "/"}
    if current is not None:
        node["value"] = current
    if children:
        node["children"] = children
    return node


def read_cgroups() -> dict | None:
    base = root("sys/fs/cgroup")
    if not os.path.isdir(base):
        return None
    return read_cgroup_node(base, "", 0)


def read_rapl() -> list[dict]:
    base = root("sys/class/powercap")
    try:
        names = sorted(os.listdir(base))
    except OSError:
        return []
    out: list[dict] = []
    for name in names:
        energy = read_line(os.path.join(base, name, "energy_uj"))
        if not energy.isdigit():
            continue
        uj = int(energy)
        if uj < 0:
            continue
        label = read_line(os.path.join(base, name, "name")) or name
        out.append({"id": name, "name": label, "uj": uj})
        if len(out) >= 12:
            break
    return out


def read_slabinfo() -> list[dict]:
    text = read_text(root("proc/slabinfo"), 256000)
    if not text:
        return []
    out: list[dict] = []
    for line in text.splitlines():
        if not line or line.startswith("#") or line.startswith("slabinfo"):
            continue
        cols = line.split()
        if len(cols) < 3:
            continue
        name = cols[0]
        if not cols[1].isdigit() or not cols[2].isdigit():
            continue
        active = int(cols[1])
        num = int(cols[2])
        if active < 0 or num < 0:
            continue
        out.append({"name": name, "active": active, "num": num})
    out.sort(key=lambda row: row["active"], reverse=True)
    return out[:40]


def read_sensors() -> list[dict]:
    out: list[dict] = []
    try:
        hwmons = sorted(os.listdir(root("sys/class/hwmon")))
    except OSError:
        hwmons = []
    for hw in hwmons:
        if not hw.startswith("hwmon"):
            continue
        base = root("sys/class/hwmon", hw)
        chip = read_line(os.path.join(base, "name")) or hw
        try:
            names = os.listdir(base)
        except OSError:
            continue
        for name in sorted(names):
            if name.startswith("temp") and name.endswith("_input"):
                idx = name[4:-6]
                temp = milli_to_c(read_line(os.path.join(base, name)))
                if temp is None:
                    continue
                out.append(
                    {
                        "id": f"{chip}:{name}",
                        "chip": chip,
                        "label": hwmon_label(base, "temp", idx),
                        "kind": "temp",
                        "value": temp,
                    }
                )
            elif name.startswith("fan") and name.endswith("_input"):
                idx = name[3:-6]
                raw = read_line(os.path.join(base, name))
                if not raw.lstrip("-").isdigit():
                    continue
                rpm = int(raw)
                if rpm <= 0:
                    continue
                out.append(
                    {
                        "id": f"{chip}:{name}",
                        "chip": chip,
                        "label": hwmon_label(base, "fan", idx),
                        "kind": "fan",
                        "value": rpm,
                    }
                )
    return out


def main() -> int:
    idle, total = read_cpu()
    mem = read_mem()
    rx, tx = read_net()
    load1, load5, load15 = read_load()
    payload = {
        "uid": uid_wanted(),
        "cpuIdle": idle,
        "cpuTotal": total,
        "cpus": read_cpus(),
        "load1": load1,
        "load5": load5,
        "load15": load15,
        "memUsed": mem["used"],
        "memTotal": mem["total"],
        "memAvail": mem["avail"],
        "memFree": mem["free"],
        "memBuffers": mem["buffers"],
        "memCached": mem["cached"],
        "memShared": mem["shared"],
        "memSReclaimable": mem["sreclaimable"],
        "memAnon": mem["anon"],
        "memDirty": mem["dirty"],
        "memSlab": mem["slab"],
        "memPageTables": mem["pagetables"],
        "memKernelStack": mem["kernelstack"],
        "memFile": mem["file"],
        "swapUsed": mem["swapUsed"],
        "swapTotal": mem["swapTotal"],
        "netRx": rx,
        "netTx": tx,
        "ifaces": read_ifaces(),
        "disks": read_disks(),
        "psi": read_psi(),
        "tcp": read_tcp(),
        "tcpSockets": read_tcp_sockets(),
        "interrupts": read_interrupts(),
        "softirqs": read_softirqs(),
        "buddy": read_buddy(),
        "cgroups": read_cgroups(),
        "rapl": read_rapl(),
        "slabs": read_slabinfo(),
        "clkTck": clk_tck(),
        "cpuTemp": read_cpu_temp(),
        "gpus": read_gpus(),
        "sensors": read_sensors(),
        "processes": read_processes(uid_wanted()),
    }
    json.dump(payload, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
