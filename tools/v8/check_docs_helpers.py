#!/usr/bin/env python3
"""Helpers for tools/v8/check_docs.sh (kept out of bash heredocs, which bash 3.2
mis-parses inside command substitutions when they contain backticks).

usage: check_docs_helpers.py arch-paths          paths named in docs/ARCHITECTURE.md that do not exist
       check_docs_helpers.py lessons SKILL...   per-skill lesson counts and unsourced rows
       check_docs_helpers.py uid                 Godot UID problems (M1 5.3): a tracked .gd/.gdshader
                                                 outside a .gdignore'd folder without its tracked .uid,
                                                 a tracked .uid without a tracked source, a tracked
                                                 .tscn whose header has no uid
       check_docs_helpers.py schema [ROOT] [--list]
                                                 M1.5 goal 8.1: docs/v8/snapshot_schema.md names every
                                                 `get_*` #[func] of rust/crates/sim-bridge (bridge_api.py's
                                                 extraction) and exactly the dictionary keys each one sets
                                                 (its own `dict.set("k", ...)` calls and those of the
                                                 `*_to_dict` / `*_to_variant_array` builders it calls in
                                                 src/ffi/*.rs); prints `schema: PASS ...` or the absent and
                                                 extra names and exits 1. `--list` prints the extraction.
"""
import re
import subprocess
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


def uid(argv):
    tracked = set(subprocess.run(["git", "ls-files", "-z"], capture_output=True, check=True).stdout.decode().split("\0"))
    tracked.discard("")

    def ignored_by_godot(path):
        parent = Path(path).parent
        while str(parent) not in ("", "."):
            if (parent / ".gdignore").exists():
                return True
            parent = parent.parent
        return False

    bad = []
    for path in sorted(tracked):
        if path.endswith((".gd", ".gdshader")) and not ignored_by_godot(path) and path + ".uid" not in tracked:
            bad.append(f"{path}: no tracked .uid")
        elif path.endswith(".uid") and path[: -len(".uid")] not in tracked:
            bad.append(f"{path}: source is not tracked")
        elif path.endswith(".tscn") and "uid=" not in Path(path).read_text().split("\n", 1)[0]:
            bad.append(f"{path}: header has no uid")
    print("\n".join(bad))


SCHEMA_DOC = "docs/v8/snapshot_schema.md"
FFI_DIR = "rust/crates/sim-bridge/src/ffi"
# A builder whose name ends like this is followed from a getter's body.
BUILDER_RE = re.compile(r"\b(\w+(?:_to_dict|_to_variant_array|_to_owned_dict))\s*\(")
KEY_SET_RE = re.compile(r"\.(?:set|insert)\(\s*\"(\w+)\"")
# `dict.set(key, ...)` with a computed key: the key comes from a literal list in the same body
# (tuples `("k", ...)`, arrays `["k", ...]`, match arms `"k" =>`) or from a function returning literals.
DYNAMIC_SET_RE = re.compile(r"\.set\(\s*(?!\")([A-Za-z_]\w*)\s*([(,])")
TUPLE_KEY_RE = re.compile(r"\(\s*\"(\w+)\"\s*,")
ARRAY_RE = re.compile(r"\[\s*\"\w+\"(?:\s*,\s*\"\w+\")*\s*,?\s*\]")
ARM_KEY_RE = re.compile(r"\"(\w+)\"\s*=>")
RETURN_ARM_RE = re.compile(r"=>\s*\"(\w+)\"")
FN_RE = re.compile(r"\bfn\s+(\w+)\s*(?:<[^>]*>)?\s*\(")
SCHEMA_HEADING_RE = re.compile(r"^###\s+`(get_\w+)`")
SCHEMA_KEY_RE = re.compile(r"^\|\s*`(\w+)`")


