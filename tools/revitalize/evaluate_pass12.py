#!/usr/bin/env python3
"""Evaluate Pass 12's J targets on schema-12 probe runs (PASS12_GOAL.md 7.6, 10). Stdlib only.

usage: evaluate_pass12.py --round10 DIR --round120 DIR --out DIR
                          [--v16-round10 DIR --v16-round120 DIR] [--v11-targets P6_V11_Y_TARGETS.json]
                          [--extra-round120 DIR ...] [--baseline-summary V16_10DAY.json]
                          [--baseline-summary-long V16_120DAY.json] [--variety DIR ...]
                          [--legacy-pin PASS|FAIL] [--v8-v17-repro PASS|FAIL] [--title TITLE]

DIR holds a schema-12 probe run of one v18 scene (`game_v18_nofission` or `game_v18`). Built on
`evaluate_pass11.py` (J1, J2, J6's comparison and escape table, J12 unchanged); what Pass 12 changes:

- J3 (`game_v18`; fission comes when the village is under pressure): share of seeds with >= 1 fission in
  [0.50, 0.90]; first-fission day = the probe's zero-based `first_fission_day` + 1 (display day) before every
  statistic: IQR >= 14 days, median in [20, 80], no single day holds more than 0.25 of the first fissions; share of
  seeds with >= 2 settlements alive at day 120 in [0.30, 0.85]; no seed above `max_settlements`; cancellations <= 0.25
  of attempts. Reported: the split rate per daily check by tercile of the hazard h (every logged check of
  `runs[].pass12.fission_checks`), and the Pearson correlation of the first-fission day with each hidden parameter.
- J4 (`game_v18`): daughters alive 28 days after founding >= 0.70; the share dissolved or merged by founding season.
- J5 (judged on `game_v18`, pooled over `--round120` and `--extra-round120`): Pass 11's stage shares and per-hour
  clause; Pass 10's H3 on this round, except that the 0.35 bound is judged per settlement-winter: the maximum over
  settlement-winters of exposure deaths <= 0.35 x that settlement's members at the winter's start
  (`runs[].pass12.settlement_winters`, members at 18:00 of the winter's first day, a settlement founded inside the
  winter with its founders); Pass 11's world-level ratio is reported beside it.
- J6 (judged on `game_v18_nofission`, reported for `game_v18`): Pass 10's H5 against game_v16 on the same seeds.
- J9 (`--variety`: the 10-day rounds): every drawn parameter's mean within +-10 % of its `RANGES_V18` range centre;
  realized region bushes and deadwood >= 0.95 of requested over the given seeds; region placement failures 0.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

import evaluate_pass10 as p10
import evaluate_pass11 as p11
from evaluate_pass4 import EvidenceError, Round, clause, quantiles, ratio, target
from evaluate_pass5 import load_round as load_round_schema

SCHEMA = 12
BASELINE_SCENE = "game_v16"
RANGES_V18 = {**p11.RANGES, "region_bushes": (8, 12), "region_deadwood": (12, 18)}
SEASONS = ("spring", "summer", "autumn", "winter")
HIDDEN = ("starting_agents", "berry_bushes", "berry_capacity", "deadwood_spots", "deadwood_capacity", "forest_permille",
          "climate", "food_regen_permille", "wild_regions", "region_bushes", "region_deadwood")

runs = p10.runs


def load_round(path: Path, duration: int) -> Round:
    return load_round_schema(path, duration, SCHEMA)


def load_reference(path: Path, duration: int) -> Round:
    """A game_v16 (or v17) reference round of schema 10, 11 or 12."""
    schema = json.loads((path / "summary.json").read_text()).get("schema_version")
    if schema not in (10, 11, 12):
        raise EvidenceError(f"{path}: reference summary schema must be 10, 11 or 12")
    return load_round_schema(path, duration, schema)


def display_first_day(row: dict) -> float | None:
    """The probe's zero-based first-fission day as a one-based display day."""
    return None if row["first_fission_day"] is None else float(row["first_fission_day"]) + 1


def pearson(xs: list[float], ys: list[float]) -> float | None:
    n = len(xs)
    if n < 3:
        return None
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    syy = sum((y - my) ** 2 for y in ys)
    if sxx <= 0 or syy <= 0:
        return None
    return sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / (sxx * syy) ** 0.5


