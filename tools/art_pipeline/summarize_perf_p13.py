#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: uv run tools/art_pipeline/summarize_perf_p13.py OUTPUT_PREFIX SESSION.json ...
# Example: ... docs/art/production13/perf/baseline evidence/baseline-W*.json
"""Preserve every raw session sample and independently recompute performance tables.

Stdlib only: repository policy forbids adding dependencies. Session order is preserved;
duplicate worlds are rejected rather than silently replacing a failed attempt.
"""
from __future__ import annotations

from session_validity_p13 import require_valid

import json
import hashlib
import math
import statistics
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Final, TypeAlias, TypedDict

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
METRICS: Final = (
    "primary_cpu_ms", "process_ms", "render_cpu_ms", "setup_cpu_ms",
    "gpu_ms", "pacing_ms", "ticks_run",
)
REQUIRED: Final = frozenset({"W0", "W1", "W200", "W500", "W-multi"})
OPTIONAL: Final = frozenset({"W1000"})


class EvidenceError(ValueError):
    """Malformed or contradictory measurement evidence."""


class Summary(TypedDict):
    count: int
    median: float | None
    p90: float | None
    p99: float | None
    maximum: float | None
    over_budget: int


@dataclass(frozen=True, slots=True)
class Session:
    source: Path
    world: str
    phase: str
    payload: Mapping[str, Json]


def mapping(value: Json) -> Mapping[str, Json]:
    """Narrow a JSON object at the evidence boundary."""
    if not isinstance(value, dict):
        raise EvidenceError(f"Expected JSON object, received {type(value).__name__}")
    return value


def sequence(value: Json) -> Sequence[Json]:
    """Narrow a JSON array at the evidence boundary."""
    if not isinstance(value, list):
        raise EvidenceError(f"Expected JSON array, received {type(value).__name__}")
    return value


def number(value: Json) -> float:
    """Reject booleans, absent values and invalid numerical evidence."""
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise EvidenceError(f"Expected finite measurement, received {value!r}")
    return float(value)


def summary(samples: Sequence[Json], metric: str) -> Summary:
    """Recompute nearest-rank percentiles rather than trusting collector summaries."""
    values = sorted(number(mapping(sample)[metric]) for sample in samples)
    if not values:
        return {"count": 0, "median": None, "p90": None, "p99": None, "maximum": None, "over_budget": 0}
    return {
        "count": len(values), "median": statistics.median(values),
        "p90": values[math.ceil(len(values) * 0.9) - 1],
        "p99": values[math.ceil(len(values) * 0.99) - 1], "maximum": values[-1],
        "over_budget": sum(number(mapping(sample)["primary_cpu_ms"]) > 16.67 for sample in samples),
    }


def display(value: float | None) -> str:
    return "N/A" if value is None else f"{value:.4f}"


def table_row(world: str, condition: str, metric: str, samples: Sequence[Json]) -> str:
    result = summary(samples, metric)
    fields = [world, condition, metric, str(result["count"])]
    fields.extend(display(value) for value in (result["median"], result["p90"], result["p99"], result["maximum"]))
    fields.append(str(result["over_budget"]))
    return "| " + " | ".join(fields) + " |"


