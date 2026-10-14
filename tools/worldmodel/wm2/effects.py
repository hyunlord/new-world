"""Effect evaluation (GOAL.md 7.4) and the effect baselines (7.5) on the dataset-v2 families. numpy only (a model enters
through `wm2.rollout_ext`).

A **case** is one (world, arm, replicate r) of a family: the arm's cast tick c (`interventions.json`, a multiple of 60)
and row(c). Rows <= row(c) of arm member r equal those of `none` member r (GOAL.md 6.2, checked by `families_v2`),
so a predictor starts at row(c) of `none_r` and rolls two branches to the last row: the factual (no action) and the arm
(the arm's requested amount at row(c), entering the transition c -> c + 60). Replicate 1's history between the fork and
c differs from replicate 0's, so each replicate is predicted from its own history.

- **true effect** of outcome o at lag L (L = 168 rows = 7 days, 720 = 30 days): arm_r(row(c) + L) - none_r(row(c) + L)
  for `food_stock` (summed store), `alive`, cumulative deaths (the five causes of `families.DEATH_COLUMNS`) and
  `settlements_alive`; a case whose row(c) + L is past the last row has no value at L (counted per arm);
- **split shift** = restricted split day (reference row(c), `v2.restricted_split_day`) of the arm minus that of the
  factual, truth and prediction alike (a prediction's `fissions_total` series is the true rows <= row(c) followed by
  the rolled-out state);
- **predicted effect** = predicted arm - predicted factual from the same start.

Metrics per arm, outcome and lag: **effect MAE** = mean over cases of |predicted - true| / std, std = the std over the
train cases of that arm's true effect (fallbacks: the std over every arm's train cases; < 1e-9 again: the cell is
dropped and recorded); **sign accuracy** over cases with a true effect != 0; split shift: MAE in days and the
right-direction share over cases with a true shift != 0, with the censored shares; the **between-replicate effect
spread** |e_0 - e_1| / sqrt(2) / std over worlds with both replicates (not a floor: it bounds the post-cast noise from
above). Intervals: world-level paired bootstrap (`wm1.bootstrap.draws`, B = 2,000, `default_rng(0)`, both replicates of a
world move together); a statistic averaged over cells is recomputed per draw from per-world sums.

Baselines (they predict the effect directly): `zero`; `arm_mean` (the mean over the arm's train cases at each lag, and
the mean train shift); `an5_arm` (the 5 nearest train cases of the same arm, at most one per world — the nearer
replicate — by `an5`'s distance: the 24-row window of the 8 V ending at the start row, scaled by the train split std of
the factual train worlds (`bench_v0.split_std`, as `climatology.CalendarFit`), each candidate compared at its own cast
row; the mean of their true effects at each lag (among candidates with that lag) and of their shifts).

usage (DGX):
  python3 tools/worldmodel/wm2/effects.py base --root ROOT --out DIR             # truths, baselines, val selection
  python3 tools/worldmodel/wm2/effects.py model --run RUN --root ROOT --out DIR  # one model's test predictions + drift
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
import bench_v0 as b
import bench_v1 as bv1
from wm_data import load_family
from wm1 import bootstrap as bs
from wm1 import common as c
from wm1 import data as d
from wm2 import v2

ARRAYS = ["global.npy", "settlements.npy", "settlement_ids.npy"]
V_FOOD, V_ALIVE = b.VARS.index("food_stock"), b.VARS.index("alive")
DRIFT_LAGS = {"7d": 168, "30d": 720, "45d": 1080}
EPS = 1e-9


@dataclass
class Case:
    seed: int
    arm: str
    rep: int
    role: str
    cast_tick: int
    cast_row: int
    truth: dict                        # outcome -> lag name -> true effect (absent past the last row)
    shift: float                       # true split shift (days)
    censored: tuple                    # (arm censored, factual censored) of the truth
    window: np.ndarray = field(repr=False)   # [24, 8] V rows cast_row - 23 .. cast_row of none_r

    @property
    def key(self) -> str:
        return f"{self.seed}:{self.arm}:{self.rep}"


def family_dirs(root: Path, role: str) -> list[Path]:
    lo, hi = v2.ROLES["game_v19"][role]
    fam = Path(root) / "v2" / "v19" / "fam"
    return [fam / f"game_v19_s{s}" for s in range(lo, hi) if (fam / f"game_v19_s{s}").is_dir()]


def cast_of(ep) -> int:
    (rec,) = ep.interventions
    tick = int(rec["tick"])
    v2.row_of_tick(tick)
    return tick


def family_cases(fam, role: str) -> list[Case]:
    """Every (arm, replicate) case of one loaded family with its true effects, shift and an5 window."""
    out = []
    none = {r: v2.member_series(ep) for r, ep in fam.factual.items()}
    vnone = {r: bv1.episode_arrays(ep)["v"] for r, ep in fam.factual.items()}
    for arm, reps in sorted(fam.variants.items()):
        for r, ep in sorted(reps.items()):
            tick = cast_of(ep)
            row = v2.row_of_tick(tick)
            if row < 23:
                raise ValueError(f"{ep.path}: cast row {row} leaves no 24-row window")
            arm_s = v2.member_series(ep)
            truth = {o: {name: float(arm_s[o][row + lag] - none[r][o][row + lag])
                         for name, lag in v2.LAGS.items() if row + lag <= v2.LAST_ROW} for o in v2.OUTCOMES}
            da, ca = v2.restricted_split_day(arm_s["fissions_total"], row)
            dn, cn = v2.restricted_split_day(none[r]["fissions_total"], row)
            out.append(Case(fam.seed, arm, r, role, tick, row, truth, float(da - dn), (ca, cn),
                            vnone[r][row - 23: row + 1].copy()))
    return out


def load_cases(root: Path, role: str, verify: bool = True) -> tuple[list[Case], list]:
    """Cases of every family of a role, and the bench V of every family's `none_r0` (the factual split std)."""
    cases, fact_v = [], []
    for path in family_dirs(root, role):
        fam = load_family(path, verify=verify, arrays=ARRAYS)
        cases += family_cases(fam, role)
        fact_v.append(bv1.episode_arrays(fam.factual[0])["v"])
    return cases, fact_v


