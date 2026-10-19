#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/art_pipeline/compare_perf_p13.py BASE AFTER IDENTITY OUTPUT_PREFIX
# Optional: --running-check FILE --activities FILE --inspector FILE; '-' means missing input.
"""Compare validated Art Pass 13 evidence without converting absence into success."""
from __future__ import annotations

import math
import json
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import TypeAlias

import summarize_perf_p13 as perf
import compare_running_identity_p13 as running_identity

Record: TypeAlias = dict[str, perf.Json]


def read(path: str) -> Record:
    if path == "-" or not Path(path).is_file():
        return {}
    raw: perf.Json = json.loads(Path(path).read_text())
    return dict(perf.mapping(raw))


def aggregates(path: str, phase: str) -> Record:
    payload = read(path)
    if not payload:
        return {}
    if payload.get("phase") != phase:
        raise perf.EvidenceError(f"Expected {phase} aggregate")
    sessions: Record = {}
    sources = perf.sequence(payload.get("sources", []))
    rows = perf.sequence(payload.get("sessions", []))
    if len(sources) != len(rows):
        raise perf.EvidenceError("Aggregate lacks original session sources")
    for source, raw in zip(sources, rows):
        session = perf.load_session(Path(str(source)))
        if dict(session.payload) != raw or session.phase != phase or session.world in sessions:
            raise perf.EvidenceError("Aggregate differs from original validated evidence")
        if session.payload.get("valid") is not True or session.payload.get("errors") != []:
            raise perf.EvidenceError("Invalid aggregate session")
        sessions[session.world] = dict(session.payload)
    return sessions


def samples(sessions: Record, world: str, condition: str, zoom: int, speed: int) -> list[perf.Json]:
    session = perf.mapping(sessions.get(world, {}))
    return [sample for raw in perf.sequence(session.get("blocks", []))
            if (block := perf.mapping(raw))["condition"] == condition and block["zoom"] == zoom and block["speed"] == speed and block.get("population_flagged", False) is False
            for sample in perf.sequence(block["samples"])]


def stats(values: list[perf.Json]) -> Record:
    return {metric: dict(perf.summary(values, metric)) for metric in perf.METRICS}


def ratio(before: perf.Json, after: perf.Json) -> float | None:
    if before is None or after is None or perf.number(after) <= 0:
        return None
    return perf.number(before) / perf.number(after)


def verdict(before: list[perf.Json], after: list[perf.Json], median_limit: float, p99_limit: float | None = None) -> Record:
    a, b = perf.summary(before, "primary_cpu_ms"), perf.summary(after, "primary_cpu_ms")
    status = "NOT_VERIFIED"
    if before and after:
        status = "PASS" if perf.number(b["median"]) <= median_limit and (p99_limit is None or perf.number(b["p99"]) <= p99_limit) else "FAIL"
    return {"status": status, "before": dict(a), "after": dict(b), "median_speedup": ratio(a["median"], b["median"]), "median_limit_ms": median_limit if math.isfinite(median_limit) else None, "p99_limit_ms": p99_limit}


def identity_verdict(payload: Record) -> Record:
    rows = [perf.mapping(row) for row in perf.sequence(payload.get("framings", []))]
    if len(rows) != 6 or {str(row.get("framing")) for row in rows} != {f"P{i}" for i in range(1, 7)}:
        return {"status": "NOT_VERIFIED", "reason": "Requires all six identity framings"}
    passed = True
    for row in rows:
        if any(key not in row for key in ("control", "candidate", "capture_frame", "state_change_frame", "rgba_sha256")):
            return {"status": "NOT_VERIFIED", "reason": "Identity provenance incomplete"}
        control, candidate = perf.mapping(row["control"]), perf.mapping(row["candidate"])
        if perf.number(row.get("capture_frame")) - perf.number(row.get("state_change_frame")) != 120 or len(perf.sequence(row.get("rgba_sha256"))) != 3:
            return {"status": "NOT_VERIFIED", "reason": "Identity provenance incomplete"}
        for key in ("pixels", "max_channel"):
            a, b = perf.number(control[key]), perf.number(candidate[key])
            if a < 0 or b < 0:
                raise perf.EvidenceError("Negative identity difference")
            passed &= a == 0 and b == 0
    return {"status": "PASS" if passed else "FAIL", "framings": list(rows)}


