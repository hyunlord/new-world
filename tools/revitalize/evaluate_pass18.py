#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_pass18.py --candidate V23_DIR --baseline V22_DIR [--tuning] --out-md OUT.md --out-json OUT.json
"""Pass 18 G1-G4: game_v23 vs game_v22 with Pass 17's definitions and bootstrap.

Both cohorts are complete `worldsim_probe --agent-trace` runs (16 seeds x 60
days for tuning 19100-19115, 32 x 120 for evaluation 19000-19031). G1/G2/G3's
F5 and G4's health rules reuse Pass 17's family and R2 code unchanged; G3's
adult-elder census and G4's daughter villages read the raw trace and summary.
G5 (identity) and G6 (performance) are judged outside this script.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Mapping, Sequence
from dataclasses import asdict, replace
from pathlib import Path
from statistics import fmean
from typing import Final

from evaluate_pass17_data import Cohort, CohortSpec, Criterion, InputError, Metric
from evaluate_pass17_family_read import read_family
from evaluate_pass17_read import JsonValue, read_json, read_jsonl
from evaluate_pass17_stage_b import evaluate as evaluate_family, fraction_metric, split_exposure
from evaluate_pass17_read import load_cohort

GROUP_MIN: Final = 6
F5_BAR: Final = Criterion(None, 0.05, "split couples / couples at completed fission departures <= 0.05", "share")


def cohort_specs(tuning: bool) -> tuple[CohortSpec, CohortSpec]:
    start, count, days = (19100, 16, 60) if tuning else (19000, 32, 120)
    seeds = tuple(range(start, start + count))
    return CohortSpec("game_v23", seeds, days), CohortSpec("game_v22", seeds, days)


def metric_json(metric: Metric) -> dict[str, JsonValue]:
    row = asdict(metric)
    row["ci95"] = list(metric.ci95) if metric.ci95 is not None else None
    return row


def fission_rows(cohort: Cohort) -> list[tuple[int, Mapping[str, JsonValue]]]:
    rows = []
    for world in cohort.worlds:
        for row in read_jsonl(cohort.source / f"{cohort.scene}_s{world.seed}_family_fissions.jsonl"):
            rows.append((world.seed, row))
    return rows


def elder_census(cohort: Cohort) -> dict[str, JsonValue]:
    """Per completed fission: elders who left, couples and adult-elder couples split."""
    completed = [(seed, row) for seed, row in fission_rows(cohort) if row["status"] == "completed"]
    cases = []
    for seed, row in completed:
        for pair in row["adult_elder_split_pairs"]:
            size = len(row["members"])
            cases.append({"seed": seed, "fission_id": row["fission_id"], "adult": pair["adult"], "elder": pair["elder"],
                "adult_departed": pair["adult_departed"], "group_size": size,
                "group_min_top_up": bool(pair["adult_departed"]) and size == GROUP_MIN})
    return {
        "completed_fissions": len(completed),
        "members_departed": sum(len(row["members"]) for _, row in completed),
        "elders_departed": sum(int(row["elders_departed"]) for _, row in completed),
        "couples_at_departure": sum(int(row["couples_total"]) for _, row in completed),
        "couples_split": sum(int(row["couples_split"]) for _, row in completed),
        "adult_elder_couples_at_departure": sum(int(row["adult_elder_couples"]) for _, row in completed),
        "adult_elder_couples_split": sum(int(row["adult_elder_couples_split"]) for _, row in completed),
        "adult_elder_split_cases": cases,
        "per_fission": [{"seed": seed, "fission_id": row["fission_id"], "size": len(row["members"]),
            "elders_departed": row["elders_departed"], "couples": row["couples_total"], "couples_split": row["couples_split"],
            "adult_elder_couples": row["adult_elder_couples"], "adult_elder_couples_split": row["adult_elder_couples_split"]}
            for seed, row in completed],
    }


def daughters(cohort: Cohort) -> dict[str, JsonValue]:
    """Daughter villages from the probe summary: founders and survival to the run's end."""
    summary = read_json(cohort.source / "summary.json")
    founded = []
    for run in summary["runs"]:
        for settlement in run["pass11"]["settlements"]:
            if settlement["parent"] is None or settlement["founders"] is None:
                continue
            members_at = settlement["members_at"]
            last = members_at[-1] if cohort.days >= 118 else None
            founded.append({"seed": run["seed"], "settlement": settlement["settlement_id"], "founded_day": settlement["founded_day"],
                "founders": settlement["founders"], "members_at": members_at, "dissolved_tick": settlement["dissolved_tick"],
                "alive_at_end": settlement["dissolved_tick"] is None and (last is None or last >= 1)})
    return {
        "daughters": len(founded),
        "mean_founders": fmean(d["founders"] for d in founded) if founded else None,
        "alive_at_end": sum(d["alive_at_end"] for d in founded),
        "survival_share": sum(d["alive_at_end"] for d in founded) / len(founded) if founded else None,
        "definition": "alive_at_end = never dissolved and >= 1 member at the day-118 checkpoint (120-day runs)",
        "villages": founded,
    }