# ------------------------------------------------------------------ baselines ---------------------------------------
def scale_stds(train: list[Case]) -> dict:
    """{(arm, outcome, lag): std} over the train cases (fallback: every arm's train cases; None: dropped)."""
    out = {}
    for o in v2.OUTCOMES:
        for lag in v2.LAGS:
            pooled = np.array([x.truth[o][lag] for x in train if lag in x.truth[o]])
            for arm in v2.ARMS:
                vals = np.array([x.truth[o][lag] for x in train if x.arm == arm and lag in x.truth[o]])
                sd = float(vals.std()) if len(vals) else 0.0
                if sd < EPS:
                    sd = float(pooled.std()) if len(pooled) else 0.0
                out[(arm, o, lag)] = sd if sd >= EPS else None
    return out


def arm_mean(train: list[Case]) -> dict:
    """{arm: {"effects": {o: {lag: mean}}, "shift": mean}} over the train cases."""
    out = {}
    for arm in v2.ARMS:
        cs = [x for x in train if x.arm == arm]
        eff = {o: {lag: float(np.mean([x.truth[o][lag] for x in cs if lag in x.truth[o]]))
                   for lag in v2.LAGS if any(lag in x.truth[o] for x in cs)} for o in v2.OUTCOMES}
        out[arm] = {"effects": eff, "shift": float(np.mean([x.shift for x in cs])) if cs else 0.0, "n": len(cs)}
    return out


def predict_zero(case: Case) -> dict:
    return {"effects": {o: {lag: 0.0 for lag in case.truth[o]} for o in v2.OUTCOMES}, "shift": 0.0}


def predict_arm_mean(case: Case, means: dict) -> dict:
    m = means[case.arm]
    return {"effects": {o: {lag: m["effects"][o][lag] for lag in case.truth[o]} for o in v2.OUTCOMES},
            "shift": m["shift"]}