def ablation_rows(session: perf.Json) -> list[perf.Json]:
    data = perf.mapping(session)
    blocks = {str(perf.mapping(block)["label"]): perf.mapping(block) for block in perf.sequence(data.get("blocks", []))}
    output: list[perf.Json] = []
    for raw in perf.sequence(data.get("ablation", [])):
        row = dict(perf.mapping(raw))
        neighbours = [sample for name in perf.sequence(row["neighbour_blocks"]) for sample in perf.sequence(blocks[str(name)]["samples"])]
        off = perf.sequence(blocks[str(row["off_block"])]["samples"])
        row["cost_ms"] = perf.number(perf.summary(neighbours, "primary_cpu_ms")["median"]) - perf.number(perf.summary(off, "primary_cpu_ms")["median"])
        output.append(row)
    return output


@dataclass(frozen=True, slots=True)
class Evidence:
    identity: Record
    running: Record
    activities: Record
    inspector: Record


def source_exists(value: perf.Json) -> bool:
    return isinstance(value, str) and bool(value) and Path(value).is_file()


def combined(rows: list[Record]) -> str:
    statuses = [row["status"] for row in rows]
    return "FAIL" if "FAIL" in statuses else "PASS" if statuses and all(status == "PASS" for status in statuses) else "NOT_VERIFIED"


def running_verdict(payload: Record) -> Record:
    rows = [perf.mapping(row) for row in perf.sequence(payload.get("worlds", []))]
    if len(rows) != len(running_identity.WORLDS) or {row.get("world") for row in rows} != {spec[0] for spec in running_identity.WORLDS}:
        return {"status": "NOT_VERIFIED", "reason": "Requires W0, W200 and W-multi running dumps plus both W500 pans"}
    checked: list[Record] = []
    for row in rows:
        world = str(row["world"])
        spec = next(spec for spec in running_identity.WORLDS if spec[0] == world)
        if not all(source_exists(row.get(key)) for key in ("base_dump", "candidate_dump")):
            return {"status": "NOT_VERIFIED", "reason": "Running dump absent"}
        paths = [Path(str(row[key])) for key in ("base_dump", "candidate_dump")]
        if any(path.name != f"{world}.jsonl" or not path.with_suffix(".json").is_file() for path in paths):
            return {"status": "NOT_VERIFIED", "reason": "Running metadata absent or nonstandard dump name"}
        result = running_identity.compare_world(paths[0].parent, paths[1].parent, spec)
        checked.append(dict(perf.mapping(json.loads(json.dumps(result)))))
    return {"status": combined(checked), "worlds": checked}


def evidence_rows(reference: perf.Json, key: str) -> list[Record]:
    source = perf.mapping(reference)
    payload = read(str(source.get("source", "-")))
    if payload.get("valid") is not True:
        return []
    return [dict(perf.mapping(row)) for row in perf.sequence(payload.get(key, []))]


def rows_status(rows: list[Record], complete: bool) -> str:
    if any(row.get("passed") is False for row in rows):
        return "FAIL"
    return "PASS" if complete and rows and all(row.get("passed") is True for row in rows) else "NOT_VERIFIED"


