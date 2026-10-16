#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/art_pipeline/test_summarize_perf_p9.py
"""Synthetic boundary checks, never performance evidence."""
from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Callable, TypeAlias

import summarize_perf_p9 as summary

Payload: TypeAlias = dict[str, summary.Json]
Mutation: TypeAlias = Callable[[Payload], None]


def fixture(world: str = "W500", phase: str = "baseline") -> Payload:
    blocks: list[summary.Json] = []
    rows: list[summary.Json] = []
    tick, frame = 1440, 0

    def block(label: str, condition: str, zoom: int = 4, speed: int = 0) -> None:
        nonlocal tick, frame
        row: Payload = {
            "label": label, "condition": condition, "zoom": zoom, "speed": speed,
            "max_fps": 60 if speed else 0, "vsync_disabled": True,
            "camera_process_frozen": True, "window_size": [1920, 1080],
        }
        for key, count in [("discarded", 30), ("samples", 300 if condition in {"frozen", "running"} else 150)]:
            samples: list[summary.Json] = []
            for _ in range(count):
                frame += 1
                ticks = int(bool(speed) and frame % 2 == 0)
                tick += ticks
                sample: Payload = {
                    "primary_cpu_ms": 3.0, "process_ms": 1.0, "render_cpu_ms": 2.0,
                    "setup_cpu_ms": 0.1, "gpu_ms": 0.0, "pacing_ms": 16.0,
                    "ticks_run": ticks, "process_frame": frame, "paired": True,
                    "state": {"tick": tick, "camera": [448, 28], "zoom": [zoom, zoom]},
                    "direct_read_context": {"inspector_visible": False, "influence_channel": -1},
                }
                if phase == "after":
                    sample["ffi_calls"] = {"get_time_snapshot": 1}
                    sample["ffi_counter_frame"] = frame
                samples.append(sample)
            row[key] = samples
        blocks.append(row)

    for zoom in (4, 1):
        for repeat in range(3):
            block(f"frozen_Z{zoom}_{repeat}", "frozen", zoom)
    if world in {"W0", "W500"}:
        before = "ablation_all_on_0"
        block(before, "ablation_reference")
        for node in ("WorldObjects", "Shadows", "BuildingRenderer/NightChillLayer", "AgentRenderer"):
            off, after = "off_" + node, "on_after_" + node
            block(off, "ablation_off")
            block(after, "ablation_reference")
            rows.append({"node": node, "layer_only": node != "AgentRenderer", "off_block": off,
                         "neighbour_blocks": [before, after], "cost_ms": 0.0,
                         "descendants_disabled_as_group": []})
            before = after
        block("ablation_floor", "ablation_floor")
        block("ablation_restored", "ablation_reference")
    for speed in (2, 4):
        for repeat in range(3):
            block(f"running_{speed}x_{repeat}", "running", 4, speed)
    return {
        "world": world, "phase": phase, "valid": True, "errors": [], "build_profile": "debug",
        "initial": {"tick": 1440, "camera": [448, 28]},
        "restoration": {key: True for key in ("vsync_restored", "layers_restored", "camera_process_restored", "probes_removed")},
        "ablation_visibility_restored": True, "ablation": rows, "blocks": blocks,
    }


def mutable(value: summary.Json) -> Payload:
    assert isinstance(value, dict)
    return value


def items(value: summary.Json) -> list[summary.Json]:
    assert isinstance(value, list)
    return value


def first_block(data: Payload) -> Payload:
    return mutable(items(data["blocks"])[0])


def first_sample(data: Payload) -> Payload:
    return mutable(items(first_block(data)["samples"])[0])


def set_block(key: str, value: summary.Json) -> Mutation:
    def change(data: Payload) -> None:
        first_block(data)[key] = value
    return change


def set_sample(key: str, value: summary.Json) -> Mutation:
    def change(data: Payload) -> None:
        first_sample(data)[key] = value
    return change


def no_ticks(data: Payload) -> None:
    for sample in items(mutable(items(data["blocks"])[-1])["samples"]):
        mutable(sample)["ticks_run"] = 0


def missing_floor(data: Payload) -> None:
    data["blocks"] = [block for block in items(data["blocks"]) if mutable(block)["label"] != "ablation_floor"]


