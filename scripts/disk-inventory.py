#!/usr/bin/env python3
"""Emit disks, LUKS paths, and swap from lsblk + df. No sudo."""

import json
import os
import subprocess
import sys


def run(argv):
    try:
        return subprocess.check_output(argv, text=True, stderr=subprocess.DEVNULL)
    except (OSError, subprocess.CalledProcessError):
        return ""


def device_paths(node, acc):
    path = node.get("path") or ""
    if path:
        acc.add(path)
    for child in node.get("children") or []:
        device_paths(child, acc)


def collect_luks(node, acc):
    if node.get("fstype") == "crypto_LUKS":
        path = node.get("path")
        if path:
            acc.append(path)
    for child in node.get("children") or []:
        collect_luks(child, acc)


def parse_df():
    rows = []
    raw = run(["df", "-B1", "-P"])
    lines = raw.splitlines()
    if len(lines) < 2:
        return rows
    for line in lines[1:]:
        parts = line.split()
        if len(parts) < 6:
            continue
        source, size, used, avail, _pct, target = parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]
        source = source.split("[", 1)[0]
        if not target.startswith("/"):
            continue
        if source.startswith("tmpfs") or source.startswith("devtmpfs") or source == "efivarfs":
            continue
        try:
            rows.append(
                {
                    "target": target,
                    "source": source,
                    "size": int(size),
                    "used": int(used),
                    "avail": int(avail),
                }
            )
        except ValueError:
            continue
    return rows


def mount_score(target):
    if target == "/":
        return 0
    if target == "/boot":
        return 1
    if target == "/home":
        return 2
    if target.startswith("/mnt/") or target == "/mnt":
        return 3
    return 10 + len(target)


def mounts_for_disk(node, df_rows):
    paths = set()
    device_paths(node, paths)
    names = {os.path.basename(p) for p in paths}
    by_source = {}
    for row in df_rows:
        src = row["source"]
        if src not in paths and os.path.basename(src) not in names:
            continue
        cur = by_source.get(src)
        if cur is None or mount_score(row["target"]) < mount_score(cur["target"]):
            by_source[src] = row
    return sorted(by_source.values(), key=lambda r: r["target"])


def main():
    raw = run(
        [
            "lsblk",
            "-J",
            "-b",
            "-o",
            "NAME,PATH,SIZE,TYPE,FSTYPE,LABEL,UUID,MOUNTPOINT,MODEL,TRAN,ROTA,HOTPLUG,PKNAME,RM",
        ]
    )
    disks = []
    swaps = []
    luks = []
    try:
        tree = json.loads(raw or "{}")
    except json.JSONDecodeError:
        tree = {}
    df_rows = parse_df()
    for node in tree.get("blockdevices") or []:
        name = str(node.get("name") or "")
        path = str(node.get("path") or "")
        ntype = str(node.get("type") or "")
        fstype = str(node.get("fstype") or "")
        collect_luks(node, luks)
        if ntype == "disk" and (fstype == "swap" or name.startswith("zram") or name.startswith("loop")):
            swaps.append(
                {
                    "path": path,
                    "name": name,
                    "size": int(node.get("size") or 0),
                    "label": str(node.get("label") or name),
                }
            )
            continue
        if ntype != "disk":
            continue
        if not path.startswith("/dev/"):
            continue
        info = run(["omarchy", "drive", "info", path]).strip()
        disks.append(
            {
                "path": path,
                "name": name,
                "model": str(node.get("model") or "").strip(),
                "tran": str(node.get("tran") or ""),
                "size": int(node.get("size") or 0),
                "info": info or path,
                "mounts": mounts_for_disk(node, df_rows),
            }
        )
    snapshots = []
    snapper_raw = run(["snapper", "--jsonout", "list"])
    if snapper_raw:
        try:
            parsed = json.loads(snapper_raw)
        except json.JSONDecodeError:
            parsed = {}
        if isinstance(parsed, dict):
            for config, entries in parsed.items():
                if not isinstance(entries, list):
                    continue
                for entry in entries:
                    if not isinstance(entry, dict):
                        continue
                    number = entry.get("number", entry.get("id"))
                    try:
                        ident = int(number)
                    except (TypeError, ValueError):
                        continue
                    if ident < 1:
                        continue
                    snapshots.append(
                        {
                            "config": str(config or ""),
                            "id": ident,
                            "date": str(entry.get("date") or ""),
                            "description": str(entry.get("description") or ""),
                            "type": str(entry.get("type") or ""),
                        }
                    )
        snapshots.sort(key=lambda s: (-s["id"], s["config"]))
        snapshots = snapshots[:40]
    json.dump(
        {"disks": disks, "luksDevices": luks, "swapDevices": swaps, "snapshots": snapshots},
        sys.stdout,
        separators=(",", ":"),
    )


if __name__ == "__main__":
    main()
