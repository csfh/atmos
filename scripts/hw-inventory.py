#!/usr/bin/env python3
"""Emit CPU, memory, chipset, and the rest of the machine. No sudo."""

import json
import os
import re
import subprocess
import sys
from io import StringIO


SYS = os.environ.get("ATMOS_SYS_ROOT") or "/"
SKIP_CMDS = bool(os.environ.get("ATMOS_HW_SKIP_CMDS"))


def root(*parts):
    return os.path.join(SYS, *[p.lstrip("/") for p in parts])


def read_text(path, limit=8192):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return fh.read(limit)
    except OSError:
        return ""


def read_strip(path, limit=512):
    return read_text(path, limit).strip()


def read_int(path):
    text = read_strip(path)
    if not text:
        return 0
    try:
        return int(text.split()[0], 0)
    except ValueError:
        return 0


def run(argv):
    if SKIP_CMDS:
        return ""
    try:
        return subprocess.check_output(argv, text=True, stderr=subprocess.DEVNULL)
    except (OSError, subprocess.CalledProcessError):
        return ""


def parse_key_values(raw, sep=":"):
    out = {}
    for line in StringIO(raw):
        cut = line.find(sep)
        if cut < 1:
            continue
        key = line[:cut].strip()
        value = line[cut + len(sep) :].strip()
        if key:
            out[key] = value
    return out


def kb_to_bytes(value):
    try:
        return int(float(re.sub(r"[^0-9.].*", "", str(value) or "0")) * 1024)
    except ValueError:
        return 0


def clean(value, limit=240):
    text = re.sub(r"[\r\n\t]+", " ", str(value or "")).strip()
    if not text:
        return ""
    lower = text.lower()
    if lower in {
        "not specified",
        "unknown",
        "none",
        "to be filled by o.e.m.",
        "default string",
        "fill by oem",
        "system product name",
        "system manufacturer",
        "oem",
        "1234567890",
    } or re.fullmatch(r"0+", text) or re.fullmatch(r"[fF]+", text):
        return ""
    return text[:limit]


CHASSIS_TYPES = {
    1: "Other",
    2: "Unknown",
    3: "Desktop",
    4: "Low-profile desktop",
    5: "Pizza box",
    6: "Mini tower",
    7: "Tower",
    8: "Portable",
    9: "Laptop",
    10: "Notebook",
    11: "Handheld",
    12: "Docking station",
    13: "All-in-one",
    14: "Sub-notebook",
    15: "Space-saving",
    16: "Lunch box",
    17: "Main server",
    18: "Expansion",
    19: "Sub-chassis",
    20: "Bus expansion",
    21: "Peripheral",
    22: "RAID",
    23: "Rack mount",
    24: "Sealed-case PC",
    25: "Multi-system",
    26: "Compact PCI",
    27: "Advanced TCA",
    28: "Blade",
    29: "Blade enclosure",
    30: "Tablet",
    31: "Convertible",
    32: "Detachable",
    33: "IoT gateway",
    34: "Embedded PC",
    35: "Mini PC",
    36: "Stick PC",
}

MEMORY_TYPES = {
    1: "Other",
    2: "Unknown",
    3: "DRAM",
    18: "DDR",
    19: "DDR2",
    24: "DDR3",
    26: "DDR4",
    27: "LPDDR",
    29: "LPDDR3",
    30: "LPDDR4",
    32: "HBM",
    33: "HBM2",
    34: "DDR5",
    35: "LPDDR5",
    36: "LPDDR5X",
    37: "HBM3",
}

MEMORY_FORMS = {
    8: "DIMM",
    13: "SODIMM",
    15: "FB-DIMM",
    16: "Die",
}


def chassis_name(value):
    text = clean(value, 40)
    if not text:
        return ""
    if text.isdigit():
        return CHASSIS_TYPES.get(int(text), "")
    return text


