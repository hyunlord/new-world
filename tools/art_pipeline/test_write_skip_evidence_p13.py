"""Write-skip evidence fixes the accepted kernel while proving each toggle."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import typer
from PIL import Image

from compare_running_identity_p13 import compare_world, write_skip_stamp_errors
from pixel_identity_p13 import FRAMINGS, main as pixel_main
from summarize_perf_p13 import EvidenceError
from test_identity_polarity_p13 import running_fixture, stamp
from validate_alternating_p13 import validate


def skip_stamp(enabled: bool, kernel: bool = True) -> dict:
    return {"requested_write_skip": enabled, "requested_kernel": kernel,
            "method_available": True, "valid": True,
            "actual": {"change_id": "write-skip", "candidate_enabled": enabled,
                       "kernel_enabled": kernel}}


class WriteSkipEvidenceTests(unittest.TestCase):
    def test_two_independent_readbacks_and_strict_booleans(self) -> None:
        self.assertEqual(write_skip_stamp_errors(skip_stamp(True), stamp(True), True, True), [])
        for candidate, kernel in ((None, stamp(True)), (skip_stamp(False), stamp(True)),
                                  (skip_stamp(True, False), stamp(True)), (skip_stamp(True), stamp(False)),
                                  ({**skip_stamp(True), "requested_write_skip": 1}, stamp(True)),
                                  ({**skip_stamp(True), "method_available": False}, stamp(True))):
            with self.subTest(candidate=candidate, kernel=kernel):
                self.assertTrue(write_skip_stamp_errors(candidate, kernel, True, True))

    def test_running_every_frame_and_fixed_kernel(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            a, b = root / "a", root / "b"
            ma, mb = running_fixture(a, True), running_fixture(b, True)
            for folder, metadata, enabled in ((a, ma, False), (b, mb, True)):
                metadata["perf_candidate_p13"] = skip_stamp(enabled)
                metadata["write_skip_states"] = [skip_stamp(enabled) for _ in range(600)]
                (folder / "W0.json").write_text(json.dumps(metadata))
            with self.assertRaises(ValueError):
                compare_world(a, b, ("W0", 40560, 20261010), "write-skip")
            def check() -> dict:
                (b / "W0.json").write_text(json.dumps(mb))
                return compare_world(a, b, ("W0", 40560, 20261010), "write-skip", "on")
            self.assertTrue(check()["valid"])
            self.assertFalse(compare_world(a, b, ("W0", 40560, 20261010), "write-skip", "off")["valid"])
            mb["write_skip_states"][599] = skip_stamp(False)
            self.assertTrue(any("frame 600" in e for e in check()["errors"]))
            mb["write_skip_states"][599] = skip_stamp(True)
            mb["candidate_states"][200] = stamp(False)
            self.assertFalse(check()["valid"])
            mb["candidate_states"][200] = stamp(True)
            mb["write_skip_states"].pop()
            self.assertFalse(check()["valid"])

    def test_pixels_p3_and_aprime_require_fixed_kernel(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            folders = [root / name for name in ("a", "ap", "b")]
            for folder, enabled in zip(folders, (False, False, True), strict=True):
                folder.mkdir()
                for framing in FRAMINGS:
                    Image.new("RGBA", (1, 1), (1, 2, 3, 255)).save(folder / f"{framing}.png")
                    (folder / f"{framing}.json").write_text(json.dumps({"capture_frame": 240,
                        "state_change_frame": 120, "tick": 100, "seed": 1, "fixed_fps": 60,
                        "perf_candidate_p12": stamp(True), "perf_candidate_p13": skip_stamp(enabled)}))
            with self.assertRaises(ValueError):
                pixel_main(*folders, root / "missing-kernel", "write-skip")
            def check(name: str) -> dict:
                with self.assertRaises(typer.Exit):
                    pixel_main(*folders, root / name, "write-skip", "on")
                return json.loads((root / name / "comparison.json").read_text())
            self.assertTrue(check("valid")["passed"])
            sidecar = folders[1] / "P3.json"
            metadata = json.loads(sidecar.read_text())
            metadata["perf_candidate_p13"] = skip_stamp(True)
            sidecar.write_text(json.dumps(metadata))
            self.assertFalse(check("bad-ap")["passed"])
            metadata["perf_candidate_p13"] = skip_stamp(False)
            metadata["perf_candidate_p12"] = stamp(False)
            sidecar.write_text(json.dumps(metadata))
            self.assertFalse(check("bad-kernel")["passed"])

    def test_alternating_checks_discarded_and_counted_frames(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            before = {"head": "a" * 40, "build_sha256": "b" * 64,
                      "sources": {"scripts/ui/agent_renderer.gd": "c" * 64}}
            launch = {"before": before, "after": before, "finished": "now", "exit_code": 0,
                      "world": "W500", "change_id": "write-skip", "kernel_enabled": True}
            initial = {"tick": 1440, "zoom": [4, 4], "population": 500, "simulation_frozen": False}
            blocks = []
            for index in range(6):
                enabled = bool(index % 2)
                samples = [{"process_frame": index * 180 + frame, "state": initial,
                            "paired": True, "ticks_run": 0, "process_ms": 2.0,
                            "render_cpu_ms": 1.0, "primary_cpu_ms": 3.0, "setup_cpu_ms": 0.0,
                            "total_with_setup_ms": 3.0, "gpu_ms": 0.0, "pacing_ms": 3.0,
                            "direct_read_context": {"inspector_visible": False, "influence_channel": -1, "layers_enabled": True},
                            "perf_candidate_p12": stamp(True), "perf_candidate_p13": skip_stamp(enabled)} for frame in range(180)]
                blocks.append({"index": index, "mode": "candidate" if enabled else "control",
                               "switch_state": skip_stamp(enabled)["actual"], "discarded": samples[:30],
                               "samples": samples[30:], "max_fps": 0, "vsync_disabled": True,
                               "camera_process_frozen": True, "window_size": [1920, 1080],
                               "fresh_setup": True, "start_state": initial, "population_flagged": False})
            data = {"build_commit": before["head"], "build_sha256": before["build_sha256"],
                    "world": "W500", "change_id": "write-skip", "kernel_enabled": True,
                    "valid": True, "errors": [], "metric": "process_ms + render_cpu_ms",
                    "initial": initial, "final": initial, "simulation_speed": 0, "blocks": blocks,
                    "restoration": {k: True for k in ("vsync_restored", "layers_restored", "camera_process_restored", "probes_removed")}}
            (folder / "session.exit").write_text("0")
            (folder / "session.log").write_text("")
            (folder / "launch.json").write_text(json.dumps(launch))
            def check() -> dict:
                (folder / "frames.json").write_text(json.dumps(data))
                # Admission is independently covered; these fixtures isolate state validation.
                with patch("validate_alternating_p13.require_valid"):
                    return validate(folder)
            self.assertTrue(check()["valid"])
            for group in ("discarded", "samples"):
                row = blocks[1][group][0]
                row["perf_candidate_p13"] = skip_stamp(False)
                with self.assertRaises(EvidenceError):
                    check()
                row["perf_candidate_p13"] = skip_stamp(True)
                row["perf_candidate_p12"] = stamp(False)
                with self.assertRaises(EvidenceError):
                    check()
                row["perf_candidate_p12"] = stamp(True)
            data["kernel_enabled"] = False
            with self.assertRaises(EvidenceError):
                check()
            launch["kernel_enabled"] = False
            (folder / "launch.json").write_text(json.dumps(launch))
            for block in blocks:
                enabled = block["mode"] == "candidate"
                block["switch_state"] = skip_stamp(enabled, False)["actual"]
                for row in block["discarded"] + block["samples"]:
                    row["perf_candidate_p12"] = stamp(False)
                    row["perf_candidate_p13"] = skip_stamp(enabled, False)
            self.assertTrue(check()["valid"])
            # Existing declump sessions remain valid without any p13-only fields.
            launch["change_id"] = data["change_id"] = "declump-kernel"
            del launch["kernel_enabled"]
            del data["kernel_enabled"]
            (folder / "launch.json").write_text(json.dumps(launch))
            for block in blocks:
                block["switch_state"] = stamp(block["mode"] == "candidate")["actual"]
                for row in block["discarded"] + block["samples"]:
                    del row["perf_candidate_p12"]
                    del row["perf_candidate_p13"]
            self.assertTrue(check()["valid"])


if __name__ == "__main__":
    unittest.main()
