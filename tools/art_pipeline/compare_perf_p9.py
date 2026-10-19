#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/art_pipeline/compare_perf_p9.py BASE AFTER IDENTITY OUTPUT_PREFIX
# Optional: --inventory FILE --a30-before FILE --a30-after FILE; '-' means missing input.
"""Compare validated Art Pass 9 evidence without converting absence into success."""
from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path
from typing import TypeAlias

import summarize_perf_p9 as perf

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
    with tempfile.TemporaryDirectory(prefix="p9-compare-validation-") as temporary:
        for raw in perf.sequence(payload.get("sessions", [])):
            scratch = Path(temporary) / "session.json"
            scratch.write_text(json.dumps(raw))
            session = perf.load_session(scratch)
            if session.phase != phase or session.world in sessions:
                raise perf.EvidenceError("Mixed phases or duplicate worlds in aggregate")
            if session.payload.get("build_profile") != "debug":
                raise perf.EvidenceError("Frame comparison requires the debug build profile")
            if session.payload.get("valid") is not True or session.payload.get("errors") != []:
                continue
            sessions[session.world] = dict(session.payload)
    return sessions


def samples(sessions: Record, world: str, condition: str, zoom: int, speed: int) -> list[perf.Json]:
    session = perf.mapping(sessions.get(world, {}))
    return [sample for raw in perf.sequence(session.get("blocks", []))
            if (block := perf.mapping(raw))["condition"] == condition and block["zoom"] == zoom and block["speed"] == speed
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
    return {"status": status, "before": dict(a), "after": dict(b), "median_speedup": ratio(a["median"], b["median"]), "median_limit_ms": median_limit, "p99_limit_ms": p99_limit}


def ffi_verdict(values: list[perf.Json], inventory: Record) -> Record:
    result: Record = {"status": "NOT_VERIFIED", "baseline_counts": "NOT_MEASURED", "inventory": inventory, "getters": {}}
    if not values:
        return result
    for value in values:
        context = perf.mapping(perf.mapping(value).get("direct_read_context", {}))
        if context.get("inspector_visible") is not False or context.get("influence_channel") != -1:
            result["reason"] = "Measured inspector/overlay context is absent or not closed"
            return result
    counts = [perf.mapping(perf.mapping(value)["ffi_calls"]) for value in values]
    getters = sorted({key for count in counts for key in count})
    by_getter: Record = {}
    for getter in getters:
        rows: list[perf.Json] = [{"primary_cpu_ms": perf.number(count.get(getter, 0))} for count in counts]
        times: list[perf.Json] = [{"primary_cpu_ms": perf.mapping(perf.mapping(value).get("ffi_call_usec", {})).get(getter, 0)} for value in values if "ffi_call_usec" in perf.mapping(value)]
        time_complete = len(times) == len(values) and all(perf.number(count.get(getter, 0)) == 0 or getter in perf.mapping(perf.mapping(value).get("ffi_call_usec", {})) for count, value in zip(counts, values, strict=True))
        by_getter[getter] = {"calls": {key: value for key, value in perf.summary(rows, "primary_cpu_ms").items() if key != "over_budget"}, "microseconds": {key: value for key, value in perf.summary(times, "primary_cpu_ms").items() if key != "over_budget"} if time_complete else None}
    result["getters"] = by_getter
    if inventory.get("confirmed") is not True or inventory.get("no_scaling") is not True or not inventory.get("source"):
        return result
    direct = perf.mapping(inventory.get("direct_calls", {}))
    if set(direct) != {"get_agent_detail", "get_influence_overlay", "get_tile_causal_events"}:
        raise perf.EvidenceError("Inventory must explicitly confirm all three direct residual getters")
    residual = {key: perf.number(value) for key, value in direct.items()}
    if residual["get_agent_detail"] != 0 or residual["get_influence_overlay"] != 0:
        raise perf.EvidenceError("F4 framing requires inspector and influence overlay closed")
    causal = residual["get_tile_causal_events"]
    source = inventory.get("causal_source")
    counted_causal = [perf.number(count.get("get_tile_causal_events", 0)) for count in counts]
    if source not in {"sim_reads", "direct", "absent"} or causal not in {0, 1}:
        raise perf.EvidenceError("Invalid causal inventory")
    if (source == "sim_reads" and (causal != 0 or max(counted_causal) > 1)) or (source == "direct" and max(counted_causal) != 0) or (source == "absent" and (causal != 0 or max(counted_causal) != 0)):
        raise perf.EvidenceError("Causal getter counted twice or inconsistent with inventory")
    totals: list[perf.Json] = [{"primary_cpu_ms": sum(perf.number(value) for value in count.values()) + causal} for count in counts]
    total = perf.summary(totals, "primary_cpu_ms")
    result.update({"status": "PASS" if perf.number(total["maximum"]) <= 20 else "FAIL", "total_calls": {key: value for key, value in total.items() if key != "over_budget"}, "limit_calls_per_frame": 20, "direct_residual_calls": causal})
    return result


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
            passed &= b <= a
    return {"status": "PASS" if passed else "FAIL", "framings": list(rows)}


def a30(path: str) -> Record:
    payload = read(path)
    if payload.get("valid") is not True or payload.get("errors") != []:
        return {}
    blocks = [perf.mapping(row) for row in perf.sequence(payload.get("blocks", []))]
    if len(blocks) != 12 or any(len(perf.sequence(block["samples"])) != 50 for block in blocks):
        raise perf.EvidenceError("A30 requires twelve 50-frame blocks")
    output: Record = {}
    for enabled, name in ((True, "on"), (False, "off")):
        selected = [sample for block in blocks if block["layers_enabled"] is enabled for sample in perf.sequence(block["samples"])]
        if len(selected) != 300:
            raise perf.EvidenceError("A30 requires 300 samples in each layer state")
        if any(perf.number(perf.mapping(sample)[metric]) < 0 for sample in selected for metric in ("process_ms", "render_cpu_ms")):
            raise perf.EvidenceError("Negative A30 measurement")
        values: list[perf.Json] = [{"primary_cpu_ms": perf.number(perf.mapping(sample)["process_ms"]) + perf.number(perf.mapping(sample)["render_cpu_ms"])} for sample in selected]
        output[name] = dict(perf.summary(values, "primary_cpu_ms"))
    return {"source": path, "layers": output}


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


def compare(before: Record, after: Record, identity: Record, inventory: Record, a30_before: Record, a30_after: Record) -> Record:
    conditions: Record = {}
    ablation: Record = {}
    for world in ("W0", "W1", "W200", "W500", "W1000", "W2000"):
        for condition, zoom, speed in (("frozen", 4, 0), ("frozen", 1, 0), ("running", 4, 2), ("running", 4, 4)):
            a, b = samples(before, world, condition, zoom, speed), samples(after, world, condition, zoom, speed)
            conditions[f"{world}/{condition}/Z{zoom}/{speed}x"] = {"before_status": "MEASURED" if a else "NOT_MEASURED", "after_status": "MEASURED" if b else "NOT_MEASURED", "before": stats(a), "after": stats(b), "median_speedup": ratio(perf.summary(a, "primary_cpu_ms")["median"], perf.summary(b, "primary_cpu_ms")["median"])}
            if condition == "running":
                at = [row for row in a if perf.number(perf.mapping(row)["ticks_run"]) > 0]
                bt = [row for row in b if perf.number(perf.mapping(row)["ticks_run"]) > 0]
                conditions[f"{world}/{condition}/Z{zoom}/{speed}x/tick-only"] = {"before_status": "MEASURED" if at else "NOT_MEASURED", "after_status": "MEASURED" if bt else "NOT_MEASURED", "before": stats(at), "after": stats(bt), "median_speedup": ratio(perf.summary(at, "primary_cpu_ms")["median"], perf.summary(bt, "primary_cpu_ms")["median"])}
        ablation[world] = {"before": ablation_rows(before.get(world, {})), "after": ablation_rows(after.get(world, {}))}
    a2, b2 = samples(before, "W500", "running", 4, 2), samples(after, "W500", "running", 4, 2)
    f: Record = {"F1": verdict(samples(before, "W0", "frozen", 4, 0), samples(after, "W0", "frozen", 4, 0), 8), "F2": verdict(samples(before, "W500", "frozen", 4, 0), samples(after, "W500", "frozen", 4, 0), 12, 16.67), "F3": verdict([row for row in a2 if perf.number(perf.mapping(row)["ticks_run"]) > 0], [row for row in b2 if perf.number(perf.mapping(row)["ticks_run"]) > 0], 16.67), "F4": ffi_verdict(samples(after, "W500", "frozen", 4, 0), inventory), "F5": identity_verdict(identity)}
    headroom: Record = {}
    for phase, sessions in (("before", before), ("after", after)):
        qualifying: list[int] = []
        populations: Record = {}
        for n in (200, 500, 1000, 2000):
            values = samples(sessions, f"W{n}", "frozen", 4, 0)
            metrics = perf.summary(values, "primary_cpu_ms")
            if values:
                populations[str(n)] = perf.mapping(perf.mapping(values[0])["state"]).get("population")
                if perf.number(metrics["median"]) <= 12 and perf.number(metrics["p99"]) <= 16.67:
                    qualifying.append(n)
        headroom[phase] = {"largest_requested_n_measured_meeting_rule": max(qualifying) if qualifying else None, "actual_alive_by_requested_n": populations, "unmeasured_n": [n for n in (200, 500, 1000, 2000) if str(n) not in populations]}
    f["F6"] = {"status": "REPORT_ONLY" if any(samples(after, f"W{n}", "frozen", 4, 0) for n in (200, 500, 1000, 2000)) else "NOT_VERIFIED", **headroom}
    f["F7"] = {"status": "REPORT_ONLY" if a30_before and a30_after else "NOT_VERIFIED", "before": a30_before, "after": a30_after, "speedup": {layer: ratio(perf.mapping(perf.mapping(a30_before["layers"])[layer])["median"], perf.mapping(perf.mapping(a30_after["layers"])[layer])["median"]) for layer in ("on", "off")} if a30_before and a30_after else {}}
    return {"verdicts": f, "conditions": conditions, "ablation": ablation, "build_profiles": {"before": {world: perf.mapping(row).get("build_profile") for world, row in before.items()}, "after": {world: perf.mapping(row).get("build_profile") for world, row in after.items()}}}


def markdown(result: Record) -> str:
    lines = ["# Art Pass 9 comparison", "", "Speed-up = before / after. Missing evidence is NOT_VERIFIED; F6/F7 have no performance verdict.", "", "| Rule | Status | Evidence |", "|---|---|---|"]
    for key, value in perf.mapping(result["verdicts"]).items():
        row = perf.mapping(value)
        details = {field: item for field, item in row.items() if field != "getters"}
        lines.append(f"| {key} | {row['status']} | {json.dumps(details, ensure_ascii=False)} |")
    lines.extend(["", "| After getter | Calls distribution | Microseconds distribution |", "|---|---|---|"])
    for getter, value in perf.mapping(perf.mapping(perf.mapping(result["verdicts"])["F4"]).get("getters", {})).items():
        row = perf.mapping(value)
        lines.append(f"| {getter} | {json.dumps(row['calls'])} | {json.dumps(row['microseconds']) if row['microseconds'] is not None else 'NOT_MEASURED'} |")
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
        raise perf.EvidenceError("Usage: compare_perf_p9.py BASE AFTER IDENTITY OUTPUT_PREFIX [--inventory FILE] [--a30-before FILE] [--a30-after FILE]")
    options = {"--inventory": "-", "--a30-before": "-", "--a30-after": "-"}
    for index in range(5, len(sys.argv), 2):
        if sys.argv[index] not in options:
            raise perf.EvidenceError(f"Unknown option: {sys.argv[index]}")
        options[sys.argv[index]] = sys.argv[index + 1]
    result = compare(aggregates(sys.argv[1], "baseline"), aggregates(sys.argv[2], "after"), read(sys.argv[3]), read(options["--inventory"]), a30(options["--a30-before"]), a30(options["--a30-after"]))
    prefix = Path(sys.argv[4])
    prefix.parent.mkdir(parents=True, exist_ok=True)
    prefix.with_suffix(".json").write_text(json.dumps(result, indent=2) + "\n")
    prefix.with_suffix(".md").write_text(markdown(result))


if __name__ == "__main__":
    main()
