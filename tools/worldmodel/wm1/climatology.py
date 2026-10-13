"""WM-1.1 calendar-family baselines (GOAL_1_1.md 6, 9.1): what a forecaster that only knows "what this world usually
looks like at this time of year" scores on the WM-1 harness. numpy only.

Nine members, all fit on A_train V arrays (rows >= 24 for every mean and coefficient; `wm1.calcore` holds the math):

- `cal_abs`   CalAbs, the per-row mean over A_train episodes (rows past the train length wrap back by whole years);
- `cal_year`  CalYear, the mean over A_train episodes and rows of the same year position p(r) = (r + 8) mod 672;
- `ap1_abs`, `ap1_year`     anomaly persistence: m(s + k) + (v[s] - m(s));
- `apfit_abs`, `apfit_year` damped anomaly persistence: m(s + k) + alpha[k, j] (v[s, j] - m(s, j)), alpha by least
  squares on A_train;
- `an1`, `an5`, `an5_off`   Lorenz (1969) analogs: the A_train episodes whose last 24 rows (8 variables, scaled by the
  A_train split std) are nearest to the test window; one shift by whole years when the window runs past the library.

Predictions are direct (no feedback); actions, severity, `wood_target`, `latent` and lifespans are never read; a
prediction for start s reads the test episode's rows <= s only. `cal_year`, `ap1_year`, `apfit_year` never read
absolute time ("fair" for models with only the season / day-of-season / hour calendar).

usage (on the DGX, inside the venv):
  python3 tools/worldmodel/wm1/climatology.py val   --out RUN_DIR    # fit + every member on A_val / B_val + selection
  python3 tools/worldmodel/wm1/climatology.py final --out RUN_DIR --selection VAL_JSON   # test splits, once

`--dataset` (GOAL_1_2.md 7.3; default `v0`, unchanged): on a non-v0 dataset (the p10 pilot) the family is fit on the
dataset's `train` group, every member is scored on `val` over T1, T2 24 / 168 h at 06:00 and every-hour starts and
their bias-removed versions (no T3 / T3b: no interventions, hidden severity), and selected per task: `cal_best`,
`cal_best_fair`, and on the bias-removed tasks `cal_best_anom` (= that task's `cal_best`). Persistence and the
severity-free ridge of `bench_v1` (refit on `train`, lambda on `val` T1) are evaluated beside them as references.
Every T2 score carries its per-world scores (GOAL_1_3.md 7.3). On a dataset with `train_seeds` (the nested WM-1.3
training sets) the family, the ridge and the analog library see only those train worlds (`common.load_groups`).
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import calcore as cc
from wm1 import common as c
from wm1 import evaluate as ev
from wm1.data import EpisodeData

MEMBERS, FAIR = cc.MEMBERS, cc.FAIR
K_MAX = 168
T_TR = 2881
# Selection tasks (GOAL_1_1.md 6.2): key -> (score path in the `evaluate` output, which val splits, metric)
TASKS = {"T1": (("T1",), ("A_val", "B_val"), "mean"), "T2_24": (("T2", "24"), ("A_val", "B_val"), "mean"),
         "T2_168": (("T2", "168"), ("A_val", "B_val"), "mean"),
         "T2any_24": (("T2_any", "24"), ("A_val", "B_val"), "mean"),
         "T2any_168": (("T2_any", "168"), ("A_val", "B_val"), "mean"),
         "T3": (("T3",), None, "brier"), "T3b": (("T3b",), ("A_val", "B_val"), "mae")}


def tasks_for(ds: c.Dataset) -> dict:
    """The selection tasks of a dataset: v0's TASKS, or for an observable dataset T1, T2 24 / 168 (06:00 and every
    hour) and the bias-removed T2 tasks (`T2br_*`, `T2anybr_*`), all on the dataset's validation groups."""
    if ds.name == "v0":
        return TASKS
    val = tuple(ds.val)
    out = {"T1": (("T1",), val, "mean")}
    for h in ("24", "168"):
        out[f"T2_{h}"] = (("T2", h), val, "mean")
        out[f"T2any_{h}"] = (("T2_any", h), val, "mean")
        out[f"T2br_{h}"] = (("T2", h), val, "bias_removed")
        out[f"T2anybr_{h}"] = (("T2_any", h), val, "bias_removed")
    return out


def key_of(ep: EpisodeData) -> tuple:
    return (ep.split, ep.role, int(ep.seed), ep.eid)


