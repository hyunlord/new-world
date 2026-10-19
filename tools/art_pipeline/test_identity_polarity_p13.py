"""Candidate identity must prove the renderer switch as well as unchanged output."""
from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import typer
from PIL import Image

from compare_running_identity_p13 import compare_world, kernel_stamp_errors
from pixel_identity_p13 import FRAMINGS, main as pixel_main


def stamp(enabled: bool) -> dict:
    return {"requested_kernel": enabled, "method_available": True, "valid": True,
            "actual": {"change_id": "declump-kernel", "candidate_enabled": enabled}}


def running_fixture(folder: Path, enabled: bool) -> dict:
    folder.mkdir()
    ticks = [40560 + i // 2 + 1 for i in range(600)]
    agent = {"id": 1, "position": [0, 0], "texture": "body.png", "region": [0, 0, 8, 8],
             "region_enabled": True, "flip_h": False, "visible": True, "visible_in_tree": True,
             "modulate": [1, 1, 1, 1], "self_modulate": [1, 1, 1, 1], "z_index": 0,
             "z_as_relative": True, "children": []}
    rows = [{"frame": i + 1, "process_frame": 241 + i, "tick": tick,
             "population": 1, "camera": [0, 0], "agents": [agent]} for i, tick in enumerate(ticks)]
    dump = ("\n".join(json.dumps(row) for row in rows) + "\n").encode()
    (folder / "W0.jsonl").write_bytes(dump)
    counts = {"pre_draw": 1, "post_draw": 1, "engine_frames_drawn": 1}
    metadata = {"world": "W0", "seed": 20261010, "start_tick": 40560, "end_tick": 40860,
                "frames": 600, "fixed_fps": 60, "advance_every_frames": 2,
                "mutation_route": "SimReads", "advance_calls": 300, "scope": "viewport_plus_64px",
                "pan": False, "pan_step_world_px": 0, "speed": 1, "render_start_frame": 120,
                "warmup_frames": 120, "process_frames": list(range(241, 841)), "ticks": ticks,
                "build_commit": "commit", "build_sha256": "binary", "world_info": {"seed": 20261010},
                "collector_sha256": "collector", "validated_delta_samples": 600,
                "dump_sha256": hashlib.sha256(dump).hexdigest(), "populations": [1] * 600,
                "camera_start": [0, 0], "hearth": [0, 0], "camera": [0, 0],
                "render_counts": [counts] * 600, "render_counts_start": counts, "render_counts_end": counts,
                "perf_candidate_p12": stamp(enabled), "candidate_states": [stamp(enabled) for _ in range(600)]}
    (folder / "W0.json").write_text(json.dumps(metadata))
    return metadata


class CandidateIdentityTests(unittest.TestCase):
    def test_strict_readback(self) -> None:
        self.assertFalse(kernel_stamp_errors(stamp(True), True))
        for value in (None, {}, {**stamp(True), "method_available": False},
                      {**stamp(True), "requested_kernel": 1},
                      {**stamp(True), "actual": {"change_id": "other", "candidate_enabled": True}},
                      {**stamp(True), "actual": {"change_id": "declump-kernel", "candidate_enabled": False}}):
            with self.subTest(value=value):
                self.assertTrue(kernel_stamp_errors(value, True))

    def test_running_polarity_every_frame_and_exact_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            a, b = root / "a", root / "b"
            running_fixture(a, False)
            metadata = running_fixture(b, False)
            reference = compare_world(a, b, ("W0", 40560, 20261010))
            self.assertTrue(reference["valid"], reference["errors"])
            self.assertEqual(reference["scope"], "reference_integrity_only")
            self.assertFalse(compare_world(a, b, ("W0", 40560, 20261010), "declump-kernel")["valid"])
            metadata["perf_candidate_p12"] = stamp(True)
            metadata["candidate_states"] = [stamp(True) for _ in range(600)]
            def check() -> dict:
                (b / "W0.json").write_text(json.dumps(metadata))
                return compare_world(a, b, ("W0", 40560, 20261010), "declump-kernel")
            self.assertTrue(check()["valid"])
            baseline_path = a / "W0.json"
            baseline = json.loads(baseline_path.read_text())
            baseline["perf_candidate_p12"] = stamp(True)
            baseline_path.write_text(json.dumps(baseline))
            self.assertFalse(check()["valid"])
            baseline["perf_candidate_p12"] = stamp(False)
            baseline_path.write_text(json.dumps(baseline))
            metadata["candidate_states"][299] = stamp(False)
            self.assertTrue(any("frame 300" in error for error in check()["errors"]))
            metadata["candidate_states"] = [stamp(True)] * 599
            self.assertFalse(check()["valid"])
            metadata.pop("candidate_states")
            self.assertFalse(check()["valid"])
            metadata["candidate_states"] = [stamp(True)] * 600
            dump_path = b / "W0.jsonl"
            dump = dump_path.read_bytes().replace(b'body.png', b'other.png', 1)
            dump_path.write_bytes(dump)
            metadata["dump_sha256"] = hashlib.sha256(dump).hexdigest()
            self.assertTrue(any("dumps differ" in error for error in check()["errors"]))

    def test_pixels_reference_candidate_missing_and_control(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            folders = [root / name for name in ("a", "ap", "b")]
            for folder in folders:
                folder.mkdir()
                for framing in FRAMINGS:
                    Image.new("RGBA", (2, 2), (1, 2, 3, 255)).save(folder / f"{framing}.png")
                    (folder / f"{framing}.json").write_text(json.dumps({"capture_frame": 240,
                        "state_change_frame": 120, "tick": 100, "seed": 1, "fixed_fps": 60,
                        "perf_candidate_p12": stamp(False)}))
            def check(name: str, candidate: str | None = "declump-kernel") -> dict:
                output = root / name
                with self.assertRaises(typer.Exit):
                    pixel_main(*folders, output, candidate)
                return json.loads((output / "comparison.json").read_text())
            reference = check("reference", None)
            self.assertTrue(reference["passed"])
            self.assertEqual(reference["scope"], "reference_integrity_only")
            self.assertFalse(check("off-off")["passed"])
            for framing in FRAMINGS:
                path = folders[2] / f"{framing}.json"
                metadata = json.loads(path.read_text())
                metadata["perf_candidate_p12"] = stamp(True)
                path.write_text(json.dumps(metadata))
            self.assertTrue(check("on")["passed"])
            path = folders[1] / "P6.json"
            metadata = json.loads(path.read_text())
            metadata.pop("perf_candidate_p12")
            path.write_text(json.dumps(metadata))
            self.assertFalse(check("missing-ap")["passed"])
            metadata["perf_candidate_p12"] = stamp(True)
            path.write_text(json.dumps(metadata))
            self.assertFalse(check("ap-on")["passed"])
            metadata["perf_candidate_p12"] = stamp(False)
            path.write_text(json.dumps(metadata))
            Image.new("RGBA", (2, 2), (1, 2, 3, 254)).save(folders[2] / "P2.png")
            self.assertFalse(check("alpha-change")["passed"])
            Image.new("RGBA", (2, 2), (1, 2, 3, 255)).save(folders[2] / "P2.png")
            Image.new("RGBA", (2, 2), (2, 2, 3, 255)).save(folders[1] / "P1.png")
            self.assertFalse(check("control-change")["passed"])
            path = folders[2] / "P3.json"
            metadata = json.loads(path.read_text())
            metadata["tick"] = 101
            path.write_text(json.dumps(metadata))
            with self.assertRaises(typer.BadParameter):
                pixel_main(*folders, root / "schedule-change", "declump-kernel")


if __name__ == "__main__":
    unittest.main()
