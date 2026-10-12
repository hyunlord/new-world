#!/usr/bin/env python3
"""v14 -> v15 shifts with bootstrap intervals (PASS9_GOAL.md section 9 item 2). Stdlib + numpy.

usage: shift_pass9.py --v14-10 DIR --v14-120 DIR --v15-10 DIR --v15-120 DIR --v11-targets JSON --out DIR [--long-days 120]

Both versions run the same seeds (worldgen and weather are identical), so the
seeds are paired. Two families of statistics:

* X3 clauses (winters): severity-only R2, the normal and harsh mode shares, the
  no-dark-night and two-or-more shares, the number of distinct first dark
  nights and the seventh-night share, and the three severity shares (weather
  only: identical by construction). Each is recomputed from the winters of a
  resampled seed list (with multiplicity).
* Y-medians (every X5 clause and Y3's child and elder shares in X2, i.e. every
  clause judged on a median over seeds). A clause with an every-seed
  counterpart takes that counterpart's stored per-seed values; otherwise the
  per-seed value is the clause's median over a one-seed round (that seed's
  value; `_10` clauses read the 10-day round, the others the long round). A
  clause that reads the run's pooled summary gives one value for every seed:
  it is reported as summary-level, without an interval.

Shift = statistic(v15) - statistic(v14); 95 % percentile interval from 2,000
resamples of the seeds with replacement (`numpy.random.default_rng(9)`, one
fresh generator per statistic), the same resampled seeds for both versions.
A shift whose interval excludes 0 is flagged.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

import numpy as np

import evaluate_pass8 as p8
from evaluate_pass4 import Round
from evaluate_pass5 import load_round

RESAMPLES = 2000
BOOT_SEED = 9


def load(path: Path, duration: int) -> Round:
    schema = json.loads((path / "summary.json").read_text())["schema_version"]
    return load_round(path, duration, schema)


def x3_statistics(winters: list[dict]) -> dict[str, float | None]:
    """Scalar statistics behind each X3 clause."""
    n = len(winters)
    out: dict[str, float | None] = {"a_r2": p8.r_squared(winters)}
    for level, name in ((1, "normal"), (2, "harsh")):
        group = [w["nights_out"] for w in winters if w["severity"] == level]
        out[f"b_mode_share_{name}"] = max(Counter(group).values()) / len(group) if group else None
    out["c_no_dark_night_share"] = sum(1 for w in winters if w["nights_out"] == 0) / n if n else None
    out["d_two_or_more_share"] = sum(1 for w in winters if w["nights_out"] >= 2) / n if n else None
    firsts = [w["first_night_out"] for w in winters if w["nights_out"] > 0]
    out["e_distinct_first_nights"] = float(len(set(firsts)))
    out["e_seventh_share"] = sum(1 for f in firsts if f == 7) / len(firsts) if firsts else None
    for level, name in enumerate(p8.SEVERITY_NAMES):
        out[f"f_share_{name}"] = sum(1 for w in winters if w["severity"] == level) / n if n else None
    return out


def one_seed(r: Round, seed) -> Round:
    return Round(r.path, r.duration, [seed], r.summary)


def median_clauses(short: Round, long: Round, v11: dict) -> tuple[dict[str, dict[int, float]], set[str]]:
    """Per-seed values of every median-judged clause: {clause: {seed: value}}. A clause
    with an every-seed counterpart (`<base>_median` / `<base>_every_seed`) takes that
    clause's stored per-seed values; the others are evaluated on one-seed rounds."""
    _, stored = p8.evaluate(short, long, {}, v11, None)
    out: dict[str, dict[int, float]] = {}
    for name, entry in stored.items():
        if name.endswith("_every_seed"):
            out[name[: -len("_every_seed")] + "_median"] = {int(label.split(":")[-1]): float(v) for label, v in entry["values"].items()}
    # `_10` clauses from one-seed rounds of every 10-day seed, the others from every
    # long-round seed (the two rounds may cover different seeds, as on the held-out set).
    jobs = [(seed, long.seeds[0], True) for seed in short.seeds] + [(short.seeds[0], seed, False) for seed in long.seeds]
    for s_seed, l_seed, ten in jobs:
        targets, _ = p8.evaluate(one_seed(short, s_seed), one_seed(long, l_seed), {}, v11, None)
        clauses = dict(targets["X5"]["clauses"])
        clauses.update({k: targets["X2"]["clauses"][k] for k in ("Y3_child_share", "Y3_elder_share")})
        for name, clause in clauses.items():
            observed = clause.get("observed")
            if not isinstance(observed, dict) or "median" not in observed or observed.get("median") is None:
                continue
            if name.endswith("_10") == ten:
                out.setdefault(f"__one_seed__{name}", {})[(s_seed if ten else l_seed).seed] = float(observed["median"])
    # One-seed values only for clauses without stored per-seed values.
    one_seed_names = set()
    for key in [k for k in out if k.startswith("__one_seed__")]:
        name = key[len("__one_seed__"):]
        values = out.pop(key)
        if name not in out:
            out[name] = values
            one_seed_names.add(name)
    return out, one_seed_names


