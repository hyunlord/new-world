"""Family evaluator for dataset v2 (GOAL_1_2.md 10): predicted intervention effects scored against the replicate
distribution of true effects. numpy only (a torch model enters as an `evaluate.Predictor`).

A branch family (Pass 9-10, `wm_data.load_family`) is one root seed's factual (`none`) and variants, each recorded
as R noise replicates forked at one tick; every member shares the rows <= the fork row. For a predictor:

- predicted effect of variant v = targets(rollout with v's recorded actions) - targets(rollout with the factual's
  actions), both rolled from the fork row of the shared prefix (rows <= fork row observed; the same prefix for every
  replicate, so one predicted effect per root seed and variant);
- true effects: per replicate k, targets(v_k) - targets(factual_k) (paired by replicate); their mean and spread
  (sample std over the R replicates, ddof 1; 0 when R = 1);
- scores per variant and target over root seeds: **mean-effect MAE** = mean |predicted - true replicate mean|,
  **sign accuracy** = share of seeds where sign(predicted) = sign(true replicate mean), **coverage** = share of seeds
  where the true replicate mean lies within predicted +- the spread of the true replicate effects.

Targets (default window = dataset v1's C_cf first winter, `bench_v1.c_cf_targets`; the p10 forks fork at day 21
08:00): dark nights = the 02:00 rows after the evenings of one-based days 22-28 where the **home settlement's**
fire is out (home = the lowest id in `settlement_ids.npy`; not the max over settlements); cold member-hours from day
22 18:00 to day 29 07:00; food store (summed over settlements) at day 29 06:00; deaths up to day 29 06:00 including
`deaths_exposure`. A predictor's own rows (> fork row) supply its fire (`fire_prob` >= 0.5 when given, else the V
`fire_lit` >= 0.5), food store, cold member-hours and deaths (both continue the fork row's true value when the
rollout carries none); rows <= fork row come from the shared prefix. The zero-effect baseline predicts 0 everywhere.

usage (DGX): python3 tools/worldmodel/wm1/families.py --families DIR [DIR ...] --out OUT_DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm_data import load_family
from wm1 import common as c
from wm1 import data as d
from wm1 import evaluate as ev

TARGETS = ("dark_nights", "cold_member_hours", "food_store", "deaths")
ARRAYS = ["global.npy", "settlements.npy", "settlement_ids.npy"]
V_FOOD = b.VARS.index("food_stock")
DEATH_COLUMNS = ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure")


def row_of(day: int, minute: int) -> int:
    """Row (every 60) of one-based display day `day` at `minute`."""
    return ((day - 1) * b.TPD + minute - b.OFFSET) // b.EVERY


@dataclass(frozen=True)
class Window:
    """Rows the targets read: the 02:00 rows counted as nights, the cold member-hours span, the food and deaths rows."""
    nights: tuple[int, ...]
    cold: tuple[int, int]
    food: int
    deaths: int

    @classmethod
    def c_cf(cls) -> "Window":
        """Dataset v1's C_cf first winter (`bench_v1.c_cf_targets`)."""
        return cls(tuple(row_of(day + 1, 120) for day in range(22, 29)), (row_of(22, 1080), row_of(29, 420)),
                   row_of(29, 360), row_of(29, 360))

    @property
    def last(self) -> int:
        return max(*self.nights, *self.cold, self.food, self.deaths)


def home_slot(settlement_ids: np.ndarray) -> int:
    """Slot of the home settlement: the lowest settlement id."""
    ids = np.asarray(settlement_ids)
    if ids.size == 0:
        raise ValueError("an episode without settlements has no home slot")
    return int(np.argmin(ids))


