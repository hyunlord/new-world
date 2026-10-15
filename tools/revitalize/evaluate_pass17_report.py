from __future__ import annotations

import json
from collections.abc import Mapping
from pathlib import Path

from evaluate_pass17_data import KINDS, Cohort, Metric, StageMetrics, Verdict
from evaluate_pass17_evidence import Evidence, digest
from evaluate_pass17_read import JsonValue, array, mapping, required
from evaluate_pass17_stage_a import group_verdict, health_values, social_exposure

JsonObject = dict[str, JsonValue]
STAGES = ("adult", "child", "elder")
STATES = ("Idle", "Seeking", "ConsumingAgent", "ConsumingOther", "Activity")


def metric_json(metric: Metric) -> JsonObject:
    details: JsonObject | None = None
    if metric.details is not None:
        details = {key: value for key, value in metric.details.items()}
    return {"value": metric.value, "ci95": list(metric.ci95) if metric.ci95 is not None else None,
        "verdict": metric.verdict, "bar": metric.bar, "unit": metric.unit,
        "candidate": metric.candidate, "baseline": metric.baseline,
        "bootstrap_valid": metric.bootstrap_valid, "bootstrap_replicates": metric.bootstrap_replicates,
        "details": details, "error": metric.error}


def evidence_json(evidence: Evidence) -> JsonObject:
    return {"verdict": evidence.verdict, "source": evidence.source, "sha256": evidence.sha256,
        "payload": dict(evidence.payload) if evidence.payload is not None else None, "error": evidence.error}


def hourly_summary(cohort: Cohort) -> list[JsonValue]:
    output: list[JsonValue] = []
    for minute in range(0, 1440, 60):
        hours = [hour for world in cohort.worlds for hour in world.hours if hour.minute == minute]
        for stage_index, stage in enumerate(STAGES):
            counts = [sum(hour.states[stage_index][index] for hour in hours) for index in range(5)]
            denominator = sum(counts)
            output.append({"minute": minute, "stage": stage, "observations": denominator,
                "counts": {state: count for state, count in zip(STATES, counts, strict=True)},
                "shares": {state: count / denominator if denominator else None for state, count in zip(STATES, counts, strict=True)}})
    return output


def activity_summary(cohort: Cohort) -> list[JsonValue]:
    all_minutes = [sum(world.all_activity_minutes[index] for world in cohort.worlds) for index in range(7)]
    adult_minutes = [sum(person.activity_minutes[index] for world in cohort.worlds for person in world.people) for index in range(7)]
    return [{"kind": kind, "all_life_stages_minutes": all_minutes[index], "adult_minutes": adult_minutes[index],
        "all_life_stages_share": all_minutes[index] / sum(all_minutes) if sum(all_minutes) else None,
        "adult_share": adult_minutes[index] / sum(adult_minutes) if sum(adult_minutes) else None,
        "all_life_stages_starts": sum(world.all_activities_started[index] for world in cohort.worlds)}
        for index, kind in enumerate(KINDS)]


def cohort_json(cohort: Cohort) -> JsonObject:
    health: list[JsonValue] = []
    files = [cohort.source / "summary.json"]
    for world in cohort.worlds:
        values: JsonObject = {key: value for key, value in health_values(world).items()}
        completions, exposure = social_exposure(world)
        values.update({"seed": world.seed, "old_age_deaths": world.old_age_deaths,
            "social_completed": completions, "adult_days": exposure,
            "social_rate": completions / exposure if exposure else None,
            "no_place_redraws": world.no_place_redraws, "no_place_failures": world.no_place_failures})
        health.append(values)
        files.extend(cohort.source / f"{cohort.scene}_s{world.seed}_{suffix}" for suffix in
            ("agents.jsonl", "people.jsonl", "hours.jsonl", "trace_meta.json"))
    hashes: JsonObject = {str(path): digest(path) for path in files}
    return {"scene": cohort.scene, "source": str(cohort.source), "days": cohort.days,
        "seeds": [world.seed for world in cohort.worlds], "worlds": len(cohort.worlds),
        "hourly_states": hourly_summary(cohort), "activity_minutes": activity_summary(cohort),
        "health_by_world": health, "inputs_sha256": hashes}


