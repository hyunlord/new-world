#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic>=2", "typer", "pillow"]
# ///
# How to run: python3 tools/art_pipeline/validate_walker_identity_p13b.py A APRIME B RUNNING_OFF RUNNING_ON NEW_OUTPUT
"""Supplement inherited reference identity checks with walker provenance/polarity."""
from __future__ import annotations

import csv
import hashlib
import json
import re
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import typer
from pydantic import BaseModel, ConfigDict, Field, JsonValue, StrictBool, StrictInt

import pixel_identity_p13 as pixels
from compare_running_identity_p13 import WORLDS, Report, compare, write_skip_stamp_errors


class Model(BaseModel):
    """Parse collector evidence without coercing booleans or dropping extensions."""

    model_config = ConfigDict(frozen=True, strict=True, extra="allow")


class Actual(Model):
    change_id: str
    candidate_enabled: StrictBool
    kernel_enabled: StrictBool
    write_skip_enabled: StrictBool


class Walker(Model):
    requested_env: str
    requested_walker: StrictBool
    requested_kernel: StrictBool
    requested_write_skip: StrictBool
    method_available: StrictBool
    valid: StrictBool
    actual: Actual


class Capture(Model):
    perf_candidate_p13b: Walker
    perf_candidate_p12: dict[str, JsonValue]
    perf_candidate_p13: dict[str, JsonValue]
    walker_states: list[Walker] = Field(default_factory=list)
    candidate_states: list[dict[str, JsonValue]] = Field(default_factory=list)
    write_skip_states: list[dict[str, JsonValue]] = Field(default_factory=list)
    frames: StrictInt = 0
    build_commit: str = ""
    build_sha256: str = ""
    collector_sha256: str = ""


class Provenance(Model):
    checkout: str
    head: str
    debug_sha256: str
    source_sha256: str
    sources: dict[str, str]
    candidate_env: dict[str, str]


@dataclass(frozen=True, slots=True)
class Run:
    folder: Path
    enabled: bool
    running: bool

    @property
    def labels(self) -> tuple[str, ...]:
        return tuple(world[0] for world in WORLDS) if self.running else pixels.FRAMINGS


@dataclass(frozen=True, slots=True)
class EvidenceError(ValueError):
    detail: str

    def __str__(self) -> str:
        return self.detail


def require(condition: bool, detail: str) -> None:
    if not condition:
        raise EvidenceError(detail)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def walker_errors(stamp: Walker, enabled: bool) -> list[str]:
    """Never accept a valid flag in place of actual renderer polarity."""
    expected = {
        "requested_env": str(int(enabled)), "requested_walker": enabled,
        "requested_kernel": True, "requested_write_skip": False,
        "method_available": True, "valid": True,
    }
    errors = [key for key, value in expected.items() if getattr(stamp, key) != value]
    actual = stamp.actual
    if (actual.change_id != "walker" or actual.candidate_enabled is not enabled
            or actual.kernel_enabled is not True or actual.write_skip_enabled is not False):
        errors.append("actual walker/kernel/write-skip polarity")
    return errors