def decode_memory_size(word, extended=0):
    try:
        size = int(word)
    except (TypeError, ValueError):
        return 0
    if size in (0, 0xFFFF):
        return 0
    if size == 0x7FFF:
        try:
            ext = int(extended)
        except (TypeError, ValueError):
            return 0
        return ext * 1024 * 1024 if ext > 0 else 0
    value = size & 0x7FFF
    if size & 0x8000:
        return value * 1024
    return value * 1024 * 1024


def dmi_string(payload, strings, index):
    if index <= 0 or index > len(strings):
        return ""
    return clean(strings[index - 1], 160)


def parse_dmi_table(blob):
    structs = []
    i = 0
    n = len(blob)
    while i + 4 <= n:
        typ = blob[i]
        length = blob[i + 1]
        handle = int.from_bytes(blob[i + 2 : i + 4], "little")
        if length < 4 or i + length > n:
            break
        formatted = blob[i : i + length]
        j = i + length
        strings = []
        if j < n and blob[j] == 0:
            j += 1
            if j < n and blob[j] == 0:
                j += 1
        else:
            start = j
            while j < n:
                if blob[j] == 0:
                    if j > start:
                        strings.append(blob[start:j].decode("latin-1", "replace"))
                    j += 1
                    start = j
                    if j < n and blob[j] == 0:
                        j += 1
                        break
                else:
                    j += 1
        structs.append({"type": typ, "handle": handle, "data": formatted, "strings": strings})
        i = j
        if typ == 127:
            break
    return structs


def read_dmi_blob():
    path = root("sys/firmware/dmi/tables/DMI")
    try:
        with open(path, "rb") as fh:
            return fh.read(1 << 20)
    except OSError:
        return b""


def apply_dmi(hw):
    blob = read_dmi_blob()
    if not blob:
        return
    for item in parse_dmi_table(blob):
        data = item["data"]
        strings = item["strings"]
        typ = item["type"]
        if typ == 0 and len(data) >= 9:
            hw["bios"]["vendor"] = hw["bios"]["vendor"] or dmi_string(data, strings, data[4])
            hw["bios"]["version"] = hw["bios"]["version"] or dmi_string(data, strings, data[5])
            hw["bios"]["date"] = hw["bios"]["date"] or dmi_string(data, strings, data[8])
        elif typ == 1 and len(data) >= 8:
            hw["machine"]["vendor"] = hw["machine"]["vendor"] or dmi_string(data, strings, data[4])
            hw["machine"]["name"] = hw["machine"]["name"] or dmi_string(data, strings, data[5])
            hw["machine"]["version"] = hw["machine"]["version"] or dmi_string(data, strings, data[6])
            hw["machine"]["serial"] = hw["machine"]["serial"] or dmi_string(data, strings, data[7])
            if len(data) > 0x19:
                hw["machine"]["sku"] = hw["machine"]["sku"] or dmi_string(data, strings, data[0x19])
            if len(data) > 0x1A:
                hw["machine"]["family"] = hw["machine"]["family"] or dmi_string(data, strings, data[0x1A])
        elif typ == 2 and len(data) >= 8:
            hw["board"]["vendor"] = hw["board"]["vendor"] or dmi_string(data, strings, data[4])
            hw["board"]["name"] = hw["board"]["name"] or dmi_string(data, strings, data[5])
            hw["board"]["version"] = hw["board"]["version"] or dmi_string(data, strings, data[6])
            hw["board"]["serial"] = hw["board"]["serial"] or dmi_string(data, strings, data[7])
        elif typ == 3 and len(data) >= 6:
            if not hw["machine"]["chassis"]:
                hw["machine"]["chassis"] = chassis_name(data[5] & 0x7F)
        elif typ == 4 and len(data) >= 0x11:
            version = dmi_string(data, strings, data[0x10])
            if version and not hw["cpu"]["model"]:
                hw["cpu"]["model"] = version
            vendor = dmi_string(data, strings, data[7])
            if vendor and not hw["cpu"]["vendor"]:
                hw["cpu"]["vendor"] = vendor
        elif typ == 17 and len(data) >= 0x13:
            size_word = int.from_bytes(data[0x0C:0x0E], "little")
            extended = 0
            if len(data) >= 0x20:
                extended = int.from_bytes(data[0x1C:0x20], "little")
            size = decode_memory_size(size_word, extended)
            locator = dmi_string(data, strings, data[0x10])
            if not size and not locator:
                continue
            form = MEMORY_FORMS.get(data[0x0E], "")
            mem_type = MEMORY_TYPES.get(data[0x12], "")
            speed = int.from_bytes(data[0x15:0x17], "little") if len(data) >= 0x17 else 0
            manufacturer = dmi_string(data, strings, data[0x17]) if len(data) > 0x17 else ""
            part = dmi_string(data, strings, data[0x1A]) if len(data) > 0x1A else ""
            bank = dmi_string(data, strings, data[0x11]) if len(data) > 0x11 else ""
            rank = (data[0x1B] & 0x0F) if len(data) > 0x1B else 0
            hw["memory"]["modules"].append(
                {
                    "locator": locator,
                    "bank": bank,
                    "size": size,
                    "type": mem_type,
                    "form": form,
                    "speed": speed if speed not in (0, 0xFFFF) else 0,
                    "manufacturer": manufacturer,
                    "part": part,
                    "rank": rank,
                }
            )


