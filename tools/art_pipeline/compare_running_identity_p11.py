#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/compare_running_identity_p11.py BASE CURRENT OUTPUT.json
"""Verify complete deterministic running captures before claiming byte identity."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Final, TypedDict

WORLDS: Final = (("W0", 40560, 20261010), ("W200", 1440, 15100), ("W500_FROZEN", 1440, 15100), ("W500_2X", 1440, 15100))
SCHEDULE_FIELDS: Final = (
    "world", "seed", "start_tick", "end_tick", "frames", "fixed_fps",
    "advance_every_frames", "mutation_route", "advance_calls", "process_frames",
    "ticks", "render_start_frame", "warmup_frames", "camera", "zoom", "debug_build",
    "populations", "scope", "pan", "camera_start", "hearth", "pan_step_world_px",
    "speed", "simulation_schedule", "collector_sha256",
)


class WorldResult(TypedDict):
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
    valid: bool
    status: str
    worlds: list[WorldResult]


def compare_world(base: Path, current: Path, spec: tuple[str, int, int]) -> WorldResult:
    """Require full frame/tick schedules, nonempty agent rows and byte equality."""
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
    for directory in (base, current):
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
        "world": label, "start_tick": start_tick, "frames": frames, "fixed_fps": 60,
        "advance_every_frames": advance_every, "mutation_route": "SimReads",
        "base_dump": str((base / f"{label}.jsonl").resolve()),
        "candidate_dump": str((current / f"{label}.jsonl").resolve()),
        "valid": not errors, "status": "PASS" if not errors else "FAIL", "errors": errors,
    }


def compare(base: Path, current: Path) -> Report:
    """Compare both running worlds and both pan scenarios; missing evidence fails."""
    worlds = [compare_world(base, current, spec) for spec in WORLDS]
    valid = all(world["valid"] for world in worlds)
    return {"valid": valid, "status": "PASS" if valid else "FAIL", "worlds": worlds}


def main() -> int:
    """Write a machine-readable report and fail closed on incomplete captures."""
    if len(sys.argv) != 4:
        print(__doc__ + "\nUsage: compare_running_identity_p11.py BASE CURRENT OUTPUT.json", file=sys.stderr)
        return 2
    result = compare(Path(sys.argv[1]), Path(sys.argv[2]))
    output = Path(sys.argv[3])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if result["valid"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
