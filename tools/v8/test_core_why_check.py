"""Synthetic adversarial why evidence only; never accesses held-out simulation data."""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import core_why_check as checker
import test_core_score as fixtures


def fixture(protocol: checker.core.SeedProtocol = checker.core.SeedProtocol.ORIGINAL) -> tuple[dict, dict]:
    """Create a complete synthetic 32-world receipt and internally consistent counters."""
    report, receipt = fixtures.fixture(protocol=protocol)
    receipt["why_checker_sha256"] = hashlib.sha256(Path(checker.__file__).read_bytes()).hexdigest()
    for row in report["seeds"]:
        row["mind_coverage"] = {
            "observed_mind_ticks": 69120, "first_choices": 10, "behaviour_switches": 990,
            "node_only_choices": 180, "same_behaviour_restarts": 20,
            "expected_records": 1200, "covered_records": 1200,
            "behaviour_expected": 1000, "behaviour_covered": 1000,
            "node_expected": 200, "node_covered": 200,
            "critical_expected": 10, "critical_covered": 10,
            "critical_choice_expected": 30, "critical_choice_covered": 30,
            "failures": 0, "examples": [], "status": "PASS",
            "coverage": 1.0, "node_coverage": 1.0, "critical_coverage": 1.0,
            "critical_choice_coverage": 1.0, "definition": "synthetic independent counters",
        }
    return report, receipt


def miss(c: dict, *, behaviour: int = 0, node: int = 0, critical_node: int = 0) -> None:
    """Make genuine uncovered transitions without malformed counters."""
    c["behaviour_covered"] -= behaviour
    c["node_covered"] -= node
    c["critical_choice_covered"] -= critical_node
    c["covered_records"] -= behaviour + node
    c["failures"] = behaviour + node
    c["examples"] = [{"tick": i, "agent": 7, "node": 1, "problem": "missing_record"}
                     for i in range(min(c["failures"], 16))]
    c["status"] = "FAIL" if c["failures"] else "PASS"
    c["coverage"] = c["behaviour_covered"] / c["behaviour_expected"]
    c["node_coverage"] = c["node_covered"] / c["node_expected"]
    c["critical_choice_coverage"] = c["critical_choice_covered"] / c["critical_choice_expected"]


