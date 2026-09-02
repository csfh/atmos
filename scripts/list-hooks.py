#!/usr/bin/env python3
"""List user hook scripts under ~/.config/omarchy/hooks."""

from __future__ import annotations

import json
import os
import re
from pathlib import Path

TYPES = (
    "theme-set",
    "font-set",
    "post-boot",
    "post-update",
    "pre-refresh-pacman",
    "battery-low",
)

HOOK_ID = re.compile(r"^[a-z0-9][a-z0-9-]*$")


def hook_types(root: Path) -> list[str]:
    found = list(TYPES)
    seen = set(TYPES)
    if not root.is_dir():
        return found
    for child in sorted(root.iterdir()):
        name = child.name
        if child.is_dir() and name.endswith(".d"):
            hook_id = name[:-2]
        elif child.is_file() and not name.endswith(".sample"):
            hook_id = name
        else:
            continue
        if hook_id in seen or not HOOK_ID.fullmatch(hook_id):
            continue
        seen.add(hook_id)
        found.append(hook_id)
    return found


def main() -> int:
    root = Path(os.environ.get("HOME", "")) / ".config/omarchy/hooks"
    items = []
    for hook_type in hook_types(root):
        flat = root / hook_type
        if flat.is_file():
            items.append(
                {
                    "type": hook_type,
                    "name": hook_type,
                    "path": str(flat),
                    "sample": False,
                    "flat": True,
                }
            )
        folder = root / f"{hook_type}.d"
        if not folder.is_dir():
            continue
        for path in sorted(folder.iterdir()):
            if not path.is_file():
                continue
            name = path.name
            if "/" in name or ".." in name:
                continue
            items.append(
                {
                    "type": hook_type,
                    "name": name,
                    "path": str(path),
                    "sample": name.endswith(".sample"),
                    "flat": False,
                }
            )
    print(json.dumps(items))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
