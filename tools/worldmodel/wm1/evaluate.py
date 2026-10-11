"""One evaluation harness for every WM-1 predictor (GOAL.md 6.2 and 7). numpy only.

A predictor implements `step(history, ctx) -> next V row` (`StepPredictor`) or, for speed, a batched
`rollout(ep, starts, horizon, actions) -> Rollout`: from each start row (rows <= start observed) predict rows
start+1 .. start+horizon autoregressively, with the calendar of the episode and the given per-row actions
(the recorded requested amounts unless a T4 branch overrides them). A predictor never reads rows > start.

Tasks: T1 (next hour), T2 (24 h / 168 h from the bench's 06:00 starts), T2-any (every hour of the same days),
T3 (18:00 -> fire out at 02:00), T3b (winter outlook from 06:00 of each winter's third day), T4 (C_cf
counterfactual rollouts). T5 (agents) lives in `wm1.agents`. Every metric uses `bench_v0`'s definitions: the
split std of V over the T1 rows as the nMAE denominator, `rollout_starts`, `winter_evenings`, `c_cf_targets`.
"""
from __future__ import annotations

from dataclasses import dataclass

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1.data import EpisodeData

BURN, BURN_HOURS = (2, 3, 4), 13
V_FIRE = b.VARS.index("fire_lit")
CF_END_ROW = (28 * b.TPD + 420 - b.OFFSET) // b.EVERY   # day 29 07:00, the last row c_cf_targets reads
RANKED_T4 = ("dark_nights", "cold_member_hours", "food_store")


@dataclass
class Rollout:
    v: np.ndarray                       # [n, H, 8] predicted V at rows start+1 .. start+H
    fire_prob: np.ndarray | None = None  # [n, H] P(fire lit)
    cold_hours: np.ndarray | None = None  # [n, H] cumulative cold member-hours (continuing the start row's value)
    deaths: np.ndarray | None = None      # [n, H] cumulative deaths


class Predictor:
    name = "predictor"
    provides_t4 = False   # True when rollouts carry cold_hours and deaths

    def rollout(self, ep: EpisodeData, starts: np.ndarray, horizon: int, actions: np.ndarray) -> Rollout:
        raise NotImplementedError


class StepPredictor(Predictor):
    """A predictor defined by one step on the V history; `rollout` feeds its own predictions back."""

    def step(self, history: np.ndarray, ctx: dict) -> np.ndarray:
        raise NotImplementedError

    def rollout(self, ep: EpisodeData, starts: np.ndarray, horizon: int, actions: np.ndarray) -> Rollout:
        out = np.zeros((len(starts), horizon, len(b.VARS)))
        for i, s in enumerate(starts):
            hist = list(ep.bench.v[: s + 1])
            for k in range(horizon):
                row = self.step(np.asarray(hist), {"ep": ep, "t": s + k, "actions": actions})
                hist.append(row)
                out[i, k] = row
        return Rollout(out)


class Persistence(StepPredictor):
    name = "persistence"

    def step(self, history, ctx):
        return history[-1]


class DailyNaive(StepPredictor):
    name = "daily_naive"

    def step(self, history, ctx):
        return history[-24]

    def rollout(self, ep, starts, horizon, actions):
        # Vectorized: the predicted row t+1 is the (true or predicted) row t+1-24; buffer index j is row s-23+j.
        n = len(starts)
        buf = np.zeros((n, 24 + horizon, len(b.VARS)))
        for i, s in enumerate(starts):
            buf[i, :24] = ep.bench.v[s - 23: s + 1]
        for k in range(horizon):
            buf[:, 24 + k] = buf[:, k]
        return Rollout(buf[:, 24:])


SEV_COLS = slice(len(b.LAGS) * len(b.VARS) + 24 + 5, len(b.LAGS) * len(b.VARS) + 24 + 5 + 4)   # severity one-hot in b.features


def drop_severity(x: np.ndarray) -> np.ndarray:
    """`bench_v0.features` rows without the severity one-hot (the severity-free ridge's inputs)."""
    return np.delete(x, np.arange(x.shape[1])[SEV_COLS], axis=1)