def validate_run(run: Run) -> Provenance:
    """Require completed launches, stable snapshots and every expected stamp."""
    initial = Provenance.model_validate_json((run.folder / "provenance/initial.json").read_text())
    expected_env = {"P13B_WALKER_KERNEL": str(int(run.enabled)),
                    "P12_DECLUMP_KERNEL": "1", "P13_WRITE_SKIP": "0"}
    require(initial.candidate_env == expected_env, f"{run.folder}: explicit candidate env mismatch")
    require(bool(re.fullmatch(r"[0-9a-f]{40}", initial.head)), "invalid HEAD")
    for value in (initial.debug_sha256, initial.source_sha256, *initial.sources.values()):
        require(bool(re.fullmatch(r"[0-9a-f]{64}", value)), "invalid provenance SHA256")
    source_hash = hashlib.sha256(json.dumps(initial.sources, sort_keys=True).encode()).hexdigest()
    require(source_hash == initial.source_sha256 and bool(initial.sources), "source manifest checksum mismatch")
    with (run.folder / "launches.tsv").open() as stream:
        launches = list(csv.DictReader(stream, delimiter="\t"))
    label_key = "world" if run.running else "framing"
    require([row[label_key] for row in launches] == list(run.labels), f"{run.folder}: incomplete/duplicate launches")
    runner = dict(line.split("=", 1) for line in (run.folder / "runner.txt").read_text().splitlines())
    require(runner.get("checkout") == initial.checkout and runner.get("fixed_fps") == "60"
            and runner.get("resolution") == "1920x1080", "runner provenance mismatch")
    if run.running:
        require(runner.get("head") == initial.head and runner.get("debug_sha256") == initial.debug_sha256,
                "running runner build differs")
    else:
        require(initial.model_extra is not None, "missing still runner provenance")
        require(initial.model_extra.get("p3_plan_sha256") == digest(run.folder / "p3-plan.json"), "P3 plan digest mismatch")
    for label, launch in zip(run.labels, launches, strict=True):
        require(all(launch[k] == v for k, v in {"exit_code": "0", "artifacts": "present",
                "log_errors": "none", "provenance": "unchanged"}.items()), f"{run.folder}/{label}: launch failed")
        start, end = (datetime.fromisoformat(launch[k]) for k in ("started_utc", "finished_utc"))
        require(start.tzinfo is not None and end.tzinfo is not None and end >= start, "invalid launch timestamps")
        log = run.folder / "logs" / label
        require(log.with_suffix(".exit").read_text().strip() == "0", "nonzero exit artifact")
        require(re.search(r"SCRIPT ERROR|ERROR:", log.with_suffix(".log").read_text()) is None, "error in capture log")
        for phase in ("before", "inside", "after"):
            observed = Provenance.model_validate_json((run.folder / "provenance" / f"{label}.{phase}.json").read_text())
            require(observed == initial, f"{run.folder}/{label}.{phase}: provenance changed")
        metadata = Capture.model_validate_json((run.folder / f"{label}.json").read_text())
        if not run.running and label == "P3":
            extra = metadata.model_extra or {}
            require(extra.get("camera_processing_disabled") is True
                    and extra.get("camera_paused") is False, "P3 camera input/pause state is not isolated")
        errors = walker_errors(metadata.perf_candidate_p13b, run.enabled)
        errors += write_skip_stamp_errors(metadata.perf_candidate_p13, metadata.perf_candidate_p12, False, True)
        if run.running:
            expected_frames = 240 if label.startswith("W500") else 600
            require(metadata.frames == expected_frames, "wrong running frame count")
            require(all(len(series) == expected_frames for series in (metadata.walker_states,
                    metadata.candidate_states, metadata.write_skip_states)), "missing per-frame stamp evidence")
            for index, (walker, kernel, skip) in enumerate(zip(metadata.walker_states,
                    metadata.candidate_states, metadata.write_skip_states, strict=True)):
                errors += [f"frame {index + 1}: {error}" for error in walker_errors(walker, run.enabled)
                           + write_skip_stamp_errors(skip, kernel, False, True)]
            require(metadata.walker_states[-1] == metadata.perf_candidate_p13b
                    and metadata.candidate_states[-1] == metadata.perf_candidate_p12
                    and metadata.write_skip_states[-1] == metadata.perf_candidate_p13, "final stamp differs")
            require(metadata.build_commit == initial.head and metadata.build_sha256 == initial.debug_sha256,
                    "metadata build differs from launch provenance")
            require(metadata.collector_sha256 == initial.sources.get("scripts/test/art/capture_running_identity_p13b.gd"),
                    "collector hash differs from source manifest")
        require(not errors, f"{run.folder}/{label}: {'; '.join(errors)}")
    return initial


