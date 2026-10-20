"""World-level paired bootstrap (GOAL_1_3.md 7.3, METRICS_1_3.md). numpy only.

A predictor's per-world score on a T2 task is that world's mean over the 8 V of the per-variable nMAE (`evaluate.t2`'s
`per_world`, the split std fixed from the full test split; bias-removed: the nMAE of the world's de-meaned errors).
A model's per-world score is the mean of its seeds' per-world scores. Intervals resample the test worlds with
replacement (B = 2,000, `numpy.random.default_rng(0)`, the same draws for every comparison of a world set) and take the
95 % percentile interval of the mean over worlds — of one predictor's score, of a difference of two predictors (paired:
the same resampled worlds on both sides), or of any per-world linear combination (Y3's difference of differences).

    from wm1 import bootstrap as bs
    a = bs.world_scores(metrics["T2_any"]["168"]["test"])           # {seed: score}
    bs.interval(bs.diff(bs.average([a0, a1, a2]), base))             # {"point", "lo", "hi", ...}
"""
from __future__ import annotations

import numpy as np

B = 2000
SEED = 0
LEVEL = 95.0


def world_scores(node: dict, bias_removed: bool = False) -> dict[str, float]:
    """{seed: mean over the 8 V} of a T2 node's `per_world` block; every world must have the same start count."""
    pw = node.get("per_world")
    if not pw:
        raise ValueError("the T2 node has no per_world scores")
    counts = {row["n"] for row in pw.values()}
    if len(counts) != 1:
        raise ValueError(f"worlds differ in start count: {sorted(counts)}")
    if bias_removed:
        return {k: float(np.nanmean(row["bias_removed"]["per_var"])) for k, row in pw.items()}
    return {k: float(np.nanmean(row["per_var"])) for k, row in pw.items()}


def world_var_scores(node: dict, var: int, bias_removed: bool = False) -> dict[str, float]:
    """{seed: nMAE of variable `var`} (per-variable skill intervals)."""
    pw = node["per_world"]
    key = (lambda r: r["bias_removed"]["per_var"]) if bias_removed else (lambda r: r["per_var"])
    return {k: float(key(row)[var]) for k, row in pw.items()}


def _same_worlds(*scores: dict) -> list[str]:
    keys = sorted(scores[0], key=int)
    for s in scores[1:]:
        if sorted(s, key=int) != keys:
            raise ValueError("the predictors were scored on different worlds")
    return keys


def average(scores: list[dict]) -> dict[str, float]:
    """Per-world mean of several predictors (a model's seeds)."""
    keys = _same_worlds(*scores)
    return {k: float(np.mean([s[k] for s in scores])) for k in keys}


def diff(a: dict, b: dict) -> dict[str, float]:
    keys = _same_worlds(a, b)
    return {k: a[k] - b[k] for k in keys}


def draws(n_worlds: int, b: int = B, seed: int = SEED) -> np.ndarray:
    """[b, n_worlds] resampled world indices (the same for every comparison on a world set of this size)."""
    return np.random.default_rng(seed).integers(0, n_worlds, size=(b, n_worlds))


def interval(scores: dict, b: int = B, seed: int = SEED) -> dict:
    """Mean over worlds and its 95 % percentile bootstrap interval."""
    keys = sorted(scores, key=int)
    x = np.array([scores[k] for k in keys], dtype=np.float64)
    if not np.isfinite(x).all():
        raise ValueError("non-finite per-world score")
    stats = x[draws(len(x), b, seed)].mean(axis=1)
    lo, hi = np.percentile(stats, [(100 - LEVEL) / 2, 100 - (100 - LEVEL) / 2])
    return {"point": float(x.mean()), "lo": float(lo), "hi": float(hi), "n_worlds": len(x), "B": b, "seed": seed}


def paired(a: dict, b: dict, nb: int = B, seed: int = SEED) -> dict:
    """Interval of mean(a - b) with the worlds resampled jointly."""
    return interval(diff(a, b), nb, seed)


def skill_interval(model: dict, base: dict, b: int = B, seed: int = SEED) -> dict:
    """Skill 1 - mean(model) / mean(base) over the resampled worlds (the same draws on both sides)."""
    keys = _same_worlds(model, base)
    m = np.array([model[k] for k in keys])
    s = np.array([base[k] for k in keys])
    idx = draws(len(keys), b, seed)
    stats = 1.0 - m[idx].mean(axis=1) / s[idx].mean(axis=1)
    lo, hi = np.percentile(stats, [(100 - LEVEL) / 2, 100 - (100 - LEVEL) / 2])
    return {"point": float(1.0 - m.mean() / s.mean()), "lo": float(lo), "hi": float(hi), "n_worlds": len(keys), "B": b}


def verdict(seed_scores: list[dict], base: dict, b: int = B, seed: int = SEED) -> dict:
    """The Y1 rule (METRICS_1_3.md): KEEP if the seed mean <= 0.8 x the baseline, every seed < the baseline and the CI
    upper < 0; WEAK if the CI upper < 0 but not KEEP; TIE if the CI contains 0; NONE if the CI lower > 0. The CI is
    the paired interval of (seed-averaged model - baseline) over the test worlds; each seed's own paired CI is also
    reported."""
    model = average(seed_scores)
    ci = paired(model, base, b, seed)
    base_mean = float(np.mean(list(base.values())))
    seeds = [float(np.mean(list(s.values()))) for s in seed_scores]
    seed_mean = float(np.mean(seeds))
    if ci["hi"] < 0:
        keep = seed_mean <= 0.8 * base_mean and all(x < base_mean for x in seeds)
        v = "KEEP" if keep else "WEAK"
    elif ci["lo"] > 0:
        v = "NONE"
    else:
        v = "TIE"
    return {"verdict": v, "model": seed_mean, "seeds": seeds, "baseline": base_mean, "ratio": seed_mean / base_mean,
            "ci_diff": ci, "seed_ci_diff": [paired(s, base, b, seed) for s in seed_scores],
            "keep_threshold": 0.8 * base_mean}