def nearest_worlds(case: Case, cands: list[Case], inv_std: np.ndarray, k: int = 5) -> list[Case]:
    """The k nearest candidates by an5's distance, at most one per world (its nearer replicate); ties by world seed
    then replicate (stable)."""
    best: dict[int, tuple[float, int, Case]] = {}
    for x in cands:
        dist = float((((case.window - x.window) * inv_std) ** 2).sum())
        cur = best.get(x.seed)
        if cur is None or (dist, x.rep) < (cur[0], cur[1]):
            best[x.seed] = (dist, x.rep, x)
    ranked = sorted(best.values(), key=lambda t: (t[0], t[2].seed, t[1]))
    return [t[2] for t in ranked[:k]]


def predict_an5_arm(case: Case, train: list[Case], inv_std: np.ndarray) -> dict:
    pool = [x for x in train if x.arm == case.arm]
    eff = {}
    for o in v2.OUTCOMES:
        eff[o] = {}
        for lag in case.truth[o]:
            near = nearest_worlds(case, [x for x in pool if lag in x.truth[o]], inv_std)
            eff[o][lag] = float(np.mean([x.truth[o][lag] for x in near]))
    near = nearest_worlds(case, pool, inv_std)
    return {"effects": eff, "shift": float(np.mean([x.shift for x in near])),
            "neighbours": [x.key for x in near]}


def baseline_predictions(cases: list[Case], train: list[Case], inv_std: np.ndarray) -> dict:
    means = arm_mean(train)
    return {"zero": {x.key: predict_zero(x) for x in cases},
            "arm_mean": {x.key: predict_arm_mean(x, means) for x in cases},
            "an5_arm": {x.key: predict_an5_arm(x, train, inv_std) for x in cases}}


# ------------------------------------------------------------------ scoring -----------------------------------------
def cell_errors(cases: list[Case], preds: dict, stds: dict) -> dict:
    """{(arm, outcome, lag): [(seed, rep, scaled |pred - true|, true, pred)]} and {(arm, "shift"): [... days]}."""
    out: dict = {}
    for x in cases:
        p = preds[x.key]
        for o in v2.OUTCOMES:
            for lag, t in x.truth[o].items():
                sd = stds[(x.arm, o, lag)]
                if sd is None:
                    continue
                out.setdefault((x.arm, o, lag), []).append((x.seed, x.rep, abs(p["effects"][o][lag] - t) / sd, t,
                                                            p["effects"][o][lag]))
        out.setdefault((x.arm, "shift"), []).append((x.seed, x.rep, abs(p["shift"] - x.shift), x.shift, p["shift"]))
    return out


def cell_summary(rows: list) -> dict:
    err = np.array([r[2] for r in rows])
    t, p = np.array([r[3] for r in rows]), np.array([r[4] for r in rows])
    nz = t != 0
    return {"mae": float(err.mean()), "n": len(rows), "worlds": len({r[0] for r in rows}),
            "sign_accuracy": float(np.mean(np.sign(p[nz]) == np.sign(t[nz]))) if nz.any() else None,
            "n_true_nonzero": int(nz.sum())}


def spread(cases: list[Case], stds: dict) -> dict:
    """Between-replicate effect spread |e_0 - e_1| / sqrt(2) / std per cell (worlds with both replicates); shift in days."""
    by: dict = {}
    for x in cases:
        by.setdefault((x.seed, x.arm), {})[x.rep] = x
    out: dict = {}
    for (seed, arm), reps in by.items():
        if 0 not in reps or 1 not in reps:
            continue
        a, bb = reps[0], reps[1]
        for o in v2.OUTCOMES:
            for lag in v2.LAGS:
                sd = stds[(arm, o, lag)]
                if sd is None or lag not in a.truth[o] or lag not in bb.truth[o]:
                    continue
                out.setdefault((arm, o, lag), []).append(abs(a.truth[o][lag] - bb.truth[o][lag]) / np.sqrt(2) / sd)
        out.setdefault((arm, "shift"), []).append(abs(a.shift - bb.shift) / np.sqrt(2))
    return {k: {"mean": float(np.mean(v)), "worlds": len(v)} for k, v in out.items()}