class CalendarFit:
    """Train-only statistics of the calendar family (A_train episodes)."""

    def __init__(self, train: list[EpisodeData], t_tr: int | None = None, split: str = "A_obs"):
        if not train or any(ep.role != "train" or ep.split != split for ep in train):
            raise ValueError(f"the calendar family is fit on the train episodes of {split} only")
        rows = {ep.rows for ep in train}
        if len(rows) != 1:
            raise ValueError(f"A_train episodes differ in length: {sorted(rows)}")
        self.t_tr = rows.pop()
        if t_tr is not None and self.t_tr != t_tr:
            raise ValueError(f"T_tr {self.t_tr} != {t_tr}")
        for ep in train:
            if not np.array_equal(ep.bench.tick, 60 * np.arange(self.t_tr)):
                raise ValueError(f"seed {ep.seed}: tick != 60 * arange(rows)")
        self.lib = np.stack([ep.bench.v for ep in train]).astype(np.float64)       # [E, T, 8]
        self.keys = [key_of(ep) for ep in train]
        self.m_abs, self.m_year = cc.calendar_means(self.lib)
        self.year_covered = bool(np.isfinite(self.m_year).all())
        rows_tr = np.arange(self.t_tr)
        self.alpha = {"abs": cc.fit_alpha(self.lib, cc.mean_at("abs", rows_tr, self.m_abs, self.m_year), K_MAX),
                      "year": cc.fit_alpha(self.lib, cc.mean_at("year", rows_tr, self.m_abs, self.m_year), K_MAX)}
        std = b.split_std([ep.bench for ep in train])
        self.std = std
        self.inv_std = np.where(std > 0, 1.0 / np.where(std > 0, std, 1.0), 1.0)

    def summary(self) -> dict:
        return {"t_tr": self.t_tr, "episodes": len(self.keys), "year_positions_covered": self.year_covered,
                "train_std": self.std, "zero_std_vars": [b.VARS[j] for j in np.nonzero(self.std == 0)[0]],
                "alpha": {kind: {str(k): dict(zip(b.VARS, a[k].tolist())) for k in (1, 24, 168)}
                          for kind, a in self.alpha.items()}}


class CalendarPredictor(ev.Predictor):
    """One member of the calendar family as a harness predictor (direct rollouts; actions ignored)."""

    def __init__(self, fit: CalendarFit, member: str):
        if member not in MEMBERS:
            raise ValueError(member)
        self.fit, self.member, self.name = fit, member, member

    def rollout(self, ep, starts, horizon, actions):
        if horizon > K_MAX:
            raise ValueError(f"horizon {horizon} > {K_MAX}")
        f = self.fit
        own = f.keys.index(key_of(ep)) if key_of(ep) in f.keys else None
        return ev.Rollout(cc.predict_member(self.member, ep.bench.v, starts, horizon, f.lib, f.m_abs, f.m_year,
                                            f.alpha, f.inv_std, exclude=own))


def predictors(fit: CalendarFit) -> dict[str, CalendarPredictor]:
    return {m: CalendarPredictor(fit, m) for m in MEMBERS}


def _get(d: dict, path: tuple):
    for k in path:
        d = d.get(k, {}) if isinstance(d, dict) else {}
    return d


def task_score(result: dict, task: str, tasks: dict = TASKS) -> float:
    """The validation score of one member's `evaluate` output for a selection task (lower is better)."""
    path, splits, metric = tasks[task]
    node = _get(result, path)
    if splits is None:                       # T3: A_val only (the harness's T3 is defined on A)
        return float(node[metric])
    if metric == "bias_removed":
        return float(np.mean([node[s]["bias_removed"]["mean"] for s in splits]))
    return float(np.mean([node[s][metric] for s in splits]))


def select(val: dict[str, dict], tasks: dict = TASKS) -> dict:
    """cal_best (all nine) and cal_best_fair (CalYear, ap1_year, apfit_year) per task, by the val score; on a
    bias-removed task the same choice is also named `cal_best_anom` / `cal_best_anom_fair`."""
    out = {}
    for task in tasks:
        scores = {m: task_score(val[m], task, tasks) for m in MEMBERS}
        best = min(MEMBERS, key=lambda m: (scores[m], MEMBERS.index(m)))
        fair = min(FAIR, key=lambda m: (scores[m], MEMBERS.index(m)))
        out[task] = {"cal_best": best, "cal_best_fair": fair, "val_scores": scores}
        if tasks[task][2] == "bias_removed":
            out[task] |= {"cal_best_anom": best, "cal_best_anom_fair": fair}
    return out


def references(train: list[EpisodeData], val: list[EpisodeData]) -> tuple[dict, dict]:
    """Persistence and the severity-free observable ridge (bench_v1's `ridge`: lags, hour, season; lambda on the
    validation T1 mean nMAE) refit on `train`."""
    ridge, lam, scores = ev.fit_ridge_without_severity(train, val, action_aware=False)
    ridge.name = "ridge"
    return {"persistence": ev.Persistence(), "ridge": ridge}, {"ridge_lambda": lam, "ridge_val_t1": scores}