def activity_verdict(payload: Record) -> Record:
    resolve = evidence_rows(payload.get("unit_tests", {}), "resolve_cases")
    poses = evidence_rows(payload.get("unit_tests", {}), "pose_cases")
    resolve_coverage = {(row.get("state"), row.get("kind")) for row in resolve}
    pose_coverage = {(row.get("kind"), row.get("body"), row.get("view")) for row in poses}
    units = rows_status(resolve + poses, {(4, kind) for kind in range(7)} <= resolve_coverage and {(kind, body, view) for kind in range(7) for body in ("adult_a", "adult_b", "child") for view in ("E", "N", "S", "W")} <= pose_coverage)
    captures = evidence_rows(payload.get("sidecar_check", {}), "captures")
    coverage = {(row.get("id"), row.get("zoom")) for row in captures}
    holders = [perf.mapping(holder) for capture in captures for holder in perf.sequence(capture.get("holders", []))]
    mismatches = sum(row.get("expected_pose") != row.get("drawn_pose") for row in holders)
    sidecar = "FAIL" if mismatches else "NOT_VERIFIED"
    if holders and all(row.get("expected_pose") and row.get("drawn_pose") and row.get("kind") in range(7) for row in holders) and {(case, zoom) for case in ("day30_1000", "day30_1500", "day60_1000") for zoom in (1, 4)} <= coverage:
        sidecar = "FAIL" if mismatches else "PASS"
    blind = read(str(perf.mapping(payload.get("blind", {})).get("source", "-")))
    answers = [perf.mapping(row) for row in perf.sequence(blind.get("answers", []))]
    complete = blind.get("valid") is True and blind.get("answers_frozen_before_key") is True and len(answers) == 14 and all(sum(row.get("expected_kind") == kind for row in answers) == 2 for kind in range(7)) and all(row.get("predicted_kind") in range(7) for row in answers)
    correct = sum(row.get("expected_kind") == row.get("predicted_kind") for row in answers)
    per_kind = {str(kind): sum(row.get("expected_kind") == kind and row.get("predicted_kind") == kind for row in answers) for kind in range(7)}
    blind_status = "NOT_VERIFIED" if not complete else "PASS" if correct >= 12 and all(value >= 1 for value in per_kind.values()) else "FAIL"
    village = read(str(perf.mapping(payload.get("village_summary", {})).get("source", "-")))
    village_answers = [perf.mapping(row) for row in perf.sequence(village.get("answers", []))]
    village_complete = village.get("valid") is True and village.get("after_blind_answers_before_key") is True and len(village_answers) == 6
    village_correct = sum(row.get("expected_kind") == row.get("predicted_kind") for row in village_answers)
    village_status = "NOT_RUN" if payload.get("village_summary_dropped") is True else "NOT_VERIFIED" if not village_complete else "PASS" if village_correct >= 5 else "FAIL"
    return {"status": combined([{"status": units}, {"status": sidecar}, {"status": blind_status}, {"status": village_status}]), "blind_status": blind_status, "village_summary_status": village_status, "village_summary_correct": village_correct, "unit_status": units, "sidecar_status": sidecar, "checked_holders": len(holders), "mismatches": mismatches, "blind_correct_share": correct / 14 if complete else None, "blind_per_kind": {str(kind): sum(row.get("expected_kind") == kind and row.get("predicted_kind") == kind for row in answers) / 2 for kind in range(7)} if complete else {}, "blind_pass_bar": {"correct": 12, "minimum_per_kind": 1}, "sources": payload}


def inspector_verdict(payload: Record) -> Record:
    rows = evidence_rows(payload.get("tests", {}), "tests")
    cases = {str(row.get("name")) for row in rows}
    complete = {"all_kinds_roles_causes_en_ko", "record_mode", "founder_record", "links", "read_discipline_120_frames"} <= cases
    review = read(str(perf.mapping(payload.get("review", {})).get("source", "-")))
    complete = complete and review.get("every_non_inference_statement_correct") is True and review.get("no_raw_keys_or_percent") is True
    return {"status": rows_status(rows, complete), "cases": sorted(cases), "source": payload}


def running_context(values: list[perf.Json]) -> Record:
    if not values:
        return {"status": "NOT_MEASURED"}
    ticks = [perf.number(perf.mapping(row)["ticks_run"]) for row in values]
    return {"population_start": perf.mapping(perf.mapping(values[0]).get("state", {})).get("population"), "population_end": perf.mapping(perf.mapping(values[-1]).get("state", {})).get("population"), "ticks_per_frame": sum(ticks) / len(ticks), "tick_carrying_frames": sum(tick > 0 for tick in ticks), "frames": len(ticks)}