def world_sums(rows: list, worlds: list[int]) -> tuple[np.ndarray, np.ndarray]:
    """Per world (in `worlds` order): the sum of the cell's errors and the number of cases."""
    pos = {w: i for i, w in enumerate(worlds)}
    s, n = np.zeros(len(worlds)), np.zeros(len(worlds))
    for r in rows:
        s[pos[r[0]]] += r[2]
        n[pos[r[0]]] += 1
    return s, n


def cell_stat(errs: dict, cells: list, worlds: list[int], counts: np.ndarray | None = None) -> np.ndarray | float:
    """Mean over `cells` of each cell's MAE; with `counts` [B, n_worlds] (bootstrap draw multiplicities) one value
    per draw (a cell's MAE recomputed from the resampled worlds' sums)."""
    vals = []
    for cell in cells:
        s, n = world_sums(errs[cell], worlds)
        if counts is None:
            vals.append(s.sum() / n.sum())
        else:
            with np.errstate(invalid="ignore", divide="ignore"):
                vals.append((counts @ s) / (counts @ n))
    return float(np.mean(vals)) if counts is None else np.nanmean(np.stack(vals), axis=0)


def draw_counts(n_worlds: int, b: int = bs.B, seed: int = bs.SEED) -> np.ndarray:
    """[b, n_worlds] multiplicity of each world in each `wm1.bootstrap.draws` resample."""
    idx = bs.draws(n_worlds, b, seed)
    return np.stack([np.bincount(row, minlength=n_worlds) for row in idx]).astype(np.float64)


def average_errors(per_seed: list[dict]) -> dict:
    """Per case, the mean over seeds of the scaled absolute error (a seed-averaged model; linear, so the cell MAE is
    the mean of the seeds' cell MAEs)."""
    out = {}
    for cell in per_seed[0]:
        rows = [sorted(e[cell], key=lambda r: (r[0], r[1])) for e in per_seed]
        if any([(r[0], r[1]) for r in x] != [(r[0], r[1]) for r in rows[0]] for x in rows):
            raise ValueError(f"seeds scored different cases in {cell}")
        out[cell] = [(r[0], r[1], float(np.mean([x[i][2] for x in rows])), r[3], float(np.mean([x[i][4] for x in rows])))
                     for i, r in enumerate(rows[0])]
    return out


def rule(seed_points: list[float], base_point: float, ci: dict) -> str:
    """The W1 rule: KEEP if the seed mean <= 0.8 x baseline, every seed below it and the CI upper < 0 (two seeds or
    more); WEAK if the CI upper < 0; TIE if the CI contains 0; NONE if the CI lower > 0."""
    if ci["hi"] < 0:
        keep = (len(seed_points) >= 2 and float(np.mean(seed_points)) <= 0.8 * base_point
                and all(s < base_point for s in seed_points))
        return "KEEP" if keep else "WEAK"
    if ci["lo"] > 0:
        return "NONE"
    return "TIE"


def compare(model_seeds: list[dict], base: dict, cells: list, worlds: list[int]) -> dict:
    """The W rule on a cell-averaged statistic: model (seed-averaged errors) vs one baseline, paired world bootstrap."""
    counts = draw_counts(len(worlds))
    avg = average_errors(model_seeds)
    m_draw, b_draw = cell_stat(avg, cells, worlds, counts), cell_stat(base, cells, worlds, counts)
    diff = m_draw - b_draw
    lo, hi = np.percentile(diff, [(100 - bs.LEVEL) / 2, 100 - (100 - bs.LEVEL) / 2])
    seeds = [cell_stat(e, cells, worlds) for e in model_seeds]
    base_point = cell_stat(base, cells, worlds)
    point = cell_stat(avg, cells, worlds)
    ci = {"point": point - base_point, "lo": float(lo), "hi": float(hi), "B": bs.B, "seed": bs.SEED,
          "n_worlds": len(worlds)}
    return {"verdict": rule(seeds, base_point, ci), "model": point, "seeds": seeds, "baseline": base_point,
            "ratio": point / base_point if base_point else None, "ci_diff": ci, "keep_threshold": 0.8 * base_point}


