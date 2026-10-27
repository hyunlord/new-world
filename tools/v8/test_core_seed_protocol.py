"""Synthetic protocol separation; never reads either held-out cohort's results."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import core_score as core
import core_story_check as story
import core_why_check as why
import test_core_score as scores
import test_core_story_check as stories
import test_core_why_check as whys

FRESH = core.SeedProtocol.P3D
OLD = core.SeedProtocol.ORIGINAL


class ProtocolTests(unittest.TestCase):
    def test_protocols_are_fixed_and_seeds_cannot_mutate_global_state(self) -> None:
        self.assertEqual(OLD.seeds, list(range(1232, 1264)))
        self.assertEqual(FRESH.seeds, list(range(1264, 1296)))
        changed = FRESH.seeds
        changed[0] = 1232
        self.assertEqual(FRESH.seeds[0], 1264)
        with self.assertRaises(ValueError):
            core.SeedProtocol("arbitrary")

    def test_fresh_output_identity_and_original_schema_stay_separate(self) -> None:
        for protocol in (OLD, FRESH):
            report, receipt = scores.fixture(protocol=protocol)
            base, br = scores.fixture("base", protocol=protocol)
            outputs = [core.score_pair(report, receipt, base, br, protocol=protocol),
                       why.check(*whys.fixture(protocol), protocol=protocol),
                       story.check_pair(*stories.fixture(protocol), protocol=protocol)]
            for output in outputs:
                self.assertEqual(output["schema_version"], 1 if protocol is OLD else 2)
                self.assertEqual(output.get("seed_protocol"), None if protocol is OLD else FRESH.value)
                self.assertFalse(output["acceptance_claim"])
            self.assertEqual(outputs[0]["seeds"], protocol.seeds)
            self.assertEqual(outputs[2]["base"]["world_years"], 64)

    def test_fresh_outcome_targets_cold_denominators_and_failure_boundary(self) -> None:
        expected = {"stockpile_day1": 31, "walled_beds_day10": 31, "zero_cold_deaths": 31,
                    "alive_day48": 31, "wolf_deaths_at_most_one": 29, "exact_ledger": 32}
        for bar, required in expected.items():
            report, receipt = scores.fixture(protocol=FRESH)
            for row in report["seeds"][:32-required]:
                scores.failed_row(row, bar)
            result = core.score(report, receipt, phase="heldout", brain="mind", protocol=FRESH)
            self.assertTrue(result["passed"])
            self.assertEqual(result["bars"][bar]["required"], required)
            scores.failed_row(report["seeds"][32-required], bar)
            self.assertFalse(core.score(report, receipt, phase="heldout", brain="mind", protocol=FRESH)["passed"])
        report, receipt = scores.fixture(protocol=FRESH)
        for row in report["seeds"][20:]:
            row["mux"]["cold_snap_observed_ticks"] = 0
        report["seeds"][0]["mux"]["cold_snap_firewood_zero_ticks"] = 1
        self.assertEqual(core.score(report, receipt, phase="heldout", brain="mind", protocol=FRESH)["bars"]["firewood_during_snap"],
                         {"passed": 19, "required": 19, "total": 20, "unexposed": 12, "status": "PASS"})
        for row in report["seeds"]:
            row["mux"].update(cold_snap_observed_ticks=0, cold_snap_firewood_zero_ticks=0)
        self.assertEqual(core.score(report, receipt, phase="heldout", brain="mind", protocol=FRESH)["bars"]["firewood_during_snap"]["status"], "NOT_EXPOSED")

    def test_fresh_why_keeps_independent_thresholds_and_critical_exposure(self) -> None:
        for failures, status in [(320, "PASS"), (321, "FAIL")]:
            report, receipt = whys.fixture(FRESH)
            whys.miss(report["seeds"][0]["mind_coverage"], behaviour=failures)
            self.assertEqual(why.check(report, receipt, protocol=FRESH)["status"], status)
        report, receipt = whys.fixture(FRESH)
        whys.miss(report["seeds"][0]["mind_coverage"], node=1, critical_node=1)
        self.assertEqual(why.check(report, receipt, protocol=FRESH)["status"], "FAIL")
        report, receipt = whys.fixture(FRESH)
        for row in report["seeds"]:
            c = row["mind_coverage"]
            for name in ("critical_expected", "critical_covered", "critical_choice_expected", "critical_choice_covered"):
                c[name] = 0
            c["critical_coverage"] = c["critical_choice_coverage"] = None
        self.assertEqual(why.check(report, receipt, protocol=FRESH)["status"], "FAIL")

    def test_missing_wrong_null_or_unknown_protocol_tags_fail_all_checkers(self) -> None:
        for value in (None, OLD.value, "unknown", True, [FRESH.value]):
            for missing in (False, True):
                report, receipt = scores.fixture(protocol=FRESH)
                wr, wm = whys.fixture(FRESH)
                args = stories.fixture(FRESH)
                for target in (receipt, wm, args[1], args[3]):
                    if missing:
                        target.pop("seed_protocol")
                    else:
                        target["seed_protocol"] = value
                for call in (lambda: core.score(report, receipt, phase="heldout", brain="mind", protocol=FRESH),
                             lambda: why.check(wr, wm, protocol=FRESH),
                             lambda: story.check_pair(*args, protocol=FRESH)):
                    with self.assertRaises(core.ScoreError):
                        call()
        for value in (None, 1, True, 2.0):
            receipt = scores.fixture(protocol=FRESH)[1]
            receipt["schema_version"] = value
            with self.assertRaises(core.ScoreError):
                core.check_protocol(receipt, FRESH)

    def test_crossed_mixed_missing_duplicate_sets_and_arms_fail(self) -> None:
        for index, replacement in [(0, 1232), (31, 1294), (0, 1296)]:
            report, receipt = scores.fixture(protocol=FRESH)
            report["seeds"][index]["seed"] = replacement
            with self.assertRaises(core.ScoreError):
                core.score(report, receipt, phase="heldout", brain="mind", protocol=FRESH)
            report, receipt = whys.fixture(FRESH)
            report["seeds"][index]["seed"] = replacement
            with self.assertRaises(core.ScoreError):
                why.check(report, receipt, protocol=FRESH)
            args = stories.fixture(FRESH)
            args[2]["worlds"][index]["seed"] = replacement
            with self.assertRaises(core.ScoreError):
                story.check_pair(*args, protocol=FRESH)
        report, receipt = scores.fixture(protocol=FRESH)
        base, br = scores.fixture("base")
        with self.assertRaises(core.ScoreError):
            core.score_pair(report, receipt, base, br, protocol=FRESH)
        args = stories.fixture(FRESH)
        old = stories.fixture()
        with self.assertRaises(core.ScoreError):
            story.check_pair(old[0], old[1], args[2], args[3], old[4], protocol=FRESH)
        for report, receipt, call in [(*scores.fixture(protocol=FRESH), lambda r,m: core.score(r,m,phase="heldout",brain="mind")),
                                      (*whys.fixture(FRESH), why.check)]:
            with self.assertRaises(core.ScoreError):
                call(report, receipt)
        with self.assertRaises(core.ScoreError):
            story.check_pair(*stories.fixture(FRESH))
        report, receipt = scores.fixture(phase="calibration", protocol=FRESH)
        with self.assertRaisesRegex(core.ScoreError, "heldout-only"):
            core.score(report, receipt, phase="calibration", brain="mind", protocol=FRESH)

    def test_fresh_story_dependency_binding_is_mandatory(self) -> None:
        report, receipt, _, _, _ = stories.fixture(FRESH)
        with tempfile.TemporaryDirectory(prefix="p3d-synthetic-") as directory:
            root = Path(directory)
            raw, binding = root / "report.json", root / "receipt.json"
            raw.write_text(json.dumps(report))
            receipt["report_sha256"] = story.sha(raw)
            for name in ("scorer_sha256", "validator_sha256"):
                bad = copy.deepcopy(receipt)
                bad[name] = "0" * 64
                binding.write_text(json.dumps(bad))
                with self.assertRaises(core.ScoreError):
                    story.bound(raw, binding, protocol=FRESH)
            binding.write_text(json.dumps(receipt))
            story.bound(raw, binding, protocol=FRESH)

    def test_fresh_cli_requires_flag_and_binds_summary_bytes(self) -> None:
        for module, factory, positional in [(core, scores.fixture, True), (why, whys.fixture, False)]:
            with tempfile.TemporaryDirectory(prefix="p3d-synthetic-") as directory:
                root = Path(directory)
                report, receipt = factory(protocol=FRESH)
                raw, binding = root / "summary.json", root / "receipt.json"
                raw.write_text(json.dumps(report))
                receipt["summary_sha256"] = story.sha(raw)
                binding.write_text(json.dumps(receipt))
                command = [sys.executable, module.__file__, *([str(raw), "--phase", "heldout"] if positional else ["--summary", str(raw)]), "--receipt", str(binding)]
                for suffix, flags, code in [("default", [], 2), ("fresh", ["--seed-protocol", FRESH.value], 0), ("unknown", ["--seed-protocol", "unknown"], 2)]:
                    output = root / (suffix + ".json")
                    run = subprocess.run([*command, *flags, "--output", str(output)], capture_output=True, text=True, check=False)
                    self.assertEqual(run.returncode, code, run.stderr)
                    self.assertEqual(output.exists(), code == 0)
                    if code == 0:
                        self.assertEqual(json.loads(output.read_text())["seed_protocol"], FRESH.value)
                raw.write_text(raw.read_text() + " ")
                run = subprocess.run([*command, "--seed-protocol", FRESH.value, "--output", str(root / "tampered.json")], capture_output=True, text=True, check=False)
                self.assertEqual(run.returncode, 2)
                self.assertFalse((root / "tampered.json").exists())


if __name__ == "__main__":
    unittest.main()