class WhyTests(unittest.TestCase):
    def test_exact_threshold_is_integer_aggregate_not_mean_of_world_percentages(self) -> None:
        report, receipt = fixture()
        miss(report["seeds"][0]["mind_coverage"], behaviour=320)
        result = checker.check(report, receipt)
        self.assertEqual(result["status"], "PASS")
        self.assertEqual(result["aggregate"]["behaviour_coverage"], .99)
        report, receipt = fixture()
        miss(report["seeds"][0]["mind_coverage"], behaviour=321)
        self.assertEqual(checker.check(report, receipt)["status"], "FAIL")

    def test_same_behaviour_critical_node_miss_fails_strict_bar(self) -> None:
        report, receipt = fixture()
        miss(report["seeds"][0]["mind_coverage"], node=1, critical_node=1)
        result = checker.check(report, receipt)
        self.assertTrue(result["bars"]["behaviour_at_least_99_percent"])
        self.assertFalse(result["bars"]["all_critical_choices_covered"])
        self.assertEqual(result["status"], "FAIL")

    def test_zero_critical_world_is_na_but_collectively_unexposed_fails(self) -> None:
        report, receipt = fixture()
        for row in report["seeds"]:
            c = row["mind_coverage"]
            for key in ("critical_expected", "critical_covered", "critical_choice_expected", "critical_choice_covered"):
                c[key] = 0
            c["critical_coverage"] = c["critical_choice_coverage"] = None
        result = checker.check(report, receipt)
        self.assertEqual(result["status"], "FAIL")
        self.assertEqual(result["bars"]["critical_exposure"], "N/A")
        self.assertIsNone(result["aggregate"]["critical_choice_coverage"])
        self.assertEqual(result["worlds"][0]["critical_status"], "N/A")
        report["seeds"][1]["mind_coverage"] = fixture()[0]["seeds"][1]["mind_coverage"]
        self.assertEqual(checker.check(report, receipt)["status"], "PASS")

    def test_missing_duplicate_wrong_duration_or_missing_coverage_rejected(self) -> None:
        for mutation in (lambda r: r["seeds"].pop(),
                         lambda r: r["seeds"].append(copy.deepcopy(r["seeds"][0])),
                         lambda r: r["seeds"][0].update(days=47),
                         lambda r: r["seeds"][0].pop("mind_coverage"),
                         lambda r: r["seeds"][0]["mux2"].update(observed_ticks=69119),
                         lambda r: r["seeds"][0]["mux2"]["inputs"].update(source_sha="e" * 40),
                         lambda r: r["seeds"][0]["mux2"]["inputs"].update(defence=False),
                         lambda r: r["seeds"][0]["mux"].update(ticks_per_day=60)):
            report, receipt = fixture()
            mutation(report)
            with self.assertRaises(checker.ScoreError):
                checker.check(report, receipt)

    def test_malformed_counters_ratios_status_and_subsets_rejected(self) -> None:
        for key, value in (("observed_mind_ticks", 69119), ("behaviour_expected", True),
                           ("node_covered", 199.0), ("failures", -1), ("first_choices", 0),
                           ("coverage", float("nan")), ("coverage", .99), ("status", "N/A"),
                           ("critical_choice_expected", 1400), ("critical_covered", 11),
                           ("expected_records", 999), ("examples", [{}]), ("node_only_choices", 2**64)):
            report, receipt = fixture()
            report["seeds"][0]["mind_coverage"][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(checker.ScoreError):
                checker.check(report, receipt)

    def test_receipt_and_exact_bytes_binding(self) -> None:
        report, receipt = fixture()
        with tempfile.TemporaryDirectory() as directory:
            summary, binding = Path(directory) / "summary.json", Path(directory) / "receipt.json"
            summary.write_text(json.dumps(report))
            receipt["summary_sha256"] = hashlib.sha256(summary.read_bytes()).hexdigest()
            binding.write_text(json.dumps(receipt))
            parsed, bound, _ = checker.read_bound(summary, binding)
            self.assertEqual(checker.check(parsed, bound)["status"], "PASS")
            summary.write_text(summary.read_text() + " ")
            with self.assertRaises(checker.ScoreError):
                checker.read_bound(summary, binding)
            receipt["summary_sha256"] = hashlib.sha256(summary.read_bytes()).hexdigest()
            receipt["why_checker_sha256"] = "0" * 64
            binding.write_text(json.dumps(receipt))
            with self.assertRaises(checker.ScoreError):
                checker.read_bound(summary, binding)

    def test_duplicate_json_key_and_nonfinite_rejected_at_boundary(self) -> None:
        report, receipt = fixture()
        with tempfile.TemporaryDirectory() as directory:
            summary, binding = Path(directory) / "summary.json", Path(directory) / "receipt.json"
            for raw in ('{"days":48,"days":48}', '{"days":NaN}'):
                summary.write_text(raw)
                receipt["summary_sha256"] = hashlib.sha256(summary.read_bytes()).hexdigest()
                binding.write_text(json.dumps(receipt))
                with self.assertRaises(ValueError):
                    checker.read_bound(summary, binding)

    def test_base_or_wrong_freeze_receipts_cannot_claim_mind_coverage(self) -> None:
        for key, value in (("brain", "base"), ("round", 1), ("policy_frozen_before_run", False),
                           ("days", 47), ("seeds", list(range(32)))):
            report, receipt = fixture()
            receipt[key] = value
            with self.assertRaises(checker.ScoreError):
                checker.check(report, receipt)


if __name__ == "__main__":
    unittest.main()