def score(cases: list[Case], preds: dict, stds: dict) -> dict:
    errs = cell_errors(cases, preds, stds)
    table = {f"{k[0]}|{k[1]}|{k[2]}" if len(k) == 3 else f"{k[0]}|{k[1]}": cell_summary(v) for k, v in errs.items()}
    for arm in v2.ARMS:
        cs = [x for x in cases if x.arm == arm]
        if not cs:
            continue
        p = np.array([preds[x.key]["shift"] for x in cs])
        t = np.array([x.shift for x in cs])
        nz = t != 0
        node = table[f"{arm}|shift"]
        node["right_direction"] = float(np.mean(np.sign(p[nz]) == np.sign(t[nz]))) if nz.any() else None
        node["censored_arm_true"] = float(np.mean([x.censored[0] for x in cs]))
        node["censored_factual_true"] = float(np.mean([x.censored[1] for x in cs]))
        node["mean_true_shift"], node["mean_pred_shift"] = float(t.mean()), float(p.mean())
    return {"cells": table}


W3_CELLS = [(arm, o, "30d") for arm in v2.W3_ARMS for o in v2.OUTCOMES]
W4_CELLS = [(arm, "shift") for arm in v2.W4_ARMS]


def w3_cells(stds: dict) -> list:
    return [cell for cell in W3_CELLS if stds[cell] is not None]


def select_baseline(val_cases: list[Case], val_preds: dict, stds: dict, cells: list) -> dict:
    """The member of {zero, arm_mean, an5_arm} with the lowest cell-averaged value on the validation families."""
    vals = {}
    for name, preds in val_preds.items():
        vals[name] = cell_stat(cell_errors(val_cases, preds, stds), cells, sorted({x.seed for x in val_cases}))
    best = min(("zero", "arm_mean", "an5_arm"), key=lambda n: (vals[n], ("zero", "arm_mean", "an5_arm").index(n)))
    return {"chosen": best, "val": vals}


def distribution(cases: list[Case]) -> dict:
    """Per arm, outcome and lag: the true effect's quantiles over the cases (and the shift)."""
    out = {}
    q = lambda a: {"mean": float(np.mean(a)), "std": float(np.std(a)), "p10": float(np.percentile(a, 10)),
                   "median": float(np.median(a)), "p90": float(np.percentile(a, 90)), "share_nonzero": float(np.mean(a != 0)),
                   "n": int(len(a))}
    for arm in v2.ARMS:
        cs = [x for x in cases if x.arm == arm]
        if not cs:
            continue
        out[arm] = {f"{o}|{lag}": q(np.array([x.truth[o][lag] for x in cs if lag in x.truth[o]]))
                    for o in v2.OUTCOMES for lag in v2.LAGS if any(lag in x.truth[o] for x in cs)}
        out[arm]["shift"] = q(np.array([x.shift for x in cs]))
        out[arm]["censored_arm"] = float(np.mean([x.censored[0] for x in cs]))
        out[arm]["censored_factual"] = float(np.mean([x.censored[1] for x in cs]))
        out[arm]["cast_day"] = q(np.array([float(v2.display_day(x.cast_tick)) for x in cs]))
        out[arm]["no_30d"] = int(sum("30d" not in x.truth["food_stock"] for x in cs))
    return out


def inv_std_of(fact_v: list[np.ndarray]) -> np.ndarray:
    """`climatology.CalendarFit.inv_std` of the factual train worlds (`bench_v0.split_std`: rows >= 24)."""
    std = np.vstack([v[24:] for v in fact_v]).std(axis=0)
    return np.where(std > 0, 1.0 / np.where(std > 0, std, 1.0), 1.0)