def load_session(path: Path) -> Session:
    raw: Json = json.loads(path.read_text())
    payload = mapping(raw)
    fragments = sequence(payload.get("split_fragments", []))
    for item in fragments:
        require_valid(Path(str(mapping(item)["source"]) + ".validity.json"))
    if not fragments:
        require_valid(Path(str(path) + ".validity.json"))
    world, phase = str(payload["world"]), str(payload["phase"])
    if world not in REQUIRED | OPTIONAL or phase not in {"baseline", "after"}:
        raise EvidenceError(f"Invalid world/phase in {path}")
    if payload.get("ablation_only"):
        raise EvidenceError(f"Part 3 ablation-only evidence cannot enter the full matrix: {path}")
    for key, length in (("build_commit", 40), ("build_sha256", 64), ("collector_sha256", 64)):
        value = payload.get(key)
        if not isinstance(value, str) or len(value) != length or any(character not in "0123456789abcdef" for character in value):
            raise EvidenceError(f"Missing or invalid measurement provenance: {key}")
    if not isinstance(payload.get("checkout"), str) or not Path(str(payload["checkout"])).is_absolute():
        raise EvidenceError("Missing absolute checkout provenance")
    if phase == "baseline" and payload["build_commit"] != payload.get("baseline_commit"):
        raise EvidenceError("Baseline must use the recorded Part 0 tooling commit")
    if world == "W1000" and phase != "after":
        raise EvidenceError("W1000 is after-only")
    if payload.get("build_profile") != "release" or payload.get("release_sha256") != payload["build_sha256"]:
        raise EvidenceError("Release profile and identical dylib SHA required")
    for raw_fragment in sequence(payload.get("split_fragments", [])):
        fragment = mapping(raw_fragment)
        source = Path(str(fragment["source"]))
        if hashlib.sha256(source.read_bytes()).hexdigest() != fragment["sha256"]:
            raise EvidenceError("Split fragment hash mismatch")
    blocks = [mapping(block) for block in sequence(payload["blocks"])]
    matrix = [block for block in blocks if str(block["condition"]) in {"frozen", "running"}]
    expected = [f"frozen_Z{zoom}_{repeat}" for zoom in (4,) for repeat in range(3)]
    expected.extend(f"running_{speed}x_{repeat}" for speed in (2,) for repeat in range(3))
    if world == "W1000":
        expected = [f"frozen_Z4_{repeat}" for repeat in range(3)]
        if len(blocks) != 3:
            raise EvidenceError("W1000 requires only three frozen Z4 blocks")
    if [str(block["label"]) for block in matrix] != expected:
        raise EvidenceError(f"Missing, duplicate or reordered matrix block in {path}")
    for block in blocks:
        count = 300 if str(block["condition"]) in {"frozen", "running"} else 150
        if len(sequence(block["samples"])) != count or len(sequence(block["discarded"])) != 30:
            raise EvidenceError(f"Wrong sample/warmup count in {path}: {block['label']}")
    session = Session(path, world, phase, payload)
    validate_measurements(session, blocks)
    return session