class RidgePredictor(Predictor):
    """`bench_v0`'s ridge (or action-aware ridge) as a predictor; the rollout is `bench_v0.rollout` vectorized
    over starts (calendar from the episode). `severity` selects what the rollout feeds as severity:
    "hold" — the start row's value (the bench's rule), "true" — each rollout row's true value (an oracle reference:
    it reads the future winter draw), "none" — the model was fit without severity features."""

    def __init__(self, model: b.Ridge, action_aware: bool, name: str, severity: str = "hold"):
        if severity not in ("hold", "true", "none"):
            raise ValueError(severity)
        self.model, self.action_aware, self.name, self.severity = model, action_aware, name, severity

    def rollout(self, ep, starts, horizon, actions):
        n = len(starts)
        lags = np.array(b.LAGS)
        buf = np.zeros((n, 24 + horizon, len(b.VARS)))
        for i, s in enumerate(starts):
            buf[i, :24] = ep.bench.v[s - 23: s + 1]
        acts24 = b.actions_window(actions) if self.action_aware else None
        held = ep.bench.severity[starts]
        idx = np.arange(n)
        for k in range(horizon):
            t = starts + k
            cur = 23 + k
            sev = ep.bench.severity[t] if self.severity == "true" else held
            parts = [buf[idx, cur - lag + 1] for lag in lags]
            parts += [b.onehot(ep.bench.hour[t], list(range(24))), b.onehot(ep.bench.season[t], [-1, 0, 1, 2, 3])]
            if self.severity != "none":
                parts.append(b.onehot(sev, [-1, 0, 1, 2]))
            if acts24 is not None:
                parts.append(acts24[t])
            x = np.concatenate(parts, axis=1)
            buf[:, cur + 1] = buf[:, cur] + self.model.predict(x)
        return Rollout(buf[:, 24:])


def fit_ridge_without_severity(train: list[EpisodeData], val: list[EpisodeData], action_aware: bool):
    """The bench's ridge recipe (lags, hour, season, optional 24-hour action sums; lambda from the bench grid on the
    validation T1 mean nMAE) with the severity one-hot removed from the features."""
    xs, ys = zip(*[b.episode_xy(ep.bench, action_aware)[:2] for ep in train])
    x, y = drop_severity(np.vstack(xs)), np.vstack(ys)
    std = b.split_std([ep.bench for ep in val])
    best, scores = None, {}
    for lam in b.LAMBDAS:
        pred = RidgePredictor(b.Ridge().fit(x, y, lam), action_aware, "ridge_nosev", "none")
        score = t1(pred, val, std)["mean"]
        scores[str(lam)] = score
        if best is None or score < best[0]:
            best = (score, lam, pred)
    return best[2], best[1], scores


# ------------------------------------------------------------------ starts ----------------------------------------
def t1_starts(ep: EpisodeData) -> np.ndarray:
    return np.arange(23, ep.rows - 1)


def bench_starts(ep: EpisodeData, horizon: int) -> np.ndarray:
    return np.array(b.rollout_starts(ep.bench, horizon), dtype=np.int64)


def any_hour_starts(ep: EpisodeData, horizon: int) -> np.ndarray:
    """Every hour (00:00 .. 23:00) of the bench's start days 3, 10, 17, ... whose horizon fits (GOAL.md 7.3)."""
    starts, day = [], 3
    while True:
        first = ((day - 1) * b.TPD - b.OFFSET) // b.EVERY
        if first + horizon >= ep.rows:
            break
        starts += [first + h for h in range(24) if first + h + horizon < ep.rows]
        day += 7
    return np.array(starts, dtype=np.int64)


def run_rollouts(pred: Predictor, ep: EpisodeData, starts: np.ndarray, horizon: int,
                 actions: np.ndarray | None = None) -> Rollout:
    if len(starts) == 0:
        return Rollout(np.zeros((0, horizon, len(b.VARS))))
    return pred.rollout(ep, starts, horizon, ep.bench.actions if actions is None else actions)


# ------------------------------------------------------------------ T1 / T2 ---------------------------------------
def summarize(per_var: np.ndarray) -> dict:
    return {"per_var": [float(x) for x in per_var], "mean": float(np.nanmean(per_var))}


def t1(pred: Predictor, eps: list[EpisodeData], std: np.ndarray) -> dict:
    preds, trues = [], []
    for ep in eps:
        s = t1_starts(ep)
        preds.append(run_rollouts(pred, ep, s, 1).v[:, 0])
        trues.append(ep.bench.v[s + 1])
    return summarize(b.nmae(np.vstack(preds), np.vstack(trues), std))


