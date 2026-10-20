#!/usr/bin/env python3
"""Reachability of Godot files from project.godot (V8-0 goal 6.1 B/C).

Starts from every `res://` path in `project.godot` (main scene, autoloads,
theme, icon) and the `.gdextension` files, then follows, transitively:

- `.gd`:   `preload("res://…")`, `load("res://…")`, `extends "res://…"`, any
           other `res://` string literal, and uses of a `class_name` declared
           by another `.gd` file (whole-word match, comments stripped);
- `.tscn` / `.tres`: `[ext_resource … path="res://…"]` and other literals;
- `.gdshader`: `#include "res://…"`;
- `.json` / `.cfg`: `res://` string literals (atlas pages, terrain sheets).

A literal that is formatted or concatenated (`"res://a/%s.json" % x`,
`"res://a/" + x`) is a *dynamic family*, turned into a glob where `*` matches
within one path segment: each `%s`/`%d` becomes `*`, and a literal ending in
`/` or followed by `+` gets a trailing `*`. Every file matching the glob is
reachable, and the family is reported.

Derived names (one rule, from `scripts/world/art_sprite_library.gd`
`_regions_for_page`): a reachable `X.png` also reaches `X_regions.png`.

A reachable file's `.uid` / `.import` companions are reachable too.

usage: gd_reach.py [ROOT] [--json OUT.json] [--seed REL ...]
       list the reachable files (tracked + untracked non-ignored)
"""
import json
import re
import subprocess
import sys
from pathlib import Path

RES_RE = re.compile(r'"\*?(res://[^"]*)"')
DYNAMIC_RE = re.compile(r'"(res://[^"]*)"\s*(?:%|\+)')
CLASS_NAME_RE = re.compile(r"^\s*class_name\s+(\w+)", re.M)
WORD_RE = re.compile(r"\b[A-Z]\w*\b")
DERIVED_SUFFIX = "_regions.png"
TEXT_SUFFIXES = {".gd", ".tscn", ".tres", ".gdshader", ".gdshaderinc", ".json", ".cfg", ".godot", ".gdextension"}


def strip_gd_comments(text):
    """Drop GDScript comments: whole-line `#` lines and a trailing `# …` outside strings."""
    out = []
    for line in text.splitlines():
        if line.lstrip().startswith("#"):
            continue
        quote = None
        for i, ch in enumerate(line):
            if quote:
                if ch == quote and line[i - 1] != "\\":
                    quote = None
            elif ch in "\"'":
                quote = ch
            elif ch == "#":
                line = line[:i]
                break
        out.append(line)
    return "\n".join(out)


def tracked_files(root):
    """Tracked files (git) plus untracked non-ignored files; plain walk outside git."""
    try:
        listed = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard"], cwd=root,
                                capture_output=True, text=True, check=True).stdout.splitlines()
        return sorted({p for p in listed if (Path(root) / p).is_file()})
    except (subprocess.CalledProcessError, FileNotFoundError):
        return sorted(str(p.relative_to(root)) for p in Path(root).rglob("*") if p.is_file() and ".git" not in p.parts)


def class_index(root, files):
    """Map class_name -> declaring .gd file."""
    index = {}
    for rel in files:
        if rel.endswith(".gd"):
            for name in CLASS_NAME_RE.findall((Path(root) / rel).read_text(errors="replace")):
                index[name] = rel
    return index


def family_glob(literal, concatenated):
    """Glob (relative to the root) for a formatted or concatenated res:// literal."""
    glob = re.sub(r"%[sd]", "*", res_to_rel(literal))
    if glob.endswith("/") or concatenated:
        glob += "*"
    return glob


def glob_regex(glob):
    return re.compile("^" + "[^/]*".join(re.escape(part) for part in glob.split("*")) + "$")


def res_to_rel(path):
    return path[len("res://"):]


def references(root, rel, classes):
    """Return (static res paths, dynamic prefixes, class files) referenced by one file."""
    path = Path(root) / rel
    if path.suffix not in TEXT_SUFFIXES or not path.is_file():
        return set(), set(), set()
    text = path.read_text(errors="replace")
    if path.suffix == ".gd":
        text = strip_gd_comments(text)
    dynamic = set()
    for match in DYNAMIC_RE.finditer(text):
        dynamic.add(family_glob(match.group(1), match.group(0).rstrip().endswith("+")))
    static = set()
    for literal in RES_RE.findall(text):
        if "%" in literal or literal.endswith("/"):
            dynamic.add(family_glob(literal, False))
        elif literal != "res://":
            static.add(res_to_rel(literal))
    used = set()
    if path.suffix == ".gd":
        own = set(CLASS_NAME_RE.findall(text))
        for word in set(WORD_RE.findall(text)) - own:
            if word in classes:
                used.add(classes[word])
    return static, dynamic, used


def reach(root, seeds=None):
    """Return a dict: reachable (sorted), families, missing references.

    `seeds` replaces the default start set (`project.godot` and every
    `.gdextension`) — used to compute the closure of a planned keep set.
    """
    root = str(root)
    files = tracked_files(root)
    file_set = set(files)
    classes = class_index(root, files)
    if seeds is None:
        seeds = {"project.godot"} | {f for f in files if f.endswith(".gdextension")}
    reachable, families, missing = set(), set(), set()
    queue = list(seeds)
    while queue:
        rel = queue.pop()
        if rel in reachable:
            continue
        reachable.add(rel)
        static, dynamic, used = references(root, rel, classes)
        for glob in dynamic:
            families.add(glob)
            pattern = glob_regex(glob)
            queue.extend(f for f in files if pattern.match(f))
        for target in static | used:
            if target in file_set:
                queue.append(target)
            elif not (Path(root) / target).exists():
                missing.add(f"{rel} -> {target}")
        if rel.endswith(".png"):
            derived = rel[:-4] + DERIVED_SUFFIX
            if derived in file_set:
                queue.append(derived)
        for companion in (rel + ".uid", rel + ".import"):
            if companion in file_set:
                queue.append(companion)
    return {"reachable": sorted(reachable), "families": sorted(families), "missing": sorted(missing)}


def main(argv):
    args = [a for a in argv[1:]]
    out = None
    if "--json" in args:
        out = args[args.index("--json") + 1]
        del args[args.index("--json"):args.index("--json") + 2]
    seeds = None
    while "--seed" in args:
        i = args.index("--seed")
        seeds = (seeds or set()) | {args[i + 1]}
        del args[i:i + 2]
    root = args[0] if args else "."
    result = reach(root, seeds)
    if out:
        Path(out).write_text(json.dumps(result, indent=1) + "\n")
    print("\n".join(result["reachable"]))
    for family in result["families"]:
        print(f"# dynamic family: res://{family}", file=sys.stderr)
    for line in result["missing"]:
        print(f"# missing: {line}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
