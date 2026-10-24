#!/usr/bin/env python3
"""Resolver parity (M-ART2 §3.4; gate step `resolver_parity`).

usage: resolver_parity.py --generate            # expected results from the art line's own resolve.py
       resolver_parity.py --check <godot.json>  # compare the GDScript resolver's output with them

`--generate` extracts `art/materials/` at the pinned art commit (`tools/v8/art_import.json` source_commit) with
`git archive` into a temporary folder, runs that commit's `art/materials/resolve.py` (pydantic) on every case of
`tools/v8/resolver_parity_fixture.json`, and writes `tools/v8/resolver_parity_expected.json`. It also asserts that
the three JSON documents it resolved against are byte-identical to the imported `assets/v8art/materials/` copies,
so the expectation belongs to the data the game loads. `--check` compares `scripts/world/art/material_resolver.gd`'s
results (written by `tools/v8/resolver_parity.gd`) field by field and exits 1 on any difference.
"""
from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tools/v8/resolver_parity_fixture.json"
EXPECTED = ROOT / "tools/v8/resolver_parity_expected.json"
FIELDS = ("style_id", "material_key", "category", "kind", "textures", "tint", "tint_source", "style_source", "top",
          "face")
DOCS = ("construction_styles.json", "material_looks.json", "category_defaults.json")


def normalise(value: object) -> object:
    """Numbers as floats, no None entries in dicts (pydantic dumps unset roles as null)."""
    if isinstance(value, bool) or value is None or isinstance(value, str):
        return value
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, dict):
        return {k: normalise(v) for k, v in sorted(value.items()) if v is not None}
    if isinstance(value, list):
        return [normalise(v) for v in value]
    return value


def generate() -> int:
    commit = json.loads((ROOT / "tools/v8/art_import.json").read_text())["source_commit"]
    archive = subprocess.run(["git", "-C", str(ROOT), "archive", commit, "art/materials"], capture_output=True,
                             check=True).stdout
    cases = json.loads(FIXTURE.read_text())["cases"]
    with tempfile.TemporaryDirectory() as tmp:
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(tmp, filter="data")
        for doc in DOCS:
            art = (Path(tmp) / "art/materials" / doc).read_bytes()
            imported = (ROOT / "assets/v8art/materials" / doc).read_bytes()
            if art != imported:
                print(f"resolver_parity: {doc} at {commit[:8]} differs from the imported copy", file=sys.stderr)
                return 1
        sys.path.insert(0, tmp)
        from art.materials.resolve import read_catalog, resolve  # type: ignore[import-not-found]

        catalog = read_catalog(Path(tmp))
        results = []
        for case in cases:
            r = resolve(catalog, case["style"], case["material"], category=case.get("category"),
                        kind=case.get("kind", "wall"))
            dumped = r.model_dump(mode="json")
            results.append({"case": case, "result": {k: dumped[k] for k in FIELDS}})
    EXPECTED.write_text(json.dumps({"source_commit": commit, "generated_by": "tools/v8/resolver_parity.py",
                                    "results": results}, indent=1, sort_keys=True) + "\n")
    print(f"resolver_parity: {len(results)} expected results from {commit[:8]}")
    return 0


def check(godot_path: Path) -> int:
    expected = json.loads(EXPECTED.read_text())["results"]
    got = json.loads(godot_path.read_text())
    if got.get("problems"):
        print(f"resolver_parity: FAIL loader problems {got['problems']}")
        return 1
    rows = got.get("results", [])
    bad = 0
    if len(rows) != len(expected) or not expected:
        print(f"resolver_parity: FAIL {len(rows)} results for {len(expected)} cases")
        return 1
    for exp, row in zip(expected, rows):
        for field in FIELDS:
            a, b = normalise(exp["result"][field]), normalise(row["result"].get(field))
            if a != b:
                bad += 1
                print(f"resolver_parity: case {exp['case']} field {field}: expected {a}, got {b}")
    print(f"resolver_parity: {'PASS' if bad == 0 else 'FAIL'} ({len(expected)} cases, {bad} differences)")
    return 1 if bad else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generate", action="store_true")
    parser.add_argument("--check")
    args = parser.parse_args()
    if args.generate:
        return generate()
    if args.check:
        return check(Path(args.check))
    parser.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
