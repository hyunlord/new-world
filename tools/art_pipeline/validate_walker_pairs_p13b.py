#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic>=2", "typer"]
# ///
# How to run: python3 tools/art_pipeline/validate_walker_pairs_p13b.py W200_DIR W500_DIR NEW_OUTPUT
"""Validate walker pair evidence without changing inherited speed or identity gates."""
from __future__ import annotations

import hashlib
import json
from datetime import datetime
from pathlib import Path

import typer
from pydantic import BaseModel, ConfigDict, StrictBool, StrictInt

from validate_alternating_p13 import validate
from summarize_perf_p13 import EvidenceError


class Model(BaseModel):
    """Strict collector boundary; preserve provenance extensions for equality."""

    model_config = ConfigDict(frozen=True, strict=True, extra="allow")


class Kernel(Model):
    change_id: str
    candidate_enabled: StrictBool


class Skip(Kernel):
    kernel_enabled: StrictBool


class Actual(Skip):
    write_skip_enabled: StrictBool


class Stamp(Model):
    requested_walker: StrictBool
    requested_kernel: StrictBool
    requested_write_skip: StrictBool
    actual: Actual
    kernel: Kernel
    write_skip: Skip
    valid: StrictBool


class Frame(Model):
    perf_candidate_p13b: Stamp


class Block(Model):
    index: StrictInt
    mode: str
    switch_state: Actual
    switch_state_after: Actual
    discarded: list[Frame]
    samples: list[Frame]


class Provenance(Model):
    checkout: str
    head: str
    build_profile: str
    build_sha256: str
    release_sha256: str
    sources: dict[str, str]


class Launch(Model):
    change_id: str
    world: str
    before: Provenance
    after: Provenance
    kernel_enabled: StrictBool
    write_skip_enabled: StrictBool
    exit_code: StrictInt
    started: str
    finished: str
    command: list[str]


class Capture(Model):
    change_id: str
    world: str
    build_profile: str
    release_sha256: str
    build_commit: str
    build_sha256: str
    kernel_enabled: StrictBool
    write_skip_enabled: StrictBool
    contended: StrictBool
    blocks: list[Block]


def require(condition: bool, detail: str) -> None:
    if not condition:
        raise EvidenceError(detail)


def actual_valid(actual: Actual, enabled: bool) -> bool:
    return (actual.change_id == "walker" and actual.candidate_enabled is enabled
            and actual.kernel_enabled is True and actual.write_skip_enabled is False)


def stamp_valid(stamp: Stamp, enabled: bool) -> bool:
    """Check actual independent readbacks even when collector valid is true."""
    return (stamp.valid is True and stamp.requested_walker is enabled
            and stamp.requested_kernel is True and stamp.requested_write_skip is False
            and actual_valid(stamp.actual, enabled)
            and stamp.kernel.change_id == "declump-kernel" and stamp.kernel.candidate_enabled is True
            and stamp.write_skip.change_id == "write-skip" and stamp.write_skip.candidate_enabled is False
            and stamp.write_skip.kernel_enabled is True)


def supplemental(folder: Path, world: str) -> Provenance:
    """Require every discarded/measured frame and launch readback to match."""
    launch = Launch.model_validate_json((folder / "launch.json").read_text())
    capture = Capture.model_validate_json((folder / "frames.json").read_text())
    require(launch.change_id == capture.change_id == "walker", "expected walker candidate")
    require(launch.world == capture.world == world, "world argument/launch/collector mismatch")
    require(launch.kernel_enabled is capture.kernel_enabled is True, "kernel must remain ON")
    require(launch.write_skip_enabled is capture.write_skip_enabled is False, "write skip must remain OFF")
    require(capture.contended is False, "contended session")
    require(launch.exit_code == 0 and launch.before == launch.after, "launch incomplete or provenance changed")
    start, end = datetime.fromisoformat(launch.started), datetime.fromisoformat(launch.finished)
    require(start.tzinfo is not None and end.tzinfo is not None and end >= start, "invalid launch times")
    require(launch.before.build_profile == capture.build_profile == "release", "release profile required")
    require(launch.before.build_sha256 == launch.before.release_sha256
            == capture.release_sha256 == capture.build_sha256, "release/debug/collector library differs")
    require(launch.before.head == capture.build_commit, "collector HEAD differs")
    command = launch.command
    require("--path" in command and "--script" in command and "--resolution" in command, "missing launch controls")
    require(command[command.index("--path") + 1] == launch.before.checkout
            and command[command.index("--script") + 1] == "res://scripts/test/art/perf_alternating_frames_p13b.gd"
            and command[command.index("--resolution") + 1] == "1920x1080", "launch route differs")
    require("scripts/test/art/perf_alternating_frames_p13b.gd" in launch.before.sources,
            "collector absent from source provenance")
    require(len(capture.blocks) == 6, "requires exactly three control/candidate pairs")
    for index, block in enumerate(capture.blocks):
        enabled = bool(index % 2)
        require(block.index == index and block.mode == ("candidate" if enabled else "control"), "block order mismatch")
        require(actual_valid(block.switch_state, enabled) and actual_valid(block.switch_state_after, enabled),
                f"block {index}: initial/final walker switch differs")
        require(block.switch_state == block.switch_state_after, f"block {index}: switch configuration changed")
        require(len(block.discarded) == 30 and len(block.samples) == 150, "requires 30 discarded + 150 counted frames")
        for number, frame in enumerate([*block.discarded, *block.samples], 1):
            require(stamp_valid(frame.perf_candidate_p13b, enabled), f"block {index}, frame {number}: walker/kernel/skip stamp mismatch")
    return launch.before