def member_arrays(ep) -> dict[str, np.ndarray]:
    """Per-row target series of one recorded member (`wm_data.Episode`): the home slot's fire, the summed food
    store, cumulative cold member-hours and cumulative deaths with exposure (schema 4; schema < 4: four causes)."""
    s = ep.arrays["settlements.npy"]
    fire = np.nan_to_num(s[:, home_slot(ep.arrays["settlement_ids.npy"]), ep.field_index("settlements.npy", "fire_lit")],
                         nan=0.0)
    food = np.nansum(s[:, :, ep.field_index("settlements.npy", "food_stock")], axis=1)
    names = ep.observable_names("global.npy")
    deaths = sum(ep.column("global.npy", k).astype(np.float64) for k in DEATH_COLUMNS if k in names)
    return {"fire_home": fire.astype(np.float64), "food": food.astype(np.float64),
            "cold_hours": ep.column("global.npy", "cold_member_hours_total").astype(np.float64), "deaths": deaths}


def targets(a: dict[str, np.ndarray], win: Window) -> dict[str, float]:
    return {"dark_nights": float(sum(a["fire_home"][r] < 0.5 for r in win.nights)),
            "cold_member_hours": float(a["cold_hours"][win.cold[1]] - a["cold_hours"][win.cold[0]]),
            "food_store": float(a["food"][win.food]), "deaths": float(a["deaths"][win.deaths])}


def fork_row(fam) -> int:
    every = int(fam.factual[0].meta.get("every", b.EVERY))
    if fam.fork_tick % every:
        raise ValueError(f"fork tick {fam.fork_tick} is not on a recorded row (every {every})")
    return fam.fork_tick // every


def check_prefix(fam, row: int) -> None:
    """Every member's global and settlement rows <= the fork row equal the factual replicate 0's."""
    ref = fam.factual[0]
    members = [*fam.factual.values(), *(ep for reps in fam.variants.values() for ep in reps.values())]
    for ep in members:
        if not np.array_equal(ep.arrays["settlement_ids.npy"], ref.arrays["settlement_ids.npy"]):
            raise ValueError(f"{ep.path}: settlement ids differ from the factual's")
        for name in ("global.npy", "settlements.npy"):
            if not np.array_equal(ep.arrays[name][: row + 1], ref.arrays[name][: row + 1], equal_nan=True):
                raise ValueError(f"{ep.path}: rows <= {row} of {name} differ from the factual's prefix")


def true_effects(fam, win: Window) -> dict[str, dict]:
    """variant -> per-replicate effects (paired by replicate), their mean and spread (ddof 1; 0 when R = 1)."""
    base = {k: targets(member_arrays(ep), win) for k, ep in fam.factual.items()}
    out = {}
    for variant, reps in sorted(fam.variants.items()):
        per = {k: {t: targets(member_arrays(ep), win)[t] - base[k][t] for t in TARGETS} for k, ep in sorted(reps.items())}
        arr = {t: np.array([per[k][t] for k in sorted(per)]) for t in TARGETS}
        out[variant] = {"per_replicate": {str(k): v for k, v in per.items()},
                        "mean": {t: float(arr[t].mean()) for t in TARGETS},
                        "spread": {t: float(arr[t].std(ddof=1)) if len(arr[t]) > 1 else 0.0 for t in TARGETS}}
    return out


def episode_data(ep, seed: int, eid: str) -> d.EpisodeData:
    return d.from_episode(ep, ep.meta.get("scene", "family"), "test", seed, eid)


def branch_targets(pred: ev.Predictor, ep: d.EpisodeData, prefix: dict[str, np.ndarray], start: int,
                   actions: np.ndarray, win: Window) -> dict[str, float]:
    """Targets of the shared prefix (rows <= start) followed by the predictor's rows start+1 .. win.last."""
    horizon = win.last - start
    if horizon < 1:
        raise ValueError("the target window ends at or before the fork row")
    ro = ev.run_rollouts(pred, ep, np.array([start]), horizon, actions)
    keep = slice(0, start + 1)
    lit = ro.fire_prob[0] if ro.fire_prob is not None else ro.v[0, :, ev.V_FIRE]
    cont = lambda series, pr: np.concatenate([prefix[series][keep], pr if pr is not None else np.full(horizon, prefix[series][start])])
    a = {"fire_home": np.concatenate([prefix["fire_home"][keep], (lit >= 0.5).astype(np.float64)]),
         "food": np.concatenate([prefix["food"][keep], ro.v[0, :, V_FOOD]]),
         "cold_hours": cont("cold_hours", None if ro.cold_hours is None else ro.cold_hours[0]),
         "deaths": cont("deaths", None if ro.deaths is None else ro.deaths[0])}
    return targets(a, win)