def g3_verdict(f5: Metric, census: Mapping[str, JsonValue]) -> str:
    cases = census["adult_elder_split_cases"]
    unexplained = [case for case in cases if not case["group_min_top_up"]]
    if f5.verdict == "ERROR":
        return "ERROR"
    return "PASS" if f5.verdict == "PASS" and not unexplained else "FAIL"


def build(candidate: Cohort, baseline: Cohort) -> dict[str, JsonValue]:
    families = tuple(read_family(candidate, world) for world in candidate.worlds)
    base_families = tuple(read_family(baseline, world) for world in baseline.worlds)
    measured = evaluate_family(candidate, baseline, families).stage.metrics
    reference = evaluate_family(baseline, baseline, base_families).stage.metrics
    f5 = fraction_metric([split_exposure(world) for world in families], F5_BAR)
    f5_base = fraction_metric([split_exposure(world) for world in base_families], F5_BAR)
    census, base_census = elder_census(candidate), elder_census(baseline)
    health = {key: value for key, value in measured.items() if key.startswith("F6-")}
    g = {
        "G1": {"verdict": measured["F1"].verdict, "F1": metric_json(measured["F1"]), "F1_game_v22": metric_json(reference["F1"])},
        "G2": {"verdict": "PASS" if measured["F2"].verdict == measured["F4"].verdict == "PASS" else "FAIL",
            "F2": metric_json(measured["F2"]), "F4": metric_json(measured["F4"]),
            "F2_game_v22": metric_json(reference["F2"]), "F4_game_v22": metric_json(reference["F4"])},
        "G3": {"verdict": g3_verdict(f5, census), "F5": metric_json(f5), "F5_game_v22": metric_json(f5_base),
            "adult_elder_couples_split": census["adult_elder_couples_split"],
            "adult_elder_split_cases": census["adult_elder_split_cases"]},
        "G4": {"verdict": "PASS" if all(metric.verdict == "PASS" for metric in health.values()) else "FAIL",
            "rules": {key: metric_json(metric) for key, metric in health.items()},
            "daughters_game_v23": daughters(candidate), "daughters_game_v22": daughters(baseline)},
    }
    return {"schema_version": 18, "candidate": candidate.scene, "baseline": baseline.scene, "days": candidate.days,
        "seeds": [world.seed for world in candidate.worlds], "G": g,
        "elder_census": {"game_v23": census, "game_v22": base_census},
        "other_family_metrics_game_v23": {key: metric_json(value) for key, value in measured.items() if key.startswith("F3")}}


def fmt(metric: Mapping[str, JsonValue]) -> str:
    value, ci = metric["value"], metric["ci95"]
    if value is None:
        return f"ERROR ({metric['error']})"
    return f"{value:.4f} [{ci[0]:.4f}, {ci[1]:.4f}]" if ci else f"{value:.4f}"