def case_json(x: Case) -> dict:
    return {"seed": x.seed, "arm": x.arm, "rep": x.rep, "role": x.role, "cast_tick": x.cast_tick, "cast_row": x.cast_row,
            "truth": x.truth, "shift": x.shift, "censored": list(x.censored), "window": x.window.tolist()}


def case_from(j: dict) -> Case:
    return Case(j["seed"], j["arm"], j["rep"], j["role"], j["cast_tick"], j["cast_row"], j["truth"], j["shift"],
                tuple(j["censored"]), np.array(j["window"]))


def stds_json(stds: dict) -> dict:
    return {"|".join(k): v for k, v in stds.items()}


def stds_from(j: dict) -> dict:
    return {tuple(k.split("|")): v for k, v in j.items()}


def cmd_base(args) -> int:
    t0 = time.time()
    root = Path(args.root)
    train, fact_v = load_cases(root, "train")
    val, _ = load_cases(root, "val")
    test, _ = load_cases(root, "test")
    print(f"cases train {len(train)} val {len(val)} test {len(test)} {time.time() - t0:.0f}s", flush=True)
    inv_std = inv_std_of(fact_v)
    stds = scale_stds(train)
    val_preds = baseline_predictions(val, train, inv_std)
    test_preds = baseline_predictions(test, train, inv_std)
    out = {"code_commit": c.git_commit(), "argv": sys.argv, "inv_std": inv_std.tolist(), "stds": stds_json(stds),
           "dropped_cells": [list(k) for k, v in stds.items() if v is None],
           "cases": {"train": [case_json(x) for x in train], "val": [case_json(x) for x in val],
                     "test": [case_json(x) for x in test]},
           "distribution": {"train": distribution(train), "val": distribution(val), "test": distribution(test)},
           "val_predictions": val_preds, "test_predictions": test_preds,
           "selection": {"W3": select_baseline(val, val_preds, stds, w3_cells(stds)),
                         "W4": select_baseline(val, val_preds, stds, W4_CELLS)},
           "val_scores": {n: score(val, p, stds) for n, p in val_preds.items()},
           "spread": {"val": {"|".join(k): v for k, v in spread(val, stds).items()},
                      "test": {"|".join(k): v for k, v in spread(test, stds).items()}}}
    out["seconds"] = time.time() - t0
    c.write_json(Path(args.out) / "effects_base.json", out)
    print(json.dumps(out["selection"]))
    return 0


