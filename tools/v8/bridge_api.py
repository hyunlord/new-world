#!/usr/bin/env python3
"""List every GDScript-visible item the sim-bridge GDExtension exports.

Scans `rust/crates/sim-bridge/src/**/*.rs` for `#[derive(GodotClass)]`
structs and for items inside `#[godot_api]` impl blocks marked `#[func]`,
`#[signal]` or `#[constant]` (other attributes may sit between the marker and
the `fn`/`const`). `#[func(rename = x)]` exports the renamed name.

Output (sorted, one per line): `class <Name>`, `func <Class>.<name>`,
`signal <Class>.<name>`, `constant <Class>.<name>`.

usage: bridge_api.py [REPO_ROOT]        print the list
       bridge_api.py REPO_ROOT --check EXPECTED.txt [--minus REMOVED.txt]
           exit 1 unless the list equals EXPECTED minus the names in REMOVED.
"""
import re
import sys
from pathlib import Path

CLASS_RE = re.compile(r"#\[derive\([^)]*GodotClass[^)]*\)\]\s*(?:#\[[^\]]*\]\s*|///[^\n]*\n\s*)*pub\s+struct\s+(\w+)")
IMPL_RE = re.compile(r"#\[godot_api\]\s*impl\s+(?:(\w+)\s+for\s+)?(\w+)")
MARK_RE = re.compile(r"#\[(func|signal|constant)(\([^)]*\))?\]")
ITEM_RE = re.compile(r"\s*(?:(?:#\[[^\]]*\]|///[^\n]*|//[^\n]*)\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:fn|const)\s+(\w+)")
RENAME_RE = re.compile(r"rename\s*=\s*(\w+)")


def block_end(text, start):
    """Index just past the brace block that opens at or after `start`."""
    i = text.index("{", start)
    depth = 0
    while i < len(text):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError("unbalanced braces")


def scan_source(text):
    """Return the set of exported item lines in one Rust source text."""
    items = {f"class {name}" for name in CLASS_RE.findall(text)}
    for impl in IMPL_RE.finditer(text):
        trait, cls = impl.group(1), impl.group(2)
        if trait is not None:
            continue  # virtual-method trait impls (INode) export no new names
        body = text[impl.end():block_end(text, impl.end())]
        for mark in MARK_RE.finditer(body):
            item = ITEM_RE.match(body, mark.end())
            if item is None:
                continue
            name = item.group(1)
            rename = RENAME_RE.search(mark.group(2) or "")
            if rename:
                name = rename.group(1)
            items.add(f"{mark.group(1)} {cls}.{name}")
    return items


def scan_repo(root):
    """Return the sorted export list of the bridge crate under `root`."""
    items = set()
    for path in sorted(Path(root, "rust/crates/sim-bridge/src").rglob("*.rs")):
        items |= scan_source(path.read_text())
    return sorted(items)


def read_list(path):
    return [line.strip() for line in Path(path).read_text().splitlines() if line.strip() and not line.startswith("#")]


def main(argv):
    root = argv[1] if len(argv) > 1 and not argv[1].startswith("--") else "."
    current = scan_repo(root)
    if "--check" not in argv:
        print("\n".join(current))
        return 0
    expected = set(read_list(argv[argv.index("--check") + 1]))
    removed = set(read_list(argv[argv.index("--minus") + 1])) if "--minus" in argv else set()
    missing_removed = removed - expected
    want = expected - removed
    extra, missing = sorted(set(current) - want), sorted(want - set(current))
    print(f"bridge api: {len(current)} items; expected {len(want)} ({len(expected)} at base minus {len(removed)} removed)")
    for line in extra:
        print(f"  unexpected: {line}")
    for line in missing:
        print(f"  missing: {line}")
    for line in sorted(missing_removed):
        print(f"  removed name not in base list: {line}")
    return 0 if not extra and not missing and not missing_removed else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
