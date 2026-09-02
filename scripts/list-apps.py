#!/usr/bin/env python3
"""List user-installed desktop, TUI, and web app launchers as JSON."""

import json
import os
import re
from urllib.parse import unquote


def applications_dir():
    home = os.environ.get("HOME") or os.path.expanduser("~")
    data = os.environ.get("XDG_DATA_HOME") or os.path.join(home, ".local", "share")
    return os.path.join(data, "applications")


def desktop_files(root):
    out = []
    if not os.path.isdir(root):
        return out
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in filenames:
            if name.endswith(".desktop"):
                out.append(os.path.join(dirpath, name))
    return out


def parse_desktop(path):
    entry = {}
    section = None
    try:
        raw = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return entry
    for line in raw.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        if section != "Desktop Entry" or "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key not in entry:
            entry[key] = value
    return entry


def truthy(value):
    return str(value or "").strip().lower() in ("true", "1", "yes")


def classify(exec_line):
    text = str(exec_line or "")
    if re.search(r"omarchy-launch-webapp|omarchy-webapp-handler", text):
        return "web"
    if re.search(r"(^|[\s=])(\$TERMINAL|xdg-terminal-exec)(\s|$).*-e(\s|$)", text):
        return "tui"
    return "desktop"


def web_url(exec_line):
    parts = str(exec_line or "").split()
    for part in parts:
        if part.startswith("http://") or part.startswith("https://"):
            return part
    return ""


def tui_command(exec_line):
    text = str(exec_line or "")
    match = re.search(r"(?:^|\s)-e\s+(.+)$", text)
    return match.group(1).strip() if match else ""


def file_id(path):
    name = os.path.basename(path)
    if name.endswith(".desktop"):
        name = name[: -len(".desktop")]
    return unquote(name)


def main():
    root = applications_dir()
    buckets = {"desktop": [], "tui": [], "web": []}
    seen = set()
    for path in desktop_files(root):
        entry = parse_desktop(path)
        if truthy(entry.get("Hidden")) or truthy(entry.get("NoDisplay")):
            continue
        kind_type = entry.get("Type") or "Application"
        if kind_type != "Application":
            continue
        exec_line = entry.get("Exec") or ""
        if not exec_line:
            continue
        kind = classify(exec_line)
        app_id = file_id(path)
        if (kind, app_id) in seen:
            continue
        seen.add((kind, app_id))
        name = (entry.get("Name") or app_id).strip() or app_id
        detail = ""
        if kind == "web":
            detail = web_url(exec_line)
        elif kind == "tui":
            detail = tui_command(exec_line)
        buckets[kind].append({"id": app_id, "name": name, "detail": detail})

    for key in buckets:
        buckets[key].sort(key=lambda item: item["name"].lower())
    print(json.dumps(buckets, separators=(",", ":")))


if __name__ == "__main__":
    main()