def dmi_id():
    base = root("sys/class/dmi/id")
    if not os.path.isdir(base):
        return {}
    keys = (
        "bios_date",
        "bios_vendor",
        "bios_version",
        "board_name",
        "board_serial",
        "board_vendor",
        "board_version",
        "chassis_type",
        "product_family",
        "product_name",
        "product_serial",
        "product_sku",
        "product_version",
        "sys_vendor",
    )
    out = {}
    for key in keys:
        out[key] = clean(read_strip(os.path.join(base, key)), 160)
    return out


def apply_dmi_id(hw, info):
    hw["bios"]["vendor"] = hw["bios"]["vendor"] or info.get("bios_vendor", "")
    hw["bios"]["version"] = hw["bios"]["version"] or info.get("bios_version", "")
    hw["bios"]["date"] = hw["bios"]["date"] or info.get("bios_date", "")
    hw["machine"]["vendor"] = hw["machine"]["vendor"] or info.get("sys_vendor", "")
    hw["machine"]["name"] = hw["machine"]["name"] or info.get("product_name", "")
    hw["machine"]["family"] = hw["machine"]["family"] or info.get("product_family", "")
    hw["machine"]["version"] = hw["machine"]["version"] or info.get("product_version", "")
    hw["machine"]["serial"] = hw["machine"]["serial"] or info.get("product_serial", "")
    hw["machine"]["sku"] = hw["machine"]["sku"] or info.get("product_sku", "")
    if not hw["machine"]["chassis"]:
        hw["machine"]["chassis"] = chassis_name(info.get("chassis_type", ""))
    hw["board"]["vendor"] = hw["board"]["vendor"] or info.get("board_vendor", "")
    hw["board"]["name"] = hw["board"]["name"] or info.get("board_name", "")
    hw["board"]["version"] = hw["board"]["version"] or info.get("board_version", "")
    hw["board"]["serial"] = hw["board"]["serial"] or info.get("board_serial", "")


def parse_meminfo():
    raw = read_text(root("proc/meminfo"), 16384)
    kv = parse_key_values(raw)
    total = kb_to_bytes(kv.get("MemTotal"))
    available = kb_to_bytes(kv.get("MemAvailable"))
    if not available:
        available = kb_to_bytes(kv.get("MemFree")) + kb_to_bytes(kv.get("Cached")) + kb_to_bytes(kv.get("Buffers"))
    if available > total:
        available = total
    used = total - available if total > available else 0
    swap_total = kb_to_bytes(kv.get("SwapTotal"))
    swap_free = kb_to_bytes(kv.get("SwapFree"))
    swap_used = swap_total - swap_free if swap_total > swap_free else 0
    return {
        "total": total,
        "used": used,
        "available": available,
        "swapTotal": swap_total,
        "swapUsed": swap_used,
        "modules": [],
    }