def t2(pred: Predictor, eps: list[EpisodeData], std: np.ndarray, horizon: int, any_hour: bool = False,
       breakdown: bool = False) -> dict:
    preds, trues, season, severity, hour = [], [], [], [], []
    for ep in eps:
        s = any_hour_starts(ep, horizon) if any_hour else bench_starts(ep, horizon)
        if len(s) == 0:
            continue
        preds.append(run_rollouts(pred, ep, s, horizon).v[:, -1])
        trues.append(ep.bench.v[s + horizon])
        season.append(ep.bench.season[s + horizon])
        severity.append(ep.bench.severity[s + horizon])
        hour.append(ep.bench.hour[s])
    if not preds:
        return {}
    p, t = np.vstack(preds), np.vstack(trues)
    out = summarize(b.nmae(p, t, std)) | {"n": int(len(t))}
    if breakdown:
        season, severity, hour = np.concatenate(season), np.concatenate(severity), np.concatenate(hour)
        out["by_target_season"] = {str(k): summarize(b.nmae(p[season == k], t[season == k], std)) | {"n": int((season == k).sum())}
                                   for k in np.unique(season)}
        out["by_target_severity"] = {str(k): summarize(b.nmae(p[severity == k], t[severity == k], std)) | {"n": int((severity == k).sum())}
                                     for k in np.unique(severity)}
        if any_hour:
            out["by_start_hour"] = {str(k): summarize(b.nmae(p[hour == k], t[hour == k], std)) | {"n": int((hour == k).sum())}
                                    for k in np.unique(hour)}
    return out


# ------------------------------------------------------------------ T3 --------------------------------------------
def t3_rows(ep: EpisodeData) -> list[dict]:
    rows = b.winter_evenings(ep.bench, BURN, BURN_HOURS)
    for r in rows:
        r["r18"] = (r["day"] * b.TPD + 1080 - b.OFFSET) // b.EVERY
    return rows


def t3_model_prob(pred: Predictor, eps: list[EpisodeData]) -> tuple[np.ndarray, np.ndarray]:
    """P(fire out at 02:00) after rolling 8 steps from 18:00, and the truth, over every winter evening."""
    probs, ys = [], []
    for ep in eps:
        rows = t3_rows(ep)
        if not rows:
            continue
        starts = np.array([r["r18"] for r in rows])
        ro = run_rollouts(pred, ep, starts, 8)
        lit = ro.fire_prob[:, 7] if ro.fire_prob is not None else np.clip(ro.v[:, 7, V_FIRE], 0.0, 1.0)
        probs.append(1.0 - lit)
        ys.append(np.array([r["out"] for r in rows]))
    return np.concatenate(probs), np.concatenate(ys)


def t3_baselines(train: list[EpisodeData], test: list[EpisodeData]) -> dict:
    """The bench's rule and logistic baselines computed through the harness's T3 rows."""
    train_rows = [r for ep in train for r in t3_rows(ep)]
    test_rows = [r for ep in test for r in t3_rows(ep)]
    y_train = np.array([r["out"] for r in train_rows])
    y_test = np.array([r["out"] for r in test_rows])
    rule = np.array([float(r["wood"] < r["tonight"]) for r in test_rows])
    logit = b.Logistic().fit(b.t3_design(train_rows), y_train)
    return {"rule": b.t3_scores(rule, y_test), "logistic": b.t3_scores(logit.predict(b.t3_design(test_rows)), y_test),
            "base_rate_train": float(y_train.mean()), "train_n": int(len(y_train))}


def t3(pred: Predictor, eps: list[EpisodeData]) -> dict:
    prob, y = t3_model_prob(pred, eps)
    return b.t3_scores(prob, y)


# ------------------------------------------------------------------ T3b -------------------------------------------
def row_of(day: int, minute: int) -> int:
    """Row of one-based display day `day` at `minute`."""
    return ((day - 1) * b.TPD + minute - b.OFFSET) // b.EVERY


def t3b_cases(ep: EpisodeData) -> list[dict]:
    """06:00 of each winter's third day (one-based days 24, 52, 80, ...) with observable features and the number of
    dark nights (fire out at 02:00) among the evenings of that day and the four after it."""
    cases, day = [], 24
    while True:
        r06 = row_of(day, 360)
        nights = [row_of(e + 1, 120) for e in range(day, day + 5)]   # 02:00 after the evening of day e
        if nights[-1] >= ep.rows:
            break
        if ep.bench.season[r06] != 3:
            raise ValueError(f"day {day} is not a winter day")
        v, wood = ep.bench.v, ep.bench.wood_stock
        dead = b.VARS.index("deadwood_total")
        drops = [wood[row_of(d, 1080)] - wood[row_of(d + 1, 360)] for d in (day - 2, day - 1)]
        dead_change = [v[row_of(d + 1, 360), dead] - v[row_of(d, 360), dead] for d in (day - 2, day - 1)]
        x = [wood[r06], v[r06, dead], *drops, *dead_change, ep.bench.members[r06]]
        dark = sum(int(v[r, V_FIRE] == 0) for r in nights)
        cases.append({"r06": r06, "nights": [r - r06 for r in nights], "x": x, "dark": float(dark),
                      "severity": int(ep.bench.severity[r06]), "seed": ep.seed, "day": day})
        day += 28
    return cases


