"""Diagnostic (post hoc, validation worlds only; no verdict): what the trained models' rollouts do with
`g.fissions_total`, the state column the split event is read from (GOAL.md 7, the split shift and the split-timing
task). From the split-timing start (tick 34,440, row 574) of every eligible validation world (replicate 0 of `none`,
no split yet), each model rolls to the last row; reported: the largest rise of the rolled-out `fissions_total` above
its start value, the mean per-hour change, and the same quantities of the truth (how many worlds split, the base rate
of a split per hour). A rollout that never rises by 0.5 never predicts a split.

usage (DGX): python3 tools/worldmodel/wm2/diag_fission.py --run RUN [RUN ...] --out JSON
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
from wm1 import common as c
from wm1 import m1_gru as g
from wm2 import rollout_ext as rx
from wm2 import split_task as st
from wm2 import v2


def q(a) -> dict:
    a = np.asarray(a, dtype=np.float64)
    return {"mean": float(a.mean()), "p10": float(np.percentile(a, 10)), "median": float(np.median(a)),
            "p90": float(np.percentile(a, 90)), "max": float(a.max()), "n": int(len(a))}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--run", nargs="+", required=True)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    val = st.eligible(c.load_groups(["val"], Path(a.root), "v2_v19_fact")["val"])
    H = v2.LAST_ROW - st.START_ROW
    truth_rise = [st.fissions(ep)[-1] - st.fissions(ep)[st.START_ROW] for ep in val]
    out = {"worlds": len(val), "start_row": st.START_ROW, "horizon": H,
           "truth": {"rise_to_last_row": q(truth_rise), "share_split": float(np.mean(np.array(truth_rise) >= 0.5)),
                     "per_hour": float(np.sum(truth_rise) / (len(val) * H))}, "runs": {}}
    for run in a.run:
        pred, cfg = g.load_predictor(Path(run))
        ros = rx.rollout_jobs(pred, [rx.Job(ep, st.START_ROW, ep.bench.actions, H) for ep in val])
        rise = [float(ro.fissions_total[0].max() - st.fissions(ep)[st.START_ROW]) for ep, ro in zip(val, ros)]
        end = [float(ro.fissions_total[0][-1] - st.fissions(ep)[st.START_ROW]) for ep, ro in zip(val, ros)]
        sa = [float(ro.settlements_alive[0].max() - ep.table[st.START_ROW, ep.names.index("g.settlements_alive")])
              for ep, ro in zip(val, ros)]
        out["runs"][Path(run).name] = {"max_rise": q(rise), "end_rise": q(end), "per_hour": float(np.sum(end) / (len(val) * H)),
                                       "share_rise_ge_0.5": float(np.mean(np.array(rise) >= 0.5)),
                                       "settlements_alive_max_rise": q(sa), "dstd_fissions": float(
                                           pred.norm.dstd[list(pred.norm.continuous).index("g.fissions_total")])}
        print(Path(run).name, json.dumps(out["runs"][Path(run).name]), flush=True)
    out["code_commit"] = c.git_commit()
    c.write_json(Path(a.out), out)
    print(json.dumps(out["truth"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