def parse_cpuinfo():
    raw = read_text(root("proc/cpuinfo"), 1 << 18)
    blocks = re.split(r"\n\n+", raw)
    model = ""
    vendor = ""
    flags = []
    mhz = 0.0
    physical = {}
    threads = 0
    for block in blocks:
        kv = parse_key_values(block)
        if "processor" not in kv and "model name" not in kv and "Hardware" not in kv:
            continue
        threads += 1
        model = model or clean(kv.get("model name") or kv.get("Hardware") or kv.get("model"), 160)
        vendor = vendor or clean(kv.get("vendor_id") or kv.get("Hardware"), 80)
        if not flags and kv.get("flags"):
            flags = [f for f in kv["flags"].split() if f]
        try:
            hz = float(kv.get("cpu MHz") or 0)
        except ValueError:
            hz = 0
        if hz > mhz:
            mhz = hz
        phys = kv.get("physical id") or "0"
        try:
            cores = int(kv.get("cpu cores") or 0)
        except ValueError:
            cores = 0
        physical[phys] = max(physical.get(phys, 0), cores or 1)
    cores = sum(physical.values()) or threads
    return {
        "model": model,
        "vendor": vendor,
        "arch": "",
        "cores": cores,
        "threads": threads,
        "sockets": len(physical) or 1,
        "mhz": int(round(mhz)),
        "maxMhz": 0,
        "caches": "",
        "flags": flags,
        "virtualization": "",
        "hypervisor": "",
    }


def apply_lscpu(cpu):
    raw = run(["lscpu"])
    if not raw:
        return
    kv = parse_key_values(raw)

    def num(key):
        text = re.sub(r"[^0-9.].*", "", kv.get(key) or "")
        try:
            return int(float(text)) if text else 0
        except ValueError:
            return 0

    cpu["model"] = cpu["model"] or clean(kv.get("Model name") or kv.get("Model Name"), 160)
    cpu["vendor"] = cpu["vendor"] or clean(kv.get("Vendor ID") or kv.get("BIOS Vendor ID"), 80)
    cpu["arch"] = clean(kv.get("Architecture"), 40)
    sockets = num("Socket(s)") or cpu["sockets"] or 1
    per = num("Core(s) per socket")
    cpu["sockets"] = sockets
    if per:
        cpu["cores"] = per * sockets
    cpu["threads"] = num("CPU(s)") or cpu["threads"]
    cpu["mhz"] = cpu["mhz"] or num("CPU MHz")
    cpu["maxMhz"] = num("CPU max MHz")
    caches = []
    for key, label in (
        ("L1d cache", "L1d"),
        ("L1i cache", "L1i"),
        ("L2 cache", "L2"),
        ("L3 cache", "L3"),
    ):
        if kv.get(key):
            caches.append(f"{label} {kv[key]}")
    cpu["caches"] = ", ".join(caches)
    if not cpu["flags"] and kv.get("Flags"):
        cpu["flags"] = [f for f in kv["Flags"].split() if f]
    cpu["virtualization"] = clean(kv.get("Virtualization"), 40)
    cpu["hypervisor"] = clean(kv.get("Hypervisor vendor"), 40)


def apply_cpufreq(cpu):
    max_hz = read_int(root("sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq"))
    cur_hz = read_int(root("sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq"))
    if max_hz:
        cpu["maxMhz"] = cpu["maxMhz"] or max_hz // 1000
    if cur_hz:
        cpu["mhz"] = cpu["mhz"] or cur_hz // 1000


def pci_ids_paths():
    return [
        root("usr/share/hwdata/pci.ids"),
        root("usr/share/misc/pci.ids"),
        "/usr/share/hwdata/pci.ids",
        "/usr/share/misc/pci.ids",
    ]