def validate_measurements(session: Session, blocks: Sequence[Mapping[str, Json]]) -> None:
    payload = session.payload
    restoration = mapping(payload.get("restoration", {}))
    for key in ("vsync_restored", "layers_restored", "camera_process_restored", "probes_removed"):
        if restoration.get(key) is not True:
            raise EvidenceError(f"Unproven runtime restoration: {key}")
    if payload.get("valid") is True and sequence(payload.get("errors", [])):
        raise EvidenceError("Session claims valid despite recorded errors")
    initial = mapping(payload["initial"])
    previous_tick = number(initial["tick"])
    previous_frame = -1.0
    running_started = False
    frozen_seen = 0
    for block in blocks:
        label, condition = str(block["label"]), str(block["condition"])
        if condition not in {"running", "frozen", "ablation_reference", "ablation_off", "ablation_floor"}:
            raise EvidenceError(f"Unknown block condition: {condition}")
        frozen_seen += condition == "frozen"
        if condition.startswith("ablation") and frozen_seen != 3:
            raise EvidenceError("Ablation must follow all three frozen blocks")
        running = condition == "running"
        if running_started and not running:
            raise EvidenceError("Frozen/ablation block after a running block")
        if running:
            start = mapping(block.get("start_state"))
            if block.get("fresh_setup") is not True or start.get("tick") != initial["tick"]:
                raise EvidenceError("Running block did not reset to the requested tick")
            previous_tick = number(start["tick"])
            if payload.get("split_fragments"):
                previous_frame = -1.0
            population_start = number(start["population"])
            population_end = number(mapping(block.get("end_state"))["population"])
            if block.get("population_start") != population_start or block.get("population_end") != population_end or block.get("population_flagged") != (population_end < 0.9 * population_start):
                raise EvidenceError("Running population metadata/flag mismatch")
        running_started |= running
        expected_speed = float(label.split("_")[1].removesuffix("x")) if running else 0.0
        expected_zoom = int(label.split("_")[1].removeprefix("Z")) if condition == "frozen" else 4
        if running and expected_speed not in {2.0, 4.0}:
            raise EvidenceError(f"Invalid running speed in {label}")
        if (number(block["speed"]), number(block["zoom"]), number(block["max_fps"])) != (expected_speed, expected_zoom, 60 if running else 0):
            raise EvidenceError(f"Control mismatch in {label}")
        if block.get("vsync_disabled") is not True or block.get("camera_process_frozen") is not True or block.get("window_size") != [1920, 1080]:
            raise EvidenceError(f"Vsync/camera/window controls unproven in {label}")
        samples = sequence(block["samples"])
        if running and not any(number(mapping(sample)["ticks_run"]) > 0 for sample in samples):
            raise EvidenceError(f"No tick-carrying frames in {label}")
        for raw in [*sequence(block["discarded"]), *samples]:
            sample = mapping(raw)
            if any(number(sample[metric]) < 0 for metric in METRICS):
                raise EvidenceError(f"Negative measurement in {label}")
            if not math.isclose(number(sample["primary_cpu_ms"]), number(sample["process_ms"]) + number(sample["render_cpu_ms"]), rel_tol=0, abs_tol=1e-9):
                raise EvidenceError(f"Primary differs from process + render in {label}")
            frame = number(sample["process_frame"])
            if sample.get("paired") is not True or frame <= previous_frame:
                raise EvidenceError(f"Unpaired/non-increasing process frame in {label}")
            previous_frame = frame
            if "ffi_calls" in sample and sample.get("ffi_counter_frame") != frame:
                raise EvidenceError(f"FFI counters do not match process frame in {label}")
            state = mapping(sample["state"])
            tick, ticks_run = number(state["tick"]), number(sample["ticks_run"])
            if ticks_run != tick - previous_tick or (not running and ticks_run != 0):
                raise EvidenceError(f"Tick delta/control mismatch in {label}")
            previous_tick = tick
            if state["camera"] != initial["camera"] or state["zoom"] != [expected_zoom, expected_zoom]:
                raise EvidenceError(f"Camera/zoom changed in {label}")
            if session.phase == "after" and session.world == "W500":
                counts = mapping(sample.get("ffi_calls"))
                if any(number(count) < 0 or not number(count).is_integer() for count in counts.values()):
                    raise EvidenceError(f"Invalid FFI counters in {label}")
                if condition == "frozen" and expected_zoom == 4:
                    context = mapping(sample.get("direct_read_context"))
                    if context.get("inspector_visible") is not False or context.get("influence_channel") != -1:
                        raise EvidenceError(f"F4 requires closed inspector and influence overlay in {label}")
    required_ablation = not payload.get("ablation_dropped", True)
    if required_ablation:
        if payload.get("ablation_dropped") is True:
            raise EvidenceError("W0/W500 ablation cannot be dropped")
        rows = [mapping(row) for row in sequence(payload["ablation"])]
        labels = {str(block["label"]) for block in blocks}
        layers = {str(row["node"]).split("/")[-1] for row in rows if row.get("layer_only") is True}
        if not {"WorldObjects", "Shadows", "NightChillLayer"}.issubset(layers) or not any(row.get("layer_only") is False for row in rows):
            raise EvidenceError("Missing mandatory node/shared-layer ablation rows")
        if not {"ablation_floor", "ablation_restored"}.issubset(labels) or payload.get("ablation_visibility_restored") is not True:
            raise EvidenceError("Missing ablation floor or exact restoration")
        for row in rows:
            references = [str(row["off_block"]), *map(str, sequence(row["neighbour_blocks"]))]
            if len(references) != 3 or not set(references).issubset(labels):
                raise EvidenceError(f"Missing ablation block references: {row['node']}")


