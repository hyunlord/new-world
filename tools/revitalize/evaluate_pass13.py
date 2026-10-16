#!/usr/bin/env python3
"""Evaluate Pass 13's J targets on schema-13 probe runs (PASS13_GOAL.md 7.5, 10). Stdlib only.

usage: evaluate_pass13.py --round10 DIR --round120 DIR --out DIR
                          [--v16-round10 DIR --v16-round120 DIR] [--v11-targets P6_V11_Y_TARGETS.json]
                          [--extra-round120 DIR ...] [--baseline-summary V16_10DAY.json]
                          [--baseline-summary-long V16_120DAY.json] [--variety DIR ...]
                          [--legacy-pin PASS|FAIL] [--v8-v18-repro PASS|FAIL] [--title TITLE]

DIR holds a schema-13 probe run of one v19 scene (`game_v19_nofission` or `game_v19`). Built on
`evaluate_pass12.py`: J1, J2, J3's clauses, J5 (judged on `game_v19`, pooled over `--round120` and
`--extra-round120`, the 0.35 bound per settlement-winter), J6 (judged on `game_v19_nofission` against game_v16 on the
same seeds, reported for `game_v19`) and J9 (`RANGES_V18`, which v19 draws from) are Pass 12's. What Pass 13 adds:

- J3 reported (`game_v19`, pressure): per settlement the first eligible check's θ and the mean scarcity term s over
  its first seven eligible checks; the Pearson correlation of the first-fission display day with the camp's θ and
  with that early scarcity; the display day P first exceeded θ / 2; P and θ at every split.
- J4 reported (the upper report of 10): the share of daughters dissolved or merged within 120 days and within the
  first winter after their founding (display days (d // 7) % 4 == 3 after the founding day), by founding season.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import evaluate_pass10 as p10
import evaluate_pass11 as p11
import evaluate_pass12 as p12
from evaluate_pass4 import EvidenceError, Round, quantiles, ratio
from evaluate_pass5 import load_round as load_round_schema

SCHEMA = 13
BASELINE_SCENE = p12.BASELINE_SCENE
SEASONS = p12.SEASONS
TPD, OFFSET = 1440, 480

runs = p10.runs


def load_round(path: Path, duration: int) -> Round:
    return load_round_schema(path, duration, SCHEMA)


def load_reference(path: Path, duration: int) -> Round:
    """A game_v16 (or v17/v18) reference round of schema 10 to 13."""
    schema = json.loads((path / "summary.json").read_text()).get("schema_version")
    if schema not in (10, 11, 12, 13):
        raise EvidenceError(f"{path}: reference summary schema must be 10 to 13")
    return load_round_schema(path, duration, schema)


def display_day(tick: int) -> int:
    """Zero-based display day of a simulation tick (480-tick clock offset)."""
    return (int(tick) + OFFSET) // TPD


def first_winter_end(founded_day: int) -> int:
    """Zero-based display day after the last day of the first winter that ends after `founded_day`."""
    d = int(founded_day)
    while (d // 7) % 4 != 3:
        d += 1
    return (d // 7) * 7 + 7


def pressure_rows(long: Round) -> list[dict]:
    """Per seed (game_v19): the camp's first θ, its early scarcity and the first-fission display day."""
    out = []
    fission = {r["seed"]: r for r in p11.fission_rows(long)}
    for run in runs(long):
        block = run.get("pass13") or {}
        checks = [c for c in block.get("pressure_checks", []) if int(c[1]) == 1]
        theta = int(checks[0][3]) if checks else None
        early = [int(c[7]) for c in checks[:7]]
        half = {int(s): int(d) for s, d in block.get("half_threshold_day", [])}
        row = fission.get(run["seed"], {})
        out.append({"seed": run["seed"], "camp_theta": theta, "camp_early_scarcity": (sum(early) / len(early)) if early else None,
                    "camp_eligible_checks": len(checks), "camp_half_threshold_day": half.get(1),
                    "first_fission_day": p12.display_first_day(row) if row else None,
                    "splits": block.get("splits", [])})
    return out


def j3_pressure(long: Round) -> dict:
    rows = pressure_rows(long)
    with_day = [r for r in rows if r["first_fission_day"] is not None]
    corr = {}
    for key in ("camp_theta", "camp_early_scarcity"):
        pairs = [(float(r[key]), float(r["first_fission_day"])) for r in with_day if r[key] is not None]
        corr[key] = {"r": p12.pearson([a for a, _ in pairs], [b for _, b in pairs]), "n": len(pairs)}
    splits = [s for r in rows for s in r["splits"]]
    over = [float(s[3]) / float(s[4]) for s in splits if int(s[4]) > 0]
    thetas = [float(r["camp_theta"]) for r in rows if r["camp_theta"] is not None]
    return {"first_fission_day_correlation": corr, "splits": len(splits),
            "pressure_over_threshold_at_split": quantiles(over) if over else None,
            "camp_theta": quantiles(thetas) if thetas else None,
            "camp_half_threshold_day": quantiles([float(r["camp_half_threshold_day"]) + 1 for r in rows
                                                  if r["camp_half_threshold_day"] is not None]) if rows else None,
            "rows": [{k: v for k, v in r.items() if k != "splits"} for r in rows]}


