#!/usr/bin/env python3
"""WM-Bench v1 (PASS8_GOAL.md section 11.2): baselines on dataset v1. numpy only.

usage: bench_v1.py [--root WM_DATA_ROOT] [--out docs/worldmodel] [--manifest docs/worldmodel/dataset_v1_manifest.json]
                   [--v0 docs/worldmodel/bench_v0.json] [--burn 2,3,4] [--skip-t5]

Changes from v0 (`bench_v0.py`):
- **Inputs are observable fields only** (`Episode.observable`, recorder schema 2): `winter_severity`, the night
  burn, the day's fall, `wood_target` and `latent.npy` are never inputs. A "severity oracle" variant of the ridge
  (and of the T3/T3b baselines) reads the hidden severity and is labelled as an upper reference.
- T1 as v0. T2 starts at every hour of one-based days 3, 10, 17, ... (24 starts per start day), horizons 24 h
  and 168 h; v0's 06:00-only number is reported beside it.
- T3 as v0 ("arithmetic: 18:00 stock vs tonight's rate", whose rule needs the hidden severity, so it is an
  oracle reference), plus a logistic regression on observable inputs only.
- T3b winter outlook: at 06:00 of each winter's third day (one-based days 24, 52, 80, 108) predict how many of the
  five remaining nights go dark; MAE and exact accuracy; baselines train mean, ridge on observable features, and
  the severity oracle (train mean per severity, a reference).
- T4 per variant and target: MAE, sign accuracy and normalized MAE (MAE / mean |true effect| over the test seeds;
  undefined when that mean is 0). Ranked on dark nights, cold member-hours and food store; deaths reported only;
  no mean across units.
- T5 agent next hour on the A agent episodes: next state tag and target kind accuracy, next position Chebyshev
  error, next hunger/thirst/fatigue nMAE; baselines persistence and 24 hours earlier.
"""
from __future__ import annotations

import argparse
import json
import os
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

from bench_v0 import LAGS, LAMBDAS, KINDS, Logistic, Ridge, actions_window, auc, brier, nmae, onehot
from wm_data import load_episode

ROOT_DIR = Path(__file__).resolve().parents[2]
VARS = ("food_stock", "wood_stock", "fire_lit", "cold_members", "alive", "deadwood_total", "food_on_bushes_total",
        "adults_working")
TPD, OFFSET, EVERY = 1440, 480, 60
SPLIT_DIRS = ("A_obs", "B_act", "C_cf", "OOD_v13", "OOD_long")
T4_TARGETS = ("dark_nights", "cold_member_hours", "food_store", "deaths")
T4_RANKED = ("dark_nights", "cold_member_hours", "food_store")
T5_AGENTS = {"train": range(10000, 10032), "val": range(10096, 10100), "test": range(10112, 10120)}


def row_of(day0: int, minute: int) -> int:
    """Row (every 60) of zero-based display day `day0` at minute-of-day `minute`."""
    return (day0 * TPD + minute - OFFSET) // EVERY


class HiddenInput(KeyError):
    """Raised when a baseline asks for a hidden field as an input."""


def observable_column(ep, array: str, name: str) -> np.ndarray:
    """Column `name` of `array`, refused unless the episode lists it as observable."""
    if name not in ep.observable_names(array):
        raise HiddenInput(f"{array}:{name} is hidden and may not be an input")
    return ep.arrays[array][..., ep.field_index(array, name)]


@dataclass
class Ep:
    split: str
    role: str
    seed: int
    eid: str
    v: np.ndarray            # [T, 8] observable
    tick: np.ndarray         # [T]
    hour: np.ndarray         # [T]
    season: np.ndarray       # [T]
    actions: np.ndarray      # [T, 7] requested amounts (interventions.json is observable)
    wood_stock: np.ndarray   # [T]
    members: np.ndarray      # [T]
    cold_hours: np.ndarray   # [T]
    deaths: np.ndarray       # [T]
    hidden_severity: np.ndarray  # [T] HIDDEN: only the oracle references and labels read it
    agents: dict = field(default_factory=dict)


