#!/usr/bin/env python3
"""Evaluate Pass 8's X targets on schema-9 probe runs (PASS9_GOAL.md 6.4, 9). Stdlib only.

usage: evaluate_pass9.py --round10 DIR --round120 DIR --out DIR
                         [--compare-to REFERENCE_X_TARGETS.json] [--v11-targets P6_V11_Y_TARGETS.json]
                         [--baseline-summary V14_10DAY.json] [--baseline-summary-long V14_120DAY.json]
                         [--long-days 120] [--cold-window START,END] [--gathered-floor 40.5]
                         [--legacy-pin PASS|FAIL] ... [--x6 PASS|FAIL] [--x7 PASS|FAIL] [--title TITLE]

The evaluation is `evaluate_pass8.evaluate` unchanged (X1-X8, the same
clauses, thresholds and comparison mode: with `--compare-to` an
evaluate_pass8 or evaluate_pass9 output of game_v12 on the same seeds, a
per-seed clause passes when this run's failing seeds number at most the
reference's + 1). Differences from evaluate_pass8: the rounds must be probe
schema 9 (the summary's `rules` block carries `keyed_draws` and
`noise_replicate`); X8's timing baseline is a same-host game_v14 summary
(Y8: v15 <= 1.2 x v14); the report records the run's `keyed_draws`.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import evaluate_pass8 as p8
from evaluate_pass4 import EvidenceError, Round
from evaluate_pass5 import load_round as load_round_schema
from evaluate_pass7 import measures as measures_pass7

SCHEMA = 9
BASELINE_SCENE = "game_v14"


def load_round(path: Path, duration: int) -> Round:
    """Load a schema-9 round; every seed must have every day checkpoint."""
    return load_round_schema(path, duration, SCHEMA)


def keyed_draws(r: Round) -> int | None:
    """`keyed_draws` of the run's rules block (0/1), None when absent."""
    value = (r.summary.get("rules") or {}).get("keyed_draws")
    return None if value is None else int(value)


def baseline_ms(path: Path | None) -> float | None:
    """Median ms per agent-tick of a same-host game_v14 summary (any schema)."""
    if path is None:
        return None
    source = json.loads(path.read_text())
    if source["scene"] != BASELINE_SCENE:
        raise EvidenceError(f"timing baseline must be a {BASELINE_SCENE} summary")
    return float(source["timing"]["ms_per_agent_tick"]["median"])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    parser.add_argument("--compare-to", type=Path, help="x_targets.json of a game_v12 evaluation on the same seeds")
    parser.add_argument("--v11-targets", type=Path, help="y_targets.json of Pass 6's game_v11 evaluation (p6-1_v11_targets)")
    parser.add_argument("--baseline-summary", type=Path, help="same-host game_v14 summary.json (10 days)")
    parser.add_argument("--baseline-summary-long", type=Path, help="same-host game_v14 summary of the long duration")
    parser.add_argument("--cold-window", default="1320,360")
    parser.add_argument("--gathered-floor", type=float, default=p8.GATHERED_FLOOR)
    parser.add_argument("--title", default="Pass 9 X targets")
    for name in (*p8.RECEIPTS, "x6", "x7"):
        parser.add_argument(f"--{name.replace('_', '-')}", choices=("PASS", "FAIL"), default="FAIL")
    args = parser.parse_args(argv)
    try:
        start, end = (int(v) for v in args.cold_window.split(","))
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        weights = p8.rules_weights(long)
        reference = None
        if args.compare_to:
            ref = json.loads(args.compare_to.read_text())
            if "per_seed" not in ref:
                raise EvidenceError("--compare-to must be an evaluate_pass8/9 output (per_seed values)")
            mine = {"round10": short.summary["seeds"], "round120": long.summary["seeds"]}
            if ref.get("seeds") != mine:
                raise EvidenceError(f"--compare-to covers seeds {ref.get('seeds')}, this run {mine}")
            reference = ref["per_seed"]
        v11 = json.loads(args.v11_targets.read_text())["measures"] if args.v11_targets else None
        receipts = {name: getattr(args, name) == "PASS" for name in p8.RECEIPTS}
        targets, per_seed = p8.evaluate(short, long, receipts, v11, weights, (start, end), reference,
                                        baseline_ms(args.baseline_summary), baseline_ms(args.baseline_summary_long),
                                        args.x6 == "PASS", args.x7 == "PASS", args.gathered_floor)
        for clause in targets["X8"]["clauses"].values():
            clause["requirement"] = f"median ms/agent-tick <= 1.2 x same-host {BASELINE_SCENE}, recorder off"
        own = measures_pass7(long, (start, end))
        winters = p8.winters_table(long)
        text = p8.markdown(targets, per_seed, long, end, args.title, weights, "summary.json rules block")
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for name, t in targets.items() if name != "Y9")
    report = {"schema_version": 1, "probe_schema": SCHEMA, "passed": passed, "targets": targets, "per_seed": per_seed,
              "measures": own, "winters": winters, "winters_by_severity": p8.winters_by_severity(winters),
              "dark_nights_by_severity": p8.dark_nights_by_severity(winters), "r2": p8.r_squared(winters),
              "work_by_season": p8.work_by_season(long), "wood_by_season": p8.wood_by_season(long, end),
              "severity_permille": list(weights) if weights else None, "severity_source": "summary.json rules block",
              "rules": long.summary.get("rules"), "keyed_draws": keyed_draws(long), "scene": long.summary.get("scene"),
              "seeds": {"round10": short.summary["seeds"], "round120": long.summary["seeds"]},
              "cold_window": [start, end],
              "inputs": {"round10": str(args.round10), "round120": str(args.round120),
                         "compare_to": str(args.compare_to) if args.compare_to else None,
                         "v11_targets": str(args.v11_targets) if args.v11_targets else None}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "x_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    (args.out / "x_targets.md").write_text(text)
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
