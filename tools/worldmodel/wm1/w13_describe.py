"""What the WM-1.3 worlds look like (GOAL_1_3.md 6.4): per scene, eta2 and R2_abs of the 8 V on the test split with
Pass 11's `tools/revitalize/diversity_p11.py` `scene_block` (fit on the train role, score on test; read-only use), the
dyn-eta2 and settlements alive of that block, and for every scene the share of worlds with >= 1 fission by day 120,
the first-fission display day distribution and the settlements alive at the last row (from `global.npy`'s observable
`fissions_total` / `settlements_alive`; labels for description only). numpy only.

usage (DGX): python3 tools/worldmodel/wm1/w13_describe.py --root WM_DATA_ROOT --out DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve()
sys.path.insert(0, str(HERE.parents[1]))
sys.path.insert(0, str(HERE.parents[2] / "revitalize"))

import numpy as np

import wm1  # noqa: F401
from wm_data import load_episode
from wm1 import common as c
from wm1 import probe_world as pw
from wm1.w13_manifest import ROLES

import diversity_p11 as p11  # noqa: E402  (Pass 11's module, read-only)


def role_range(scene: str, role: str) -> range:
    return dict(ROLES[scene])[role]


def fission_block(root: Path, scene: str) -> dict:
    out = {}
    for role, seeds in ROLES[scene]:
        first, alive_last, alive_2832, total = [], [], [], []
        for seed in seeds:
            ep = load_episode(root / "w13" / scene / f"{scene}_s{seed}_none", arrays=["global.npy"])
            names = [f["name"] for f in ep.meta["fields"]["global.npy"]]
            if "fissions_total" not in names:
                return {"recorded": False}
            fis = ep.column("global.npy", "fissions_total").astype(np.float64)
            alive = ep.column("global.npy", "settlements_alive").astype(np.float64)
            first.append(pw.first_split_day(fis))
            total.append(float(fis[-1]))
            alive_last.append(float(alive[-1]))
            alive_2832.append(float(alive[pw.SETTLEMENTS_ROW]))
        first = np.array(first)
        split = np.isfinite(first)
        hist = lambda a: {str(int(k)): int(v) for k, v in zip(*np.unique(np.asarray(a), return_counts=True))}
        out[role] = {"worlds": len(seeds), "with_fission": int(split.sum()), "share_with_fission": float(split.mean()),
                     "first_fission_day": hist(first[split]) if split.any() else {},
                     "first_fission_day_variance": float(first[split].var()) if split.any() else 0.0,
                     "fissions_total_last_row": hist(total), "settlements_alive_last_row": hist(alive_last),
                     "settlements_alive_row2832": hist(alive_2832),
                     "settlements_alive_row2832_variance": float(np.var(alive_2832))}
    out["recorded"] = True
    return out


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    t0 = time.time()
    root = Path(args.root)
    out = {"vars": list(p11.NAMES), "scenes": {}, "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha("w13_v16_n400")}
    for scene in ROLES:
        block = p11.scene_block(root / "w13", scene, role_range(scene, "train"), role_range(scene, "test"))
        keep = {k: block[k] for k in ("eta2", "r2_abs", "dyn_eta2", "settlements_alive", "deaths_mean_by_day120", "schema")
                if k in block}
        out["scenes"][scene] = {"diversity_p11": keep, "fission": fission_block(root, scene),
                                "train_worlds": len(role_range(scene, "train")), "test_worlds": len(role_range(scene, "test"))}
        print(scene, {k: round(v, 3) if v is not None else None for k, v in keep["eta2"].items()},
              f"{time.time() - t0:.0f}s", flush=True)
    out["seconds"] = time.time() - t0
    target = Path(args.out) / "describe.json"
    c.write_json(target, out)
    print(f"done {out['seconds']:.0f}s -> {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