def markdown(report: Mapping[str, JsonValue], tuning: bool) -> str:
    g = report["G"]
    lines = [f"# Pass 18 {'tuning' if tuning else 'evaluation'}: game_v23 vs game_v22",
        "", f"Seeds {report['seeds'][0]}-{report['seeds'][-1]} ({len(report['seeds'])} worlds), {report['days']} days, paired by seed; "
        "Pass 17 definitions and bootstrap (2,000 world resamples, 95 % percentile interval).", "",
        "| # | rule | game_v23 | game_v22 | verdict |", "|---|---|---|---|---|",
        f"| G1 | F1 partnered share of non-elder adults at day 60 >= 0.50 | {fmt(g['G1']['F1'])} | {fmt(g['G1']['F1_game_v22'])} | {g['G1']['verdict']} |",
        f"| G2 | F2 couple-parent births from day 20 >= 0.90 | {fmt(g['G2']['F2'])} | {fmt(g['G2']['F2_game_v22'])} | {g['G2']['F2']['verdict']} |",
        f"| G2 | F4 couples in the same shelter >= 0.90 | {fmt(g['G2']['F4'])} | {fmt(g['G2']['F4_game_v22'])} | {g['G2']['F4']['verdict']} |",
        f"| G3 | F5 split couples / couples at completed departures <= 0.05 | {fmt(g['G3']['F5'])} | {fmt(g['G3']['F5_game_v22'])} | {g['G3']['F5']['verdict']} |",
        f"| G3 | adult-elder couples split (only group_min top-ups allowed) | {g['G3']['adult_elder_couples_split']} | {report['elder_census']['game_v22']['adult_elder_couples_split']} | {g['G3']['verdict']} |"]
    for key, metric in g["G4"]["rules"].items():
        lines.append(f"| G4 | {key}: {metric['bar']} | {metric['candidate']:.3f} | {metric['baseline']:.3f} | {metric['verdict']} (stat {fmt(metric)}) |")
    lines += ["", "## Elders at completed fissions", "", "| | game_v23 | game_v22 |", "|---|---|---|"]
    for key in ("completed_fissions", "members_departed", "elders_departed", "couples_at_departure", "couples_split",
            "adult_elder_couples_at_departure", "adult_elder_couples_split"):
        lines.append(f"| {key} | {report['elder_census']['game_v23'][key]} | {report['elder_census']['game_v22'][key]} |")
    lines += ["", "Adult-elder split cases in game_v23 (each must be a group_min top-up):", ""]
    cases = g["G3"]["adult_elder_split_cases"]
    lines += [f"- seed {c['seed']} fission {c['fission_id']}: adult {c['adult']} {'left' if c['adult_departed'] else 'stayed'}, elder {c['elder']} {'stayed' if c['adult_departed'] else 'left'}; group {c['group_size']}; top-up {c['group_min_top_up']}" for c in cases] or ["- none"]
    d23, d22 = g["G4"]["daughters_game_v23"], g["G4"]["daughters_game_v22"]
    lines += ["", "## Daughter villages", "", "| | game_v23 | game_v22 |", "|---|---|---|",
        f"| daughters founded | {d23['daughters']} | {d22['daughters']} |",
        f"| mean founders | {d23['mean_founders']} | {d22['mean_founders']} |",
        f"| alive at end | {d23['alive_at_end']} ({d23['survival_share']}) | {d22['alive_at_end']} ({d22['survival_share']}) |",
        "", d23["definition"], "", "G5 (identity) and G6 (performance) are judged outside this script.", ""]
    return "\n".join(lines)


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--tuning", action="store_true")
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    candidate_spec, baseline_spec = cohort_specs(args.tuning)
    try:
        report = build(load_cohort(args.candidate, candidate_spec), load_cohort(args.baseline, baseline_spec))
    except InputError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    args.out_json.write_text(json.dumps(report, indent=2) + "\n")
    args.out_md.write_text(markdown(report, args.tuning))
    print(" ".join(f"{key}={value['verdict']}" for key, value in report["G"].items()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
