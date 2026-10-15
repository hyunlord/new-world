#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path

from evaluate_pass17_data import CohortSpec, InputError
from evaluate_pass17_evidence import load_evidence
from evaluate_pass17_read import load_cohort
from evaluate_pass17_report import JsonObject, build_report, write_report
from evaluate_pass17_stage_a import evaluate


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Strict Pass 17 Stage A metrics from complete traced paired cohorts.")
    parser.add_argument("--stage", choices=("a",), required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--tuning", action="store_true", help="Use seeds 17100..17115, 60 days; absent identity/performance stays PENDING.")
    parser.add_argument("--identity-json", type=Path, help="Verified schema-1 identity manifest with I1-I4, v19_pins and v20_pins hash-bound checks.")
    parser.add_argument("--perf-json", type=Path, help="perf.json from evaluate_perf_pass17.py; schema 1, stage a.")
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    seeds = tuple(range(17100, 17116)) if args.tuning else tuple(range(17000, 17032))
    days = 60 if args.tuning else 120
    try:
        candidate = load_cohort(args.candidate, CohortSpec("game_v21", seeds, days))
        baseline = load_cohort(args.baseline, CohortSpec("game_v20", seeds, days))
        metrics = evaluate(candidate, baseline)
        identity = load_evidence(args.identity_json, "identity", args.tuning)
        performance = load_evidence(args.perf_json, "performance", args.tuning)
        report = build_report(candidate, baseline, metrics, identity, performance, args.tuning)
    except InputError as error:
        report: JsonObject = {"schema_version": 17, "stage": "a", "mode": "tuning" if args.tuning else "evaluation",
            "verdict": "ERROR", "error": str(error)}
    write_report(report, args.out_json, args.out_md)
    verdict = report["verdict"]
    print(f"{verdict}: {args.out_json}")
    return 2 if verdict == "ERROR" else 1 if verdict == "FAIL" else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