def main() -> None:
    count = 0
    with tempfile.TemporaryDirectory(prefix="p9-collector-checks-") as directory:
        root = Path(directory)
        def load(data: Payload) -> summary.Session:
            path = root / "session.json"
            path.write_text(json.dumps(data))
            return summary.load_session(path)

        # Given a complete matrix, when parsed, then baseline fallback and after counters work.
        for world, phase in (("W500", "baseline"), ("W0", "baseline"), ("W1", "baseline"), ("W500", "after")):
            load(fixture(world, phase))
            count += 1
            print(f"PASS valid {world} {phase}")
        cases: list[tuple[str, Mutation]] = [
            ("restoration false", lambda data: mutable(data["restoration"]).__setitem__("camera_process_restored", False)),
            ("restoration missing", lambda data: data.__delitem__("restoration")),
            ("matrix reordered", lambda data: items(data["blocks"]).reverse()),
            ("wrong speed", set_block("speed", 2)), ("wrong fps", set_block("max_fps", 60)),
            ("vsync enabled", set_block("vsync_disabled", False)),
            ("camera processing", set_block("camera_process_frozen", False)),
            ("wrong window", set_block("window_size", [960, 540])),
            ("primary mismatch", set_sample("primary_cpu_ms", 4)),
            ("negative measurement", set_sample("gpu_ms", -1)),
            ("missing ablation", lambda data: data.__setitem__("ablation", [])),
            ("missing floor", missing_floor),
            ("ablation restoration false", lambda data: data.__setitem__("ablation_visibility_restored", False)),
            ("unpaired", set_sample("paired", False)), ("frame reuse", set_sample("process_frame", 1)),
            ("frozen tick", set_sample("ticks_run", 1)),
            ("camera move", lambda data: mutable(first_sample(data)["state"]).__setitem__("camera", [0, 0])),
            ("valid with errors", lambda data: data.__setitem__("errors", ["failure"])),
            ("short block", lambda data: items(first_block(data)["samples"]).__delitem__(-1)),
            ("no tick frames", no_ticks),
        ]
        original = fixture()
        # Given one invalid boundary, when parsed, then the evidence is rejected.
        for label, change in cases:
            data = copy.deepcopy(original)
            change(data)
            try:
                load(data)
            except summary.EvidenceError as error:
                print(f"PASS rejects {label}: {error}")
                count += 1
            else:
                raise AssertionError(f"Accepted {label}")
        data = fixture(phase="after")
        first_sample(data).__delitem__("ffi_calls")
        try:
            load(data)
        except summary.EvidenceError:
            print("PASS rejects missing after W500 ffi_calls")
            count += 1
        else:
            raise AssertionError("Accepted missing after W500 ffi_calls")
        # Given empty and even-sized samples, when summarized, then absence and median remain exact.
        assert summary.summary([], "primary_cpu_ms")["median"] is None
        print("PASS empty median remains null")
        count += 1
        assert summary.summary([{"primary_cpu_ms": 1.0}, {"primary_cpu_ms": 9.0}], "primary_cpu_ms")["median"] == 5.0
        print("PASS independent even median is 5.0")
        count += 1
        # Given four mandatory worlds, when aggregated, then raw evidence and pooled tables survive.
        names: list[str] = []
        for world in ("W0", "W1", "W200", "W500"):
            path = root / f"{world}.json"
            path.write_text(json.dumps(fixture(world)))
            names.append(str(path))
        prefix = root / "baseline"
        original_args = sys.argv
        try:
            sys.argv = ["summarize_perf_p9.py", str(prefix), *names]
            assert summary.main() == 0
        finally:
            sys.argv = original_args
        raw: summary.Json = json.loads(prefix.with_suffix(".json").read_text())
        sessions = items(mutable(raw)["sessions"])
        assert len(sessions) == 4
        assert first_sample(mutable(sessions[0]))["primary_cpu_ms"] == 3.0
        assert "POOLED" in prefix.with_suffix(".md").read_text()
        assert mutable(raw)["optional_worlds_not_measured"] == ["W1000", "W2000"]
        print("PASS CLI aggregate preserves raw samples, pooled tables and optional omissions")
        count += 1
    assert count == 28
    print(f"TOTAL: {count} focused checks passed; synthetic evidence only, no performance claims")


class CollectorBoundaryTests(unittest.TestCase):
    def test_synthetic_measurement_boundaries(self) -> None:
        main()

    def test_ffi_counter_alignment(self) -> None:
        for counter_frame in (None, -1):
            with self.subTest(counter_frame=counter_frame), tempfile.TemporaryDirectory(prefix="p9-counter-frame-") as directory:
                payload = fixture(phase="after")
                sample = first_sample(payload)
                if counter_frame is None:
                    del sample["ffi_counter_frame"]
                else:
                    sample["ffi_counter_frame"] = counter_frame
                path = Path(directory) / "session.json"
                path.write_text(json.dumps(payload))
                with self.assertRaisesRegex(summary.EvidenceError, "FFI counters do not match process frame"):
                    summary.load_session(path)

    def test_f4_requires_measured_closed_context(self) -> None:
        for context in ({}, {"inspector_visible": True, "influence_channel": -1}, {"inspector_visible": False, "influence_channel": 0}):
            with self.subTest(context=context), tempfile.TemporaryDirectory(prefix="p9-f4-context-") as directory:
                payload = fixture(phase="after")
                first_sample(payload)["direct_read_context"] = context
                path = Path(directory) / "session.json"
                path.write_text(json.dumps(payload))
                with self.assertRaisesRegex(summary.EvidenceError, "F4 requires closed"):
                    summary.load_session(path)


if __name__ == "__main__":
    main()