def compare(before: Record, after: Record, evidence: Evidence) -> Record:
    conditions: Record = {}
    ablation: Record = {}
    for world in ("W0", "W1", "W200", "W500", "W-multi"):
        for condition, zoom, speed in (("frozen", 4, 0), ("running", 4, 2)):
            a, b = samples(before, world, condition, zoom, speed), samples(after, world, condition, zoom, speed)
            conditions[f"{world}/{condition}/Z{zoom}/{speed}x"] = {"before_status": "MEASURED" if a else "NOT_MEASURED", "after_status": "MEASURED" if b else "NOT_MEASURED", "before": stats(a), "after": stats(b), "median_speedup": ratio(perf.summary(a, "primary_cpu_ms")["median"], perf.summary(b, "primary_cpu_ms")["median"])}
            if condition == "running":
                at = [row for row in a if perf.number(perf.mapping(row)["ticks_run"]) > 0]
                bt = [row for row in b if perf.number(perf.mapping(row)["ticks_run"]) > 0]
                conditions[f"{world}/{condition}/Z{zoom}/{speed}x/tick-only"] = {"before": stats(at), "after": stats(bt), "before_context": running_context(a), "after_context": running_context(b)}
        ablation[world] = {"before": ablation_rows(before.get(world, {})), "after": ablation_rows(after.get(world, {}))}
    running_blocks = {side: [perf.mapping(raw) for raw in perf.sequence(perf.mapping(source.get("W500", {})).get("blocks", [])) if perf.mapping(raw).get("condition") == "running" and perf.mapping(raw).get("speed") == 2] for side, source in (("before", before), ("after", after))}
    unflagged = {side: [block for block in blocks if block.get("population_flagged") is False] for side, blocks in running_blocks.items()}
    a2 = [sample for block in unflagged["before"] for sample in perf.sequence(block["samples"])]
    b2 = [sample for block in unflagged["after"] for sample in perf.sequence(block["samples"])]
    identity, running = identity_verdict(evidence.identity), running_verdict(evidence.running)
    h: Record = {}
    for key, world, condition, limit, p99 in (("H1", "W0", "frozen", 6.0, None), ("H2", "W1", "running", math.inf, 16.67), ("H3", "W200", "running", 16.67, None), ("H4", "W200", "frozen", 10.0, None), ("H5", "W500", "frozen", 16.67, None)):
        speed = 2 if condition == "running" else 0
        h[key] = verdict(samples(before, world, condition, 4, speed), samples(after, world, condition, 4, speed), limit, p99)
    h["H6"] = {"status": "REPORTED", "worlds": {world: conditions[f"{world}/running/Z4/2x"] for world in ("W500", "W-multi")}, "population_blocks": {side: {world: [{key: block.get(key) for key in ("label", "population_start", "population_end", "population_flagged")} for raw in perf.sequence(perf.mapping(source.get(world, {})).get("blocks", [])) if (block := perf.mapping(raw)).get("condition") == "running"] for world in ("W500", "W-multi")} for side, source in (("before", before), ("after", after))}}
    h["H7"] = {"status": "NOT_VERIFIED", "pixel_identity": identity, "running_equality": running, "reason": "Requires exact identity evidence for every accepted candidate including 240-frame pans and W-multi for memo"}
    h["H8"] = {"status": "NOT_VERIFIED", "reason": "Requires stage A 7/7, stage B >=12/14 with every kind >=1/2, and recorded icon size/contrast metrics"}
    h["H9"] = {"status": "NOT_VERIFIED", "reason": "Requires glyph pair differences >=12px, correct non-name facts, and real-input evidence"}
    h["H10"] = {"status": "NOT_VERIFIED", "reason": "Requires game_v26 porter glyph/pose, multiple relation colours and kind-21 sentence evidence"}
    return {"verdicts": h, "conditions": conditions, "ablation": ablation, "build_profiles": {"before": {world: perf.mapping(row).get("build_profile") for world, row in before.items()}, "after": {world: perf.mapping(row).get("build_profile") for world, row in after.items()}}}


