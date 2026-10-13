"""Recorder equivalence (GOAL_1_3.md 6.1): compare episodes recorded by the WM-1.3 binary with their p10 twins.

For every array file present in both episodes: arrays with named fields (`meta["fields"]`) are compared column by
column, matched by name (the other axes must have the same shape); arrays without fields are compared whole. Integers
must be equal, floats bit-identical (the raw bytes, so NaN == NaN and -0.0 != 0.0). Columns or files only one
recording has are listed, not compared. numpy only.

usage: python3 tools/worldmodel/wm1/recorder_equiv.py --new DIR --old DIR [--out JSON]
       (DIR holds `<scene>_s<seed>_none` episode directories; every episode of --new is compared)
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
from wm_data import load_episode


def identical(a: np.ndarray, b: np.ndarray) -> bool:
    """Exact for integers and booleans, bit-identical for floats (same dtype and shape required)."""
    if a.dtype != b.dtype or a.shape != b.shape:
        return False
    return np.ascontiguousarray(a).tobytes() == np.ascontiguousarray(b).tobytes()


def fields_of(ep, name: str) -> list[str] | None:
    entry = ep.meta.get("fields", {}).get(name)
    return None if not entry else [f["name"] for f in entry]


def compare_episode(new_path: Path, old_path: Path) -> dict:
    """One episode pair: the compared columns, the differing ones and what only one side has."""
    new, old = load_episode(new_path), load_episode(old_path)
    out = {"episode": new_path.name, "differ": [], "compared": 0, "only_new": [], "only_old": [],
           "schema_new": new.meta.get("schema_version"), "schema_old": old.meta.get("schema_version")}
    for name in sorted(set(new.arrays) | set(old.arrays)):
        if name not in old.arrays:
            out["only_new"].append(name)
            continue
        if name not in new.arrays:
            out["only_old"].append(name)
            continue
        a, b = new.arrays[name], old.arrays[name]
        fa, fb = fields_of(new, name), fields_of(old, name)
        if fa is None or fb is None:
            out["compared"] += 1
            if not identical(a, b):
                out["differ"].append(name)
            continue
        if a.shape[:-1] != b.shape[:-1]:
            out["differ"].append(f"{name} (shape {list(a.shape)} vs {list(b.shape)})")
            continue
        for col in fa:
            if col not in fb:
                out["only_new"].append(f"{name}:{col}")
                continue
            out["compared"] += 1
            if not identical(a[..., fa.index(col)], b[..., fb.index(col)]):
                out["differ"].append(f"{name}:{col}")
        out["only_old"] += [f"{name}:{col}" for col in fb if col not in fa]
    if new.interventions != old.interventions:
        out["differ"].append("interventions.json")
    return out


def compare_dirs(new_dir: Path, old_dir: Path) -> dict:
    pairs = sorted(p for p in Path(new_dir).iterdir() if (p / "meta.json").exists())
    if not pairs:
        raise SystemExit(f"{new_dir}: no episodes")
    episodes = []
    for p in pairs:
        twin = Path(old_dir) / p.name
        if not (twin / "meta.json").exists():
            raise SystemExit(f"{twin}: no p10 twin")
        episodes.append(compare_episode(p, twin))
    only_new = sorted({x for e in episodes for x in e["only_new"]})
    return {"episodes": episodes, "identical": all(not e["differ"] for e in episodes), "only_new": only_new,
            "only_old": sorted({x for e in episodes for x in e["only_old"]})}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--new", required=True)
    ap.add_argument("--old", required=True)
    ap.add_argument("--out")
    args = ap.parse_args(argv)
    res = compare_dirs(Path(args.new), Path(args.old))
    if args.out:
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
        Path(args.out).write_text(json.dumps(res, indent=1, sort_keys=True) + "\n")
    for e in res["episodes"]:
        print(e["episode"], f"schema {e['schema_old']} -> {e['schema_new']}", f"compared {e['compared']}",
              "differ:", e["differ"] or "none")
    print("only in the new recording:", res["only_new"])
    print("only in the old recording:", res["only_old"])
    print("IDENTICAL" if res["identical"] else "DIFFERENT")
    return 0 if res["identical"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