def build_report(candidate: Cohort, baseline: Cohort, metrics: StageMetrics,
                 identity: Evidence, performance: Evidence, tuning: bool) -> JsonObject:
    verdicts: dict[str, Verdict] = dict(metrics.verdicts)
    verdicts.update({"R4": identity.verdict, "R5": performance.verdict})
    return {"schema_version": 17, "stage": "a", "mode": "tuning" if tuning else "evaluation",
        "verdict": group_verdict(list(verdicts.values())),
        "rules": {key: value for key, value in verdicts.items()},
        "metrics": {key: metric_json(value) for key, value in metrics.metrics.items()},
        "identity": evidence_json(identity), "performance": evidence_json(performance),
        "candidate": cohort_json(candidate), "baseline": cohort_json(baseline),
        "methods": {"bootstrap": "2,000 world resamples; fresh numpy.default_rng(0) per statistic; percentile 95% CI; undefined draws excluded and count reported",
            "R1": "Pooled nonelder-adult hourly observations from 08:00 through 17:00 inclusive; Idle state 0",
            "R2": "Paired worlds; equal-world health means; daily noon stores sum settlements; social primary pools completions/adult-days, mean world rate is sensitivity",
            "shelters": "Final minus initial complete Shelter count, including daughter-camp boot shelters",
            "R3": "Pooled-person Spearman average tie ranks; >=14,400 nonelder-adult ticks; zero-activity NS/RD shares=0; P requires >=3 normally completed activities",
            "activity_minutes": "Consuming Activity ticks only, excluding travel; adult and all-life-stage totals shown separately"}}


def display(value: JsonValue) -> str:
    if value is None:
        return "undefined"
    if isinstance(value, float):
        return f"{value:.6g}"
    if isinstance(value, list):
        return "[" + ", ".join(display(item) for item in value) + "]"
    return str(value)


def render(report: Mapping[str, JsonValue]) -> str:
    lines = [f"# Pass 17 Stage A — {report['verdict']}", "", f"Mode: {report['mode']}.", "",
        "| Rule | Verdict |", "|---|---|"]
    for key, value in mapping(required(report, "rules"), "rules").items():
        lines.append(f"| {key} | {value} |")
    lines += ["", "| Metric | Candidate | Baseline | Estimate | 95% CI | Valid bootstrap draws | Bar | Verdict |",
        "|---|---:|---:|---:|---|---:|---|---|"]
    for key, value in mapping(required(report, "metrics"), "metrics").items():
        row = mapping(value, key)
        lines.append(f"| {key} | {display(row['candidate'])} | {display(row['baseline'])} | {display(row['value'])} | {display(row['ci95'])} | {row['bootstrap_valid']}/{row['bootstrap_replicates']} | {row['bar']} | {row['verdict']} |")
        if row["error"]:
            lines += ["", f"{key}: {row['error']}", ""]
    lines += ["", "## Methods", ""]
    lines.extend(f"- {key}: {value}" for key, value in mapping(required(report, "methods"), "methods").items())
    lines += ["", "## Eligibility and sensitivity", "", "```json",
        json.dumps({key: mapping(value, key)["details"] for key, value in mapping(report["metrics"], "metrics").items() if mapping(value, key)["details"] is not None}, indent=2, allow_nan=False), "```"]
    for side in ("baseline", "candidate"):
        cohort = mapping(required(report, side), side)
        lines += ["", f"## {cohort['scene']}", "", f"{cohort['worlds']} worlds × {cohort['days']} days; seeds {display(cohort['seeds'])}.", "",
            "| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |",
            "|---|---|---:|---:|---:|---:|---:|---:|"]
        for value in array(cohort["hourly_states"], "hourly_states"):
            row = mapping(value, "hourly_states")
            shares = mapping(row["shares"], "shares")
            lines.append("| " + " | ".join([display(row["minute"]), display(row["stage"]), display(row["observations"])] + [display(shares[state]) for state in STATES]) + " |")
        for field in ("activity_minutes", "health_by_world"):
            lines += ["", f"### {field}", "", "```json", json.dumps(cohort[field], indent=2, allow_nan=False), "```"]
    for field in ("identity", "performance"):
        lines += ["", f"## {field}", "", "```json", json.dumps(report[field], indent=2, allow_nan=False), "```"]
    return "\n".join(lines) + "\n"


def write_report(report: Mapping[str, JsonValue], out_json: Path, out_md: Path) -> None:
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(report, indent=2, allow_nan=False) + "\n")
    if report.get("error") is not None:
        out_md.write_text(f"# Pass 17 Stage A — ERROR\n\n{report['error']}\n")
    else:
        out_md.write_text(render(report))