def markdown(result: Record) -> str:
    lines = ["# Art Pass 13 comparison", "", "Speed-up = before / after. Missing evidence is NOT_VERIFIED; H6 has no performance bar; blind identification requires 12/14 and every kind at least 1/2. Frozen worlds never invalidate tick caches; running 2x typically invalidates about every frame. Stress worlds use game_v19 and measure no activity cost.", "", "| Rule | Status | Evidence |", "|---|---|---|"]
    for key, value in perf.mapping(result["verdicts"]).items():
        row = perf.mapping(value)
        details = {field: item for field, item in row.items() if field != "getters"}
        lines.append(f"| {key} | {row['status']} | {json.dumps(details, ensure_ascii=False)} |")
    lines.extend(["", "| World / condition | Metric | Before median / p90 / p99 / max / >16.67 | After median / p90 / p99 / max / >16.67 | Median speed-up |", "|---|---|---|---|---|"])
    for condition, value in perf.mapping(result["conditions"]).items():
        row = perf.mapping(value)
        for metric in perf.METRICS:
            cells = []
            for side in ("before", "after"):
                numbers = perf.mapping(perf.mapping(row[side])[metric])
                cells.append(" / ".join(str(numbers[key]) for key in ("median", "p90", "p99", "maximum", "over_budget")) if numbers["count"] else "NOT_MEASURED")
            speedup = ratio(perf.mapping(perf.mapping(row["before"])[metric])["median"], perf.mapping(perf.mapping(row["after"])[metric])["median"])
            lines.append(f"| {condition} | {metric} | {cells[0]} | {cells[1]} | {speedup} |")
    lines.extend(["", "## Ablation before / after", "", "| World | Node/group | Before ms | After ms |", "|---|---|---:|---:|"])
    for world, value in perf.mapping(result["ablation"]).items():
        row = perf.mapping(value)
        a = {str(perf.mapping(item)["node"]): perf.mapping(item)["cost_ms"] for item in perf.sequence(row["before"])}
        b = {str(perf.mapping(item)["node"]): perf.mapping(item)["cost_ms"] for item in perf.sequence(row["after"])}
        for node in sorted(a.keys() | b.keys()):
            lines.append(f"| {world} | {node} | {a.get(node, 'NOT_MEASURED')} | {b.get(node, 'NOT_MEASURED')} |")
    lines.extend(["", "## Build profiles", "", json.dumps(result["build_profiles"]), ""])
    return "\n".join(lines)


def main() -> None:
    if len(sys.argv) < 5 or (len(sys.argv) - 5) % 2:
        raise perf.EvidenceError("Usage: compare_perf_p13.py BASE AFTER IDENTITY OUTPUT_PREFIX [--running-check FILE] [--activities FILE] [--inspector FILE]")
    options = {"--running-check": "-", "--activities": "-", "--inspector": "-"}
    for index in range(5, len(sys.argv), 2):
        if sys.argv[index] not in options:
            raise perf.EvidenceError(f"Unknown option: {sys.argv[index]}")
        options[sys.argv[index]] = sys.argv[index + 1]
    result = compare(aggregates(sys.argv[1], "baseline"), aggregates(sys.argv[2], "after"), Evidence(read(sys.argv[3]), read(options["--running-check"]), read(options["--activities"]), read(options["--inspector"])))
    prefix = Path(sys.argv[4])
    prefix.parent.mkdir(parents=True, exist_ok=True)
    prefix.with_suffix(".json").write_text(json.dumps(result, indent=2) + "\n")
    prefix.with_suffix(".md").write_text(markdown(result))


if __name__ == "__main__":
    main()
