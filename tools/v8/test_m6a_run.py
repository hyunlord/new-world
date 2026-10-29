"""Round register validation without launching any cohort."""
import argparse
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from types import SimpleNamespace

from m6a_evidence import EvidenceError
from m6a_run import cohort, probe_environment, register_rows, round_start, seeds


class RegisterTests(unittest.TestCase):
    def test_conflicting_environment_rejected_before_any_round_or_process(self):
        for key in ("WORLDSIM_CONTENT_PACK", "WORLDSIM_MIND_HISTORY_SATURATE",
                    "WORLDSIM_MIND_CONTINUATION_DIAGNOSTICS", "WORLDSIM_MIND_GUARD_CACHE_DIAGNOSTIC",
                    "WORLDSIM_MUX2_PROFILE", "WORLDSIM_POLICY_OVERRIDE"):
            for value in ("1", ""):
                with self.subTest(key=key, value=value), tempfile.TemporaryDirectory() as folder:
                    out = Path(folder) / "round1"
                    register = Path(folder) / "register.jsonl"
                    args = argparse.Namespace(out=out, calibration_register=register)
                    with patch.dict(os.environ, {key: value}, clear=True), \
                         patch("m6a_run.subprocess.run") as run, \
                         patch("m6a_run.round_start") as start, \
                         patch("m6a_run.source_hash") as source:
                        with self.assertRaises(EvidenceError):
                            cohort(args)
                        run.assert_not_called()
                        start.assert_not_called()
                        source.assert_not_called()
                    self.assertFalse(out.exists())
                    self.assertFalse(register.exists())

    def test_admitted_environment_is_copied_and_preserves_git_provenance(self):
        host = {"PATH": "/bin", "WORLDSIM_GIT_SHA": "source-head"}
        admitted = probe_environment(host)
        host["WORLDSIM_CONTENT_PACK"] = "/external/changed.ron"
        self.assertEqual(admitted, {"PATH": "/bin", "WORLDSIM_GIT_SHA": "source-head"})

    def test_legacy_fixture_locators_are_not_passed_to_the_probe(self):
        host = {"WORLDSIM_MUX_FIXTURES": "/external/stage-b",
                "WORLDSIM_B5_LEGACY9": "/external/format9.bin", "WORLDSIM_GIT_SHA": "head"}
        self.assertEqual(probe_environment(host), {"WORLDSIM_GIT_SHA": "head"})
        host["WORLDSIM_CONTENT_PACK"] = "/external/pack.ron"
        with self.assertRaises(EvidenceError):
            probe_environment(host)

    def test_gate_metadata_does_not_admit_selected_content_or_diagnostics(self):
        gate = {"WORLDSIM_MUX_FIXTURES": "/external/stage-b", "WORLDSIM_B5_LEGACY9": "/external/old.bin",
                "WORLDSIM_TEST_SETTINGS_DIR": "/external/settings", "WORLDSIM_MUX2_LANE_FD": "9",
                "WORLDSIM_GIT_SHA": "head"}
        self.assertEqual(probe_environment(gate), {"WORLDSIM_GIT_SHA": "head"})
        for key in ("WORLDSIM_CONTENT_PACK", "WORLDSIM_MIND_HISTORY_SATURATE", "WORLDSIM_MUX2_PROFILE"):
            with self.subTest(key=key), self.assertRaises(EvidenceError):
                probe_environment(dict(gate, **{key: "1"}))

    def test_cohort_launch_passes_the_admitted_environment_explicitly(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            root = base / "source"
            root.mkdir()
            binary = base / "probe"
            binary.write_text("mock binary; never executed")
            args = argparse.Namespace(source_root=root, source_head="head", binary=binary,
                seeds="0..2", days=72, purpose="fixture", arm="candidate", out=base / "out")
            expected = {"PATH": "/bin", "WORLDSIM_GIT_SHA": "head"}
            gate_environment = dict(expected, WORLDSIM_MUX_FIXTURES="/external/stage-b",
                WORLDSIM_B5_LEGACY9="/external/format9.bin",
                WORLDSIM_TEST_SETTINGS_DIR="/external/gate/settings", WORLDSIM_MUX2_LANE_FD="9")
            with patch.dict(os.environ, gate_environment, clear=True), \
                 patch("m6a_run.source_hash", return_value="source-seal"), \
                 patch("m6a_run.subprocess.run", return_value=SimpleNamespace(returncode=0)) as run:
                self.assertEqual(cohort(args), 0)
                self.assertEqual(dict(os.environ), gate_environment, "parent ownership metadata stays intact")
                self.assertEqual(run.call_count, 2)
                for call in run.call_args_list:
                    self.assertEqual(call.kwargs["env"], expected)

    def test_seed_ranges_are_half_open(self):
        self.assertEqual(seeds("0..2"), [0, 1])
        self.assertEqual(seeds("1296..1328"), list(range(1296, 1328)))
        with self.assertRaises(EvidenceError):
            seeds("0..33")

    def test_unfinished_round_cannot_restart(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "register.jsonl"
            self.assertEqual(round_start(path, Path(folder) / "round1"), 1)
            with self.assertRaises(EvidenceError):
                round_start(path, Path(folder) / "round2")

    def test_duplicate_or_orphan_completion_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "register.jsonl"
            for text in ('{"event":"finished","round":1}\n',
                         '{"event":"started","round":1}\n{"event":"finished","round":2}\n'):
                path.write_text(text)
                with self.assertRaises(EvidenceError):
                    register_rows(path)


if __name__ == "__main__":
    unittest.main()
