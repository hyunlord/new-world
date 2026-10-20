#!/usr/bin/env python3
"""Localization keys the kept game can show (V8-0 goal 6.1 D, 7.4).

A key is used when
- a string literal equal to it appears in a reachable `.gd` file
  (`gd_reach.py`; comments stripped), or
- it belongs to a *family* built at runtime in a reachable `.gd` file: a
  literal containing `%s`/`%d` (`"CHRONICLE_KIND_%s" % k`) is a pattern, and a
  literal ending in `_` that is concatenated (`"PREFIX_" + x` / `x + "_SUFFIX"`)
  is a prefix (or suffix) family; every key matching is kept, or
- a string literal equal to it appears in the simulation or bridge Rust
  sources (`rust/crates/{sim-core,sim-systems,sim-engine,sim-bridge}/src`),
  because the bridge sends such keys to GDScript.

The key universe is `localization/compiled/<default>.json` `strings`.

usage: locale_keys.py [ROOT] [--json OUT.json] [--seed REL ...]
       print used keys and families (`--seed` as in gd_reach.py)
"""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gd_reach  # noqa: E402

LITERAL_RE = re.compile(r'"((?:[^"\\]|\\.)*)"')
CONCAT_RE = re.compile(r'"([A-Z0-9_]+_)"\s*\+|\+\s*"(_[A-Z0-9_]+)"')
RUST_DIRS = ("sim-core/src", "sim-systems/src", "sim-engine/src", "sim-bridge/src")


def key_universe(root):
    manifest = json.loads(Path(root, "localization/manifest.json").read_text())
    default = manifest.get("default_locale", "ko")
    return set(json.loads(Path(root, f"localization/compiled/{default}.json").read_text())["strings"])


def families_in(text):
    """Return regexes of runtime-built key families in one GDScript text."""
    patterns = set()
    for literal in LITERAL_RE.findall(text):
        if re.fullmatch(r"[A-Z0-9_]*%[sd][A-Z0-9_%sd]*", literal) and re.search(r"[A-Z]{2}", literal):
            patterns.add("^" + re.sub(r"%[sd]", "[A-Za-z0-9_]+", re.escape(literal).replace("%", "%")) + "$")
    for prefix, suffix in CONCAT_RE.findall(text):
        if prefix:
            patterns.add("^" + re.escape(prefix) + "[A-Za-z0-9_]+$")
        if suffix:
            patterns.add("^[A-Za-z0-9_]+" + re.escape(suffix) + "$")
    return patterns


def used_keys(root, seeds=None):
    root = Path(root)
    keys = key_universe(root)
    reach = gd_reach.reach(root, seeds)["reachable"]
    literal_hits, families = set(), set()
    for rel in reach:
        if not rel.endswith(".gd"):
            continue
        text = gd_reach.strip_gd_comments((root / rel).read_text(errors="replace"))
        literal_hits |= keys & set(LITERAL_RE.findall(text))
        families |= families_in(text)
    family_hits = {k for k in keys for f in families if re.match(f, k)}
    rust_hits = set()
    for sub in RUST_DIRS:
        for path in sorted((root / "rust/crates").joinpath(sub).rglob("*.rs")):
            rust_hits |= keys & set(LITERAL_RE.findall(path.read_text(errors="replace")))
    return {
        "universe": len(keys),
        "used": sorted(literal_hits | family_hits | rust_hits),
        "gdscript_literal": len(literal_hits),
        "family_patterns": sorted(families),
        "family_keys": len(family_hits),
        "rust_literal": len(rust_hits),
    }


def main(argv):
    args = argv[1:]
    out = None
    if "--json" in args:
        out = args[args.index("--json") + 1]
        del args[args.index("--json"):args.index("--json") + 2]
    seeds = None
    while "--seed" in args:
        i = args.index("--seed")
        seeds = (seeds or set()) | {args[i + 1]}
        del args[i:i + 2]
    result = used_keys(args[0] if args else ".", seeds)
    if out:
        Path(out).write_text(json.dumps(result, indent=1, ensure_ascii=False) + "\n")
    print(f"universe {result['universe']} used {len(result['used'])} "
          f"(gdscript literals {result['gdscript_literal']}, family keys {result['family_keys']}, rust literals {result['rust_literal']})")
    for pattern in result["family_patterns"]:
        print(f"family {pattern}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