def main(a: Path, aprime: Path, b: Path, running_off: Path, running_on: Path, output: Path) -> None:
    """Write a compound identity report; this does not accept performance/art gates."""
    output.mkdir(parents=True, exist_ok=False)
    runs = (Run(a, False, False), Run(aprime, False, False), Run(b, True, False),
            Run(running_off, False, True), Run(running_on, True, True))
    errors: list[str] = []
    provenance: list[Provenance] = []
    for run in runs:
        try:
            provenance.append(validate_run(run))
        except (OSError, ValueError, KeyError, TypeError, AttributeError, IndexError) as exc:
            errors.append(f"{run.folder}: {exc}")
    if len(provenance) == len(runs):
        cores = [p.model_dump(include={"checkout", "head", "debug_sha256", "source_sha256", "sources"}) for p in provenance]
        if any(core != cores[0] for core in cores[1:]):
            errors.append("cross-run checkout/HEAD/source/build differs")
        still = [p.model_dump(exclude={"candidate_env"}) for p in provenance[:3]]
        if any(p != still[0] for p in still[1:]):
            errors.append("still provenance differs beyond candidate env")
        running = [p.model_dump(exclude={"candidate_env"}) for p in provenance[3:]]
        if running[0] != running[1]:
            errors.append("running provenance differs beyond candidate env")
    pixel_pass = False
    running_report: Report = {"candidate": None, "kernel_state": None,
                              "scope": "reference_integrity_only", "valid": False,
                              "status": "FAIL", "worlds": []}
    try:
        running_report = compare(running_off, running_on)
    except (OSError, ValueError, KeyError, TypeError, AttributeError, IndexError) as exc:
        errors.append(f"inherited running comparator: {type(exc).__name__}: {exc}")
    try:
        pixels.main(a, aprime, b, output / "rgba-reference")
    except typer.Exit as exc:
        pixel_pass = exc.exit_code == 0
    except (OSError, ValueError, KeyError, TypeError, AttributeError, IndexError, typer.BadParameter) as exc:
        errors.append(f"inherited RGBA comparator: {type(exc).__name__}: {exc}")
    (output / "running-reference.json").write_text(json.dumps(running_report, indent=2) + "\n")
    evidence_hashes: dict[str, str] = {}
    for run in runs:
        for path in [run.folder / "launches.tsv", run.folder / "provenance/initial.json",
                     *(run.folder / f"{label}.json" for label in run.labels)]:
            try:
                if path.is_file():
                    evidence_hashes[str(path.resolve())] = digest(path)
            except OSError as exc:
                errors.append(f"evidence checksum {path}: {exc}")
    passed = not errors and pixel_pass and running_report["valid"]
    report = {"scope": "compound_walker_candidate_identity", "candidate": "walker", "passed": passed,
              "errors": errors, "kernel": "ON", "write_skip": "OFF", "still_polarity": [False, False, True],
              "running_polarity": [False, True], "inherited_scope": "reference_integrity_only",
              "rgba_reference_passed": pixel_pass, "running_reference_passed": running_report["valid"],
              "expected_stamp_coverage": "18 capture-time still stamps; 4560 running-frame stamps plus 10 final stamps",
              "limitations": ["Still collectors do not emit per-frame candidate stamps.",
                              "Identity only: no performance, visual acceptance or candidate installation approval."],
              "inputs": [str(run.folder.resolve()) for run in runs],
              "evidence_sha256": evidence_hashes,
              "validator_sha256": digest(Path(__file__)),
              "inherited_comparator_sha256": {name: digest(Path(__file__).with_name(name)) for name in
                                              ("pixel_identity_p13.py", "compare_running_identity_p13.py")}}
    (output / "walker-identity.json").write_text(json.dumps(report, indent=2) + "\n")
    typer.echo(json.dumps(report, indent=2))
    raise typer.Exit(0 if passed else 1)


if __name__ == "__main__":
    typer.run(main)