def report(sessions: Sequence[Session], missing: set[str]) -> list[str]:
    """Render all blocks, pooled conditions, tick-only subsets and ablation rows."""
    lines = ["# Art Pass 13 frame measurements", "", "Profile: release (identical release/debug dylib SHA). Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.", "", f"Missing worlds: {', '.join(sorted(missing)) or 'none'}.", ""]
    for session in sessions:
        payload = session.payload
        initial = mapping(payload.get("initial", {}))
        lines.extend([f"## {session.world}", "", f"Source: `{session.source}`; valid: `{payload.get('valid')}`; measured tick: {initial.get('tick')}; alive population: {initial.get('population')}; contended: {payload.get('contended')}.", "", f"Machine: `{json.dumps(payload.get('machine'), ensure_ascii=False)}`", "", f"Godot: `{json.dumps(payload.get('godot'), ensure_ascii=False)}`; load before: `{payload.get('load_before')}`; load after: `{payload.get('load_after')}`.", "", "| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |", "|---|---|---|---:|---:|---:|---:|---:|---:|"])
        pooled: dict[str, list[Json]] = {}
        for raw_block in sequence(payload["blocks"]):
            block = mapping(raw_block)
            samples = sequence(block["samples"])
            for metric in METRICS:
                lines.append(table_row(session.world, str(block["label"]), metric, samples))
            condition = str(block["condition"])
            if condition == "running":
                lines.append(f"\nPopulation {block['label']}: {block['population_start']} → {block['population_end']}; flagged={block['population_flagged']}.\n")
                ticks = [sample for sample in samples if number(mapping(sample)["ticks_run"]) > 0]
                for metric in METRICS:
                    lines.append(table_row(session.world, str(block["label"]) + " tick-only", metric, ticks))
            if condition in {"running", "frozen"}:
                key = f"{condition}_Z{block['zoom']}_{block['speed']}x"
                pooled.setdefault(key, [])
                if condition == "running" and block["population_flagged"] is True:
                    lines.append(f"\nEXCLUDED from pooled and tick-only results: {block['label']} (population loss); raw rows retained above.\n")
                else:
                    pooled[key].extend(samples)
        for key, samples in pooled.items():
            for metric in METRICS:
                lines.append(table_row(session.world, "POOLED " + key, metric, samples))
            if key.startswith("running"):
                ticks = [sample for sample in samples if number(mapping(sample)["ticks_run"]) > 0]
                for metric in METRICS:
                    lines.append(table_row(session.world, "POOLED " + key + " tick-only", metric, ticks))
        lines.extend(["", "| Ablation node/group | Cost ms | Layer only | Group descendants |", "|---|---:|---|---|"])
        for raw_row in sequence(payload["ablation"]):
            row = mapping(raw_row)
            lines.append(f"| {row['node']} | {number(row['cost_ms']):.4f} | {row['layer_only']} | {row['descendants_disabled_as_group']} |")
        lines.extend(["", f"Errors: `{json.dumps(payload.get('errors'), ensure_ascii=False)}`", ""])
    return lines


def main() -> int:
    if len(sys.argv) < 3:
        raise EvidenceError("Usage: summarize_perf_p13.py OUTPUT_PREFIX SESSION.json ...")
    prefix = Path(sys.argv[1])
    sessions = [load_session(Path(name)) for name in sys.argv[2:]]
    worlds = {session.world for session in sessions}
    if len(worlds) != len(sessions):
        raise EvidenceError("Duplicate world: retain rejected attempts separately; select one explicitly")
    if len({session.phase for session in sessions}) != 1:
        raise EvidenceError("Cannot mix baseline and after sessions")
    missing = (REQUIRED | OPTIONAL) - worlds
    valid = not (REQUIRED - worlds) and all(session.payload.get("valid") is True for session in sessions)
    aggregate = {"schema_version": 1, "phase": sessions[0].phase, "valid": valid, "missing_worlds": sorted(missing), "optional_worlds_not_measured": sorted(OPTIONAL - worlds), "sources": [str(session.source) for session in sessions], "sessions": [session.payload for session in sessions]}
    prefix.parent.mkdir(parents=True, exist_ok=True)
    prefix.with_suffix(".json").write_text(json.dumps(aggregate, ensure_ascii=False, indent=2) + "\n")
    prefix.with_suffix(".md").write_text("\n".join(report(sessions, missing)))
    return 0 if valid else 1


if __name__ == "__main__":
    sys.exit(main())
