"""Synthetic CORE scoring counterexamples; never reads held-out simulation data."""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_core_score.py
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import core_score as scorer


def fixture(brain: str = "mind", phase: str = "heldout") -> tuple[dict, dict]:
    """Construct synthetic observations directly in the CORE seed domain."""
    seeds = list(range(1232, 1264)) if phase == "heldout" else list(range(8))
    source = scorer.BASE_SHA if brain == "base" else "a" * 40
    receipt = {"brain": brain, "phase": phase, "defence": True, "seeds": seeds,
               "days": 48, "round": None if phase == "heldout" else 1,
               "policy_frozen_before_run": True, "source_sha": source,
               "policy_sha256": "b" * 64, "binary_sha256": "c" * 64,
               "scorer_sha256": hashlib.sha256(Path(scorer.__file__).read_bytes()).hexdigest(),
               "validator_sha256": hashlib.sha256(Path(scorer.__file__).with_name("mux_safety_validate.py").read_bytes()).hexdigest()}
    inputs = {"steward": True, "defence": True, "defence_enabled_end": True,
              "player_script": False, "story": "on", "neighbours": "on",
              "source_sha": source, "policy_sha256": "b" * 64}
    old = {"ticks_per_day": 1440, "stockpile_first_tick": 1440,
           "first_winter_observed": True, "first_winter_complete": True,
           "first_winter_cold_deaths": 0, "cold_snap_observed_ticks": 10,
           "cold_snap_firewood_zero_ticks": 0, "alive_end": 10, "alive_at_day48": True}
    observed = {"schema_version": 1, "start_tick": 0, "end_tick": 69120,
                "observed_ticks": 69120, "observer_integrity_errors": [], "inputs": inputs,
                "walled_beds_for_all_first_tick": 14400, "wolf_deaths": 0,
                "wolf_death_events": [], "fatal_animal_events": 0, "combat_deaths": 0}
    rows = [{"seed": seed, "days": 48, "mux": copy.deepcopy(old),
             "mux2": copy.deepcopy(observed), "alive_end": 10, "deaths": {"combat": 0},
             "ledger_violation_ticks": 0, "reservation_violation_ticks": 0} for seed in seeds]
    return {"scene": "colony", "days": 48, "seeds": rows}, receipt


def failed_row(row: dict, bar: str) -> None:
    """Mutate one synthetic world into a valid failure for the named outcome."""
    match bar:
        case "stockpile_day1": row["mux"]["stockpile_first_tick"] = 1441
        case "walled_beds_day10": row["mux2"]["walled_beds_for_all_first_tick"] = 14401
        case "zero_cold_deaths": row["mux"]["first_winter_cold_deaths"] = 1
        case "alive_day48":
            row["mux"].update(alive_end=0, alive_at_day48=False)
            row["alive_end"] = 0
        case "exact_ledger": row["ledger_violation_ticks"] = 1
        case "wolf_deaths_at_most_one":
            row["mux2"].update(wolf_deaths=2, fatal_animal_events=2, combat_deaths=2,
                wolf_death_events=[{"tick": 0, "victim": i, "attacker": 9,
                                    "kind": 1, "mechanic": "predator"} for i in (1, 2)])
            row["deaths"]["combat"] = 2
        case _: raise AssertionError(bar)