def main(w200: Path, w500: Path, output: Path) -> None:
    """Preserve inherited .97 verdict; separately assess GOAL walker guardrails."""
    output.mkdir(parents=True, exist_ok=False)
    results = []
    provenance = []
    errors = []
    for world, folder in (("W200", w200), ("W500", w500)):
        try:
            provenance.append(supplemental(folder, world))
            inherited = validate(folder)
            (output / f"{world}-inherited.json").write_text(json.dumps(inherited, indent=2) + "\n")
            ratio = inherited["pooled_median_ratio"]
            pair_ratios = inherited["pair_ratios"]
            # The unchanged validator computes these; JSON boundary models keep arithmetic strict.
            metrics = Metrics.model_validate({"ratio": ratio, "pair_ratios": pair_ratios})
            faster = sum(value < 1 for value in metrics.pair_ratios)
            goal_pass = metrics.ratio <= (0.97 if world == "W200" else 1.00) and faster >= 2
            results.append({"world": world, "evidence_valid": True, "pairs": 3, "frame_stamps_checked": 1080,
                            "pooled_median_ratio": metrics.ratio, "pair_ratios": metrics.pair_ratios,
                            "faster_pairs": faster, "inherited_speed_pass": inherited["speed_pass"],
                            "walker_goal_threshold": 0.97 if world == "W200" else 1.00,
                            "walker_goal_pass": goal_pass, "source": str(folder.resolve())})
        except (OSError, ValueError, KeyError, IndexError) as exc:
            errors.append(f"{world}: {exc}")
    if len(provenance) == 2 and provenance[0] != provenance[1]:
        errors.append("W200/W500 source, build or checkout provenance differs")
    evidence_valid = not errors and len(results) == 2
    speed_pass = evidence_valid and all(row["walker_goal_pass"] for row in results)
    files = [(folder / name) for folder in (w200, w500) for name in
             ("launch.json", "frames.json", "frames.json.validity.json", "session.log", "session.exit")]
    report = {"scope": "supplemental_walker_performance", "evidence_valid": evidence_valid,
              "walker_performance_pass": speed_pass, "errors": errors, "worlds": results,
              "inherited_validator_unchanged": True, "inherited_speed_pass_rule": "pooled <=0.97 and >=2/3 faster, for both worlds",
              "walker_goal_rule": "W200 pooled <=0.97 and >=2/3 faster; W500 pooled <=1.00 and >=2/3 faster",
              "candidate_acceptance": "NOT_ESTABLISHED_REQUIRES_SEPARATE_IDENTITY",
              "evidence_sha256": {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in files if p.is_file()},
              "validator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "inherited_validator_sha256": hashlib.sha256(Path(__file__).with_name("validate_alternating_p13.py").read_bytes()).hexdigest()}
    (output / "walker-pairs.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = ["# Supplemental walker performance", "", f"Evidence valid: {evidence_valid}; walker performance pass: {speed_pass}.",
             "", "Candidate acceptance remains NOT_ESTABLISHED: separate identity acceptance is required.", "",
             "Inherited .97 speed_pass values are preserved; W500's walker-specific <=1.00 guardrail is separate; both worlds require >=2/3 faster pairs.", ""]
    lines.extend(json.dumps(row, sort_keys=True) for row in results)
    lines.extend(["", *errors])
    (output / "walker-pairs.md").write_text("\n".join(lines) + "\n")
    typer.echo(json.dumps(report, indent=2))
    raise typer.Exit(0 if speed_pass else 1)


class Metrics(Model):
    ratio: float
    pair_ratios: list[float]


if __name__ == "__main__":
    typer.run(main)
