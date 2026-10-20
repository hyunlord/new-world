"""WM-2 Part 0.3 (GOAL.md 5.3): the recorder built at the merge commit reproduces Pass 14's Stage A episodes.

For every episode directory of `--new` (the identity recording), the directory of the same name under `--ref`
(`worldsim-wm-data/p14/stageA/`) must exist and carry: the same set of `.npy` files with equal sha256 (computed from
the files, not read from meta.json), an equal `interventions.json` (parsed), and a `meta.json` equal in every key but
`git_sha`. Stdlib only.

usage: python3 tools/worldmodel/wm2/identity.py --new DIR --ref DIR --out OUT.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

MAY_DIFFER = ("git_sha",)


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def compare_episode(new: Path, ref: Path) -> dict:
    """One episode: the per-file verdicts and the list of differences (empty = identical)."""
    diffs: list[str] = []
    if not ref.is_dir():
        return {"episode": new.name, "identical": False, "diffs": ["missing in the reference"], "npy": 0}
    new_npy = sorted(p.name for p in new.glob("*.npy"))
    ref_npy = sorted(p.name for p in ref.glob("*.npy"))
    if new_npy != ref_npy:
        diffs.append(f"npy file sets differ: {sorted(set(new_npy) ^ set(ref_npy))}")
    for name in sorted(set(new_npy) & set(ref_npy)):
        if sha256(new / name) != sha256(ref / name):
            diffs.append(f"{name}: sha256 differs")
    if json.loads((new / "interventions.json").read_text()) != json.loads((ref / "interventions.json").read_text()):
        diffs.append("interventions.json differs")
    mn, mr = json.loads((new / "meta.json").read_text()), json.loads((ref / "meta.json").read_text())
    for key in sorted(set(mn) | set(mr)):
        if key not in MAY_DIFFER and mn.get(key) != mr.get(key):
            diffs.append(f"meta.json {key} differs")
    return {"episode": new.name, "identical": not diffs, "diffs": diffs, "npy": len(new_npy),
            "git_sha": [mn.get("git_sha"), mr.get("git_sha")]}


def compare(new_root: Path, ref_root: Path) -> dict:
    eps = sorted(p for p in new_root.iterdir() if (p / "meta.json").exists())
    rows = [compare_episode(p, ref_root / p.name) for p in eps]
    return {"new": str(new_root), "ref": str(ref_root), "episodes": len(rows),
            "identical": sum(r["identical"] for r in rows), "all_identical": bool(rows) and all(r["identical"] for r in rows),
            "npy_files_compared": sum(r["npy"] for r in rows), "rows": rows}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--new", required=True)
    ap.add_argument("--ref", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    out = compare(Path(a.new), Path(a.ref))
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps({k: out[k] for k in ("episodes", "identical", "all_identical", "npy_files_compared")}))
    return 0 if out["all_identical"] else 1


if __name__ == "__main__":
    sys.exit(main())