class CoreScoreTests(unittest.TestCase):
    def test_thresholds_match_published_outcomes_minus_one_except_ledger(self) -> None:
        expected = {"stockpile_day1": 31, "walled_beds_day10": 31, "zero_cold_deaths": 31,
                    "alive_day48": 31, "wolf_deaths_at_most_one": 29, "exact_ledger": 32}
        self.assertEqual(scorer.TARGETS, expected)
        report, receipt = fixture()
        result = scorer.score(report, receipt, phase="heldout", brain="mind")
        self.assertEqual({key: result["bars"][key]["required"] for key in expected}, expected)

    def test_fixed_bars_accept_exact_boundary_and_reject_one_more_failure(self) -> None:
        for bar, required in scorer.TARGETS.items():
            with self.subTest(bar=bar):
                report, receipt = fixture()
                for row in report["seeds"][:32-required]:
                    failed_row(row, bar)
                result = scorer.score(report, receipt, phase="heldout", brain="mind")
                self.assertTrue(result["passed"])
                self.assertEqual(result["bars"][bar]["passed"], required)
                failed_row(report["seeds"][32-required], bar)
                self.assertFalse(scorer.score(report, receipt, phase="heldout", brain="mind")["passed"])

    def test_cold_denominator_excludes_unexposed_worlds_and_allows_one_loss(self) -> None:
        report, receipt = fixture()
        for row in report["seeds"][20:]:
            row["mux"]["cold_snap_observed_ticks"] = 0
        report["seeds"][0]["mux"]["cold_snap_firewood_zero_ticks"] = 1
        result = scorer.score(report, receipt, phase="heldout", brain="mind")
        self.assertEqual(result["bars"]["firewood_during_snap"],
                         {"passed": 19, "required": 19, "total": 20, "unexposed": 12, "status": "PASS"})
        self.assertIsNone(result["seeds"][20]["bars"]["firewood_during_snap"])
        report["seeds"][1]["mux"]["cold_snap_firewood_zero_ticks"] = 1
        self.assertFalse(scorer.score(report, receipt, phase="heldout", brain="mind")["passed"])

    def test_zero_exposure_is_neither_success_nor_failure(self) -> None:
        report, receipt = fixture()
        for row in report["seeds"]:
            row["mux"]["cold_snap_observed_ticks"] = 0
        bar = scorer.score(report, receipt, phase="heldout", brain="mind")["bars"]["firewood_during_snap"]
        self.assertEqual(bar, {"passed": 0, "required": 0, "total": 0, "unexposed": 32, "status": "NOT_EXPOSED"})

    def test_rejects_malformed_coverage_observer_inputs_and_provenance(self) -> None:
        changes = [lambda r, m: r["seeds"].pop(), lambda r, m: r["seeds"][0].update(seed=1233),
                   lambda r, m: r["seeds"][0].update(seed=1200),
                   lambda r, m: r.update(days=True), lambda r, m: m.update(policy_frozen_before_run=False),
                   lambda r, m: m.update(defence=False), lambda r, m: m.pop("binary_sha256"),
                   lambda r, m: r["seeds"][0]["mux2"].update(observed_ticks=69119),
                   lambda r, m: r["seeds"][0]["mux2"].update(wolf_deaths=1),
                   lambda r, m: r["seeds"][0]["mux2"].update(observer_integrity_errors=["bad"]),
                   lambda r, m: r["seeds"][0]["mux2"]["inputs"].update(player_script=True),
                   lambda r, m: r["seeds"][0]["mux2"]["inputs"].update(source_sha="d"*40),
                   lambda r, m: r["seeds"][0]["mux"].update(cold_snap_firewood_zero_ticks=11),
                   lambda r, m: r["seeds"][0]["mux"].update(ticks_per_day=True)]
        for index, change in enumerate(changes):
            with self.subTest(index=index):
                report, receipt = fixture()
                change(report, receipt)
                with self.assertRaises(ValueError):
                    scorer.score(report, receipt, phase="heldout", brain="mind")
        for mutation in ({"victim": 1}, {"tick": 69120}, {"mechanic": "other"}):
            report, receipt = fixture()
            failed_row(report["seeds"][0], "wolf_deaths_at_most_one")
            report["seeds"][0]["mux2"]["wolf_death_events"][1].update(mutation)
            with self.assertRaises(ValueError):
                scorer.score(report, receipt, phase="heldout", brain="mind")

    def test_calibration_round_limits_and_no_acceptance_claim(self) -> None:
        report, receipt = fixture(phase="calibration")
        for round_number in range(1, 5):
            receipt["round"] = round_number
            self.assertFalse(scorer.score(report, receipt, phase="calibration", brain="mind")["acceptance_claim"])
        receipt["round"] = 5
        with self.assertRaises(ValueError):
            scorer.score(report, receipt, phase="calibration", brain="mind")

    def test_byte_reader_rejects_ambiguous_json_and_unbound_scorer(self) -> None:
        with tempfile.TemporaryDirectory(prefix="core-score-synthetic-") as directory:
            root = Path(directory)
            report, receipt = fixture()
            for raw in (b'{"scene":"colony","scene":"colony"}', b'{"scene":NaN}'):
                receipt["summary_sha256"] = hashlib.sha256(raw).hexdigest()
                (root / "run.json").write_bytes(raw)
                (root / "receipt.json").write_text(json.dumps(receipt))
                with self.assertRaises(ValueError):
                    scorer.read_bound_run(root / "run.json", root / "receipt.json")
            raw = json.dumps(report).encode()
            receipt["summary_sha256"] = hashlib.sha256(raw).hexdigest()
            receipt["scorer_sha256"] = "0" * 64
            (root / "run.json").write_bytes(raw)
            (root / "receipt.json").write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, "bind this scorer"):
                scorer.read_bound_run(root / "run.json", root / "receipt.json")

    def test_validator_hash_mismatch_and_changed_imported_file_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="core-score-synthetic-") as directory:
            root = Path(directory)
            report, receipt = fixture()
            raw = json.dumps(report).encode()
            receipt["summary_sha256"] = hashlib.sha256(raw).hexdigest()
            (root / "run.json").write_bytes(raw)
            valid_hash = receipt["validator_sha256"]
            receipt["validator_sha256"] = "0" * 64
            (root / "receipt.json").write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, "bind this validator"):
                scorer.read_bound_run(root / "run.json", root / "receipt.json")
            receipt["validator_sha256"] = valid_hash
            (root / "receipt.json").write_text(json.dumps(receipt))
            copied = root / "isolated" / "tools" / "v8"
            copied.mkdir(parents=True)
            source = Path(scorer.__file__)
            (copied / "core_score.py").write_bytes(source.read_bytes())
            helper = source.with_name("mux_safety_validate.py").read_bytes()
            (copied / "mux_safety_validate.py").write_bytes(helper + b"\n# Synthetic dependency mutation.\n")
            output = root / "must-not-exist.json"
            run = subprocess.run([sys.executable, str(copied / "core_score.py"), str(root / "run.json"),
                                  "--receipt", str(root / "receipt.json"), "--phase", "heldout",
                                  "--output", str(output)], capture_output=True, text=True, check=False)
            self.assertEqual(run.returncode, 2)
            self.assertIn("bind this validator", run.stderr)
            self.assertFalse(output.exists())

    def test_pair_allows_different_sources_but_requires_exact_base_and_same_calendar(self) -> None:
        new, nr = fixture()
        old, br = fixture("base")
        failed_row(old["seeds"][0], "exact_ledger")
        result = scorer.score_pair(new, nr, old, br)
        self.assertTrue(result["evaluation_bars_passed"])
        self.assertFalse(result["arms"]["base"]["passed"])
        self.assertFalse(result["acceptance_claim"])
        br["source_sha"] = "d" * 40
        with self.assertRaises(ValueError): scorer.score_pair(new, nr, old, br)
        old, br = fixture("base")
        for row in old["seeds"]:
            row["mux"]["ticks_per_day"] = 720
            row["mux2"].update(end_tick=34560, observed_ticks=34560)
        with self.assertRaisesRegex(ValueError, "calendar"):
            scorer.score_pair(new, nr, old, br)

    def test_real_cli_binds_raw_bytes_and_rejects_missing_or_corrupt_receipts(self) -> None:
        with tempfile.TemporaryDirectory(prefix="core-score-synthetic-") as directory:
            root = Path(directory)
            for brain in ("mind", "base"):
                report, receipt = fixture(brain)
                raw = json.dumps(report).encode()
                receipt["summary_sha256"] = hashlib.sha256(raw).hexdigest()
                (root / f"{brain}.json").write_bytes(raw)
                (root / f"{brain}-receipt.json").write_text(json.dumps(receipt))
            args = [sys.executable, scorer.__file__, str(root / "mind.json"), "--receipt",
                    str(root / "mind-receipt.json"), "--phase", "heldout", "--base-summary",
                    str(root / "base.json"), "--base-receipt", str(root / "base-receipt.json")]
            out = root / "score.json"
            run = subprocess.run([*args, "--output", str(out)], capture_output=True, text=True, check=False)
            self.assertEqual(run.returncode, 0, run.stderr)
            result = json.loads(out.read_text())
            self.assertEqual(result["worlds"], 64)
            self.assertEqual(set(result["input_hashes"]), {"mind", "base"})
            report, receipt = fixture()
            failed_row(report["seeds"][0], "exact_ledger")
            raw = json.dumps(report).encode()
            receipt["summary_sha256"] = hashlib.sha256(raw).hexdigest()
            (root / "mind.json").write_bytes(raw)
            (root / "mind-receipt.json").write_text(json.dumps(receipt))
            failed = root / "failed.json"
            run = subprocess.run([*args, "--output", str(failed)], capture_output=True, text=True, check=False)
            self.assertEqual(run.returncode, 1)
            self.assertFalse(json.loads(failed.read_text())["evaluation_bars_passed"])
            (root / "base.json").write_text((root / "base.json").read_text() + " ")
            bad = root / "bad.json"
            run = subprocess.run([*args, "--output", str(bad)], capture_output=True, text=True, check=False)
            self.assertEqual(run.returncode, 2)
            self.assertFalse(bad.exists())
            (root / "base-receipt.json").unlink()
            run = subprocess.run([*args, "--output", str(bad)], capture_output=True, text=True, check=False)
            self.assertEqual(run.returncode, 2)
            self.assertFalse(bad.exists())


if __name__ == "__main__":
    unittest.main()
