#!/usr/bin/env python3
"""Emit the session user, avatar path, human logins, and groups. No sudo."""

from __future__ import annotations

import json
import os
import pwd
import sys

ALWAYS_GROUPS = {"wheel", "docker"}


def human_uid(uid: int) -> bool:
    return 1000 <= uid < 65534


def removable_gid(gid: int, name: str) -> bool:
    if name in ("wheel", "docker"):
        return False
    return 1000 <= gid < 65534


def gecos_full_name(gecos: str) -> str:
    return (gecos or "").split(",", 1)[0]


def current_login() -> str:
    env = os.environ.get("USER") or os.environ.get("LOGNAME") or ""
    if env:
        return env
    try:
        return pwd.getpwuid(os.getuid()).pw_name
    except KeyError:
        return ""


def avatar_path(home: str, username: str) -> str:
    candidates = []
    if username:
        candidates.append(f"/var/lib/AccountsService/icons/{username}")
    if home:
        candidates.append(os.path.join(home, ".face.icon"))
        candidates.append(os.path.join(home, ".face"))
    for path in candidates:
        if path and os.path.isfile(path):
            return path
    return ""


def parse_passwd(current: str) -> list[dict]:
    users = []
    try:
        fh = open("/etc/passwd", encoding="utf-8")
    except OSError:
        return users
    with fh:
        for line in fh:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split(":")
            if len(parts) < 7:
                continue
            name = parts[0]
            try:
                uid = int(parts[2])
                gid = int(parts[3])
            except ValueError:
                continue
            if name != current and not human_uid(uid):
                continue
            users.append(
                {
                    "name": name,
                    "uid": uid,
                    "gid": gid,
                    "fullName": gecos_full_name(parts[4]),
                    "home": parts[5],
                    "shell": parts[6],
                    "wheel": False,
                    "current": name == current,
                }
            )
    return users


def parse_group() -> list[dict]:
    groups = []
    try:
        fh = open("/etc/group", encoding="utf-8")
    except OSError:
        return groups
    with fh:
        for line in fh:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split(":")
            if len(parts) < 4:
                continue
            name = parts[0]
            try:
                gid = int(parts[2])
            except ValueError:
                continue
            members = [m for m in parts[3].split(",") if m]
            groups.append({"name": name, "gid": gid, "members": members})
    return groups


def main() -> int:
    current = current_login()
    users = parse_passwd(current)
    groups = parse_group()
    wheel_members = set()
    for g in groups:
        if g["name"] == "wheel":
            wheel_members.update(g["members"])
    for u in users:
        u["wheel"] = u["name"] in wheel_members
    humans = {u["name"] for u in users}
    visible = []
    for g in groups:
        if g["name"] not in ALWAYS_GROUPS and any(
            u["name"] == g["name"] and u["gid"] == g["gid"] for u in users
        ):
            continue
        keep = g["name"] in ALWAYS_GROUPS or removable_gid(g["gid"], g["name"])
        if not keep and any(m in humans for m in g["members"]):
            keep = True
        if keep:
            visible.append(g)
    session = next((u for u in users if u["current"]), None)
    home = session["home"] if session else os.path.expanduser("~")
    payload = {
        "currentUser": current,
        "avatarPath": avatar_path(home, current),
        "users": users,
        "groups": visible,
    }
    json.dump(payload, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception:
        json.dump({"currentUser": "", "avatarPath": "", "users": [], "groups": []}, sys.stdout)
        sys.stdout.write("\n")
        raise SystemExit(0)
