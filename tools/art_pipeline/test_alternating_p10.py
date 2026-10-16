#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest discover -s tools/art_pipeline -p test_alternating_p10.py
"""Synthetic experiment evidence and fail-before-launch contract checks."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import summarize_perf_p10 as perf
from validate_alternating_p10 import Record, validate


def fixture() -> Record:
    state: Record = {"tick": 1440, "camera": [448, 28], "population": 500, "zoom": [4, 4], "simulation_frozen": False}
    blocks: list[perf.Json] = []
    for index in range(6):
        rows: list[perf.Json] = [{"state": state, "process_frame": index * 180 + frame + 1, "paired": True,
            "ticks_run": 0, "process_ms": 4, "render_cpu_ms": 1, "primary_cpu_ms": 5,
            "direct_read_context": {"inspector_visible": False, "influence_channel": -1, "layers_enabled": True}, "setup_cpu_ms": 0.2, "gpu_ms": 0, "pacing_ms": 6} for frame in range(180)]
        blocks.append({"index": index, "mode": "candidate" if index % 2 else "control", "switch_state": {"change_id": "wood-cache", "candidate_enabled": bool(index % 2)}, "discarded": rows[:30], "samples": rows[30:], "max_fps": 0, "vsync_disabled": True, "camera_process_frozen": True, "window_size": [1920, 1080]})
    return {"valid": True, "errors": [], "world": "W500", "change_id": "wood-cache", "build_commit": "a" * 40, "build_sha256": "b" * 64, "initial": state, "final": state, "restoration": {key: True for key in ("vsync_restored", "layers_restored", "camera_process_restored", "probes_removed")}, "blocks": blocks, "metric": "process_ms + render_cpu_ms", "simulation_speed": 0}


class AlternatingTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="p10-alternating-test-")
        self.addCleanup(temporary.cleanup)
        self.folder = Path(temporary.name)
        self.data = fixture()
        source: Record = {"head": "a" * 40, "build_sha256": "b" * 64, "sources": {"scripts/ui/agent_renderer.gd": "c" * 64}}
        self.launch: Record = {"before": source, "after": source, "exit_code": 0, "finished": "synthetic", "world": "W500", "change_id": "wood-cache"}
        (self.folder / "session.exit").write_text("0\n")
        (self.folder / "session.log").write_text("synthetic complete\n")
        self.save()

    def save(self) -> None:
        (self.folder / "frames.json").write_text(json.dumps(self.data))
        (self.folder / "launch.json").write_text(json.dumps(self.launch))

    def test_complete_pairs_preserve_metric_and_do_not_claim_identity(self) -> None:
        result = validate(self.folder)
        self.assertTrue(result["valid"])
        self.assertEqual(perf.mapping(result["control"])["count"], 450)
        self.assertEqual(perf.mapping(result["candidate"])["median"], 5)
        self.assertEqual(result["median_speedup"], 1)
        self.assertEqual(result["pixel_identity"], "NOT_VERIFIED")

    def test_missing_or_nonzero_terminal_status_cannot_pass(self) -> None:
        (self.folder / "session.exit").unlink()
        with self.assertRaisesRegex(perf.EvidenceError, "Missing terminal"):
            validate(self.folder)
        (self.folder / "session.exit").write_text("124\n")
        with self.assertRaisesRegex(perf.EvidenceError, "Nonzero terminal"):
            validate(self.folder)

    def test_missing_block_rejected(self) -> None:
        self.data["blocks"] = list(perf.sequence(self.data["blocks"]))[:5]
        self.save()
        with self.assertRaisesRegex(perf.EvidenceError, "three ordered"):
            validate(self.folder)

    def test_wrong_order_metric_or_frame_count_rejected(self) -> None:
        for field, value in (("mode", "candidate"), ("samples", []), ("max_fps", 60)):
            self.data = fixture()
            rows = list(perf.sequence(self.data["blocks"]))
            row = dict(perf.mapping(rows[0]))
            row[field] = value
            rows[0] = row
            self.data["blocks"] = rows
            self.save()
            with self.assertRaises(perf.EvidenceError):
                validate(self.folder)

    def test_changed_source_or_unfinished_session_rejected(self) -> None:
        self.launch["after"] = {}
        self.save()
        with self.assertRaisesRegex(perf.EvidenceError, "source/build changed"):
            validate(self.folder)

    def test_real_runner_without_integration_never_creates_evidence(self) -> None:
        checkout = self.folder / "checkout"
        renderer = checkout / "scripts/ui/agent_renderer.gd"
        renderer.parent.mkdir(parents=True)
        renderer.write_text("extends Node2D\n")
        output = self.folder / "new-evidence"
        runner = Path(__file__).with_name("run_alternating_p10.py")
        result = subprocess.run([sys.executable, str(runner), str(checkout), "wood-cache", str(output)], capture_output=True, text=True, check=False, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no hold or Godot launch started", result.stderr)
        self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