def j4_upper(long: Round) -> dict:
    """Daughters dissolved or merged within the run and within their first winter, by founding season."""
    rows = p11.daughters(long)
    def first_winter(d: dict) -> bool:
        return d["dissolved_tick"] is not None and display_day(d["dissolved_tick"]) < first_winter_end(d["founded_day"])
    out = {}
    for name in (*SEASONS, "all"):
        group = rows if name == "all" else [d for d in rows if SEASONS[(int(d["founded_day"]) // 7) % 4] == name]
        gone = sum(1 for d in group if d["dissolved_tick"] is not None)
        winter = sum(1 for d in group if first_winter(d))
        out[name] = {"daughters": len(group), "dissolved_or_merged": gone, "share": ratio(gone, len(group)) if group else None,
                     "dissolved_in_first_winter": winter, "first_winter_share": ratio(winter, len(group)) if group else None}
    return out


def relabel_text(text: str) -> str:
    return text.replace("v16", "\0").replace("v15", "game_v16").replace("\0", "v19")


def relabel(value):
    if isinstance(value, dict):
        return {relabel_text(k): relabel(v) for k, v in value.items()}
    if isinstance(value, list):
        return [relabel(v) for v in value]
    return relabel_text(value) if isinstance(value, str) else value


def j6(short: Round, long: Round, v16_short: Round | None, v16_long: Round | None, v11: dict | None) -> tuple[dict, dict]:
    targets, reported = p10.h5(short, long, v16_short, v16_long, v11)
    return relabel(targets), relabel(reported)


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
    parser.add_argument("--v8-v18-repro", choices=("PASS", "FAIL"), default="FAIL")
    parser.add_argument("--title", default="Pass 13 J targets")
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
        targets = {"J1": p11.j1(short, long, {"legacy_pin": args.legacy_pin == "PASS", "v8_v16_repro": args.v8_v18_repro == "PASS"})}
        targets["J2"], j2_seeds = p11.j2(long)
        extra: dict = {}
        if scene == "game_v19":
            targets["J3"], extra["j3"] = p12.j3(long)
            extra["fission"] = extra["j3"].pop("rows")
            extra["j3_pressure"] = j3_pressure(long)
            targets["J4"], extra["daughters"] = p12.j4(long)
            extra["j4_upper"] = j4_upper(long)
        targets["J5"], j5_detail = p12.j5(short, long, [long, *extra_long])
        extra["stage_pool"] = j5_detail["stage_pool"]
        targets["J6"], j6_reported = j6(short, long, v16[0], v16[1], v11)
        if args.variety:
            targets["J9"], extra["variety"] = p12.j9([load_round(p, 10) for p in args.variety])
        if args.baseline_summary:
            targets["J12"] = p11.j12(short, long, p11.baseline_ms(args.baseline_summary), p11.baseline_ms(args.baseline_summary_long))
        text = p11.markdown(args.title, scene, targets, extra)
        if "j3" in extra:
            pr = {k: v for k, v in extra["j3_pressure"].items() if k != "rows"}
            text += ("\n## J3 reported\n\n" + f"Split rate per daily check by tercile of h: {json.dumps(extra['j3']['hazard_terciles'], sort_keys=True)}\n\n"
                     + f"First-fission display day correlation with hidden parameters: {json.dumps(extra['j3']['first_fission_day_correlation'], sort_keys=True)}\n\n"
                     + f"First-fission display day histogram: {json.dumps(extra['j3']['first_fission_day_histogram'])}\n\n"
                     + f"Pressure: {json.dumps(pr, sort_keys=True)}\n")
        if "daughters" in extra:
            text += (f"\n## J4 by founding season\n\n{json.dumps(extra['daughters']['by_founding_season'], sort_keys=True)}\n"
                     + f"\n## J4 upper report (dissolved or merged within the run / the first winter)\n\n{json.dumps(extra['j4_upper'], sort_keys=True)}\n")
        text += f"\n## J5 world-level ratio (Pass 11's, reported)\n\n{json.dumps(j5_detail['world_level_max_share'], sort_keys=True)[:800]}\n"
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    reported_only = {"game_v19": {"J6"}, "game_v19_nofission": {"J5"}}.get(scene, set())
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
