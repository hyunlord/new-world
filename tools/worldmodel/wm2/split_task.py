"""The split-timing task (GOAL.md 7.7, Q3): from display day 25, 06:00 (tick 34,440, row 574), when does a village
that has not split yet split first?

Worlds: replicate 0 of `none` (`v2_v19_fact`) with `g.fissions_total` = 0 at the start row. Target: the restricted split
day (reference = the start row, censored at the last row's day; `v2.restricted_split_day`). Predictors:

- the models (M1-long, M1b-act): a rollout from the start row to the last row with the factual actions (none); the
  split event of the rolled-out `fissions_total` (`wm2.rollout_ext`);
- `median_day`: the median restricted split day of the eligible train worlds;
- `an5_day`: the mean restricted split day of the 5 eligible train worlds nearest by `an5`'s distance at the start row
  (the 24-row window of the 8 V, scaled by the train split std; every world is at the same calendar row, no shift);
- `ridge_day`: a ridge (`bench_v0.Ridge`, standardized features) from the same window (24 x 8, flattened) to the
  restricted split day, lambda from `bench_v0.LAMBDAS` selected on the eligible validation worlds (MAE).

Metric: MAE in days and the share within +-7 days; per-world absolute errors for the world-level bootstrap. The hidden
threshold theta (`settlements.npy` `fission_threshold` of the home settlement, the lowest id, at the start row) is read
only by `explain` after every prediction exists: leave-one-out ridge R^2 of each predictor's signed error from (a) the
window summary (mean of the 8 V over the window) and (b) theta plus the summary; theta's share = (b) - (a).

usage (DGX):
  python3 tools/worldmodel/wm2/split_task.py base --out DIR
  python3 tools/worldmodel/wm2/split_task.py model --run RUN --out DIR
  python3 tools/worldmodel/wm2/split_task.py explain --base DIR/split_base.json --model NAME=JSON ... --out DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
import bench_v0 as b
from wm1 import common as c
from wm1 import data as d
from wm2 import v2

START_ROW = v2.row_of_tick(v2.SPLIT_START_TICK)
WINDOW = 24
FT = "g.fissions_total"


def fissions(ep: d.EpisodeData) -> np.ndarray:
    return ep.table[:, ep.names.index(FT)]


def eligible(eps: list[d.EpisodeData]) -> list[d.EpisodeData]:
    return [ep for ep in eps if fissions(ep)[START_ROW] == 0]


def target(ep: d.EpisodeData) -> tuple[int, bool]:
    return v2.restricted_split_day(fissions(ep), START_ROW)


def window(ep: d.EpisodeData) -> np.ndarray:
    return ep.bench.v[START_ROW - WINDOW + 1: START_ROW + 1]


def an5_day(test: list[d.EpisodeData], train: list[d.EpisodeData], inv_std: np.ndarray) -> dict:
    """{seed: (prediction, neighbour seeds)}: mean target of the 5 nearest train worlds (stable ties)."""
    lib = np.stack([window(ep) for ep in train])
    days = np.array([target(ep)[0] for ep in train], dtype=np.float64)
    out = {}
    for ep in test:
        dist = (((lib - window(ep)[None]) * inv_std) ** 2).sum(axis=(1, 2))
        near = np.argsort(dist, kind="stable")[:5]
        out[ep.seed] = (float(days[near].mean()), [int(train[i].seed) for i in near])
    return out


def ridge_day(train: list[d.EpisodeData], val: list[d.EpisodeData]) -> tuple[b.Ridge, float, dict]:
    x = np.stack([window(ep).ravel() for ep in train])
    y = np.array([[target(ep)[0]] for ep in train], dtype=np.float64)
    xv = np.stack([window(ep).ravel() for ep in val])
    yv = np.array([target(ep)[0] for ep in val], dtype=np.float64)
    best, scores = None, {}
    for lam in b.LAMBDAS:
        m = b.Ridge().fit(x, y, lam)
        mae = float(np.abs(m.predict(xv)[:, 0] - yv).mean())
        scores[str(lam)] = mae
        if best is None or mae < best[0]:
            best = (mae, lam, m)
    return best[2], best[1], scores


def summarize(pred: dict, truth: dict) -> dict:
    seeds = sorted(truth)
    err = np.array([abs(pred[s] - truth[s]) for s in seeds])
    return {"mae": float(err.mean()), "within_7d": float(np.mean(err <= 7)), "n": len(seeds),
            "per_world": {str(s): float(abs(pred[s] - truth[s])) for s in seeds},
            "signed": {str(s): float(pred[s] - truth[s]) for s in seeds}}


def cmd_base(args) -> int:
    t0 = time.time()
    g = c.load_groups(["train", "val", "test"], Path(args.root), "v2_v19_fact")
    el = {k: eligible(v) for k, v in g.items()}
    truth = {k: {ep.seed: target(ep)[0] for ep in v} for k, v in el.items()}
    cens = {k: float(np.mean([target(ep)[1] for ep in v])) for k, v in el.items()}
    std = b.split_std([ep.bench for ep in g["train"]])
    inv_std = np.where(std > 0, 1.0 / np.where(std > 0, std, 1.0), 1.0)
    median = float(np.median([truth["train"][s] for s in truth["train"]]))
    ridge, lam, lam_scores = ridge_day(el["train"], el["val"])
    out = {"start_row": START_ROW, "start_tick": v2.SPLIT_START_TICK, "last_day": v2.LAST_DAY,
           "eligible": {k: [ep.seed for ep in v] for k, v in el.items()},
           "eligible_counts": {k: [len(v), len(g[k])] for k, v in el.items()}, "censored_share": cens,
           "truth": {k: {str(s): t for s, t in v.items()} for k, v in truth.items()},
           "ridge_lambda": lam, "ridge_val_mae": lam_scores, "median_day": median, "inv_std": inv_std.tolist(),
           "code_commit": c.git_commit(), "argv": sys.argv, "scores": {}}
    for role in ("val", "test"):
        preds = {"median_day": {ep.seed: median for ep in el[role]},
                 "an5_day": {s: v[0] for s, v in an5_day(el[role], el["train"], inv_std).items()},
                 "ridge_day": {ep.seed: float(ridge.predict(window(ep).ravel()[None])[0, 0]) for ep in el[role]}}
        out["scores"][role] = {n: summarize(p, truth[role]) | {"predictions": {str(s): x for s, x in p.items()}}
                               for n, p in preds.items()}
    out["selection"] = min(("median_day", "an5_day", "ridge_day"), key=lambda n: (out["scores"]["val"][n]["mae"],
                                                                                 ("median_day", "an5_day", "ridge_day").index(n)))
    out["seconds"] = time.time() - t0
    c.write_json(Path(args.out) / "split_base.json", out)
    print(json.dumps({"eligible": out["eligible_counts"], "censored": cens, "selection": out["selection"],
                      "val": {n: v["mae"] for n, v in out["scores"]["val"].items()},
                      "test": {n: v["mae"] for n, v in out["scores"]["test"].items()}}))
    return 0


def cmd_model(args) -> int:
    from wm1 import m1_gru as g
    from wm2 import rollout_ext as rx
    t0 = time.time()
    target_json = Path(args.out) / "split.json"
    if target_json.exists():
        raise SystemExit(f"{target_json} exists: the test split-timing prediction is computed once per run")
    pred, config = g.load_predictor(Path(args.run), args.device)
    gr = c.load_groups(["test"], Path(args.root), "v2_v19_fact")
    el = eligible(gr["test"])
    jobs = [rx.Job(ep, START_ROW, ep.bench.actions, v2.LAST_ROW - START_ROW) for ep in el]
    ros = rx.rollout_jobs(pred, jobs)
    preds, cens, truth = {}, {}, {}
    for ep, ro in zip(el, ros):
        f = fissions(ep)
        day, cz = v2.restricted_split_day(np.concatenate([f[: START_ROW + 1], ro.fissions_total[0]]), START_ROW)
        preds[ep.seed], cens[ep.seed] = float(day), cz
        truth[ep.seed] = target(ep)[0]
    out = {"run": args.run, "model": config["model"], "seed": config["seed"], "hyper": config["hyper"],
           "code_commit": c.git_commit(), "argv": sys.argv, "predictions": {str(s): v for s, v in preds.items()},
           "censored_pred_share": float(np.mean(list(cens.values()))), **summarize(preds, truth),
           "seconds": time.time() - t0}
    c.write_json(target_json, out)
    print(f"split done {time.time() - t0:.0f}s mae {out['mae']:.2f} within7 {out['within_7d']:.3f}")
    return 0


def loo_r2(x: np.ndarray, y: np.ndarray) -> dict:
    """Leave-one-out R^2 of a ridge (`bench_v0.Ridge`, lambda from `bench_v0.LAMBDAS` by the LOO error) of y on x."""
    n = len(y)
    best = None
    for lam in b.LAMBDAS:
        pred = np.array([b.Ridge().fit(np.delete(x, i, 0), np.delete(y, i)[:, None], lam).predict(x[i: i + 1])[0, 0]
                         for i in range(n)])
        sse = float(((pred - y) ** 2).sum())
        if best is None or sse < best[0]:
            best = (sse, lam)
    sst = float(((y - y.mean()) ** 2).sum())
    return {"r2_loo": 1.0 - best[0] / sst if sst > 0 else None, "lambda": best[1], "n": n}


def cmd_explain(args) -> int:
    """After every prediction exists: how much of each predictor's signed error the hidden theta explains."""
    from wm_data import load_episode
    base = json.loads(Path(args.base).read_text())
    m = json.loads(c.V2_MANIFEST.read_text())
    paths = {e["seed"]: e["path"] for e in m["episodes"] if e["split"] == "v19_fact" and e["role"] == "test"}
    seeds = sorted(int(s) for s in base["truth"]["test"])
    theta, summary = [], []
    gr = {ep.seed: ep for ep in c.load_groups(["test"], Path(args.root), "v2_v19_fact")["test"]}
    for s in seeds:
        ep = load_episode(Path(args.root) / paths[s], arrays=["settlements.npy", "settlement_ids.npy"])
        home = int(np.argmin(ep.arrays["settlement_ids.npy"]))
        theta.append(float(ep.column("settlements.npy", "fission_threshold")[START_ROW, home]))
        summary.append(window(gr[s]).mean(axis=0))
    theta, summary = np.array(theta), np.array(summary)
    errs = {n: base["scores"]["test"][n]["signed"] for n in ("median_day", "an5_day", "ridge_day")}
    for spec in args.model:
        name, path = spec.split("=", 1)
        errs[name] = json.loads(Path(path).read_text())["signed"]
    out = {"theta": {"mean": float(theta.mean()), "std": float(theta.std()), "n": len(theta),
                     "corr_with_truth": float(np.corrcoef(theta, [base["truth"]["test"][str(s)] for s in seeds])[0, 1])},
           "per_predictor": {}, "note": "theta read after every test prediction existed (labels only)"}
    for name, e in errs.items():
        y = np.array([e[str(s)] for s in seeds])
        a = loo_r2(summary, y)
        bb = loo_r2(np.column_stack([theta, summary]), y)
        th = loo_r2(theta[:, None], y)
        out["per_predictor"][name] = {"summary_only": a, "theta_plus_summary": bb, "theta_only": th,
                                      "theta_share": None if a["r2_loo"] is None or bb["r2_loo"] is None
                                      else bb["r2_loo"] - a["r2_loo"],
                                      "corr_theta_error": float(np.corrcoef(theta, y)[0, 1])}
    c.write_json(Path(args.out) / "split_explain.json", out)
    print(json.dumps({n: v["theta_share"] for n, v in out["per_predictor"].items()}))
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("base"); p.add_argument("--root", default=str(c.DEFAULT_ROOT)); p.add_argument("--out", required=True)
    p = sub.add_parser("model"); p.add_argument("--run", required=True); p.add_argument("--root", default=str(c.DEFAULT_ROOT))
    p.add_argument("--device", default="auto"); p.add_argument("--out", required=True)
    p = sub.add_parser("explain"); p.add_argument("--base", required=True); p.add_argument("--model", nargs="*", default=[])
    p.add_argument("--root", default=str(c.DEFAULT_ROOT)); p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    return {"base": cmd_base, "model": cmd_model, "explain": cmd_explain}[a.cmd](a)


if __name__ == "__main__":
    raise SystemExit(main())