def _strip_tests(text):
    """Drop `#[cfg(test)]` modules (their helpers are not the bridge's)."""
    import bridge_api
    out, pos = [], 0
    for m in re.finditer(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{", text):
        if m.start() < pos:
            continue
        out.append(text[pos:m.start()])
        pos = bridge_api.block_end(text, m.start())
    out.append(text[pos:])
    return "".join(out)


def _ffi_functions(root):
    """Every non-test `fn` in src/ffi/*.rs: name -> (parameter names, body text)."""
    import bridge_api
    fns = {}
    for path in sorted(Path(root, FFI_DIR).glob("*.rs")):
        text = _strip_tests(path.read_text())
        for m in FN_RE.finditer(text):
            try:
                end = bridge_api.block_end(text, m.end())
            except ValueError:
                continue
            brace = text.find("{", m.end())
            semi = text.find(";", m.end())
            if semi != -1 and semi < brace:
                continue  # a declaration without a body
            params = re.findall(r"(\w+)\s*:", _balanced(text, m.end() - 1))
            fns.setdefault(m.group(1), (params, text[brace:end]))
    return fns


def _balanced(text, open_at):
    """The text inside the parenthesis group that opens at `open_at`."""
    depth = 0
    for i in range(open_at, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return text[open_at + 1:i]
    return ""


def _split_args(args):
    """Top-level comma-separated arguments of a call."""
    out, depth, cur = [], 0, ""
    for ch in args:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def _drop_false_branches(body, params, args):
    """Remove `if <param> { ... }` blocks of `body` whose parameter is passed a literal `false`."""
    import bridge_api
    for param, arg in zip(params, args):
        if arg != "false":
            continue
        while True:
            m = re.search(r"\bif\s+" + re.escape(param) + r"\s*\{", body)
            if m is None:
                break
            body = body[:m.start()] + body[bridge_api.block_end(body, m.start()):]
    return body


def _getter_bodies(root):
    """`get_*` #[func] methods of the bridge: name -> (return type, body)."""
    import bridge_api
    out = {}
    for path in sorted(Path(root, "rust/crates/sim-bridge/src").rglob("*.rs")):
        text = _strip_tests(path.read_text())
        for impl in bridge_api.IMPL_RE.finditer(text):
            if impl.group(1) is not None:
                continue
            body = text[impl.end():bridge_api.block_end(text, impl.end())]
            for mark in bridge_api.MARK_RE.finditer(body):
                if mark.group(1) != "func":
                    continue
                item = bridge_api.ITEM_RE.match(body, mark.end())
                if item is None or not item.group(1).startswith("get_"):
                    continue
                brace = body.index("{", item.end())
                ret = re.search(r"->\s*([\w:<>]+)", body[item.end():brace])
                out[item.group(1)] = (ret.group(1) if ret else "()", body[brace:bridge_api.block_end(body, item.end())])
    return out


def _keys_of(body, fns, seen):
    keys = set(KEY_SET_RE.findall(body))
    for name, follow in DYNAMIC_SET_RE.findall(body):
        if follow == "(" and name in fns:
            keys |= set(RETURN_ARM_RE.findall(fns[name][1]))
        else:
            keys |= set(TUPLE_KEY_RE.findall(body)) | set(ARM_KEY_RE.findall(body))
            for array in ARRAY_RE.findall(body):
                keys |= set(re.findall(r"\"(\w+)\"", array))
    for call in BUILDER_RE.finditer(body):
        name = call.group(1)
        if name not in fns:
            continue
        params, builder = fns[name]
        args = _split_args(_balanced(body, call.end() - 1))
        key = (name, tuple(a == "false" for a in args))
        if key in seen:
            continue
        seen.add(key)
        keys |= _keys_of(_drop_false_branches(builder, params, args), fns, seen)
    return keys


def schema_extract(root):
    """getter -> (return type, sorted keys) for every `get_*` #[func]; plus problems."""
    import bridge_api
    fns = _ffi_functions(root)
    bodies = _getter_bodies(root)
    listed = {line.split(".", 1)[1] for line in bridge_api.scan_repo(root)
              if line.startswith("func ") and line.split(".", 1)[1].startswith("get_")}
    problems = [f"bridge_api lists {name} but its body was not found" for name in sorted(listed - set(bodies))]
    out = {}
    for name in sorted(listed & set(bodies)):
        ret, body = bodies[name]
        out[name] = (ret, sorted(_keys_of(body, fns, set())))
    return out, problems


def schema_documented(text):
    """getter -> set of keys named in the first table column of its `### get_x` section (name in backticks)."""
    doc, current = {}, None
    for line in text.splitlines():
        heading = SCHEMA_HEADING_RE.match(line)
        if heading:
            current = heading.group(1)
            doc.setdefault(current, set())
        elif line.startswith("#"):
            current = None
        elif current is not None:
            key = SCHEMA_KEY_RE.match(line)
            if key:
                doc[current].add(key.group(1))
    return doc


def schema_problems(root):
    """Every absent or extra getter or key of the schema doc against the bridge source."""
    extracted, problems = schema_extract(root)
    path = Path(root, SCHEMA_DOC)
    if not path.exists():
        return extracted, problems + [f"{SCHEMA_DOC} does not exist"]
    doc = schema_documented(path.read_text())
    for name in sorted(set(extracted) - set(doc)):
        problems.append(f"absent getter: {name}")
    for name in sorted(set(doc) - set(extracted)):
        problems.append(f"extra getter: {name} (no such get_* #[func])")
    for name in sorted(set(extracted) & set(doc)):
        keys = set(extracted[name][1])
        for key in sorted(keys - doc[name]):
            problems.append(f"absent key: {name}.{key}")
        for key in sorted(doc[name] - keys):
            problems.append(f"extra key: {name}.{key} (not set by the bridge)")
    return extracted, problems


def schema(argv):
    root = next((a for a in argv if not a.startswith("--")), ".")
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    extracted, problems = schema_problems(root)
    if "--list" in argv:
        for name, (ret, keys) in extracted.items():
            print(f"{name} -> {ret}: {' '.join(keys) if keys else '(no keys)'}")
        return
    keys = sum(len(k) for _, k in extracted.values())
    if problems:
        for line in problems:
            print(f"schema: {line}")
        print(f"schema: FAIL ({len(problems)} problems; {len(extracted)} getters, {keys} keys in the source)")
        sys.exit(1)
    print(f"schema: PASS ({len(extracted)} getters, {keys} keys documented in {SCHEMA_DOC})")


def main(argv):
    commands = {"arch-paths": arch_paths, "lessons": lessons, "uid": uid, "schema": schema}
    if not argv or argv[0] not in commands:
        sys.exit(__doc__)
    commands[argv[0]](argv[1:])


if __name__ == "__main__":
    main(sys.argv[1:])