def predicted_effects(pred: ev.Predictor | None, fam, win: Window) -> dict[str, dict[str, float]]:
    """variant -> target -> predicted effect from the fork row (None: the zero-effect baseline)."""
    if pred is None:
        return {v: {t: 0.0 for t in TARGETS} for v in sorted(fam.variants)}
    row = fork_row(fam)
    check_prefix(fam, row)
    factual = episode_data(fam.factual[0], fam.seed, "none")
    prefix = member_arrays(fam.factual[0])
    off = branch_targets(pred, factual, prefix, row, factual.bench.actions, win)
    out = {}
    for variant, reps in sorted(fam.variants.items()):
        acts = episode_data(reps[0], fam.seed, variant).bench.actions
        if acts[: row + 1].any():
            raise ValueError(f"{variant}: an action at or before the fork row")
        on = branch_targets(pred, factual, prefix, row, acts, win)
        out[variant] = {t: on[t] - off[t] for t in TARGETS}
    return out


def score(pred: dict[int, dict], true: dict[int, dict]) -> dict:
    """Per variant and target over root seeds: mean-effect MAE, sign accuracy, coverage (see the module doc)."""
    out = {}
    variants = sorted({v for s in true.values() for v in s})
    for v in variants:
        seeds = sorted(s for s in true if v in true[s])
        out[v] = {}
        for t in TARGETS:
            p = np.array([pred[s][v][t] for s in seeds])
            m = np.array([true[s][v]["mean"][t] for s in seeds])
            sp = np.array([true[s][v]["spread"][t] for s in seeds])
            out[v][t] = {"mean_effect_mae": float(np.abs(p - m).mean()),
                         "sign_accuracy": float(np.mean(np.sign(p) == np.sign(m))),
                         "coverage": float(np.mean(np.abs(p - m) <= sp)), "seeds": len(seeds),
                         "true_mean_abs": float(np.abs(m).mean()), "true_spread_mean": float(sp.mean())}
    return out


def evaluate_families(predictors: dict[str, ev.Predictor | None], families: list, win: Window) -> dict:
    true = {fam.seed: true_effects(fam, win) for fam in families}
    out = {"true_effects": {str(s): t for s, t in true.items()}, "predictors": {}}
    for name, pred in predictors.items():
        eff = {fam.seed: predicted_effects(pred, fam, win) for fam in families}
        out["predictors"][name] = {"effects": {str(s): e for s, e in eff.items()}, "scores": score(eff, true)}
    return out


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--families", nargs="+", required=True, help="family directories (one per root seed)")
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    t0 = time.time()
    fams = [load_family(Path(p), arrays=ARRAYS) for p in args.families]
    win = Window.c_cf()
    out = evaluate_families({"persistence": ev.Persistence(), "zero_effect": None}, fams, win)
    out |= {"families": [{"path": str(f.path), "seed": f.seed, "scene": f.scene, "fork_tick": f.fork_tick,
                          "fork_row": fork_row(f), "replicates": f.replicates, "variants": sorted(f.variants),
                          "home_slot": home_slot(f.factual[0].arrays["settlement_ids.npy"]),
                          "settlement_ids": f.factual[0].arrays["settlement_ids.npy"].tolist()} for f in fams],
            "window": {"nights": list(win.nights), "cold": list(win.cold), "food": win.food, "deaths": win.deaths},
            "code_commit": c.git_commit(), "argv": sys.argv, "seconds": time.time() - t0,
            "note": "real-data smoke of the evaluator (GOAL_1_2.md 10.3): no model claims"}
    c.write_json(Path(args.out) / "families_smoke.json", out)
    print(json.dumps({n: {v: {t: s["mean_effect_mae"] for t, s in x.items()} for v, x in p["scores"].items()}
                      for n, p in out["predictors"].items()}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