def lookup_pci_names(pairs):
    needed = {vendor: set() for vendor, _device in pairs}
    for vendor, device in pairs:
        needed.setdefault(vendor, set()).add(device)
    names = {}
    path = next((p for p in pci_ids_paths() if os.path.isfile(p)), "")
    if not path:
        return names
    vendor = ""
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            for line in fh:
                if not line or line[0] == "#" or line.startswith("C "):
                    continue
                if line[0] != "\t":
                    parts = line.split(None, 1)
                    if len(parts) != 2:
                        continue
                    vendor = parts[0].lower()
                    if vendor in needed:
                        names[vendor] = parts[1].strip()
                    continue
                if line.startswith("\t\t") or vendor not in needed:
                    continue
                parts = line.strip().split(None, 1)
                if len(parts) != 2:
                    continue
                device = parts[0].lower()
                if device in needed[vendor]:
                    names[f"{vendor}:{device}"] = parts[1].strip()
    except OSError:
        return names
    return names


def sysfs_pci_devices():
    base = root("sys/bus/pci/devices")
    if not os.path.isdir(base):
        return []
    rows = []
    for name in sorted(os.listdir(base)):
        path = os.path.join(base, name)
        vendor = read_strip(os.path.join(path, "vendor")).replace("0x", "").lower()
        device = read_strip(os.path.join(path, "device")).replace("0x", "").lower()
        cls = read_strip(os.path.join(path, "class")).replace("0x", "").lower()
        if len(vendor) < 4 or len(device) < 4:
            continue
        rows.append(
            {
                "slot": name.replace("0000:", ""),
                "classId": cls[:4] if len(cls) >= 4 else cls,
                "className": "",
                "vendor": "",
                "name": "",
                "pciId": f"{vendor}:{device}",
                "vendorId": vendor,
                "deviceId": device,
            }
        )
    names = lookup_pci_names([(r["vendorId"], r["deviceId"]) for r in rows])
    class_names = {
        "0600": "Host bridge",
        "0601": "ISA bridge",
        "0300": "VGA compatible controller",
        "0302": "3D controller",
        "0380": "Display controller",
        "0200": "Ethernet controller",
        "0280": "Network controller",
        "0403": "Audio device",
        "0c03": "USB controller",
        "0c05": "SMBus",
    }
    for row in rows:
        row["vendor"] = names.get(row["vendorId"], "")
        row["name"] = names.get(row["pciId"], "")
        row["className"] = class_names.get(row["classId"], "")
    return rows


def parse_lspci():
    raw = run(["lspci", "-nnmm"]) or run(["lspci", "-nn"])
    if not raw:
        return sysfs_pci_devices()
    rows = []
    mm = re.compile(
        r'^(\S+)\s+"([^"]*)"\s+"([^"]*)"\s+"([^"]*)"'
    )
    classic = re.compile(r"^(\S+)\s+([^:]+):\s+(.*)$")
    for line in raw.splitlines():
        match = mm.match(line)
        if match:
            cls, ven, dev = match.group(2), match.group(3), match.group(4)
            class_id = (re.search(r"\[([0-9a-fA-F]{4})\]", cls) or [None, ""])[1].lower()
            vendor_id = (re.search(r"\[([0-9a-fA-F]{4})\]", ven) or [None, ""])[1].lower()
            device_id = (re.search(r"\[([0-9a-fA-F]{4,8})\]", dev) or [None, ""])[1].lower()
            rows.append(
                {
                    "slot": match.group(1),
                    "className": clean(re.sub(r"\s*\[[0-9a-fA-F]{4}\]\s*$", "", cls), 80),
                    "classId": class_id,
                    "vendor": clean(re.sub(r"\s*\[[0-9a-fA-F]{4}\]\s*$", "", ven), 80),
                    "name": clean(re.sub(r"\s*\[[0-9a-fA-F]{4,8}\]\s*$", "", dev), 160),
                    "pciId": f"{vendor_id}:{device_id}" if vendor_id and device_id else "",
                }
            )
            continue
        match = classic.match(line)
        if not match:
            continue
        ids = re.search(r"\[([0-9a-fA-F]{4}):([0-9a-fA-F]{4})\]", match.group(3))
        rows.append(
            {
                "slot": match.group(1),
                "className": clean(match.group(2), 80),
                "classId": "",
                "vendor": "",
                "name": clean(re.sub(r"\s*\[[0-9a-fA-F]{4}:[0-9a-fA-F]{4}\]\s*$", "", match.group(3)), 160),
                "pciId": f"{ids.group(1).lower()}:{ids.group(2).lower()}" if ids else "",
            }
        )
    return rows