def t3b_scores(pred: np.ndarray, true: np.ndarray) -> dict:
    return {"mae": float(np.abs(pred - true).mean()), "exact_accuracy": float(np.mean(np.round(pred) == true)),
            "n": int(len(true)), "mean_true": float(true.mean())}


def t3b_baselines(train: list[EpisodeData], val: list[EpisodeData], tests: dict[str, list[EpisodeData]]) -> dict:
    """Train mean, ridge on observables (lambda on val), severity oracle (train mean per severity)."""
    tr = [c for ep in train for c in t3b_cases(ep)]
    va = [c for ep in val for c in t3b_cases(ep)]
    xtr, ytr = np.array([c["x"] for c in tr]), np.array([[c["dark"]] for c in tr])
    xva, yva = np.array([c["x"] for c in va]), np.array([c["dark"] for c in va])
    best = None
    for lam in b.LAMBDAS:
        m = b.Ridge().fit(xtr, ytr, lam)
        mae = float(np.abs(np.clip(m.predict(xva)[:, 0], 0, 5) - yva).mean())
        if best is None or mae < best[0]:
            best = (mae, lam, m)
    mean = float(ytr.mean())
    by_sev = {k: float(ytr[[c["severity"] == k for c in tr]].mean()) for k in sorted({c["severity"] for c in tr})}
    out = {"ridge_lambda": best[1], "train_n": len(tr), "train_mean": mean, "oracle_means": by_sev}
    for name, eps in tests.items():
        cs = [c for ep in eps for c in t3b_cases(ep)]
        if not cs:
            continue
        y = np.array([c["dark"] for c in cs])
        x = np.array([c["x"] for c in cs])
        out[name] = {"train_mean": t3b_scores(np.full(len(y), mean), y),
                     "ridge": t3b_scores(np.clip(best[2].predict(x)[:, 0], 0, 5), y)}
        if all(c["severity"] in by_sev for c in cs):
            out[name]["severity_oracle"] = t3b_scores(np.array([by_sev[c["severity"]] for c in cs]), y)
    return out


def t3b(pred: Predictor, eps: list[EpisodeData]) -> dict:
    """Roll out from 06:00 to the last of the five nights and count predicted dark 02:00 rows."""
    preds, trues, sevs = [], [], []
    for ep in eps:
        cs = t3b_cases(ep)
        if not cs:
            continue
        horizon = max(cs[0]["nights"])
        ro = run_rollouts(pred, ep, np.array([c["r06"] for c in cs]), horizon)
        for i, c in enumerate(cs):
            idx = np.array(c["nights"]) - 1
            lit = ro.fire_prob[i, idx] >= 0.5 if ro.fire_prob is not None else ro.v[i, idx, V_FIRE] >= 0.5
            preds.append(float((~lit).sum()))
            trues.append(c["dark"])
            sevs.append(c["severity"])
    if not trues:
        return {}
    p, t, s = np.array(preds), np.array(trues), np.array(sevs)
    out = t3b_scores(p, t)
    out["by_severity"] = {str(k): t3b_scores(p[s == k], t[s == k]) for k in np.unique(s)}
    return out


# ------------------------------------------------------------------ T4 --------------------------------------------
def cf_groups(eps: list[EpisodeData]) -> dict[int, dict[str, EpisodeData]]:
    groups: dict[int, dict[str, EpisodeData]] = {}
    for ep in eps:
        groups.setdefault(ep.seed, {})[ep.eid] = ep
    return groups


def cf_branch_targets(pred: Predictor, factual: EpisodeData, start: int, actions: np.ndarray) -> dict[str, float]:
    """c_cf_targets on factual rows <= start followed by the predictor's rows start+1 .. day 29 07:00."""
    ro = run_rollouts(pred, factual, np.array([start]), CF_END_ROW - start, actions)
    fb = factual.bench
    keep = slice(0, start + 1)
    ep = b.Ep(fb.split, fb.role, fb.seed, "branch",
              v=np.vstack([fb.v[keep], ro.v[0]]), tick=fb.tick[: CF_END_ROW + 1], hour=fb.hour[: CF_END_ROW + 1],
              season=fb.season[: CF_END_ROW + 1], severity=fb.severity[: CF_END_ROW + 1],
              actions=actions[: CF_END_ROW + 1], wood_stock=np.concatenate([fb.wood_stock[keep], ro.v[0][:, 1]]),
              members=fb.members[: CF_END_ROW + 1],
              cold_hours=np.concatenate([fb.cold_hours[keep], ro.cold_hours[0]]),
              deaths=np.concatenate([fb.deaths[keep], ro.deaths[0]]))
    return b.c_cf_targets(ep)


