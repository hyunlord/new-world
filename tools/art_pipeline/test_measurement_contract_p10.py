#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest discover -s tools/art_pipeline -p test_measurement_contract_p10.py
"""Synthetic provenance/resume checks; no Godot, builds or actual lock acquisition."""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import summarize_perf_p10 as perf
from test_summarize_perf_p9 import fixture

MERGE = "0caabf5fd1c62b9d3c99845daab1805e79799db7"
SOURCE = Path(__file__).resolve().parent


class MeasurementContractTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="p10-measurement-contract-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.tools = self.root / "tools/art_pipeline"
        self.tools.mkdir(parents=True)
        for name in ("run_perf_matrix_p10.sh", "summarize_perf_p10.py"):
            shutil.copy2(SOURCE / name, self.tools / name)
        self.collector = self.root / "scripts/test/art/perf_frames_p10.gd"
        self.collector.parent.mkdir(parents=True)
        self.collector.write_text("synthetic collector\n")
        self.library = self.root / "rust/target/debug/libsim_bridge.dylib"
        self.library.parent.mkdir(parents=True)
        self.library.write_bytes(b"synthetic library")
        self.evidence = self.root / "evidence"
        self.evidence.mkdir()
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        fake_git = fake_bin / "git"
        fake_git.write_text('#!/usr/bin/env bash\nprintf "%s\\n" "${MOCK_HEAD:-' + MERGE + '}"\n')
        fake_git.chmod(0o755)
        (self.tools / "with_quiet_lock_p10.sh").write_text('#!/usr/bin/env bash\necho "synthetic launch rejected" >&2\nexit 97\n')
        self.env = dict(os.environ, PATH=str(fake_bin) + os.pathsep + os.environ["PATH"])
        self.env.pop("P10_DEADLINE_EPOCH", None)
        for world in ("W0", "W500", "W1", "W200"):
            self.path(world).write_text(json.dumps(self.payload(world)))
            self.path(world, "log").write_text("synthetic completed\n")
            self.path(world, "exit").write_text("0\n")

    def payload(self, world: str, phase: str = "baseline") -> dict[str, perf.Json]:
        data = fixture(world, phase)
        data.update({"checkout": str(self.root), "build_commit": MERGE,
                     "build_sha256": hashlib.sha256(self.library.read_bytes()).hexdigest(),
                     "collector_sha256": hashlib.sha256(self.collector.read_bytes()).hexdigest()})
        return data

    def path(self, world: str, suffix: str = "json") -> Path:
        return self.evidence / f"baseline-{world}-attempt1.{suffix}"

    def run_matrix(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["bash", str(self.tools / "run_perf_matrix_p10.sh"), "baseline", str(self.root), str(self.evidence)], env=self.env, capture_output=True, text=True, timeout=30, check=False)

    def load(self, data: dict[str, perf.Json]) -> perf.Session:
        path = self.root / "session.json"
        path.write_text(json.dumps(data))
        return perf.load_session(path)

    def test_complete_resumes_require_matching_provenance_and_terminal_exit(self) -> None:
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.count("Resuming with accepted evidence"), 4)

    def test_missing_exit_rejected_and_original_preserved(self) -> None:
        self.path("W0", "exit").unlink()
        original = self.path("W0").read_bytes()
        result = self.run_matrix()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Rejected missing/nonzero session status", result.stderr)
        self.assertEqual(self.path("W0").read_bytes(), original)

    def test_wrong_baseline_checkout_rejected_before_launch(self) -> None:
        self.env["MOCK_HEAD"] = "1" * 40
        result = self.run_matrix()
        self.assertEqual(result.returncode, 2)
        self.assertIn("Baseline requires the Part 0 merge", result.stderr)

    def test_resume_rejects_changed_library_or_collector(self) -> None:
        for target, field in ((self.library, "build_sha256"), (self.collector, "collector_sha256")):
            with self.subTest(field=field):
                original = target.read_bytes()
                target.write_bytes(original + b"changed")
                result = self.run_matrix()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(f"mismatched {field}", result.stderr)
                target.write_bytes(original)

    def test_missing_head_hash_and_checkout_provenance_rejected(self) -> None:
        for field in ("build_commit", "build_sha256", "collector_sha256", "checkout"):
            with self.subTest(field=field):
                data = self.payload("W1")
                del data[field]
                with self.assertRaises(perf.EvidenceError):
                    self.load(data)

    def test_ablation_cannot_be_dropped_in_either_phase(self) -> None:
        for world in ("W0", "W500"):
            for phase in ("baseline", "after"):
                data = self.payload(world, phase)
                data["ablation_dropped"] = True
                with self.assertRaisesRegex(perf.EvidenceError, "cannot be dropped"):
                    self.load(data)

    def test_optional_w1000_requires_only_after_frozen_z4(self) -> None:
        data = self.payload("W1000", "after")
        with self.assertRaisesRegex(perf.EvidenceError, "only three frozen Z4"):
            self.load(data)
        data["blocks"] = list(perf.sequence(data["blocks"]))[:3]
        self.assertEqual(self.load(data).world, "W1000")
        data["phase"] = "baseline"
        with self.assertRaisesRegex(perf.EvidenceError, "after-only"):
            self.load(data)


if __name__ == "__main__":
    unittest.main()
