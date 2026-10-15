#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Literal

from evaluate_pass17_data import CohortSpec, InputError
from evaluate_pass17_evidence import load_evidence
from evaluate_pass17_family_read import load_families
from evaluate_pass17_family_report import build_family_report
from evaluate_pass17_read import load_cohort
from evaluate_pass17_report import JsonObject, build_report, write_report
from evaluate_pass17_stage_a import evaluate
from evaluate_pass17_stage_b import evaluate as evaluate_family


def cohort_specs(stage: Literal["a", "b"], tuning: bool) -> tuple[CohortSpec, CohortSpec]:
    start = (17100 if tuning else 17000) if stage == "a" else (18100 if tuning else 18000)
    seeds = tuple(range(start, start + (16 if tuning else 32)))
    days = 60 if tuning else 120
    candidate, baseline = ("game_v21", "game_v20") if stage == "a" else ("game_v22", "game_v21")
    return CohortSpec(candidate, seeds, days), CohortSpec(baseline, seeds, days)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Strict Pass 17 Stage A/B metrics from complete traced paired cohorts.")
    parser.add_argument("--stage", choices=("a", "b"), required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--tuning", action="store_true", help="Use 16 seeds, 60 days: A 17100..17115, B 18100..18115; absent identity/performance stays PENDING.")
    parser.add_argument("--identity-json", type=Path, help="Verified schema-1 matching-stage manifest: I1-I4 (B also I5), v19_pins and v20_pins.")
    parser.add_argument("--perf-json", type=Path, help="perf.json from evaluate_perf_pass17.py; schema 1, matching stage.")
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    candidate_spec, baseline_spec = cohort_specs(args.stage, args.tuning)
    try:
        candidate = load_cohort(args.candidate, candidate_spec)
        baseline = load_cohort(args.baseline, baseline_spec)
        identity = load_evidence(args.identity_json, "identity", args.tuning, args.stage)
        performance = load_evidence(args.perf_json, "performance", args.tuning, args.stage)
        if args.stage == "a":
            report = build_report(candidate, baseline, evaluate(candidate, baseline), identity, performance, args.tuning)
        else:
            families = load_families(candidate)
            measured = evaluate_family(candidate, baseline, families)
            report = build_family_report(candidate, baseline, measured, families, identity, performance, args.tuning)
    except InputError as error:
        report: JsonObject = {"schema_version": 17, "stage": args.stage, "mode": "tuning" if args.tuning else "evaluation",
            "verdict": "ERROR", "error": str(error)}
    write_report(report, args.out_json, args.out_md)
    verdict = report["verdict"]
    print(f"{verdict}: {args.out_json}")
    return 2 if verdict == "ERROR" else 1 if verdict == "FAIL" else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