def episode_arrays(ep) -> dict:
    """Per-row arrays of one loaded episode; every input column goes through `observable_column`."""
    g = lambda name: observable_column(ep, "global.npy", name)
    s = ep.arrays["settlements.npy"]
    t = ep.arrays["global.npy"].shape[0]
    sf = (lambda name: observable_column(ep, "settlements.npy", name)) if s.shape[1] else (lambda name: np.zeros((t, 0)))
    food = np.nansum(sf("food_stock"), axis=1)
    wood = np.nansum(sf("wood_stock"), axis=1)
    fire = np.nan_to_num(sf("fire_lit"), nan=0.0).max(axis=1, initial=0.0)
    members = np.nansum(sf("members"), axis=1)
    v = np.stack([food, wood, fire, g("cold_members"), g("alive"), g("deadwood_total"), g("food_on_bushes_total"),
                  g("adults_working")], axis=1).astype(np.float64)
    tick = g("tick").astype(np.int64)
    actions = np.zeros((len(tick), len(KINDS)))
    index = {int(x): i for i, x in enumerate(tick)}
    for rec in ep.interventions:
        if int(rec["tick"]) in index:
            actions[index[int(rec["tick"])], KINDS.index(rec["kind"])] += float(rec["amount"])
    deaths = sum(g(k) for k in ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age"))
    severity = ep.column("global.npy", "winter_severity").astype(np.int64)  # hidden: labels and oracles only
    return dict(v=v, tick=tick, hour=(g("minute_of_day") // 60).astype(np.int64), season=g("season").astype(np.int64),
                actions=actions, wood_stock=wood, members=members, cold_hours=g("cold_member_hours_total"),
                deaths=deaths, hidden_severity=severity)


def agent_arrays(ep) -> dict:
    """The T5 agent table (observable columns only)."""
    names = ("x", "y", "state_tag", "target_kind", "hunger", "thirst", "fatigue")
    return {"alive": ep.arrays["agent_alive.npy"].astype(bool),
            **{name: observable_column(ep, "agents.npy", name) for name in names}}


def load_split(root: Path, manifest: dict, split: str, agents: bool = False) -> list[Ep]:
    eps = []
    for entry in manifest["episodes"]:
        if entry["split"] != split:
            continue
        want = ["global.npy", "settlements.npy"]
        if agents and entry["agents"]:
            want += ["agents.npy", "agent_alive.npy"]
        ep = load_episode(root / entry["path"], arrays=want)
        e = Ep(split, entry["role"], entry["seed"], entry["schedule_id"], **episode_arrays(ep))
        if agents and entry["agents"]:
            e.agents = agent_arrays(ep)
        eps.append(e)
    return eps


# ------------------------------------------------------------------ features / ridge ----------------------------
def features(v, hour, season, t, actions24=None, severity=None) -> np.ndarray:
    """Rows `t` (each >= 23): lags of V, hour and season one-hots, optionally the last-24-hour action sums and
    (severity oracle only) the hidden severity one-hot."""
    parts = [v[t - lag + 1] for lag in LAGS]
    parts += [onehot(hour[t], list(range(24))), onehot(season[t], [-1, 0, 1, 2, 3])]
    if actions24 is not None:
        parts.append(actions24[t])
    if severity is not None:
        parts.append(onehot(severity[t], [-1, 0, 1, 2]))
    return np.concatenate(parts, axis=1)


@dataclass(frozen=True)
class Variant:
    name: str
    action_aware: bool = False
    oracle: bool = False


RIDGES = (Variant("ridge"), Variant("ridge_action", action_aware=True),
          Variant("ridge_severity_oracle", oracle=True))


def episode_xy(ep: Ep, variant: Variant):
    t = np.arange(23, len(ep.tick) - 1)
    x = features(ep.v, ep.hour, ep.season, t, actions_window(ep.actions) if variant.action_aware else None,
                 ep.hidden_severity if variant.oracle else None)
    return x, ep.v[t + 1] - ep.v[t], t


def split_std(eps: list[Ep]) -> np.ndarray:
    return np.vstack([ep.v[24:] for ep in eps]).std(axis=0)


def t1_scores(eps, models: dict, std) -> dict:
    preds = {"persistence": [], "daily_naive": [], **{v.name: [] for v in models}}
    trues = []
    for ep in eps:
        t = np.arange(23, len(ep.tick) - 1)
        trues.append(ep.v[t + 1])
        preds["persistence"].append(ep.v[t])
        preds["daily_naive"].append(ep.v[t + 1 - 24])
        for variant, model in models.items():
            x, _, _ = episode_xy(ep, variant)
            preds[variant.name].append(ep.v[t] + model.predict(x))
    true = np.vstack(trues)
    return {name: nmae(np.vstack(p), true, std) for name, p in preds.items() if p}


def fit_ridge(train, val, variant: Variant):
    x, y = (np.vstack(a) for a in zip(*[episode_xy(ep, variant)[:2] for ep in train]))
    std = split_std(val)
    best, scores = None, {}
    for lam in LAMBDAS:
        model = Ridge().fit(x, y, lam)
        score = float(np.nanmean(t1_scores(val, {variant: model}, std)[variant.name]))
        scores[str(lam)] = score
        if best is None or score < best[0]:
            best = (score, lam, model)
    return best[1], best[2], scores


# ------------------------------------------------------------------ T2 -------------------------------------------
def rollout_starts(n_rows: int, horizon: int, every_hour: bool) -> list[int]:
    """Rows of one-based days 3, 10, 17, ...: every hour (24 per day) or 06:00 only (v0), whose horizon fits."""
    starts = []
    day1 = 3
    while True:
        rows = [row_of(day1 - 1, 60 * h) for h in (range(24) if every_hour else (6,))]
        rows = [r for r in rows if r >= 23 and r + horizon < n_rows]
        if not rows:
            break
        starts.extend(rows)
        day1 += 7
    return starts


def rollout_batch(ep: Ep, starts: np.ndarray, horizon: int, model: Ridge, variant: Variant) -> np.ndarray:
    """Autoregressive ridge rollouts of every start at once; V after `horizon` steps, one row per start. The calendar
    follows the clock; actions are those recorded; the oracle holds the start row's severity."""
    buf = np.zeros((len(starts), 24 + horizon, ep.v.shape[1]))
    for i, s in enumerate(starts):
        buf[i, :24] = ep.v[s - 23: s + 1]
    acts = actions_window(ep.actions) if variant.action_aware else None
    for k in range(horizon):
        rows = starts + k
        parts = [buf[:, 23 + k - lag + 1] for lag in LAGS]
        parts += [onehot(ep.hour[rows], list(range(24))), onehot(ep.season[rows], [-1, 0, 1, 2, 3])]
        if acts is not None:
            parts.append(acts[rows])
        if variant.oracle:
            parts.append(onehot(ep.hidden_severity[starts], [-1, 0, 1, 2]))
        x = np.concatenate(parts, axis=1)
        buf[:, 24 + k] = buf[:, 23 + k] + model.predict(x)
    return buf[:, 23 + horizon]


def t2_scores(eps, models: dict, std, horizon: int, every_hour: bool) -> dict:
    pred = {"persistence": [], "daily_naive": [], **{v.name: [] for v in models}}
    true = []
    for ep in eps:
        starts = np.array(rollout_starts(len(ep.tick), horizon, every_hour), dtype=np.int64)
        if not len(starts):
            continue
        true.append(ep.v[starts + horizon])
        pred["persistence"].append(ep.v[starts])
        pred["daily_naive"].append(ep.v[starts + horizon - 24 * ((horizon + 23) // 24)])
        for variant, model in models.items():
            pred[variant.name].append(rollout_batch(ep, starts, horizon, model, variant))
    if not true:
        return {}
    t = np.vstack(true)
    return {name: nmae(np.vstack(p), t, std) for name, p in pred.items()}


# ------------------------------------------------------------------ T3 ------------------------------------------
def winter_evenings(ep: Ep, burn, hours: int) -> list[dict]:
    """Every winter evening (18:00 of a winter display day) with the fire state at 02:00 that night."""
    rows = []
    days = int((ep.tick[-1] + OFFSET) // TPD)
    fire, dead = VARS.index("fire_lit"), VARS.index("deadwood_total")
    for d in range(1, days):
        r18, r02 = row_of(d, 1080), row_of(d + 1, 120)
        if r02 >= len(ep.tick) or ep.season[r18] != 3:
            continue
        level = int(ep.hidden_severity[r18])
        rows.append({"wood": ep.wood_stock[r18], "tonight": burn[level if level >= 0 else 1] * hours,
                     "members": ep.members[r18], "deadwood": ep.v[r18, dead], "severity": level, "night": d % 7 + 1,
                     "last_drop": ep.wood_stock[row_of(d - 1, 1080)] - ep.wood_stock[row_of(d, 360)],
                     "dead_change": ep.v[r18, dead] - ep.v[row_of(d - 1, 1080), dead],
                     "out": float(ep.v[r02, fire] == 0), "seed": ep.seed, "day": d})
    return rows


def t3_observable_design(rows) -> np.ndarray:
    return np.column_stack([[r[k] for r in rows] for k in ("wood", "members", "deadwood", "night", "last_drop", "dead_change")])


def t3_oracle_design(rows) -> np.ndarray:
    ratio = np.array([r["wood"] / r["tonight"] for r in rows])
    sev = onehot(np.array([r["severity"] for r in rows]), [0, 1, 2])
    return np.column_stack([ratio, [r["members"] for r in rows], [r["deadwood"] for r in rows], sev,
                            [float(r["night"]) for r in rows]])


def t3_scores(prob, y) -> dict:
    return {"brier": brier(prob, y), "auc": auc(prob, y), "accuracy": float(np.mean((prob >= .5) == (y == 1))),
            "n": int(len(y)), "positives": int(y.sum())}


# ------------------------------------------------------------------ T3b -----------------------------------------
T3B_FEATURES = ("wood_stock", "deadwood_total", "drop_last_night", "drop_night_before", "deadwood_change_1",
                "deadwood_change_2", "members")


def winter_outlooks(ep: Ep) -> list[dict]:
    """At 06:00 of each winter's third day (zero-based 28y + 23): features, the hidden severity and the number of
    dark nights among the five remaining nights (evenings 28y + 23 .. 28y + 27, fire out at the 02:00 row)."""
    out = []
    fire, dead = VARS.index("fire_lit"), VARS.index("deadwood_total")
    year = 0
    while True:
        d = 28 * year + 23
        last = row_of(28 * year + 28, 120)
        if last >= len(ep.tick):
            break
        r = row_of(d, 360)
        if ep.season[r] == 3:
            wood = ep.wood_stock
            out.append({
                "seed": ep.seed, "year": year, "severity": int(ep.hidden_severity[r]),
                "x": [wood[r], ep.v[r, dead], wood[row_of(d - 1, 1080)] - wood[r],
                      wood[row_of(d - 2, 1080)] - wood[row_of(d - 1, 360)],
                      ep.v[r, dead] - ep.v[row_of(d - 1, 360), dead], ep.v[row_of(d - 1, 360), dead] - ep.v[row_of(d - 2, 360), dead],
                      ep.members[r]],
                "dark": int(sum(ep.v[row_of(e + 1, 120), fire] == 0 for e in range(d, d + 5))),
            })
        year += 1
    return out


def t3b_scores(pred: np.ndarray, truth: np.ndarray) -> dict:
    rounded = np.clip(np.rint(pred), 0, 5)
    return {"mae": float(np.abs(pred - truth).mean()), "exact_accuracy": float(np.mean(rounded == truth)), "n": int(len(truth))}


def t3b(train, val, test) -> dict:
    rows = {name: [o for ep in eps for o in winter_outlooks(ep)] for name, eps in (("train", train), ("val", val), ("test", test))}
    x = {k: np.array([o["x"] for o in v], dtype=float) for k, v in rows.items()}
    y = {k: np.array([o["dark"] for o in v], dtype=float) for k, v in rows.items()}
    sev = {k: np.array([o["severity"] for o in v]) for k, v in rows.items()}
    best = None
    lam_scores = {}
    for lam in LAMBDAS:
        model = Ridge().fit(x["train"], y["train"][:, None], lam)
        score = float(np.abs(model.predict(x["val"])[:, 0] - y["val"]).mean())
        lam_scores[str(lam)] = score
        if best is None or score < best[0]:
            best = (score, lam, model)
    mean = float(y["train"].mean())
    per_sev = {level: float(y["train"][sev["train"] == level].mean()) if np.any(sev["train"] == level) else mean
               for level in (0, 1, 2)}
    oracle = np.array([per_sev.get(int(s), mean) for s in sev["test"]])
    hist = {str(level): {str(k): int(np.sum((sev["test"] == level) & (y["test"] == k))) for k in range(6)}
            for level in (0, 1, 2)}
    return {"train_mean": t3b_scores(np.full(len(y["test"]), mean), y["test"]),
            "ridge": t3b_scores(best[2].predict(x["test"])[:, 0], y["test"]),
            "severity_oracle_reference": t3b_scores(oracle, y["test"]),
            "lambda": best[1], "val_scores": lam_scores, "features": list(T3B_FEATURES),
            "train_n": int(len(y["train"])), "test_dark_by_severity": hist, "train_mean_value": mean,
            "oracle_means": {str(k): v for k, v in per_sev.items()}}


# ------------------------------------------------------------------ T4 ------------------------------------------
def c_cf_targets(ep: Ep) -> dict[str, float]:
    """First-winter outcomes (as v0): dark nights of evenings of one-based days 22-28, cold member-hours from day 22
    18:00 to day 29 07:00, food store at day 29 06:00, deaths up to then."""
    dark = sum(int(ep.v[row_of(d + 1, 120), VARS.index("fire_lit")] == 0) for d in range(21, 28))
    start, end, food = row_of(21, 1080), row_of(28, 420), row_of(28, 360)
    return {"dark_nights": float(dark), "cold_member_hours": float(ep.cold_hours[end] - ep.cold_hours[start]),
            "food_store": float(ep.v[food, VARS.index("food_stock")]), "deaths": float(ep.deaths[food])}


def c_cf_effects(eps) -> dict:
    by_seed: dict[int, dict[str, Ep]] = {}
    for ep in eps:
        by_seed.setdefault(ep.seed, {})[ep.eid] = ep
    effects: dict = {}
    for seed, group in sorted(by_seed.items()):
        base = c_cf_targets(group["none"])
        for variant, ep in group.items():
            if variant != "none":
                target = c_cf_targets(ep)
                effects.setdefault(variant, {})[seed] = {k: target[k] - base[k] for k in T4_TARGETS}
    return effects


def t4_scores(test: dict, val: dict) -> dict:
    """Per variant, baseline and target: MAE, sign accuracy and nMAE = MAE / mean |true effect| (None when 0)."""
    out = {}
    for variant, seeds in test.items():
        truth = np.array([[e[k] for k in T4_TARGETS] for e in seeds.values()])
        mean = np.array([np.mean([e[k] for e in val.get(variant, {}).values()]) if val.get(variant) else 0.0
                         for k in T4_TARGETS])
        scale = np.abs(truth).mean(axis=0)
        out[variant] = {}
        for name, pred in (("zero_effect", np.zeros_like(truth)), ("val_mean_effect", np.broadcast_to(mean, truth.shape))):
            out[variant][name] = {}
            for i, k in enumerate(T4_TARGETS):
                mae = float(np.abs(pred[:, i] - truth[:, i]).mean())
                out[variant][name][k] = {"mae": mae, "sign_accuracy": float(np.mean(np.sign(pred[:, i]) == np.sign(truth[:, i]))),
                                         "nmae": mae / float(scale[i]) if scale[i] > 0 else None,
                                         "ranked": k in T4_RANKED}
    return out


def effect_distribution(effects: dict) -> dict:
    out = {}
    for variant, seeds in effects.items():
        out[variant] = {}
        for k in T4_TARGETS:
            vals = np.array([e[k] for e in seeds.values()])
            out[variant][k] = {"mean": float(vals.mean()), "mean_abs": float(np.abs(vals).mean()), "median": float(np.median(vals)),
                               "min": float(vals.min()), "max": float(vals.max()), "nonzero_share": float(np.mean(vals != 0))}
        out[variant]["changed_ranked_share"] = float(np.mean([any(e[k] != 0 for k in T4_RANKED) for e in seeds.values()]))
    return out


# ------------------------------------------------------------------ T5 ------------------------------------------
def t5_pairs(eps, lag: int):
    """(truth at t+1, prediction from row t+1-lag) for every agent alive at t+1 and at t+1-lag."""
    out = {k: ([], []) for k in ("x", "y", "state_tag", "target_kind", "hunger", "thirst", "fatigue")}
    for ep in eps:
        a = ep.agents
        alive = a["alive"]
        n = alive.shape[0]
        for t in range(23, n - 1):
            src = t + 1 - lag
            mask = alive[t + 1] & alive[src]
            for k in out:
                out[k][0].append(a[k][t + 1][mask])
                out[k][1].append(a[k][src][mask])
    return {k: (np.concatenate(v[0]), np.concatenate(v[1])) for k, v in out.items()}


def t5_scores(eps) -> dict:
    result = {}
    truth_std = None
    for name, lag in (("persistence", 1), ("day_earlier", 24)):
        p = t5_pairs(eps, lag)
        if truth_std is None:
            truth_std = {k: float(p[k][0].std()) for k in ("hunger", "thirst", "fatigue")}
        cheb = np.maximum(np.abs(p["x"][0] - p["x"][1]), np.abs(p["y"][0] - p["y"][1]))
        result[name] = {
            "state_tag_accuracy": float(np.mean(p["state_tag"][0] == p["state_tag"][1])),
            "target_kind_accuracy": float(np.mean(p["target_kind"][0] == p["target_kind"][1])),
            "position_chebyshev_mean": float(cheb.mean()),
            **{f"{k}_nmae": float(np.abs(p[k][0] - p[k][1]).mean() / truth_std[k]) if truth_std[k] > 0 else None
               for k in ("hunger", "thirst", "fatigue")},
            "n": int(len(cheb)),
        }
    return result


# ------------------------------------------------------------------ driver --------------------------------------
def run(root: Path, manifest: dict, burn, hours: int, skip_t5: bool = False) -> dict:
    present = {e["split"] for e in manifest["episodes"]}
    data = {s: load_split(root, manifest, s, agents=(s == "A_obs" and not skip_t5)) for s in SPLIT_DIRS if s in present}
    role = lambda split, r: [e for e in data.get(split, []) if e.role == r]
    a_train, a_val, a_test = role("A_obs", "train"), role("A_obs", "val"), role("A_obs", "test")
    b_train, b_val, b_test = role("B_act", "train"), role("B_act", "val"), role("B_act", "test")
    tests = {"A_test": a_test, "B_test": b_test, "OOD_v13": data.get("OOD_v13", []), "OOD_long": data.get("OOD_long", [])}
    tests = {k: v for k, v in tests.items() if v}
    stds = {name: split_std(eps) for name, eps in tests.items()}
    models, lambdas = {}, {}
    for variant in RIDGES:
        train, val = (a_train + b_train, a_val + b_val) if variant.action_aware else (a_train, a_val)
        lam, model, scores = fit_ridge(train, val, variant)
        models[variant], lambdas[variant.name] = model, {"lambda": lam, "val_scores": scores}
    report: dict = {"vars": VARS, "lambda": lambdas,
                    "inputs": "observable fields only (Episode.observable); ridge_severity_oracle adds the hidden severity (reference)"}
    report["T1"] = {name: {k: v.tolist() for k, v in t1_scores(eps, models, stds[name]).items()} for name, eps in tests.items()}
    report["T2"] = {}
    for horizon in (24, 168):
        for mode, every in (("every_hour", True), ("0600_only", False)):
            report["T2"].setdefault(str(horizon), {})[mode] = {
                name: {k: v.tolist() for k, v in t2_scores(eps, models, stds[name], horizon, every).items()}
                for name, eps in tests.items()}
    train_rows = [r for ep in a_train for r in winter_evenings(ep, burn, hours)]
    test_rows = [r for ep in a_test for r in winter_evenings(ep, burn, hours)]
    y_train, y_test = np.array([r["out"] for r in train_rows]), np.array([r["out"] for r in test_rows])
    rule = np.array([float(r["wood"] < r["tonight"]) for r in test_rows])
    obs = Logistic().fit(t3_observable_design(train_rows), y_train)
    oracle = Logistic().fit(t3_oracle_design(train_rows), y_train)
    report["T3"] = {"rule_severity_oracle": t3_scores(rule, y_test),
                    "logistic_observable": t3_scores(obs.predict(t3_observable_design(test_rows)), y_test),
                    "logistic_severity_oracle": t3_scores(oracle.predict(t3_oracle_design(test_rows)), y_test),
                    "base_rate_train": float(y_train.mean()), "train_n": int(len(y_train))}
    report["T3b"] = t3b(a_train, a_val, a_test)
    cf = data.get("C_cf", [])
    val_eff = c_cf_effects([e for e in cf if e.role == "val"])
    test_eff = c_cf_effects([e for e in cf if e.role == "test"])
    report["T4"] = t4_scores(test_eff, val_eff)
    report["T4_effects_test"] = effect_distribution(test_eff)
    report["T4_effects_val"] = effect_distribution(val_eff)
    if not skip_t5:
        agent_eps = {r: [e for e in data["A_obs"] if e.agents and e.seed in seeds] for r, seeds in T5_AGENTS.items()}
        report["T5"] = {"test": t5_scores(agent_eps["test"]), "val": t5_scores(agent_eps["val"]),
                        "episodes": {r: [e.seed for e in eps] for r, eps in agent_eps.items()}}
    report["sanity"] = {
        "ridge_beats_persistence_T1_food_wood": {
            name: [report["T1"][name]["ridge"][i] < report["T1"][name]["persistence"][i] for i in (0, 1)] for name in tests},
        "T3b_ridge_beats_train_mean": report["T3b"]["ridge"]["mae"] < report["T3b"]["train_mean"]["mae"],
        "every_variant_changes_a_ranked_target_in_25pct": {
            v: d["changed_ranked_share"] >= .25 for v, d in report["T4_effects_test"].items()},
        "severity_oracle_T3b_mae_nonzero": report["T3b"]["severity_oracle_reference"]["mae"] > 0,
    }
    report["counts"] = {name: len(eps) for name, eps in data.items()} | {"A_train": len(a_train), "B_train": len(b_train)}
    return report


def fmt(x) -> str:
    return "—" if x is None or (isinstance(x, float) and np.isnan(x)) else f"{x:.3f}"


def mean_of(vals) -> float:
    return float(np.nanmean(np.array(vals, dtype=float)))


def table(scores: dict, v) -> list[str]:
    lines = ["| Baseline | " + " | ".join(v) + " | mean |", "|---|" + "---:|" * (len(v) + 1)]
    for name, vals in scores.items():
        label = f"{name} (reference: reads hidden severity)" if "oracle" in name else name
        lines.append(f"| {label} | " + " | ".join(fmt(x) for x in vals) + f" | {fmt(mean_of(vals))} |")
    return lines


def markdown(r: dict, v0: dict | None) -> str:
    v = r["vars"]
    lam = {k: x["lambda"] for k, x in r["lambda"].items()}
    lines = ["# WM-Bench v1", "",
             "Second benchmark for WorldSim world models, on dataset v1 (`DATASET_v1.md`, `game_v14`: winters whose "
             "severity, nightly burn and daily deadwood fall are hidden). Produced by `tools/worldmodel/bench_v1.py` "
             "(numpy only). Numbers are nMAE = MAE / standard deviation of the variable over the split's scored rows "
             "(lower is better) unless stated.", "",
             "**Inputs are observable fields only** (`wm_data.Episode.observable`): `winter_severity`, "
             "`night_burn_per_hour`, `winter_fall_today`, `wood_target` and `latent.npy` are never inputs. Rows marked "
             "*reference* read the hidden severity (an oracle) and bound what reading the weather is worth; they are not "
             "numbers to beat.", "",
             f"Episodes: {json.dumps(r['counts'])}. Ridge λ chosen on validation: {json.dumps(lam)}.", "",
             "Variables V: " + ", ".join(f"`{x}`" for x in v) + ".", "",
             "## T1 — next hour", "", "Predict V at row t+1 from rows up to t. Train: A train (action-aware: A + B train).", ""]
    for split, scores in r["T1"].items():
        lines += [f"### {split}", "", *table(scores, v), ""]
    lines += ["## T2 — rollout", "",
              "From every hour of one-based days 3, 10, 17, … (24 starts per start day; `every_hour`) and, as in v0, from "
              "06:00 only (`0600_only`), roll the one-step model forward 24 h and 168 h; nMAE at the horizon. The oracle "
              "holds the start row's severity.", ""]
    for horizon, modes in r["T2"].items():
        for mode, splits in modes.items():
            for split, scores in splits.items():
                lines += [f"### {horizon} h — {mode} — {split}", "", *table(scores, v), ""]
    t3 = r["T3"]
    lines += ["## T3 — dark night (arithmetic: 18:00 stock vs tonight's rate)", "",
              "At 18:00 of every winter evening of A test: P(fire out at 02:00). The rule (stock < severity rate × 13) needs "
              "the hidden severity and is a reference; `logistic_observable` reads the stock, members, deadwood, night index, "
              "last night's stock drop and the deadwood change since yesterday; `logistic_severity_oracle` is v0's logistic. "
              f"Fit on A train ({t3['train_n']} evenings, base rate {t3['base_rate_train']:.3f}).", "",
              "| Baseline | Brier | AUC | Accuracy | Evenings | Dark |", "|---|---:|---:|---:|---:|---:|"]
    for name in ("logistic_observable", "rule_severity_oracle", "logistic_severity_oracle"):
        s = t3[name]
        label = f"{name} (reference)" if "oracle" in name else name
        lines.append(f"| {label} | {fmt(s['brier'])} | {fmt(s['auc'])} | {fmt(s['accuracy'])} | {s['n']} | {s['positives']} |")
    b = r["T3b"]
    lines += ["", "## T3b — winter outlook", "",
              "At 06:00 of each winter's third day (one-based days 24, 52, 80, 108) of A test: how many of the five remaining "
              "nights go dark (fire out at 02:00)? Ridge features (observable): " + ", ".join(f"`{f}`" for f in b["features"])
              + f" (λ {b['lambda']} on A val). Train: {b['train_n']} winters; train mean {b['train_mean_value']:.3f}.", "",
              "| Baseline | MAE | Exact accuracy | Winters |", "|---|---:|---:|---:|"]
    for name in ("train_mean", "ridge", "severity_oracle_reference"):
        s = b[name]
        lines.append(f"| {name} | {fmt(s['mae'])} | {fmt(s['exact_accuracy'])} | {s['n']} |")
    lines += ["", "Dark nights of the five by severity (A test, count of winters): " + json.dumps(b["test_dark_by_severity"]), ""]
    lines += ["## T4 — counterfactual", "",
              "On C_cf test seeds, each variant's effect (variant − factual) on the first winter: dark nights, cold "
              "member-hours over those nights, food store at day 29 06:00 (ranked), deaths up to then (reported only). "
              "nMAE = MAE / mean |true effect| over the test seeds (— when every effect is 0). No mean across units.", ""]
    for variant, preds in r["T4"].items():
        lines += [f"### {variant}", "", "| Baseline | Target | MAE | Sign accuracy | nMAE |", "|---|---|---:|---:|---:|"]
        for name, scores in preds.items():
            for k in T4_TARGETS:
                s = scores[k]
                tag = "" if s["ranked"] else " (reported only)"
                lines.append(f"| {name} | {k}{tag} | {fmt(s['mae'])} | {fmt(s['sign_accuracy'])} | {fmt(s['nmae'])} |")
        lines.append("")
    lines += ["### Ground-truth effects (C_cf test)", "", "| Variant | Target | Mean | Mean abs | Min | Max | Non-zero share |",
              "|---|---|---:|---:|---:|---:|---:|"]
    for variant, d in r["T4_effects_test"].items():
        for k in T4_TARGETS:
            e = d[k]
            lines.append(f"| {variant} | {k} | {fmt(e['mean'])} | {fmt(e['mean_abs'])} | {fmt(e['min'])} | {fmt(e['max'])} | {fmt(e['nonzero_share'])} |")
        lines.append(f"| {variant} | any ranked target changed | | | | | {fmt(d['changed_ranked_share'])} |")
    if "T5" in r:
        lines += ["", "## T5 — agent next hour", "",
                  f"A agent episodes (test seeds {r['T5']['episodes']['test']}): every agent alive at t+1 and at the source row. "
                  "Persistence predicts row t, `day_earlier` row t+1−24. nMAE = MAE / std of the true next value.", "",
                  "| Baseline | State tag acc. | Target kind acc. | Position Chebyshev | Hunger nMAE | Thirst nMAE | Fatigue nMAE | Pairs |",
                  "|---|---:|---:|---:|---:|---:|---:|---:|"]
        for name, s in r["T5"]["test"].items():
            lines.append(f"| {name} | {fmt(s['state_tag_accuracy'])} | {fmt(s['target_kind_accuracy'])} | {fmt(s['position_chebyshev_mean'])} | "
                         f"{fmt(s['hunger_nmae'])} | {fmt(s['thirst_nmae'])} | {fmt(s['fatigue_nmae'])} | {s['n']} |")
    s = r["sanity"]
    lines += ["", "## Sanity checks", "",
              f"- Ridge beats persistence on T1 food and wood stock: {json.dumps(s['ridge_beats_persistence_T1_food_wood'])}",
              f"- T3b ridge beats the train mean: {s['T3b_ridge_beats_train_mean']}",
              f"- Every C_cf variant changes at least one ranked target in ≥ 25 % of test seeds: {json.dumps(s['every_variant_changes_a_ranked_target_in_25pct'])}",
              f"- The severity oracle's T3b MAE is not 0: {s['severity_oracle_T3b_mae_nonzero']} (MAE {fmt(b['severity_oracle_reference']['mae'])})", ""]
    if v0:
        lines += side_by_side(r, v0)
    return "\n".join(lines)


def best_of(scores: dict) -> tuple[float, str]:
    return min((mean_of(v), k) for k, v in scores.items() if "oracle" not in k)


def side_by_side(r: dict, v0: dict) -> list[str]:
    lines = ["## v0 and v1 side by side", "",
             "Best non-oracle baseline (mean nMAE over V for T1/T2). v0: dataset v0 (`game_v13`, severity an input); "
             "v1: dataset v1 (`game_v14`, observable inputs only). OOD_v12 (v0) and OOD_v13 (v1) are each pass's previous "
             "ruleset.", "", "| Task | Split | v0 | v1 |", "|---|---|---:|---:|"]
    pair = {"A_test": "A_test", "B_test": "B_test", "OOD_long": "OOD_long", "OOD_v13": "OOD_v12"}
    for split1, split0 in pair.items():
        if split1 in r["T1"] and split0 in v0["T1"]:
            a, an = best_of(v0["T1"][split0])
            b, bn = best_of(r["T1"][split1])
            lines.append(f"| T1 next hour | {split1} (v0 {split0}) | {a:.3f} ({an}) | {b:.3f} ({bn}) |")
    for horizon in ("24", "168"):
        for split1, split0 in pair.items():
            if split1 in r["T2"][horizon]["0600_only"] and split0 in v0["T2"][horizon]:
                a, an = best_of(v0["T2"][horizon][split0])
                b, bn = best_of(r["T2"][horizon]["0600_only"][split1])
                c, cn = best_of(r["T2"][horizon]["every_hour"][split1])
                lines.append(f"| T2 {horizon} h, 06:00 starts | {split1} | {a:.3f} ({an}) | {b:.3f} ({bn}); every hour {c:.3f} ({cn}) |")
    t30, t31 = v0["T3"], r["T3"]
    lines.append(f"| T3 Brier / AUC | A_test | logistic (severity input) {t30['logistic']['brier']:.3f} / {t30['logistic']['auc']:.3f} | "
                 f"observable {t31['logistic_observable']['brier']:.3f} / {t31['logistic_observable']['auc']:.3f}; "
                 f"oracle {t31['logistic_severity_oracle']['brier']:.3f} / {t31['logistic_severity_oracle']['auc']:.3f} |")
    for variant in v0["T4"]:
        if variant in r["T4"]:
            for k in T4_RANKED:
                a = min(v0["T4"][variant][n][k]["mae"] for n in v0["T4"][variant])
                b = min(r["T4"][variant][n][k]["mae"] for n in r["T4"][variant])
                lines.append(f"| T4 {variant} {k} MAE | C_cf test | {a:.3f} | {b:.3f} |")
    lines.append("")
    return lines


def to_beat(r: dict) -> str:
    lines = ["## Numbers to beat (WM-1)", "",
             "- **Inputs.** Observable fields only (`Episode.observable` of each array, `interventions.json` for actions). "
             "A model that reads `winter_severity`, `night_burn_per_hour`, `winter_fall_today`, `wood_target`, "
             "`latent.npy` or `lifespan_days` is an oracle and must say so.",
             "- **Splits.** Fit on `train` (A_obs; A + B for action-aware), choose on `val`, report on `test`, OOD_v13 and "
             "OOD_long untouched; C_cf val may calibrate T4.",
             "- **Best non-oracle baseline:**"]
    for split, scores in r["T1"].items():
        value, name = best_of(scores)
        lines.append(f"  - T1 {split}: mean nMAE {value:.3f} ({name})")
    for horizon, modes in r["T2"].items():
        for split, scores in modes["every_hour"].items():
            if scores:
                value, name = best_of(scores)
                lines.append(f"  - T2 {horizon} h every-hour starts {split}: {value:.3f} ({name})")
    t3 = r["T3"]["logistic_observable"]
    lines.append(f"  - T3 (observable): Brier {t3['brier']:.3f}, AUC {t3['auc']:.3f}")
    b = r["T3b"]
    top = min(("train_mean", "ridge"), key=lambda k: b[k]["mae"])
    lines.append(f"  - T3b: MAE {b[top]['mae']:.3f}, exact accuracy {b[top]['exact_accuracy']:.3f} ({top}); the severity "
                 f"oracle reference reaches MAE {b['severity_oracle_reference']['mae']:.3f}")
    for variant, preds in r["T4"].items():
        parts = []
        for k in T4_RANKED:
            name = min(preds, key=lambda n: preds[n][k]["mae"])
            s = preds[name][k]
            parts.append(f"{k} MAE {s['mae']:.3f} (nMAE {fmt(s['nmae'])}, {name})")
        lines.append(f"  - T4 {variant}: " + "; ".join(parts))
    if "T5" in r:
        p = r["T5"]["test"]["persistence"]
        lines.append(f"  - T5 (persistence): state tag {p['state_tag_accuracy']:.3f}, target kind {p['target_kind_accuracy']:.3f}, "
                     f"Chebyshev {p['position_chebyshev_mean']:.3f}, hunger/thirst/fatigue nMAE {fmt(p['hunger_nmae'])} / "
                     f"{fmt(p['thirst_nmae'])} / {fmt(p['fatigue_nmae'])}")
    lines.append("")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=Path(os.environ.get("WM_DATA_ROOT", "/Users/rexxa/work/worldsim-wm-data")))
    parser.add_argument("--out", type=Path, default=ROOT_DIR / "docs/worldmodel")
    parser.add_argument("--manifest", type=Path, default=ROOT_DIR / "docs/worldmodel/dataset_v1_manifest.json")
    parser.add_argument("--v0", type=Path, default=ROOT_DIR / "docs/worldmodel/bench_v0.json")
    parser.add_argument("--burn", default="2,3,4", help="winter_severity_burn_per_hour of game_v14() (T3 oracle rule)")
    parser.add_argument("--burn-hours", type=int, default=13)
    parser.add_argument("--skip-t5", action="store_true")
    args = parser.parse_args(argv)
    burn = tuple(int(x) for x in args.burn.split(","))
    report = run(args.root, json.loads(args.manifest.read_text()), burn, args.burn_hours, args.skip_t5)
    v0 = json.loads(args.v0.read_text()) if args.v0 and args.v0.exists() else None
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "bench_v1.json").write_text(json.dumps(report, indent=1, sort_keys=True, default=float) + "\n")
    (args.out / "BENCH_v1.md").write_text(markdown(report, v0) + "\n" + to_beat(report))
    print(json.dumps(report["sanity"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