def hazard_terciles(long: Round) -> dict:
    """Split rate per daily check by tercile of h (equal-count groups over every logged check, sorted by h)."""
    checks = [(int(c[8]), int(c[10])) for run in runs(long) for c in run["pass12"]["fission_checks"]]
    if not checks:
        return {"n": 0}
    checks.sort()
    n = len(checks)
    out = {"n": n}
    for i, name in enumerate(("low", "mid", "high")):
        group = checks[i * n // 3:(i + 1) * n // 3]
        out[name] = {"n": len(group), "h_range": [group[0][0], group[-1][0]] if group else None,
                     "split_rate": ratio(sum(s for _, s in group), len(group)) if group else None}
    out["checks_per_seed"] = quantiles([float(len(run["pass12"]["fission_checks"])) for run in runs(long)])
    return out


def j3(long: Round) -> tuple[dict, dict]:
    rows = p11.fission_rows(long)
    n = len(rows)
    with_fission = ratio(sum(1 for r in rows if r["fissions"] >= 1), n) if n else None
    days = [d for d in (display_first_day(r) for r in rows) if d is not None]
    q = quantiles(days) if days else {"n": 0}
    iqr = (p11.percentile(days, .75) - p11.percentile(days, .25)) if days else None
    counts = Counter(days)
    top_day, top_n = counts.most_common(1)[0] if counts else (None, 0)
    top_share = ratio(top_n, len(days)) if days else None
    two = ratio(sum(1 for r in rows if (r["alive_last"] or 0) >= 2), n) if n else None
    over = [r["seed"] for r in rows if r["max_settlements"] is not None and r["max_alive_settlements"] > r["max_settlements"]]
    attempts = sum(r["attempts"] for r in rows)
    cancelled = sum(r["cancelled"] for r in rows)
    cancel_share = ratio(cancelled, attempts) if attempts else 0.0
    variety = {run["seed"]: run["seed_info"].get("world_variety") or {} for run in runs(long)}
    corr = {}
    for key in HIDDEN:
        pairs = [(float(variety[r["seed"]][key]), display_first_day(r)) for r in rows
                 if r["first_fission_day"] is not None and key in variety[r["seed"]]]
        corr[key] = pearson([a for a, _ in pairs], [b for _, b in pairs])
    reported = {"hazard_terciles": hazard_terciles(long), "first_fission_day_correlation": corr,
                "first_fission_day_histogram": {str(int(k)): v for k, v in sorted(counts.items())}}
    return target({
        "share_with_fission": clause(with_fission is not None and .50 <= with_fission <= .90, {"share": with_fission, "n": n},
                                     "share of 120-day seeds with >= 1 fission in [0.50, 0.90]"),
        "first_fission_iqr": clause(iqr is not None and iqr >= 14, {"iqr": iqr, "p25": p11.percentile(days, .25) if days else None,
                                                                   "p75": p11.percentile(days, .75) if days else None},
                                    "first-fission display day interquartile range >= 14 days"),
        "first_fission_median": clause(bool(days) and 20 <= q["median"] <= 80, q, "first-fission display day median in [20, 80]"),
        "first_fission_no_peak": clause(top_share is not None and top_share <= .25, {"day": top_day, "count": top_n, "share": top_share},
                                        "no single display day holds more than 0.25 of the first fissions"),
        "two_alive_day_end": clause(two is not None and .30 <= two <= .85, {"share": two, "n": n},
                                    f"share of seeds with >= 2 settlements alive on day {long.duration} in [0.30, 0.85]"),
        "never_above_max": clause(not over, {"over": over}, "no seed above max_settlements alive at any daily sample"),
        "cancellations": clause(cancel_share <= .25, {"cancelled": cancelled, "attempts": attempts, "share": cancel_share},
                                "cancellations <= 0.25 of attempts"),
    }), {"rows": rows, **reported}


def j4(long: Round) -> tuple[dict, dict]:
    rows = p11.daughters(long)
    judged = [d for d in rows if d["alive_28_days_after_founding"] is not None]
    alive = sum(1 for d in judged if d["alive_28_days_after_founding"])
    share = ratio(alive, len(judged)) if judged else None
    by_season = {}
    for name in SEASONS:
        group = [d for d in rows if SEASONS[((int(d["founded_day"])) // 7) % 4] == name]
        gone = sum(1 for d in group if d["dissolved_tick"] is not None)
        j = [d for d in group if d["alive_28_days_after_founding"] is not None]
        by_season[name] = {"daughters": len(group), "dissolved_or_merged": gone,
                           "dissolved_or_merged_share": ratio(gone, len(group)) if group else None,
                           "alive_28d_share": ratio(sum(1 for d in j if d["alive_28_days_after_founding"]), len(j)) if j else None}
    gone = sum(1 for d in rows if d["dissolved_tick"] is not None)
    detail = {"daughters": len(rows), "judged_28d": len(judged), "alive_28d": alive,
              "dissolved_or_merged": gone, "dissolved_or_merged_share": ratio(gone, len(rows)) if rows else None,
              "by_founding_season": by_season,
              "founders": quantiles([float(d["founders"]) for d in rows if d["founders"] is not None]) if rows else None}
    return target({"daughters_viable": clause(share is not None and share >= .70, {"share": share, **detail},
                                              "share of daughters alive 28 days after founding >= 0.70")}), detail


def settlement_winters(rounds: list[Round]) -> list[dict]:
    out = []
    for r in rounds:
        for run in runs(r):
            for w in run["pass12"]["settlement_winters"]:
                for s in w["settlements"]:
                    m = int(s["members_at_winter_start"])
                    d = int(s["exposure_deaths"])
                    out.append({"seed": run["seed"], "first_day": w["first_day"], "settlement": s["settlement"],
                                "members": m, "deaths": d, "founded_inside": s["founded_inside"],
                                "ratio": (d / m) if m else (0.0 if d == 0 else float("inf"))})
    return out


def j5(short: Round, long: Round, pooled: list[Round]) -> tuple[dict, dict]:
    target_, detail = p11.j5(short, long, pooled)
    clauses = dict(target_["clauses"])
    world = clauses.pop("H3_max_share_of_members")
    sw = settlement_winters(pooled)
    over = [w for w in sw if w["ratio"] > .35]
    worst = max((w["ratio"] for w in sw), default=None)
    clauses["max_share_settlement_winter"] = clause(bool(sw) and not over,
                                                    {"settlement_winters": len(sw), "over": over[:20], "n_over": len(over), "max": worst},
                                                    "every settlement-winter: exposure deaths <= 0.35 x that settlement's members at the winter's start")
    detail["world_level_max_share"] = world["observed"]
    detail["world_level_max_share_passed"] = world["passed"]
    return target(clauses), detail


def relabel(value):
    """p10.h5 labels the candidate `v16` and the reference `v15`; here the candidate is a v18 scene."""
    if isinstance(value, dict):
        return {relabel_text(k): relabel(v) for k, v in value.items()}
    if isinstance(value, list):
        return [relabel(v) for v in value]
    return relabel_text(value) if isinstance(value, str) else value


def relabel_text(text: str) -> str:
    return text.replace("v16", "\0").replace("v15", "game_v16").replace("\0", "v18")


def j6(short: Round, long: Round, v16_short: Round | None, v16_long: Round | None, v11: dict | None) -> tuple[dict, dict]:
    targets, reported = p10.h5(short, long, v16_short, v16_long, v11)
    return relabel(targets), relabel(reported)


def j9(rounds: list[Round]) -> tuple[dict, dict]:
    h6, detail = p10.h6(rounds, RANGES_V18, p10.CLIMATE_PERMILLE)
    regions = [run["seed_info"].get("wild_regions") for r in rounds for run in runs(r)]
    missing = sum(1 for g in regions if g is None)
    failures = [{"requested": g["regions_requested"], "placed": g["regions_placed"]} for g in regions
                if g and g["regions_placed"] < g["regions_requested"]]
    b_req = sum(int(g["region_bushes_requested"]) for g in regions if g)
    b_got = sum(int(g["region_bushes_placed"]) for g in regions if g)
    w_req = sum(int(g["region_deadwood_requested"]) for g in regions if g)
    w_got = sum(int(g["region_deadwood_placed"]) for g in regions if g)
    fill = Counter()
    for r in rounds:
        for run in runs(r):
            for i, n in enumerate(run["pass12"]["region_fill_bushes"]):
                fill[f"bush_tier_{i + 2}"] += int(n)
            fill["deadwood_tier_2"] += int(run["pass12"]["region_fill_deadwood"])
    water = [int(c["water_distance"]) for g in regions if g for c in g["centres"]]
    detail["regions"] = {"seeds": len(regions), "without_block": missing, "placement_failures": failures,
                         "bushes": [b_got, b_req], "deadwood": [w_got, w_req], "fill_by_tier": dict(fill),
                         "water_distance": quantiles([float(w) for w in water]) if water else None}
    clauses = dict(h6["clauses"])
    clauses["region_bushes_realized"] = clause(b_req > 0 and b_got >= .95 * b_req, {"placed": b_got, "requested": b_req,
                                               "share": ratio(b_got, b_req) if b_req else None},
                                               "realized region bushes >= 0.95 of requested")
    clauses["region_deadwood_realized"] = clause(w_req > 0 and w_got >= .95 * w_req, {"placed": w_got, "requested": w_req,
                                                 "share": ratio(w_got, w_req) if w_req else None},
                                                 "realized region deadwood >= 0.95 of requested")
    clauses["region_placement"] = clause(missing == 0 and not failures, detail["regions"], "region placement failures 0")
    return target(clauses), detail


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    parser.add_argument("--extra-round120", type=Path, nargs="*", default=[])
    parser.add_argument("--v16-round10", type=Path)
    parser.add_argument("--v16-round120", type=Path)
    parser.add_argument("--v11-targets", type=Path)
    parser.add_argument("--baseline-summary", type=Path)
    parser.add_argument("--baseline-summary-long", type=Path)
    parser.add_argument("--variety", type=Path, nargs="*", default=[])
    parser.add_argument("--legacy-pin", choices=("PASS", "FAIL"), default="FAIL")
    parser.add_argument("--v8-v17-repro", choices=("PASS", "FAIL"), default="FAIL")
    parser.add_argument("--title", default="Pass 12 J targets")
    args = parser.parse_args(argv)
    try:
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        scene = long.summary.get("scene")
        if short.summary.get("scene") != scene:
            raise EvidenceError("both rounds must be the same scene")
        v16 = (load_reference(args.v16_round10, 10), load_reference(args.v16_round120, args.long_days)) \
            if args.v16_round10 else (None, None)
        if v16[0] is not None and (v16[0].summary.get("scene"), v16[1].summary.get("scene")) != (BASELINE_SCENE, BASELINE_SCENE):
            raise EvidenceError(f"--v16-round* must be {BASELINE_SCENE} runs")
        v11 = json.loads(args.v11_targets.read_text())["measures"] if args.v11_targets else None
        extra_long = [load_round(p, args.long_days) for p in args.extra_round120]
        targets = {"J1": p11.j1(short, long, {"legacy_pin": args.legacy_pin == "PASS", "v8_v16_repro": args.v8_v17_repro == "PASS"})}
        targets["J2"], j2_seeds = p11.j2(long)
        extra: dict = {}
        if scene == "game_v18":
            targets["J3"], extra["j3"] = j3(long)
            extra["fission"] = extra["j3"].pop("rows")
            targets["J4"], extra["daughters"] = j4(long)
        targets["J5"], j5_detail = j5(short, long, [long, *extra_long])
        extra["stage_pool"] = j5_detail["stage_pool"]
        targets["J6"], j6_reported = j6(short, long, v16[0], v16[1], v11)
        if args.variety:
            targets["J9"], extra["variety"] = j9([load_round(p, 10) for p in args.variety])
        if args.baseline_summary:
            targets["J12"] = p11.j12(short, long, p11.baseline_ms(args.baseline_summary), p11.baseline_ms(args.baseline_summary_long))
        text = p11.markdown(args.title, scene, targets, extra)
        if "j3" in extra:
            text += ("\n## J3 reported\n\n" + f"Split rate per daily check by tercile of h: {json.dumps(extra['j3']['hazard_terciles'], sort_keys=True)}\n\n"
                     + f"First-fission display day correlation with hidden parameters: {json.dumps(extra['j3']['first_fission_day_correlation'], sort_keys=True)}\n\n"
                     + f"First-fission display day histogram: {json.dumps(extra['j3']['first_fission_day_histogram'])}\n")
        if "daughters" in extra:
            text += f"\n## J4 by founding season\n\n{json.dumps(extra['daughters']['by_founding_season'], sort_keys=True)}\n"
        text += f"\n## J5 world-level ratio (Pass 11's, reported)\n\n{json.dumps(j5_detail['world_level_max_share'], sort_keys=True)[:800]}\n"
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    reported_only = {"game_v18": {"J6"}, "game_v18_nofission": {"J5"}}.get(scene, set())
    for name in reported_only & set(targets):
        targets[name]["reported_only"] = True
    passed = all(t["passed"] for name, t in targets.items() if name not in reported_only)
    report = {"schema_version": 1, "probe_schema": SCHEMA, "scene": scene, "passed": passed, "targets": targets,
              "j2_per_seed": j2_seeds, "j5": j5_detail, "j6_reported": j6_reported, "extra": extra,
              "seeds": {"round10": short.summary["seeds"], "round120": long.summary["seeds"]},
              "inputs": {k: (str(v) if isinstance(v, Path) else [str(p) for p in v] if isinstance(v, list) else v)
                         for k, v in vars(args).items() if k != "out"}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "j_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True, default=str) + "\n")
    (args.out / "j_targets.md").write_text(text)
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'}{' (reported only)' if entry.get('reported_only') else ''} "
              + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