def bootstrap(values: dict[int, tuple[float, float]], stat, one_seed: bool = False) -> dict:
    """Shift of `stat` over paired per-seed values with a percentile interval."""
    seeds = sorted(values)
    a = [values[s][0] for s in seeds]
    b = [values[s][1] for s in seeds]
    rng = np.random.default_rng(BOOT_SEED)
    shifts = []
    for _ in range(RESAMPLES):
        index = rng.integers(0, len(seeds), len(seeds))
        x, y = stat([a[i] for i in index]), stat([b[i] for i in index])
        if x is not None and y is not None:
            shifts.append(y - x)
    v14, v15 = stat(a), stat(b)
    lo = float(np.quantile(shifts, .025, method="inverted_cdf")) if shifts else None
    hi = float(np.quantile(shifts, .975, method="inverted_cdf")) if shifts else None
    shift = None if v14 is None or v15 is None else v15 - v14
    # A clause computed from the run's summary (pooled over every seed) gives the
    # same value for every one-seed round: no per-seed interval exists.
    summary_level = one_seed and len(seeds) > 1 and len(set(a)) == 1 and len(set(b)) == 1
    if summary_level:
        lo = hi = None
    return {"v14": v14, "v15": v15, "shift": shift, "lo": lo, "hi": hi, "n_seeds": len(seeds),
            "summary_level": summary_level, "excludes_zero": lo is not None and (lo > 0 or hi < 0)}


def x3_bootstrap(w14: list[dict], w15: list[dict]) -> dict:
    by14, by15 = {}, {}
    for w in w14:
        by14.setdefault(w["seed"], []).append(w)
    for w in w15:
        by15.setdefault(w["seed"], []).append(w)
    seeds = sorted(set(by14) & set(by15))
    names = list(x3_statistics(w14))
    out = {}
    for name in names:
        rng = np.random.default_rng(BOOT_SEED)
        shifts = []
        for _ in range(RESAMPLES):
            index = rng.integers(0, len(seeds), len(seeds))
            pick = [seeds[i] for i in index]
            x = x3_statistics([w for s in pick for w in by14[s]])[name]
            y = x3_statistics([w for s in pick for w in by15[s]])[name]
            if x is not None and y is not None:
                shifts.append(y - x)
        v14, v15 = x3_statistics(w14)[name], x3_statistics(w15)[name]
        lo = float(np.quantile(shifts, .025, method="inverted_cdf")) if shifts else None
        hi = float(np.quantile(shifts, .975, method="inverted_cdf")) if shifts else None
        out[name] = {"v14": v14, "v15": v15, "shift": None if v14 is None or v15 is None else v15 - v14,
                     "lo": lo, "hi": hi, "n_seeds": len(seeds), "excludes_zero": lo is not None and (lo > 0 or hi < 0)}
    return out


def median(values: list[float]) -> float:
    return float(np.median(values))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    for name in ("--v14-10", "--v14-120", "--v15-10", "--v15-120", "--out"):
        parser.add_argument(name, required=True, type=Path)
    parser.add_argument("--v11-targets", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    parser.add_argument("--title", default="v14 -> v15 shifts")
    args = parser.parse_args(argv)
    v11 = json.loads(args.v11_targets.read_text())["measures"]
    s14, l14 = load(args.v14_10, 10), load(args.v14_120, args.long_days)
    s15, l15 = load(args.v15_10, 10), load(args.v15_120, args.long_days)
    if [s.seed for s in s14.seeds] != [s.seed for s in s15.seeds] or [s.seed for s in l14.seeds] != [s.seed for s in l15.seeds]:
        print("the two versions must cover the same seeds", file=sys.stderr)
        return 2
    report = {"x3": x3_bootstrap(p8.winters_table(l14), p8.winters_table(l15)), "medians": {}}
    (m14, one14), (m15, one15) = median_clauses(s14, l14, v11), median_clauses(s15, l15, v11)
    for name in sorted(set(m14) & set(m15)):
        paired = {s: (m14[name][s], m15[name][s]) for s in m14[name] if s in m15[name]}
        report["medians"][name] = bootstrap(paired, median, name in one14 and name in one15)
    flagged = [f"X3:{k}" for k, v in report["x3"].items() if v["excludes_zero"]] + \
              [f"median:{k}" for k, v in report["medians"].items() if v["excludes_zero"]]
    report["flagged"] = flagged
    lines = [f"# {args.title}", "", "Shift = v15 − v14 on the same seeds; 95 % interval from 2,000 seed resamples (paired).", "",
             "| statistic | v14 | v15 | shift | 95 % CI | seeds | excludes 0 |", "|---|---:|---:|---:|---|---:|---|"]
    f = lambda x: "–" if x is None else f"{x:.4g}"
    for group, rows in (("X3", report["x3"]), ("median", report["medians"])):
        for name, r in rows.items():
            lines.append(f"| {group}:{name} | {f(r['v14'])} | {f(r['v15'])} | {f(r['shift'])} | [{f(r['lo'])}, {f(r['hi'])}] | "
                         f"{r['n_seeds']} | {'**yes**' if r['excludes_zero'] else ('summary-level' if r.get('summary_level') else 'no')} |")
    lines += ["", f"Intervals excluding 0: {len(flagged)} ({', '.join(flagged) or 'none'})."]
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "shifts.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    (args.out / "shifts.md").write_text("\n".join(lines) + "\n")
    print(f"flagged {len(flagged)}: {', '.join(flagged)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
