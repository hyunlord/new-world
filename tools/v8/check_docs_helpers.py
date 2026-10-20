#!/usr/bin/env python3
"""Helpers for tools/v8/check_docs.sh (kept out of bash heredocs, which bash 3.2
mis-parses inside command substitutions when they contain backticks).

usage: check_docs_helpers.py arch-paths          paths named in docs/ARCHITECTURE.md that do not exist
       check_docs_helpers.py lessons SKILL...   per-skill lesson counts and unsourced rows
"""
import re
import sys
from pathlib import Path


def arch_paths(argv):
    text = Path("docs/ARCHITECTURE.md").read_text()
    prefixes = ("rust/", "scripts/", "scenes/", "tools/", "docs/", "assets/", "localization/", "shaders/", ".claude/", ".agents/")
    bad = []
    for token in re.findall(r"`([^`\s]+)`", text):
        path = token.split(":")[0].rstrip(".,;)")
        if path.startswith(prefixes) and "*" not in path and "<" not in path and "{" not in path:
            if not Path(path).exists():
                bad.append(path)
    print("\n".join(sorted(set(bad))))


def lessons(argv):
    out = []
    for name in argv:
        path = Path(f".claude/skills/{name}/SKILL.md")
        if not path.exists():
            continue
        text = path.read_text()
        if "## Lessons" not in text:
            out.append(f"{name}: no Lessons section")
            continue
        rows = [l for l in text.split("## Lessons", 1)[1].splitlines() if l.startswith("|") and not set(l) <= set("|-: ")]
        rows = rows[1:]
        unsourced = [r for r in rows if not re.search(r"`[^`]+`|\b[0-9a-f]{7,40}\b", r.strip().strip("|").split("|")[-1])]
        out.append(f"{name}: {len(rows)} lessons" + (f", {len(unsourced)} without a source" if unsourced else ""))
    print("\n".join(out))


def main(argv):
    if not argv or argv[0] not in ("arch-paths", "lessons"):
        sys.exit(__doc__)
    (arch_paths if argv[0] == "arch-paths" else lessons)(argv[1:])


if __name__ == "__main__":
    main(sys.argv[1:])