def is_host_bridge(row):
    return row.get("classId") == "0600" or "host bridge" in (row.get("className") or "").lower()


def is_isa_bridge(row):
    name = (row.get("className") or "").lower()
    return row.get("classId") == "0601" or "isa bridge" in name or "lpc" in name


def is_gpu(row):
    if row.get("classId") in {"0300", "0302", "0380"}:
        return True
    name = (row.get("className") or "").lower()
    return "vga" in name or "3d controller" in name or "display controller" in name


def pick_chipset(devices):
    host = next((r for r in devices if is_host_bridge(r)), None)
    south = next((r for r in devices if is_isa_bridge(r)), None)
    primary = host or south
    if not primary:
        return {"name": "", "vendor": "", "pciId": "", "role": "", "southbridge": ""}
    south_name = ""
    if host and south and south.get("name") and south.get("name") != host.get("name"):
        south_name = south.get("name") or ""
    return {
        "name": clean(primary.get("name"), 160),
        "vendor": clean(primary.get("vendor"), 80),
        "pciId": clean(primary.get("pciId"), 20),
        "role": "Host bridge" if host else "ISA bridge",
        "southbridge": clean(south_name, 160),
    }


def drm_driver(pci_id):
    base = root("sys/class/drm")
    if not os.path.isdir(base):
        return ""
    for name in os.listdir(base):
        if not name.startswith("card") or "-" in name:
            continue
        device = os.path.join(base, name, "device")
        vendor = read_strip(os.path.join(device, "vendor")).replace("0x", "").lower()
        dev = read_strip(os.path.join(device, "device")).replace("0x", "").lower()
        if pci_id and f"{vendor}:{dev}" != pci_id:
            continue
        driver = os.path.basename(os.path.realpath(os.path.join(device, "driver")))
        if driver and driver != "driver":
            return driver
    return ""


def collect_gpus(devices):
    out = []
    for row in devices:
        if not is_gpu(row):
            continue
        pci_id = row.get("pciId") or ""
        out.append(
            {
                "name": clean(row.get("name"), 160),
                "vendor": clean(row.get("vendor"), 80),
                "pciId": pci_id,
                "driver": drm_driver(pci_id),
                "vram": 0,
            }
        )
    if out:
        return out
    base = root("sys/class/drm")
    if not os.path.isdir(base):
        return out
    seen = set()
    for name in sorted(os.listdir(base)):
        if not name.startswith("card") or "-" in name:
            continue
        if name in seen:
            continue
        seen.add(name)
        device = os.path.join(base, name, "device")
        driver = os.path.basename(os.path.realpath(os.path.join(device, "driver")))
        if driver == "driver":
            driver = ""
        out.append({"name": name, "vendor": "", "pciId": "", "driver": driver, "vram": 0})
    return out


VIRTUAL_NET = re.compile(
    r"^(lo|docker\d*|veth|br-|virbr|cni|flannel|tun|tap|wg|tailscale|zt)"
)


def collect_nics():
    base = root("sys/class/net")
    if not os.path.isdir(base):
        return []
    rows = []
    for name in sorted(os.listdir(base)):
        path = os.path.join(base, name)
        if VIRTUAL_NET.match(name):
            continue
        if not os.path.isdir(path):
            continue
        iface_type = read_strip(os.path.join(path, "type"))
        if iface_type == "772":
            continue
        driver_path = os.path.join(path, "device", "driver")
        driver = ""
        if os.path.exists(driver_path):
            driver = os.path.basename(os.path.realpath(driver_path))
            if driver == "driver":
                driver = ""
        wireless = os.path.isdir(os.path.join(path, "wireless"))
        speed = read_int(os.path.join(path, "speed"))
        if speed < 0:
            speed = 0
        rows.append(
            {
                "name": name,
                "iface": name,
                "mac": clean(read_strip(os.path.join(path, "address")), 20),
                "driver": driver,
                "speed": speed,
                "wireless": wireless,
            }
        )
    return rows