def t4_predicted_effects(pred: Predictor, eps: list[EpisodeData]) -> dict[str, dict[int, dict[str, float]]]:
    """variant -> seed -> target -> predicted effect (rollout with the intervention minus rollout without), both
    branches from the factual episode's row at the intervention tick."""
    effects: dict[str, dict[int, dict[str, float]]] = {}
    for seed, group in sorted(cf_groups(eps).items()):
        factual = group["none"]
        for variant, ep in sorted(group.items()):
            if variant == "none":
                continue
            (rec,) = ep.interventions
            start = int(rec["tick"]) // b.EVERY
            if not np.array_equal(ep.bench.v[: start + 1], factual.bench.v[: start + 1]):
                raise ValueError(f"seed {seed} {variant}: prefix differs from the factual episode")
            with_act = np.zeros_like(factual.bench.actions)
            with_act[start, b.KINDS.index(rec["kind"])] = float(rec["amount"])
            on = cf_branch_targets(pred, factual, start, with_act)
            off = cf_branch_targets(pred, factual, start, np.zeros_like(factual.bench.actions))
            effects.setdefault(variant, {})[seed] = {k: on[k] - off[k] for k in b.T4_TARGETS}
    return effects


def t4_score(pred_eff: dict, true_eff: dict) -> dict:
    """Per variant and target: MAE, sign accuracy, MAE / mean |true effect| (None when that mean is 0)."""
    out = {}
    for variant, seeds in true_eff.items():
        out[variant] = {}
        for k in b.T4_TARGETS:
            truth = np.array([seeds[s][k] for s in sorted(seeds)])
            p = np.array([pred_eff[variant][s][k] for s in sorted(seeds)])
            mae = float(np.abs(p - truth).mean())
            scale = float(np.abs(truth).mean())
            out[variant][k] = {"mae": mae, "sign_accuracy": float(np.mean(np.sign(p) == np.sign(truth))),
                               "nmae": mae / scale if scale > 0 else None}
    return out


def t4_baseline_effects(true_test: dict, true_val: dict) -> dict[str, dict]:
    """The bench's zero-effect and val-mean-effect predictions as effect tables."""
    zero = {v: {s: {k: 0.0 for k in b.T4_TARGETS} for s in seeds} for v, seeds in true_test.items()}
    mean = {}
    for v, seeds in true_test.items():
        m = {k: float(np.mean([e[k] for e in true_val[v].values()])) if true_val.get(v) else 0.0 for k in b.T4_TARGETS}
        mean[v] = {s: dict(m) for s in seeds}
    return {"zero_effect": zero, "val_mean_effect": mean}


def t4(pred: Predictor, test_eps: list[EpisodeData]) -> dict:
    true = b.c_cf_effects([ep.bench for ep in test_eps])
    eff = t4_predicted_effects(pred, test_eps)
    return {"scores": t4_score(eff, true), "effects": {v: {str(s): e for s, e in d.items()} for v, d in eff.items()}}


# ------------------------------------------------------------------ driver ----------------------------------------
def evaluate(pred: Predictor, splits: dict[str, list[EpisodeData]], stds: dict[str, np.ndarray],
             cf_test: list[EpisodeData] | None, t3_split: str = "A_test", any_hour: bool = True,
             t1_on: bool = True) -> dict:
    """Every task for one predictor on the given (test or val) splits."""
    out: dict = {"T1": {}, "T2": {"24": {}, "168": {}}, "T2_any": {"24": {}, "168": {}}, "T3b": {}}
    for name, eps in splits.items():
        if t1_on:
            out["T1"][name] = t1(pred, eps, stds[name])
        for h in (24, 168):
            out["T2"][str(h)][name] = t2(pred, eps, stds[name], h, breakdown=True)
            if any_hour:
                out["T2_any"][str(h)][name] = t2(pred, eps, stds[name], h, any_hour=True, breakdown=True)
        out["T3b"][name] = t3b(pred, eps)
    if t3_split in splits:
        out["T3"] = t3(pred, splits[t3_split])
    if cf_test is not None and pred.provides_t4:
        out["T4"] = t4(pred, cf_test)
    return out


def mean_of(d: dict) -> float:
    return float(d["mean"]) if d else float("nan")
