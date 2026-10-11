#!/usr/bin/env python3
"""WM-Bench v0: tasks and baselines on dataset v0 (PASS7_GOAL.md section 12.2). numpy only.

usage: bench_v0.py [--root WM_DATA_ROOT] [--out docs/worldmodel] [--burn 2,3,4] [--burn-hours 13]

Reads WM_DATA_ROOT/v0/<split>/<episode> (see DATASET_v0.md) and writes BENCH_v0.md and
bench_v0.json into --out. Settlement level throughout; the eight variables V are
food stock, wood stock, fire lit (summed / max over settlements, 0 before the first
settlement forms), cold members, alive, deadwood total, food on bushes and adults working.

T1 next hour     predict V at row t+1 from rows <= t; nMAE = MAE / std of the variable over the test rows.
T2 rollout       from 06:00 of days 3, 10, 17, ... predict 24 h and 168 h ahead autoregressively; nMAE at the horizon.
T3 dark night    at 18:00 of every winter evening of A test, P(fire out at 02:00); Brier, rank AUC, accuracy at 0.5.
T4 counterfactual  on C_cf test seeds, each variant's effect on four targets; MAE and sign accuracy.

Baselines: persistence; daily naive (24 h earlier); ridge on lags {1,2,3,6,12,24} of V plus hour, season and
severity one-hots, predicting the hourly change (lambda from {0.1, 1, 10, 100} on validation); the action-aware
ridge adds, per intervention kind, the amount applied over the last 24 hours. T3: the rule "wood at 18:00 <
tonight's burn" and a logistic regression; T4: zero effect and the per-variant mean effect of C_cf val.
"""
from __future__ import annotations

import argparse
import json
import os
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from wm_data import load_episode

ROOT_DIR = Path(__file__).resolve().parents[2]
VARS = ("food_stock", "wood_stock", "fire_lit", "cold_members", "alive", "deadwood_total", "food_on_bushes_total",
        "adults_working")
KINDS = ("gift_food", "spoil_food", "gift_wood", "rot_wood", "rain", "cold_snap", "windfall")
LAGS = (1, 2, 3, 6, 12, 24)
LAMBDAS = (0.1, 1.0, 10.0, 100.0)
TPD, OFFSET, EVERY = 1440, 480, 60
SPLIT_DIRS = ("A_obs", "B_act", "C_cf", "OOD_v12", "OOD_long")


@dataclass
class Ep:
    split: str
    role: str
    seed: int
    eid: str
    v: np.ndarray            # [T, 8]
    tick: np.ndarray         # [T]
    hour: np.ndarray         # [T] int 0..23
    season: np.ndarray       # [T] int -1..3
    severity: np.ndarray     # [T] int -1..2
    actions: np.ndarray      # [T, 7]: amount applied at the row's tick (after the row), per kind
    wood_stock: np.ndarray   # [T]
    members: np.ndarray      # [T]
    cold_hours: np.ndarray   # [T] cumulative cold member-hours
    deaths: np.ndarray       # [T] cumulative deaths, four causes