def collect_audio():
    cards = read_text(root("proc/asound/cards"), 8192)
    out = []
    for match in re.finditer(r"^\s*(\d+)\s+\[([^\]]+)\]:\s+(.*)$", cards, re.M):
        out.append({"id": match.group(1), "name": clean(match.group(3) or match.group(2), 160), "driver": clean(match.group(2), 40)})
    if out:
        return out
    base = root("sys/class/sound")
    if not os.path.isdir(base):
        return out
    for name in sorted(os.listdir(base)):
        if not name.startswith("card") or name.endswith("c"):
            continue
        ident = read_strip(os.path.join(base, name, "id"))
        if ident:
            out.append({"id": name, "name": ident, "driver": ""})
    return out


def collect_usb():
    base = root("sys/bus/usb/devices")
    if not os.path.isdir(base):
        return []
    out = []
    for name in sorted(os.listdir(base)):
        path = os.path.join(base, name)
        product = clean(read_strip(os.path.join(path, "product")), 160)
        manufacturer = clean(read_strip(os.path.join(path, "manufacturer")), 80)
        if not product and not manufacturer:
            continue
        if name.startswith("usb"):
            continue
        speed = clean(read_strip(os.path.join(path, "speed")), 20)
        out.append({"name": product or manufacturer, "vendor": manufacturer, "product": product, "speed": speed})
    return out


def collect_batteries():
    base = root("sys/class/power_supply")
    if not os.path.isdir(base):
        return []
    out = []
    for name in sorted(os.listdir(base)):
        path = os.path.join(base, name)
        kind = read_strip(os.path.join(path, "type")).lower()
        if kind and kind != "battery":
            continue
        if not os.path.isfile(os.path.join(path, "capacity")) and kind != "battery":
            continue
        if not os.path.isfile(os.path.join(path, "capacity")):
            continue
        out.append(
            {
                "name": clean(read_strip(os.path.join(path, "model_name")) or name, 80),
                "status": clean(read_strip(os.path.join(path, "status")), 20),
                "capacity": read_int(os.path.join(path, "capacity")),
                "technology": clean(read_strip(os.path.join(path, "technology")), 20),
                "energy": read_int(os.path.join(path, "energy_now")),
                "voltage": read_int(os.path.join(path, "voltage_now")),
            }
        )
    return out


def collect_thermals():
    base = root("sys/class/thermal")
    if not os.path.isdir(base):
        return []
    out = []
    for name in sorted(os.listdir(base)):
        if not name.startswith("thermal_zone"):
            continue
        path = os.path.join(base, name)
        milli = read_int(os.path.join(path, "temp"))
        if milli <= 0:
            continue
        kind = clean(read_strip(os.path.join(path, "type")), 40)
        out.append({"name": kind or name, "type": kind, "temp": milli / 1000.0})
    return out


def collect_tpm():
    tpm0 = root("sys/class/tpm/tpm0")
    if os.path.isdir(tpm0):
        version = clean(read_strip(os.path.join(tpm0, "tpm_version_major")), 12)
        return {"present": True, "version": version}
    if os.path.exists(root("dev/tpm0")) or os.path.exists(root("dev/tpmrm0")):
        return {"present": True, "version": ""}
    return {"present": False, "version": ""}


