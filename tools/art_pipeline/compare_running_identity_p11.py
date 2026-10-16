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

WORLDS: Final = (("W0", 40560, 20261010), ("W200", 1440, 15100))
SCHEDULE_FIELDS: Final = (
    "world", "seed", "start_tick", "end_tick", "frames", "fixed_fps",
    "advance_every_frames", "mutation_route", "advance_calls", "process_frames",
    "ticks", "render_start_frame", "warmup_frames", "camera", "zoom", "debug_build",
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
            "end_tick": start_tick + 300, "frames": 600, "fixed_fps": 60,
            "advance_every_frames": 2, "mutation_route": "SimReads", "advance_calls": 300,
            "render_start_frame": 120, "warmup_frames": 120,
            "process_frames": list(range(241, 841)),
            "ticks": [start_tick + i // 2 + 1 for i in range(600)],
        }
        for key, value in expected.items():
            if metadata.get(key) != value:
                errors.append(f"{metadata_path}: invalid {key}")
        for key in ("build_commit", "build_sha256", "world_info"):
            if not metadata.get(key):
                errors.append(f"{metadata_path}: missing build/world provenance {key}")
        if metadata.get("validated_delta_samples", 0) < 600:
            errors.append(f"{metadata_path}: fewer than 600 validated deltas")
        if metadata.get("dump_sha256") != hashlib.sha256(dump).hexdigest():
            errors.append(f"{metadata_path}: dump checksum differs")
        if len(rows) != 600:
            errors.append(f"{dump_path}: expected 600 rows, got {len(rows)}")
        for index, row in enumerate(rows):
            if not isinstance(row, dict):
                errors.append(f"{dump_path}: frame {index + 1} is not an object")
                break
            expected_row = {"frame": index + 1, "process_frame": 241 + index,
                            "tick": start_tick + index // 2 + 1}
            if any(row.get(key) != value for key, value in expected_row.items()):
                errors.append(f"{dump_path}: invalid schedule at frame {index + 1}")
                break
            agents = row.get("agents")
            if not isinstance(agents, list) or not agents:
                errors.append(f"{dump_path}: frame {index + 1} has no agents")
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
        "world": label, "start_tick": start_tick, "frames": 600, "fixed_fps": 60,
        "advance_every_frames": 2, "mutation_route": "SimReads",
        "base_dump": str((base / f"{label}.jsonl").resolve()),
        "candidate_dump": str((current / f"{label}.jsonl").resolve()),
        "valid": not errors, "status": "PASS" if not errors else "FAIL", "errors": errors,
    }


def compare(base: Path, current: Path) -> Report:
    """Compare both required worlds; a missing world is a failure."""
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