def episode_arrays(ep) -> dict:
    """The bench's per-row arrays of one loaded episode (wm_data.Episode)."""
    g = ep.arrays["global.npy"]
    col = lambda name: g[:, ep.field_index("global.npy", name)]
    s = ep.arrays["settlements.npy"]
    sf = lambda name: s[:, :, ep.field_index("settlements.npy", name)] if s.shape[1] else np.zeros((g.shape[0], 0))
    food = np.nansum(sf("food_stock"), axis=1)
    wood = np.nansum(sf("wood_stock"), axis=1)
    fire = np.nan_to_num(sf("fire_lit"), nan=0.0).max(axis=1, initial=0.0)
    members = np.nansum(sf("members"), axis=1)
    v = np.stack([food, wood, fire, col("cold_members"), col("alive"), col("deadwood_total"),
                  col("food_on_bushes_total"), col("adults_working")], axis=1).astype(np.float64)
    tick = col("tick").astype(np.int64)
    actions = np.zeros((len(tick), len(KINDS)))
    index = {int(t): i for i, t in enumerate(tick)}
    for rec in ep.interventions:
        if rec["applied"] and int(rec["tick"]) in index:
            actions[index[int(rec["tick"])], KINDS.index(rec["kind"])] += float(rec["amount"])
    deaths = sum(col(k) for k in ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age"))
    return dict(v=v, tick=tick, hour=(col("minute_of_day") // 60).astype(np.int64), season=col("season").astype(np.int64),
                severity=col("winter_severity").astype(np.int64), actions=actions, wood_stock=wood, members=members,
                cold_hours=col("cold_member_hours_total"), deaths=deaths)


def load_split(root: Path, manifest: dict, split: str) -> list[Ep]:
    eps = []
    for entry in manifest["episodes"]:
        if entry["split"] != split:
            continue
        ep = load_episode(root / entry["path"], arrays=["global.npy", "settlements.npy"])
        eps.append(Ep(split, entry["role"], entry["seed"], entry["schedule_id"], **episode_arrays(ep)))
    return eps


# ------------------------------------------------------------------ metrics -------------------------------------
def nmae(pred: np.ndarray, true: np.ndarray, std: np.ndarray) -> np.ndarray:
    """Per-variable MAE / std (NaN where the std is 0)."""
    mae = np.abs(pred - true).mean(axis=0)
    with np.errstate(divide="ignore", invalid="ignore"):
        return np.where(std > 0, mae / np.where(std > 0, std, 1.0), np.nan)


def auc(scores: np.ndarray, labels: np.ndarray) -> float:
    """Rank AUC (Mann-Whitney) with average ranks for ties; NaN with one class."""
    labels = labels.astype(bool)
    pos, neg = labels.sum(), (~labels).sum()
    if pos == 0 or neg == 0:
        return float("nan")
    order = np.argsort(scores, kind="mergesort")
    ranks = np.empty(len(scores))
    sorted_scores = scores[order]
    i = 0
    while i < len(scores):
        j = i
        while j + 1 < len(scores) and sorted_scores[j + 1] == sorted_scores[i]:
            j += 1
        ranks[order[i:j + 1]] = (i + j) / 2 + 1
        i = j + 1
    return float((ranks[labels].sum() - pos * (pos + 1) / 2) / (pos * neg))


def brier(prob: np.ndarray, labels: np.ndarray) -> float:
    return float(np.mean((prob - labels) ** 2))


# ------------------------------------------------------------------ features / ridge ----------------------------
def onehot(values: np.ndarray, levels: list[int]) -> np.ndarray:
    return (values[:, None] == np.array(levels)[None, :]).astype(np.float64)


def features(v: np.ndarray, hour: np.ndarray, season: np.ndarray, severity: np.ndarray, actions24: np.ndarray | None,
             t: np.ndarray) -> np.ndarray:
    """Features of rows `t` (each t >= 24): lags of V (v[t - lag + 1] for lag 1..24, lag 1 = the row itself),
    hour, season and severity one-hots, and optionally the last-24-hour action sums."""
    parts = [v[t - lag + 1] for lag in LAGS]
    parts += [onehot(hour[t], list(range(24))), onehot(season[t], [-1, 0, 1, 2, 3]), onehot(severity[t], [-1, 0, 1, 2])]
    if actions24 is not None:
        parts.append(actions24[t])
    return np.concatenate(parts, axis=1)


def actions_window(actions: np.ndarray) -> np.ndarray:
    """Per row t: amounts applied at rows t-23 .. t (the 24 hours ending with row t's tick)."""
    c = np.cumsum(np.vstack([np.zeros((1, actions.shape[1])), actions]), axis=0)
    t = np.arange(actions.shape[0])
    return c[t + 1] - c[np.maximum(t - 23, 0)]


def episode_xy(ep: Ep, action_aware: bool) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    t = np.arange(23, len(ep.tick) - 1)
    x = features(ep.v, ep.hour, ep.season, ep.severity, actions_window(ep.actions) if action_aware else None, t)
    return x, ep.v[t + 1] - ep.v[t], t


class Ridge:
    """Multi-output ridge on standardized features; the intercept is not penalised."""

    def fit(self, x: np.ndarray, y: np.ndarray, lam: float) -> "Ridge":
        self.mu, self.sd = x.mean(axis=0), x.std(axis=0)
        self.sd[self.sd == 0] = 1.0
        z = (x - self.mu) / self.sd
        self.ymu = y.mean(axis=0)
        a = z.T @ z + lam * np.eye(z.shape[1])
        self.w = np.linalg.solve(a, z.T @ (y - self.ymu))
        return self

    def predict(self, x: np.ndarray) -> np.ndarray:
        return ((x - self.mu) / self.sd) @ self.w + self.ymu


def stack(eps: list[Ep], action_aware: bool):
    xs, ys = zip(*[episode_xy(ep, action_aware)[:2] for ep in eps])
    return np.vstack(xs), np.vstack(ys)


def split_std(eps: list[Ep]) -> np.ndarray:
    return np.vstack([ep.v for ep in eps]).std(axis=0)


def t1_scores(eps: list[Ep], model: Ridge | None, action_aware: bool, std: np.ndarray) -> dict[str, np.ndarray]:
    preds = {"persistence": [], "daily_naive": [], "ridge": []}
    trues = []
    for ep in eps:
        x, _, t = episode_xy(ep, action_aware)
        trues.append(ep.v[t + 1])
        preds["persistence"].append(ep.v[t])
        preds["daily_naive"].append(ep.v[t + 1 - 24])
        if model is not None:
            preds["ridge"].append(ep.v[t] + model.predict(x))
    true = np.vstack(trues)
    return {name: nmae(np.vstack(p), true, std) for name, p in preds.items() if p}


def choose_lambda(train: list[Ep], val: list[Ep], action_aware: bool) -> tuple[float, Ridge, dict]:
    x, y = stack(train, action_aware)
    std = split_std(val)
    best, scores = None, {}
    for lam in LAMBDAS:
        model = Ridge().fit(x, y, lam)
        score = float(np.nanmean(t1_scores(val, model, action_aware, std)["ridge"]))
        scores[str(lam)] = score
        if best is None or score < best[0]:
            best = (score, lam, model)
    return best[1], best[2], scores


# ------------------------------------------------------------------ T2 rollout -----------------------------------
def rollout_starts(ep: Ep, horizon: int) -> list[int]:
    """Rows at 06:00 of one-based days 3, 10, 17, ... whose horizon fits in the episode."""
    starts = []
    day = 3
    while True:
        tick = (day - 1) * TPD + 360 - OFFSET
        row = tick // EVERY
        if row + horizon >= len(ep.tick):
            break
        starts.append(row)
        day += 7
    return starts


def rollout(ep: Ep, start: int, horizon: int, model: Ridge, action_aware: bool) -> np.ndarray:
    """Autoregressive ridge rollout from row `start`; hour and season follow the calendar, severity stays at the
    start row's value (a model cannot know a future winter's draw); actions are those recorded (none in the
    no-action splits)."""
    # Rows after `start` are overwritten before they are read (features read rows <= t only).
    v = ep.v[: start + horizon + 1].copy()
    hour, season = ep.hour, ep.season
    sev = np.full(len(ep.tick), ep.severity[start])
    acts = actions_window(ep.actions) if action_aware else None
    for step in range(horizon):
        t = start + step
        x = features(v, hour, season, sev, acts, np.array([t]))
        v[t + 1] = v[t] + model.predict(x)[0]
    return v[start + horizon]


def t2_scores(eps: list[Ep], model: Ridge, action_aware: bool, std: np.ndarray, horizon: int) -> dict[str, np.ndarray]:
    pred = {"persistence": [], "daily_naive": [], "ridge": []}
    true = []
    for ep in eps:
        for start in rollout_starts(ep, horizon):
            true.append(ep.v[start + horizon])
            pred["persistence"].append(ep.v[start])
            # The value at the same hour of the last observed day (horizons are whole days).
            pred["daily_naive"].append(ep.v[start + horizon - 24 * ((horizon + 23) // 24)])
            pred["ridge"].append(rollout(ep, start, horizon, model, action_aware))
    if not true:
        return {}
    t = np.array(true)
    return {name: nmae(np.array(p), t, std) for name, p in pred.items()}


# ------------------------------------------------------------------ T3 dark night --------------------------------
def winter_evenings(ep: Ep, burn: tuple[int, int, int], hours: int) -> list[dict]:
    """Every winter evening (18:00 of a winter display day) with the fire state at 02:00 that night."""
    rows = []
    days = (ep.tick[-1] + OFFSET) // TPD
    for d in range(days):
        r18 = (d * TPD + 1080 - OFFSET) // EVERY
        r02 = ((d + 1) * TPD + 120 - OFFSET) // EVERY
        if r02 >= len(ep.tick) or ep.season[r18] != 3:
            continue
        level = int(ep.severity[r18])
        tonight = burn[level] * hours if level >= 0 else burn[1] * hours
        # Night index 1..7 within the winter: display day d's position in its season.
        night = d % 7 + 1
        rows.append({"wood": ep.wood_stock[r18], "tonight": tonight, "members": ep.members[r18],
                     "deadwood": ep.v[r18, VARS.index("deadwood_total")], "severity": level, "night": night,
                     "out": float(ep.v[r02, VARS.index("fire_lit")] == 0), "seed": ep.seed, "day": d})
    return rows


def t3_design(rows: list[dict]) -> np.ndarray:
    ratio = np.array([r["wood"] / r["tonight"] for r in rows])
    members = np.array([r["members"] for r in rows])
    dead = np.array([r["deadwood"] for r in rows])
    sev = onehot(np.array([r["severity"] for r in rows]), [0, 1, 2])
    night = np.array([r["night"] for r in rows], dtype=float)
    return np.column_stack([ratio, members, dead, sev, night])


class Logistic:
    """L2-regularised logistic regression by Newton's method on standardized features."""

    def fit(self, x: np.ndarray, y: np.ndarray, lam: float = 1e-3, iters: int = 50) -> "Logistic":
        self.mu, self.sd = x.mean(axis=0), x.std(axis=0)
        self.sd[self.sd == 0] = 1.0
        z = np.column_stack([np.ones(len(x)), (x - self.mu) / self.sd])
        w = np.zeros(z.shape[1])
        reg = lam * np.eye(z.shape[1])
        reg[0, 0] = 0.0
        for _ in range(iters):
            p = 1 / (1 + np.exp(-np.clip(z @ w, -30, 30)))
            grad = z.T @ (p - y) + reg @ w
            hess = (z * (p * (1 - p))[:, None]).T @ z + reg + 1e-9 * np.eye(z.shape[1])
            step = np.linalg.solve(hess, grad)
            w -= step
            if np.abs(step).max() < 1e-10:
                break
        self.w = w
        return self

    def predict(self, x: np.ndarray) -> np.ndarray:
        z = np.column_stack([np.ones(len(x)), (x - self.mu) / self.sd])
        return 1 / (1 + np.exp(-np.clip(z @ self.w, -30, 30)))


def t3_scores(prob: np.ndarray, y: np.ndarray) -> dict:
    return {"brier": brier(prob, y), "auc": auc(prob, y), "accuracy": float(np.mean((prob >= .5) == (y == 1))),
            "n": int(len(y)), "positives": int(y.sum())}


# ------------------------------------------------------------------ T4 counterfactual ----------------------------
T4_TARGETS = ("dark_nights", "cold_member_hours", "food_store", "deaths")


def c_cf_targets(ep: Ep) -> dict[str, float]:
    """First-winter outcomes: dark nights (evenings of one-based days 22-28, fire out at 02:00), cold member-hours
    from day 22 18:00 to day 29 07:00, food store at day 29 06:00 (tick 40,200), deaths up to that tick."""
    row = {int(t): i for i, t in enumerate(ep.tick)}
    dark = 0
    for d in range(21, 28):  # zero-based evenings of one-based days 22..28
        r02 = row[(d + 1) * TPD + 120 - OFFSET]
        dark += int(ep.v[r02, VARS.index("fire_lit")] == 0)
    start, end, food = row[21 * TPD + 1080 - OFFSET], row[28 * TPD + 420 - OFFSET], row[28 * TPD + 360 - OFFSET]
    return {"dark_nights": float(dark), "cold_member_hours": float(ep.cold_hours[end] - ep.cold_hours[start]),
            "food_store": float(ep.v[food, VARS.index("food_stock")]), "deaths": float(ep.deaths[food])}


def c_cf_effects(eps: list[Ep]) -> dict[str, dict[int, dict[str, float]]]:
    """variant -> seed -> target -> effect (variant minus the same seed's factual)."""
    by_seed: dict[int, dict[str, Ep]] = {}
    for ep in eps:
        by_seed.setdefault(ep.seed, {})[ep.eid] = ep
    effects: dict[str, dict[int, dict[str, float]]] = {}
    for seed, group in sorted(by_seed.items()):
        base = c_cf_targets(group["none"])
        for variant, ep in group.items():
            if variant == "none":
                continue
            target = c_cf_targets(ep)
            effects.setdefault(variant, {})[seed] = {k: target[k] - base[k] for k in T4_TARGETS}
    return effects


def t4_scores(test: dict, val: dict) -> dict:
    out = {}
    for variant, seeds in test.items():
        truth = np.array([[e[k] for k in T4_TARGETS] for e in seeds.values()])
        mean = np.array([np.mean([e[k] for e in val.get(variant, {}).values()]) if val.get(variant) else 0.0
                         for k in T4_TARGETS])
        out[variant] = {}
        for name, pred in (("zero_effect", np.zeros_like(truth)), ("val_mean_effect", np.broadcast_to(mean, truth.shape))):
            out[variant][name] = {k: {"mae": float(np.abs(pred[:, i] - truth[:, i]).mean()),
                                      "sign_accuracy": float(np.mean(np.sign(pred[:, i]) == np.sign(truth[:, i])))}
                                  for i, k in enumerate(T4_TARGETS)}
    return out


def effect_distribution(effects: dict) -> dict:
    out = {}
    for variant, seeds in effects.items():
        out[variant] = {}
        for k in T4_TARGETS:
            vals = np.array([e[k] for e in seeds.values()])
            out[variant][k] = {"mean": float(vals.mean()), "median": float(np.median(vals)), "min": float(vals.min()),
                               "max": float(vals.max()), "nonzero_share": float(np.mean(vals != 0))}
        out[variant]["changed_share"] = float(np.mean([any(e[k] != 0 for k in T4_TARGETS) for e in seeds.values()]))
    return out


# ------------------------------------------------------------------ driver --------------------------------------
def run(root: Path, manifest: dict, burn: tuple[int, int, int], hours: int) -> dict:
    data = {s: load_split(root, manifest, s) for s in SPLIT_DIRS}
    role = lambda split, r: [e for e in data[split] if e.role == r]
    a_train, a_val, a_test = role("A_obs", "train"), role("A_obs", "val"), role("A_obs", "test")
    b_train, b_val, b_test = role("B_act", "train"), role("B_act", "val"), role("B_act", "test")
    tests = {"A_test": a_test, "B_test": b_test, "OOD_v12": data["OOD_v12"], "OOD_long": data["OOD_long"]}
    stds = {name: split_std(eps) for name, eps in tests.items()}
    lam, ridge, lam_scores = choose_lambda(a_train, a_val, False)
    lam_act, ridge_act, lam_act_scores = choose_lambda(a_train + b_train, a_val + b_val, True)
    report: dict = {"vars": VARS, "lambda": {"ridge": lam, "ridge_action": lam_act,
                                             "val_scores": lam_scores, "val_scores_action": lam_act_scores}}
    report["T1"] = {}
    for name, eps in tests.items():
        scores = t1_scores(eps, ridge, False, stds[name])
        scores["ridge_action"] = t1_scores(eps, ridge_act, True, stds[name])["ridge"]
        report["T1"][name] = {k: v.tolist() for k, v in scores.items()}
    report["T2"] = {}
    for horizon in (24, 168):
        report["T2"][str(horizon)] = {}
        for name, eps in tests.items():
            scores = t2_scores(eps, ridge, False, stds[name], horizon)
            if scores:
                scores["ridge_action"] = t2_scores(eps, ridge_act, True, stds[name], horizon)["ridge"]
            report["T2"][str(horizon)][name] = {k: v.tolist() for k, v in scores.items()}
    train_rows = [r for ep in a_train for r in winter_evenings(ep, burn, hours)]
    test_rows = [r for ep in a_test for r in winter_evenings(ep, burn, hours)]
    y_train = np.array([r["out"] for r in train_rows])
    y_test = np.array([r["out"] for r in test_rows])
    rule = np.array([float(r["wood"] < r["tonight"]) for r in test_rows])
    logit = Logistic().fit(t3_design(train_rows), y_train)
    report["T3"] = {"rule": t3_scores(rule, y_test), "logistic": t3_scores(logit.predict(t3_design(test_rows)), y_test),
                    "base_rate_train": float(y_train.mean()), "train_n": int(len(y_train))}
    cf = data["C_cf"]
    val_eff = c_cf_effects([e for e in cf if e.role == "val"])
    test_eff = c_cf_effects([e for e in cf if e.role == "test"])
    report["T4"] = t4_scores(test_eff, val_eff)
    report["T4_effects_test"] = effect_distribution(test_eff)
    report["T4_effects_val"] = effect_distribution(val_eff)
    report["sanity"] = {
        "ridge_beats_persistence_T1_food_wood": {
            name: [report["T1"][name]["ridge"][i] < report["T1"][name]["persistence"][i] for i in (0, 1)]
            for name in tests},
        "T3_logistic_auc_ge_0.70": report["T3"]["logistic"]["auc"] >= 0.70,
        "every_variant_changes_a_target_in_25pct": {v: d["changed_share"] >= .25 for v, d in report["T4_effects_test"].items()},
    }
    report["counts"] = {name: len(eps) for name, eps in data.items()} | {"A_train": len(a_train), "B_train": len(b_train)}
    return report


def fmt(x) -> str:
    return "–" if x is None or (isinstance(x, float) and np.isnan(x)) else f"{x:.3f}"


def markdown(r: dict) -> str:
    v = r["vars"]
    lines = ["# WM-Bench v0", "",
             "First benchmark for WorldSim world models, on dataset v0 (`DATASET_v0.md`). Settlement level; numbers are "
             "nMAE = MAE / standard deviation of the variable over the test split's rows (lower is better) unless stated. "
             "Produced by `tools/worldmodel/bench_v0.py` (numpy only).", "",
             f"Episodes: {json.dumps(r['counts'])}. Ridge λ chosen on validation: {r['lambda']['ridge']} "
             f"(action-aware {r['lambda']['ridge_action']}).", "",
             "Variables V: " + ", ".join(f"`{x}`" for x in v) + ".", ""]
    lines += ["## T1 — next hour", "", "Predict V at row t+1 (one hour later) from rows up to t. Train: A train (action-aware: A + B train).", ""]
    for split, scores in r["T1"].items():
        lines += [f"### {split}", "", "| Baseline | " + " | ".join(v) + " | mean |", "|---|" + "---:|" * (len(v) + 1)]
        for name, vals in scores.items():
            lines.append(f"| {name} | " + " | ".join(fmt(x) for x in vals) + f" | {fmt(float(np.nanmean(vals)))} |")
        lines.append("")
    lines += ["## T2 — rollout", "", "From 06:00 of days 3, 10, 17, … of every test episode, roll the one-step model forward "
              "24 h and 168 h; nMAE at the horizon. Daily naive equals persistence at whole-day horizons.", ""]
    for horizon, splits in r["T2"].items():
        for split, scores in splits.items():
            lines += [f"### {horizon} h — {split}", "", "| Baseline | " + " | ".join(v) + " | mean |", "|---|" + "---:|" * (len(v) + 1)]
            for name, vals in scores.items():
                lines.append(f"| {name} | " + " | ".join(fmt(x) for x in vals) + f" | {fmt(float(np.nanmean(vals)))} |")
            lines.append("")
    t3 = r["T3"]
    lines += ["## T3 — dark night", "", "At 18:00 of every winter evening of A test: P(fire out at 02:00). Rule: wood stock at 18:00 < "
              "tonight's burn (severity rate × 13). Logistic: wood / tonight's burn, members, deadwood total, severity one-hot, "
              f"night index; fit on A train ({t3['train_n']} evenings, base rate {t3['base_rate_train']:.3f}).", "",
              "| Baseline | Brier | AUC | Accuracy | Evenings | Dark |", "|---|---:|---:|---:|---:|---:|"]
    for name in ("rule", "logistic"):
        s = t3[name]
        lines.append(f"| {name} | {fmt(s['brier'])} | {fmt(s['auc'])} | {fmt(s['accuracy'])} | {s['n']} | {s['positives']} |")
    lines += ["", "## T4 — counterfactual", "", "On C_cf test seeds, each variant's effect (variant − factual) on: dark nights of the "
              "first winter, cold member-hours over those nights, food store at day 29 06:00, deaths up to then.", ""]
    for variant, preds in r["T4"].items():
        lines += [f"### {variant}", "", "| Baseline | " + " | ".join(f"{k} MAE | {k} sign" for k in T4_TARGETS) + " |",
                  "|---|" + "---:|---:|" * len(T4_TARGETS)]
        for name, scores in preds.items():
            lines.append(f"| {name} | " + " | ".join(f"{fmt(scores[k]['mae'])} | {fmt(scores[k]['sign_accuracy'])}" for k in T4_TARGETS) + " |")
        lines.append("")
    lines += ["### Ground-truth effects (C_cf test)", "", "| Variant | Target | Mean | Median | Min | Max | Non-zero share |",
              "|---|---|---:|---:|---:|---:|---:|"]
    for variant, d in r["T4_effects_test"].items():
        for k in T4_TARGETS:
            e = d[k]
            lines.append(f"| {variant} | {k} | {fmt(e['mean'])} | {fmt(e['median'])} | {fmt(e['min'])} | {fmt(e['max'])} | {fmt(e['nonzero_share'])} |")
        lines.append(f"| {variant} | any target changed | | | | | {fmt(d['changed_share'])} |")
    s = r["sanity"]
    lines += ["", "## Sanity checks", "",
              f"- Ridge beats persistence on T1 food and wood stock: {json.dumps(s['ridge_beats_persistence_T1_food_wood'])}",
              f"- T3 logistic AUC ≥ 0.70: {s['T3_logistic_auc_ge_0.70']} (AUC {fmt(t3['logistic']['auc'])})",
              f"- Every C_cf variant changes at least one target in ≥ 25 % of test seeds: {json.dumps(s['every_variant_changes_a_target_in_25pct'])}",
              ""]
    return "\n".join(lines)


def to_beat(r: dict) -> str:
    """The "how to beat this" note: inputs, splits, metrics and the best baseline numbers."""
    best = lambda scores: min((float(np.nanmean(v)), k) for k, v in scores.items())
    lines = ["## How to beat this (WM-1)", "",
             "- **Inputs.** Per row (one simulated hour): the eight settlement-level variables V above, the calendar "
             "(`minute_of_day`, `season`, `day_of_season`, `year`), `winter_severity`, and for B_act the interventions "
             "(`interventions.json`, applied after the row of their tick). Richer models may read every column of "
             "`global.npy` and `settlements.npy`, the agent tables of the `--agents` episodes and `latent.npy`; say which. "
             "`lifespan_days` is hidden ground truth: a model of what a player sees must not read it.",
             "- **Splits.** Fit on the `train` role only (A_obs train; A + B train for action-aware models), choose "
             "hyper-parameters on `val`, report on `test`, OOD_v12 and OOD_long untouched. C_cf val may be used for T4 "
             "calibration, C_cf test only for the report.",
             "- **Metrics.** As above: T1 and T2 per-variable nMAE (and the mean over V), T3 Brier / AUC / accuracy, T4 MAE "
             "and sign accuracy per target and variant.",
             "- **Numbers to beat** (best baseline, mean nMAE over V unless stated):"]
    for split, scores in r["T1"].items():
        value, name = best(scores)
        lines.append(f"  - T1 {split}: {value:.3f} ({name})")
    for horizon, splits in r["T2"].items():
        for split, scores in splits.items():
            if scores:
                value, name = best(scores)
                lines.append(f"  - T2 {horizon} h {split}: {value:.3f} ({name})")
    t3 = r["T3"]
    top = min(("rule", "logistic"), key=lambda k: t3[k]["brier"])
    lines.append(f"  - T3: Brier {t3[top]['brier']:.3f}, AUC {max(t3['rule']['auc'], t3['logistic']['auc']):.3f} ({top} / best AUC)")
    for variant, preds in r["T4"].items():
        mae = {name: np.mean([preds[name][k]["mae"] for k in T4_TARGETS]) for name in preds}
        name = min(mae, key=mae.get)
        lines.append(f"  - T4 {variant}: mean MAE over the four targets {mae[name]:.3f} ({name})")
    lines += ["- **T3 is nearly arithmetic in v0.** The fire burns only stockpile wood from 18:00, one hourly withdrawal "
              "at a time, so whether it is out at the 02:00 row follows from the 18:00 stock and the night's rate (eight "
              "withdrawals before that row); the logistic baseline learns the threshold (AUC 1.000) where the rule's "
              "whole-night threshold (13 hours) is miscalibrated. A world model must reproduce this; a harder dark-night "
              "task (a lead time of a day or more) belongs in v1.",
              "- T2's daily-naive prediction equals persistence at whole-day horizons, and both are exact for "
              "`cold_members` and `adults_working` at 06:00 (both are always 0 then); the 24-hour 'numbers to beat' "
              "therefore favour persistence on those two variables.",
              "- A model that predicts severity-dependent winters, the claim dynamics of wood gatherers and the response to "
              "an intervention (T4) should beat the ridge most where the ridge is closest to persistence: the fire and cold "
              "variables at 24 h and 168 h, and every T4 target.", ""]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=Path(os.environ.get("WM_DATA_ROOT", "/Users/rexxa/work/worldsim-wm-data")))
    parser.add_argument("--out", type=Path, default=ROOT_DIR / "docs/worldmodel")
    parser.add_argument("--burn", default="2,3,4", help="winter_severity_burn_per_hour of game_v13()")
    parser.add_argument("--burn-hours", type=int, default=13)
    parser.add_argument("--manifest", type=Path, default=ROOT_DIR / "docs/worldmodel/dataset_v0_manifest.json")
    args = parser.parse_args(argv)
    burn = tuple(int(x) for x in args.burn.split(","))
    report = run(args.root, json.loads(args.manifest.read_text()), burn, args.burn_hours)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "bench_v0.json").write_text(json.dumps(report, indent=1, sort_keys=True, default=float) + "\n")
    (args.out / "BENCH_v0.md").write_text(markdown(report) + "\n" + to_beat(report))
    print(json.dumps(report["sanity"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