def _run_observable(args, ds: c.Dataset, roles: list[str], selection: dict | None, t0: float) -> dict:
    """Fit on `train`, evaluate the nine members and the references on `roles` (val or test groups)."""
    g = c.load_groups(["train", "val"] + [r for r in roles if r not in ("train", "val")], Path(args.root), ds.name)
    fit = CalendarFit(g["train"], T_TR, split=ds.groups["train"][0])
    if not fit.year_covered:
        raise SystemExit("m_year leaves a year position uncovered")
    splits = {n: g[n] for n in roles}
    stds = c.split_stds(splits)
    refs, ref_info = references(g["train"], g["val"])
    results, ref_results = {}, {}
    kw = dict(cf_test=None, t3_split=None, tasks=ds.tasks, bias_removed=True, per_world=True)
    for m, p in predictors(fit).items():
        results[m] = ev.evaluate(p, splits, stds, **kw)
        print(f"{m} {time.time() - t0:.0f}s", flush=True)
    for name, p in refs.items():
        ref_results[name] = ev.evaluate(p, splits, stds, **kw)
        print(f"{name} {time.time() - t0:.0f}s", flush=True)
    return {"fit": fit.summary(), "results": results, "references": ref_results, "reference_info": ref_info,
            "dataset": ds.name, "groups": {n: [int(ep.seed) for ep in g[n]] for n in g},
            "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(ds.name), "argv": sys.argv}


def cmd_val(args) -> int:
    t0 = time.time()
    ds = c.dataset(args.dataset)
    if ds.name != "v0":
        tasks = tasks_for(ds)
        run = _run_observable(args, ds, list(ds.val), None, t0)
        out = {**{k: v for k, v in run.items() if k != "results"}, "val": run["results"],
               "selection": select(run["results"], tasks), "tasks": {k: list(v[0]) + [v[2]] for k, v in tasks.items()},
               "seconds": time.time() - t0}
        c.write_json(Path(args.out) / "calendar_val.json", out)
        for task, sel in out["selection"].items():
            print(task, sel["cal_best"], round(sel["val_scores"][sel["cal_best"]], 4), sel["cal_best_fair"],
                  round(sel["val_scores"][sel["cal_best_fair"]], 4))
        return 0
    g = c.load_groups(["A_train", "A_val", "B_val"], Path(args.root))
    fit = CalendarFit(g["A_train"], T_TR)
    if not fit.year_covered:
        raise SystemExit("m_year leaves a year position uncovered")
    val = {n: g[n] for n in c.VAL_SPLITS}
    stds = c.split_stds(val)
    results = {}
    for m, p in predictors(fit).items():
        results[m] = ev.evaluate(p, val, stds, cf_test=None, t3_split="A_val", any_hour=True)
        print(f"{m} {time.time() - t0:.0f}s", flush=True)
    out = {"fit": fit.summary(), "val": results, "selection": select(results), "code_commit": c.git_commit(),
           "manifest_sha256": c.manifest_sha(), "seconds": time.time() - t0, "argv": sys.argv}
    c.write_json(Path(args.out) / "calendar_val.json", out)
    for task, s in out["selection"].items():
        print(task, s["cal_best"], round(s["val_scores"][s["cal_best"]], 4), s["cal_best_fair"],
              round(s["val_scores"][s["cal_best_fair"]], 4))
    return 0


def cmd_final(args) -> int:
    """Every member on the four test splits through the harness (T1, T2 24/168 both start sets with breakdowns, T3 on
    A_test, T3b). Run once, after METRICS_1_1.md (with the selection) is pushed."""
    target = Path(args.out) / "calendar.json"
    if target.exists() and not args.force:
        raise SystemExit(f"{target} exists: test numbers are computed once")
    sel_json = json.loads(Path(args.selection).read_text())
    sel = sel_json["selection"]
    t0 = time.time()
    ds = c.dataset(args.dataset)
    if sel_json.get("dataset", "v0") != ds.name:
        raise SystemExit(f"the selection is of dataset {sel_json.get('dataset', 'v0')}, not {ds.name}")
    if ds.name != "v0":
        run = _run_observable(args, ds, list(ds.test), sel, t0)
        out = {**{k: v for k, v in run.items() if k != "results"}, "test": run["results"], "selection": sel,
               "seconds": time.time() - t0, "forced": bool(args.force)}
        c.write_json(target, out)
        return 0
    g = c.load_groups(["A_train"] + list(c.TEST_SPLITS), Path(args.root))
    fit = CalendarFit(g["A_train"], T_TR)
    tests = {n: g[n] for n in c.TEST_SPLITS}
    stds = c.split_stds(tests)
    results = {}
    for m, p in predictors(fit).items():
        results[m] = ev.evaluate(p, tests, stds, cf_test=None, t3_split="A_test", any_hour=True)
        print(f"{m} {time.time() - t0:.0f}s", flush=True)
    out = {"fit": fit.summary(), "test": results, "selection": sel,
           "T4": "not run: the calendar family ignores actions, so every predicted effect is 0 = zero_effect",
           "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(), "seconds": time.time() - t0,
           "argv": sys.argv, "forced": bool(args.force)}
    c.write_json(target, out)
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("val", "final"):
        p = sub.add_parser(name)
        p.add_argument("--out", required=True)
        p.add_argument("--root", default=str(c.DEFAULT_ROOT))
        p.add_argument("--dataset", default="v0", choices=sorted(c.DATASETS))
        if name == "final":
            p.add_argument("--selection", required=True, help="calendar_val.json of the pre-registered selection")
            p.add_argument("--force", action="store_true")
    args = ap.parse_args(argv)
    return cmd_val(args) if args.cmd == "val" else cmd_final(args)


if __name__ == "__main__":
    raise SystemExit(main())
