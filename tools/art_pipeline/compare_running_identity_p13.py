#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/compare_running_identity_p13.py BASE CURRENT OUTPUT.json
"""Verify complete deterministic running captures before claiming byte identity."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Final, TypedDict

from summarize_perf_p13 import Json

WORLDS: Final = (("W0", 40560, 20261010), ("W200", 1440, 15100), ("W-multi", 90840, 19603), ("W500_FROZEN", 1440, 15100), ("W500_2X", 1440, 15100))
SCHEDULE_FIELDS: Final = (
    "world", "seed", "start_tick", "end_tick", "frames", "fixed_fps",
    "advance_every_frames", "mutation_route", "advance_calls", "process_frames",
    "ticks", "render_start_frame", "warmup_frames", "camera", "zoom", "debug_build",
    "populations", "scope", "pan", "camera_start", "hearth", "pan_step_world_px",
    "speed", "simulation_schedule", "collector_sha256",
)


class WorldResult(TypedDict):
    scope: str
    world: str
    start_tick: int
    frames: int
    fixed_fps: int
    advance_every_frames: int
    mutation_route: str
    base_dump: str
    candidate_dump: str
    valid: bool
    status: str
    errors: list[str]


class Report(TypedDict):
    candidate: str | None
    kernel_state: str | None
    scope: str
    valid: bool
    status: str
    worlds: list[WorldResult]


def identity_scope(candidate: str | None) -> str:
    """Keep reference reproduction distinct from validation of a named change."""
    if candidate not in (None, "declump-kernel", "write-skip"):
        raise ValueError(f"Unsupported candidate: {candidate}")
    return "candidate_validation" if candidate else "reference_integrity_only"


def kernel_stamp_errors(stamp: object, enabled: bool) -> list[str]:
    """Require renderer readback, strict booleans and the requested polarity."""
    if not isinstance(stamp, dict):
        return ["missing or malformed perf_candidate_p12 stamp"]
    errors: list[str] = []
    if stamp.get("requested_kernel") is not enabled:
        errors.append(f"requested_kernel must be {enabled}")
    if stamp.get("method_available") is not True:
        errors.append("candidate readback method unavailable")
    if stamp.get("valid") is not True:
        errors.append("collector candidate stamp is not valid")
    actual = stamp.get("actual")
    if not isinstance(actual, dict):
        errors.append("missing candidate actual readback")
    elif actual.get("change_id") != "declump-kernel" or actual.get("candidate_enabled") is not enabled:
        errors.append(f"actual must identify declump-kernel with candidate_enabled={enabled}")
    return errors


def expected_kernel(candidate: str | None, kernel_state: str | None) -> bool | None:
    """Write-skip must name its accepted kernel baseline explicitly."""
    if kernel_state not in (None, "on", "off"):
        raise ValueError("kernel-state must be on or off")
    if candidate == "write-skip" and kernel_state is None:
        raise ValueError("write-skip requires --kernel-state on|off")
    return None if kernel_state is None else kernel_state == "on"


def write_skip_stamp_errors(stamp: Json, kernel_stamp: Json,
                           enabled: bool, kernel_enabled: bool) -> list[str]:
    """Check independent candidate and fixed-kernel readbacks without coercion."""
    errors = kernel_stamp_errors(kernel_stamp, kernel_enabled)
    if not isinstance(stamp, dict):
        return [*errors, "missing or malformed perf_candidate_p13 stamp"]
    if stamp.get("requested_write_skip") is not enabled:
        errors.append(f"requested_write_skip must be {enabled}")
    if stamp.get("requested_kernel") is not kernel_enabled:
        errors.append(f"requested_kernel must remain {kernel_enabled}")
    if stamp.get("method_available") is not True or stamp.get("valid") is not True:
        errors.append("write-skip readback unavailable or invalid")
    actual = stamp.get("actual")
    if not isinstance(actual, dict):
        errors.append("missing write-skip actual readback")
    elif (actual.get("change_id") != "write-skip"
          or actual.get("candidate_enabled") is not enabled
          or actual.get("kernel_enabled") is not kernel_enabled):
        errors.append("write-skip actual candidate/kernel state mismatch")
    return errors


def compare_world(base: Path, current: Path, spec: tuple[str, int, int],
                  candidate: str | None = None, kernel_state: str | None = None) -> WorldResult:
    """Require full frame/tick schedules, nonempty agent rows and byte equality."""
    scope = identity_scope(candidate)
    fixed_kernel = expected_kernel(candidate, kernel_state)
    label, start_tick, seed = spec
    pan = label.startswith("W500")
    frozen = label == "W500_FROZEN"
    frames = 240 if pan else 600
    advance_every = 0 if frozen else (1 if pan else 2)
    advances = 0 if frozen else (240 if pan else 300)
    ticks = [start_tick + (0 if frozen else i // advance_every + 1) for i in range(frames)]
    errors: list[str] = []
    captures = []
    dumps = []
    for capture_index, directory in enumerate((base, current)):
        metadata_path = directory / f"{label}.json"
        dump_path = directory / f"{label}.jsonl"
        try:
            metadata = json.loads(metadata_path.read_text())
            dump = dump_path.read_bytes()
            rows = [json.loads(line) for line in dump.splitlines()]
        except (OSError, json.JSONDecodeError) as exc:
            errors.append(f"{directory}: {exc}")
            continue
        if not isinstance(metadata, dict):
            errors.append(f"{metadata_path}: metadata must be an object")
            continue
        if candidate:
            enabled = capture_index == 1
            if candidate == "write-skip":
                assert isinstance(fixed_kernel, bool)
            kernel_enabled = fixed_kernel if candidate == "write-skip" else enabled
            assert isinstance(kernel_enabled, bool)
            errors.extend(f"{metadata_path}: {error}" for error in
                          kernel_stamp_errors(metadata.get("perf_candidate_p12"), kernel_enabled))
            states = metadata.get("candidate_states")
            if not isinstance(states, list) or len(states) != frames:
                errors.append(f"{metadata_path}: expected {frames} candidate frame stamps")
            if isinstance(states, list):
                for index, stamp in enumerate(states):
                    errors.extend(f"{metadata_path}: frame {index + 1}: {error}" for error in
                                  kernel_stamp_errors(stamp, kernel_enabled))
            if candidate == "write-skip":
                assert isinstance(fixed_kernel, bool)
                errors.extend(f"{metadata_path}: {error}" for error in write_skip_stamp_errors(
                    metadata.get("perf_candidate_p13"), metadata.get("perf_candidate_p12"), enabled, fixed_kernel))
                skip_states = metadata.get("write_skip_states")
                if not isinstance(skip_states, list) or len(skip_states) != frames:
                    errors.append(f"{metadata_path}: expected {frames} write-skip frame stamps")
                if isinstance(skip_states, list):
                    for index, stamp in enumerate(skip_states):
                        kernel_stamp = states[index] if isinstance(states, list) and index < len(states) else None
                        errors.extend(f"{metadata_path}: frame {index + 1}: {error}" for error in
                                      write_skip_stamp_errors(stamp, kernel_stamp, enabled, fixed_kernel))
        expected = {
            "world": label, "seed": seed, "start_tick": start_tick,
            "end_tick": start_tick + advances, "frames": frames, "fixed_fps": 60,
            "advance_every_frames": advance_every, "mutation_route": "SimReads", "advance_calls": advances,
            "scope": "viewport_plus_64px", "pan": pan, "pan_step_world_px": 10 if pan else 0,
            "speed": 0 if frozen else (2 if pan else 1),
            "render_start_frame": 120, "warmup_frames": 120,
            "process_frames": list(range(241, 241 + frames)),
            "ticks": ticks,
        }
        for key, value in expected.items():
            if metadata.get(key) != value:
                errors.append(f"{metadata_path}: invalid {key}")
        for key in ("build_commit", "build_sha256", "world_info", "collector_sha256"):
            if not metadata.get(key):
                errors.append(f"{metadata_path}: missing build/world provenance {key}")
        if metadata.get("validated_delta_samples", 0) < frames:
            errors.append(f"{metadata_path}: fewer validated deltas than required frames")
        if metadata.get("dump_sha256") != hashlib.sha256(dump).hexdigest():
            errors.append(f"{metadata_path}: dump checksum differs")
        if len(rows) != frames:
            errors.append(f"{dump_path}: expected {frames} rows, got {len(rows)}")
        if any(not isinstance(row, dict) for row in rows):
            errors.append(f"{dump_path}: rows must be objects")
            continue
        for index, row in enumerate(rows):
            if not isinstance(row, dict):
                errors.append(f"{dump_path}: frame {index + 1} is not an object")
                break
            expected_row = {"frame": index + 1, "process_frame": 241 + index,
                            "tick": ticks[index] if index < frames else -1}
            if any(row.get(key) != value for key, value in expected_row.items()):
                errors.append(f"{dump_path}: invalid schedule at frame {index + 1}")
                break
            agents = row.get("agents")
            if not isinstance(agents, list):
                errors.append(f"{dump_path}: frame {index + 1} has malformed agents")
                break
            ids: list[int] = []
            for agent in agents:
                if not isinstance(agent, dict) or not isinstance(agent.get("id"), int):
                    errors.append(f"{dump_path}: malformed agent at frame {index + 1}")
                    break
                ids.append(agent["id"])
                required = {"position", "texture", "region", "region_enabled", "flip_h",
                            "visible", "visible_in_tree", "modulate", "self_modulate",
                            "z_index", "z_as_relative", "children"}
                if not required <= agent.keys():
                    errors.append(f"{dump_path}: missing sprite fields at frame {index + 1}")
                    break
            if ids != sorted(set(ids)):
                errors.append(f"{dump_path}: agent IDs must be unique and sorted")
                break
        if not any(row.get("agents") for row in rows):
            errors.append(f"{dump_path}: no agent entered viewport/margin in the full session")
        populations = metadata.get("populations", [])
        if len(populations) != frames or any(not isinstance(n, int) or n <= 0 for n in populations):
            errors.append(f"{metadata_path}: invalid population series")
        camera_start = metadata.get("camera_start", [])
        hearth = metadata.get("hearth", [])
        if len(camera_start) != 2 or len(hearth) != 2:
            errors.append(f"{metadata_path}: missing camera start/hearth")
        else:
            if camera_start != [hearth[0] - (1200 if pan else 0), hearth[1]]:
                errors.append(f"{metadata_path}: invalid initial hearth-relative camera")
            for index, row in enumerate(rows):
                camera = [camera_start[0] + (10 * index if pan else 0), camera_start[1]]
                if row.get("camera") != camera:
                    errors.append(f"{dump_path}: invalid pan schedule at frame {index + 1}")
                    break
                if index >= len(populations) or row.get("population") != populations[index]:
                    errors.append(f"{dump_path}: invalid population provenance at frame {index + 1}")
                    break
            if rows and metadata.get("camera") != rows[-1].get("camera"):
                errors.append(f"{metadata_path}: final camera differs from dump")
        if len(metadata.get("render_counts", [])) != frames:
            errors.append(f"{metadata_path}: missing per-frame render count series")
        for key in ("render_counts_start", "render_counts_end"):
            counts = metadata.get(key, {})
            if not isinstance(counts, dict) or any(not isinstance(counts.get(field), int)
                    or counts[field] < 0 for field in ("pre_draw", "post_draw", "engine_frames_drawn")):
                errors.append(f"{metadata_path}: missing render-clock evidence {key}")
        captures.append(metadata)
        dumps.append(dump)
    if len(captures) == 2:
        for key in SCHEDULE_FIELDS:
            if captures[0].get(key) != captures[1].get(key):
                errors.append(f"{label}: baseline/candidate {key} differ")
        if dumps[0] != dumps[1]:
            first_difference = next((i + 1 for i, (left, right) in enumerate(
                zip(dumps[0].splitlines(), dumps[1].splitlines(), strict=False)) if left != right), None)
            errors.append(f"{label}: dumps differ; first differing frame={first_difference}")
    else:
        errors.append(f"{label}: both complete captures required")
    return {
        "scope": scope, "world": label, "start_tick": start_tick, "frames": frames, "fixed_fps": 60,
        "advance_every_frames": advance_every, "mutation_route": "SimReads",
        "base_dump": str((base / f"{label}.jsonl").resolve()),
        "candidate_dump": str((current / f"{label}.jsonl").resolve()),
        "valid": not errors, "status": "PASS" if not errors else "FAIL", "errors": errors,
    }


def compare(base: Path, current: Path, include_multi: bool = True,
            candidate: str | None = None, kernel_state: str | None = None) -> Report:
    """Compare both running worlds and both pan scenarios; missing evidence fails."""
    scope = identity_scope(candidate)
    expected_kernel(candidate, kernel_state)
    worlds = [compare_world(base, current, spec, candidate, kernel_state) for spec in WORLDS]
    valid = all(world["valid"] for world in worlds)
    return {"scope": scope, "candidate": candidate, "kernel_state": kernel_state, "valid": valid, "status": "PASS" if valid else "FAIL", "worlds": worlds}


def main() -> int:
    """Write a machine-readable report and fail closed on incomplete captures."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--include-multi", action="store_true", help="Legacy option; all worlds are always required")
    parser.add_argument("--candidate", choices=("declump-kernel", "write-skip"))
    parser.add_argument("--kernel-state", choices=("on", "off"))
    args = parser.parse_args()
    result = compare(args.base, args.current, True, args.candidate, args.kernel_state)
    output = args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if result["valid"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