def collect_secure_boot():
    efi = root("sys/firmware/efi")
    if not os.path.isdir(efi):
        return {"available": False, "enabled": False}
    var_dir = os.path.join(efi, "efivars")
    enabled = False
    if os.path.isdir(var_dir):
        for name in os.listdir(var_dir):
            if not name.startswith("SecureBoot-"):
                continue
            try:
                with open(os.path.join(var_dir, name), "rb") as fh:
                    blob = fh.read(8)
                if len(blob) >= 5:
                    enabled = blob[4] == 1
            except OSError:
                pass
            break
    return {"available": True, "enabled": enabled}


def collect_virt(cpu):
    hypervisor = clean(cpu.get("hypervisor"), 40)
    flags = cpu.get("flags") or []
    guest = bool(hypervisor) or "hypervisor" in flags
    kvm = os.path.exists(root("dev/kvm"))
    if not hypervisor:
        for path in (
            root("sys/hypervisor/type"),
            root("sys/class/dmi/id/product_name"),
        ):
            text = clean(read_strip(path), 40)
            if text.lower() in {"kvm", "qemu", "vmware", "virtualbox", "xen", "microsoft", "bhyve", "parallels"}:
                hypervisor = text
                guest = True
                break
    detect = run(["systemd-detect-virt"])
    if detect:
        virt = clean(detect.splitlines()[0], 40)
        if virt and virt.lower() != "none":
            hypervisor = hypervisor or virt
            guest = True
    return {"hypervisor": hypervisor, "guest": guest, "kvm": kvm}


def empty_hardware():
    return {
        "machine": {
            "vendor": "",
            "name": "",
            "family": "",
            "version": "",
            "serial": "",
            "sku": "",
            "chassis": "",
        },
        "board": {"vendor": "", "name": "", "version": "", "serial": ""},
        "chipset": {"name": "", "vendor": "", "pciId": "", "role": "", "southbridge": ""},
        "bios": {"vendor": "", "version": "", "date": "", "uefi": os.path.isdir(root("sys/firmware/efi"))},
        "cpu": {
            "model": "",
            "vendor": "",
            "arch": "",
            "cores": 0,
            "threads": 0,
            "sockets": 1,
            "mhz": 0,
            "maxMhz": 0,
            "caches": "",
            "flags": [],
            "virtualization": "",
            "hypervisor": "",
        },
        "memory": {"total": 0, "used": 0, "available": 0, "swapTotal": 0, "swapUsed": 0, "modules": []},
        "gpus": [],
        "nics": [],
        "audio": [],
        "usb": [],
        "batteries": [],
        "thermals": [],
        "tpm": {"present": False, "version": ""},
        "secureBoot": {"available": False, "enabled": False},
        "virtualization": {"hypervisor": "", "guest": False, "kvm": False},
        "pci": [],
    }


def main():
    hw = empty_hardware()
    hw["memory"] = parse_meminfo()
    hw["cpu"] = parse_cpuinfo()
    apply_lscpu(hw["cpu"])
    apply_cpufreq(hw["cpu"])
    apply_dmi_id(hw, dmi_id())
    apply_dmi(hw)
    devices = parse_lspci()
    hw["chipset"] = pick_chipset(devices)
    hw["gpus"] = collect_gpus(devices)
    hw["pci"] = [
        {
            "slot": r.get("slot", ""),
            "className": r.get("className", ""),
            "classId": r.get("classId", ""),
            "vendor": r.get("vendor", ""),
            "name": r.get("name", ""),
            "pciId": r.get("pciId", ""),
        }
        for r in devices
        if is_host_bridge(r) or is_isa_bridge(r) or is_gpu(r) or r.get("classId") in {"0200", "0280", "0403", "0c05"}
    ]
    hw["nics"] = collect_nics()
    hw["audio"] = collect_audio()
    hw["usb"] = collect_usb()
    hw["batteries"] = collect_batteries()
    hw["thermals"] = collect_thermals()
    hw["tpm"] = collect_tpm()
    hw["secureBoot"] = collect_secure_boot()
    hw["virtualization"] = collect_virt(hw["cpu"])
    json.dump(hw, sys.stdout, separators=(",", ":"))


if __name__ == "__main__":
    main()
