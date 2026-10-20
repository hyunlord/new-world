"""Diagnostic (post hoc, validation families only; no verdict): the predicted and true effect of each arm on the food
store, `alive` and cumulative deaths as a function of the hours since the cast (1, 2, 6, 24, 72, 168, 720), averaged
over the validation cases — how much of an intervention's immediate, mechanical effect (a spoil removes food from the
store at once) the models carry into their rollouts and how fast it fades.

usage (DGX): python3 tools/worldmodel/wm2/diag_effect_lag.py --run RUN [RUN ...] [--worlds N] --out JSON
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
import bench_v0 as b
from wm_data import load_family
from wm1 import common as c
from wm1 import data as d
from wm1 import m1_gru as g
from wm2 import effects as ef
from wm2 import rollout_ext as rx
from wm2 import v2

LAGS = (1, 2, 6, 24, 72, 168, 720)
OUT = ("food_stock", "alive", "deaths")


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--run", nargs="+", required=True)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--worlds", type=int, default=40)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    fams = [load_family(p, arrays=ef.ARRAYS) for p in ef.family_dirs(Path(a.root), "val")[: a.worlds]]
    truth: dict = {}
    jobs, keys = [], []
    for fam in fams:
        for r, raw in fam.factual.items():
            ep = d.from_episode(raw, "v19_fam", "val", fam.seed, "none", c.V17_EXTRA)
            none_s = v2.member_series(raw)
            zeros = np.zeros_like(ep.bench.actions)
            for arm, reps in fam.variants.items():
                arm_raw = reps[r]
                row = v2.row_of_tick(ef.cast_of(arm_raw))
                arm_s = v2.member_series(arm_raw)
                act = zeros.copy()
                kind, amount = v2.ARM_SPEC[arm]
                act[row, b.KINDS.index(kind)] = float(amount)
                h = max(LAGS)
                jobs += [rx.Job(ep, row, zeros, h), rx.Job(ep, row, act, h)]
                keys.append((arm, fam.seed, r))
                truth.setdefault(arm, []).append({o: [arm_s[o][row + L] - none_s[o][row + L] for L in LAGS] for o in OUT})
    out = {"lags": list(LAGS), "worlds": len(fams), "truth": {}, "runs": {}, "code_commit": c.git_commit()}
    for arm, rows in truth.items():
        out["truth"][arm] = {o: np.mean([x[o] for x in rows], axis=0).tolist() for o in OUT} | {"n": len(rows)}
    for run in a.run:
        pred, _ = g.load_predictor(Path(run))
        ros = rx.rollout_jobs(pred, jobs)
        per: dict = {}
        for i, (arm, _, _) in enumerate(keys):
            fa, ar = ros[2 * i], ros[2 * i + 1]
            get = {"food_stock": lambda ro: ro.v[0, :, ef.V_FOOD], "alive": lambda ro: ro.v[0, :, ef.V_ALIVE],
                   "deaths": lambda ro: ro.deaths[0]}
            per.setdefault(arm, []).append({o: [float(get[o](ar)[L - 1] - get[o](fa)[L - 1]) for L in LAGS] for o in OUT})
        out["runs"][Path(run).name] = {arm: {o: np.mean([x[o] for x in rows], axis=0).tolist() for o in OUT}
                                       for arm, rows in per.items()}
        print(Path(run).name, json.dumps({arm: [round(x, 1) for x in v["food_stock"]] for arm, v in out["runs"][Path(run).name].items()}), flush=True)
    print("truth", json.dumps({arm: [round(x, 1) for x in v["food_stock"]] for arm, v in out["truth"].items()}))
    c.write_json(Path(a.out), out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