# ------------------------------------------------------------------ models ------------------------------------------
def model_predictions(pred, root: Path, cases: list[Case], test_std: np.ndarray, chunk: int = 128) -> tuple[dict, dict]:
    """A model's predicted effects and shifts for every case, and the factual-rollout drift (T2 nMAE of the 8 V at 7,
    30 and 45 days after each start, test split std)."""
    from wm2 import rollout_ext as rx
    by_fam: dict[int, list[Case]] = {}
    for x in cases:
        by_fam.setdefault(x.seed, []).append(x)
    preds, drift = {}, {k: [] for k in DRIFT_LAGS}
    for path in family_dirs(root, "test"):
        fam = load_family(path, verify=True, arrays=ARRAYS)
        mine = by_fam.get(fam.seed, [])
        for r, raw in sorted(fam.factual.items()):
            cs = [x for x in mine if x.rep == r]
            if not cs:
                continue
            ep = d.from_episode(raw, "v19_fam", "test", fam.seed, "none", c.V17_EXTRA)
            truth = v2.member_series(raw)
            zeros = np.zeros_like(ep.bench.actions)
            jobs, keys = [], []
            for row in sorted({x.cast_row for x in cs}):
                jobs.append(rx.Job(ep, row, zeros, v2.LAST_ROW - row))
                keys.append(("factual", row))
            for x in cs:
                act = zeros.copy()
                kind, amount = v2.ARM_SPEC[x.arm]
                act[x.cast_row, b.KINDS.index(kind)] = float(amount)
                jobs.append(rx.Job(ep, x.cast_row, act, v2.LAST_ROW - x.cast_row))
                keys.append(("arm", x.key))
            ros = dict(zip(keys, rx.rollout_jobs(pred, jobs, chunk)))
            for row in sorted({x.cast_row for x in cs}):
                ro = ros[("factual", row)]
                for name, lag in DRIFT_LAGS.items():
                    if row + lag <= v2.LAST_ROW:
                        drift[name].append(np.abs(ro.v[0, lag - 1] - ep.bench.v[row + lag]) / np.where(test_std > 0, test_std, np.nan))
            for x in cs:
                fa, ar = ros[("factual", x.cast_row)], ros[("arm", x.key)]
                eff = {}
                for o in v2.OUTCOMES:
                    eff[o] = {}
                    for lag_name in x.truth[o]:
                        i = v2.LAGS[lag_name] - 1
                        get = {"food_stock": lambda ro: ro.v[0, i, V_FOOD], "alive": lambda ro: ro.v[0, i, V_ALIVE],
                               "deaths": lambda ro: ro.deaths[0, i],
                               "settlements_alive": lambda ro: ro.settlements_alive[0, i]}[o]
                        eff[o][lag_name] = float(get(ar) - get(fa))
                f_true = truth["fissions_total"][: x.cast_row + 1]
                days = {}
                for name, ro in (("arm", ar), ("factual", fa)):
                    days[name] = v2.restricted_split_day(np.concatenate([f_true, ro.fissions_total[0]]), x.cast_row)
                preds[x.key] = {"effects": eff, "shift": float(days["arm"][0] - days["factual"][0]),
                                "censored": [days["arm"][1], days["factual"][1]]}
    drift_out = {k: {"mean": float(np.nanmean(np.vstack(v))) if v else None, "per_var": np.nanmean(np.vstack(v), axis=0).tolist() if v else None,
                     "n": len(v)} for k, v in drift.items()}
    return preds, drift_out


def cmd_model(args) -> int:
    from wm1 import m1_gru as g
    t0 = time.time()
    run = Path(args.run)
    base = json.loads(Path(args.base).read_text())
    cases = [case_from(j) for j in base["cases"]["test"]]
    pred, config = g.load_predictor(run, args.device)
    if config.get("dataset") != "v2_v19":
        raise SystemExit(f"{run}: trained on {config.get('dataset')}, not v2_v19")
    tests = c.load_groups(["test"], Path(args.root), "v2_v19_fact")
    test_std = b.split_std([ep.bench for ep in tests["test"]])
    preds, drift = model_predictions(pred, Path(args.root), cases, test_std)
    out = {"run": str(run), "model": config["model"], "variant": config["variant"], "seed": config["seed"],
           "hyper": config["hyper"], "round_counts": pred.round_counts, "code_commit": c.git_commit(),
           "train_code_commit": config.get("code_commit"), "argv": sys.argv, "predictions": preds, "drift": drift,
           "test_std": test_std.tolist(), "base_sha": _sha(Path(args.base)), "seconds": time.time() - t0}
    target = Path(args.out) / "effects.json"
    if target.exists():
        raise SystemExit(f"{target} exists: test effects are computed once per run")
    c.write_json(target, out)
    print(f"effects done {time.time() - t0:.0f}s; drift {json.dumps({k: v['mean'] for k, v in drift.items()})}")
    return 0


def _sha(path: Path) -> str:
    import hashlib
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("base")
    p.add_argument("--root", default=str(c.DEFAULT_ROOT))
    p.add_argument("--out", required=True)
    p = sub.add_parser("model")
    p.add_argument("--run", required=True)
    p.add_argument("--base", required=True, help="effects_base.json (the test cases and their truths)")
    p.add_argument("--root", default=str(c.DEFAULT_ROOT))
    p.add_argument("--device", default="auto")
    p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    return cmd_base(a) if a.cmd == "base" else cmd_model(a)


if __name__ == "__main__":
    raise SystemExit(main())
