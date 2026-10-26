"""Pure synthetic story-yield failures; no real held-out reports are read."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import core_story_check as checker


def fixture() -> tuple[dict, dict, dict, dict, dict]:
    """Supply synthetic valid reports and independently labeled arm receipts."""
    source = "a" * 40
    inputs = copy.deepcopy(checker.INPUTS)
    definition = copy.deepcopy(checker.DEFINITION)
    worlds = [{"seed": seed, "end_tick": 69120, "ticks_per_year": 34560,
               "fingerprint": "0" * 16 + "-" + "1" * 16,
               "years": [{"year": year, "observed_ticks": 34560, "complete_year": True,
                          "entries": 10, "deep_entries": 3, "distinct_deep_chains": 2,
                          "people_with_three_entries": 2, "missing_chains": 1, "cyclic_chains": 0}
                         for year in [0, 1]]} for seed in checker.SEEDS]
    base = {"schema_version": 2, "scene": "colony", "days": 48, "source_sha": source,
            "inputs": inputs, "definition": definition, "worlds": worlds}
    mind = copy.deepcopy(base)
    mind["source_sha"] = "b" * 40
    receipt = {"schema_version": 1, "role": "base_instrument", "source_sha": source,
               "runtime_base_sha": checker.BASE, "policy_sha256": None,
               "binary_sha256": "c" * 64, "report_sha256": "d" * 64,
               "metric_sha256": checker.sha(checker.ROOT / "rust/crates/sim-test/src/story_yield/metric.rs"),
               "cli_sha256": checker.sha(checker.ROOT / "rust/crates/sim-test/src/story_yield/cli.rs"),
               "checker_sha256": checker.sha(Path(checker.__file__)),
               "base_instrument_provenance_sha256": "e" * 64,
               "seeds": checker.SEEDS, "days": 48, "inputs": copy.deepcopy(inputs)}
    mind_receipt = copy.deepcopy(receipt)
    mind_receipt.update(role="mind", source_sha=mind["source_sha"], runtime_base_sha=None,
                        policy_sha256="f" * 64, base_instrument_provenance_sha256=None)
    provenance = {"schema_version": 1, "runtime_base_sha": checker.BASE, "source_sha": source,
                  "binary_sha256": receipt["binary_sha256"], "checkout_vs_base_only_docs": True,
                  "source_manifest": {"rows": 506, "git_blob_matches": 505,
                                      "git_blob_missing": ["rust/Cargo.lock"], "git_blob_mismatches": [],
                                      "untracked_lock_matches_manifest": True},
                  "safety": {"binary_matches_frozen": True, "comparison_log_reports_pass": True},
                  "steward_on": {"summary_byte_identical": True, "determinism_log_pass": True,
                                 "days": 24, "seeds": [0, 1]}}
    return base, receipt, mind, mind_receipt, provenance


class StoryTests(unittest.TestCase):
    def test_measures_two_years_per_world_without_numerical_bar(self) -> None:
        result = checker.check_pair(*fixture())
        self.assertEqual(result["status"], "MEASURED")
        self.assertIsNone(result["numerical_bar"])
        self.assertEqual(result["base"]["world_years"], 64)
        self.assertEqual(result["base"]["totals"]["deep_entries"], 192)
        self.assertEqual(result["base"]["per_world_year"]["deep_entries"], 3)
        self.assertFalse(result["acceptance_claim"])

    def test_zero_yield_is_valid_measurement_not_missing_evidence(self) -> None:
        args = fixture()
        for report in [args[0], args[2]]:
            for world in report["worlds"]:
                for year in world["years"]:
                    year.update(dict.fromkeys(checker.COUNTS, 0))
        self.assertEqual(checker.check_pair(*args)["status"], "MEASURED")

    def test_omitted_duplicate_world_year_and_wrong_windows_fail(self) -> None:
        mutations = [lambda r: r["worlds"].pop(),
                     lambda r: r["worlds"].append(copy.deepcopy(r["worlds"][0])),
                     lambda r: r["worlds"][0]["years"].pop(),
                     lambda r: r["worlds"][0]["years"][1].update(year=0),
                     lambda r: r["worlds"][0].update(end_tick=69119),
                     lambda r: r["worlds"][0].update(ticks_per_year=40320),
                     lambda r: r["worlds"][0]["years"][0].update(observed_ticks=34559),
                     lambda r: r["worlds"][0]["years"][0].update(complete_year=False)]
        for change in mutations:
            args = fixture()
            change(args[2])
            with self.assertRaises(checker.ScoreError):
                checker.check_pair(*args)

    def test_counts_and_definition_cannot_be_fabricated(self) -> None:
        for key, value in [("entries", True), ("entries", -1), ("entries", 10.0),
                           ("deep_entries", 11), ("distinct_deep_chains", 4),
                           ("people_with_three_entries", 11), ("cyclic_chains", 9),
                           ("missing_chains", 11), ("entries", 2**64), ("extra", 0)]:
            args = fixture()
            args[2]["worlds"][0]["years"][0][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(checker.ScoreError):
                checker.check_pair(*args)
        args = fixture()
        args[0]["definition"]["minimum_depth"] = 2
        with self.assertRaises(checker.ScoreError):
            checker.check_pair(*args)

    def test_unknown_inputs_wrong_sources_and_incomplete_provenance_fail(self) -> None:
        for mutation in (lambda a: a[2]["inputs"].update(unused="ignored"),
                         lambda a: a[2]["inputs"].update(steward=1),
                         lambda a: a[2].update(source_sha="e" * 40),
                         lambda a: a[3].update(cli_sha256="0" * 64),
                         lambda a: a[1].update(runtime_base_sha="0" * 40),
                         lambda a: a[4]["source_manifest"].update(git_blob_matches=504),
                         lambda a: a[4]["safety"].update(binary_matches_frozen=False),
                         lambda a: a[4]["steward_on"].update(seeds=[False, True])):
            args = fixture()
            mutation(args)
            with self.assertRaises(checker.ScoreError):
                checker.check_pair(*args)

    def test_bound_bytes_checker_and_instrument_changes_fail(self) -> None:
        report, receipt, _, _, _ = fixture()
        with tempfile.TemporaryDirectory() as directory:
            output, binding = Path(directory) / "report.json", Path(directory) / "receipt.json"
            output.write_text(json.dumps(report))
            receipt["report_sha256"] = checker.sha(output)
            binding.write_text(json.dumps(receipt))
            checker.bound(output, binding)
            output.write_text(output.read_text() + " ")
            with self.assertRaises(checker.ScoreError):
                checker.bound(output, binding)
            receipt["report_sha256"] = checker.sha(output)
            for field in ["checker_sha256", "metric_sha256", "cli_sha256"]:
                altered = copy.deepcopy(receipt)
                altered[field] = "0" * 64
                binding.write_text(json.dumps(altered))
                with self.assertRaises(checker.ScoreError):
                    checker.bound(output, binding)

    def test_cli_checks_complete_provenance_and_rejects_changed_artifact(self) -> None:
        base, br, mind, mr, provenance = fixture()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifact = root / "synthetic-artifact"
            artifact.write_text("synthetic fixture, never a real binary")
            artifact_hash = checker.sha(artifact)
            provenance["binary_sha256"] = br["binary_sha256"] = artifact_hash
            provenance["artifacts"] = {name: {"path": str(artifact), "sha256": artifact_hash}
                                       for name in checker.ARTIFACTS}
            provenance["instrument"] = {name: {"sha256": checker.sha(checker.ROOT / name)}
                                        for name in checker.INSTRUMENT}
            proof = root / "provenance.json"
            proof.write_text(json.dumps(provenance))
            br["base_instrument_provenance_sha256"] = checker.sha(proof)
            paths = []
            for name, report, receipt in [("base", base, br), ("mind", mind, mr)]:
                raw, binding = root / (name + ".json"), root / (name + "-receipt.json")
                raw.write_text(json.dumps(report))
                receipt["report_sha256"] = checker.sha(raw)
                binding.write_text(json.dumps(receipt))
                paths.extend(["--" + name + "-report", str(raw), "--" + name + "-receipt", str(binding)])
            command = [sys.executable, checker.__file__, *paths, "--base-provenance", str(proof), "--output"]
            passed = subprocess.run([*command, str(root / "success.json")], capture_output=True, text=True, check=False)
            self.assertEqual(passed.returncode, 0, passed.stderr)
            artifact.write_text("changed")
            failed = subprocess.run([*command, str(root / "failure.json")], capture_output=True, text=True, check=False)
            self.assertEqual(failed.returncode, 2)
            self.assertIn("Changed BASE provenance artifact", failed.stderr)
            self.assertFalse((root / "failure.json").exists())

    def test_duplicate_keys_and_nonfinite_json_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "input.json"
            for raw in ('{"days":48,"days":48}', '{"entries":NaN}'):
                output.write_text(raw)
                with self.assertRaises(ValueError):
                    checker.read(output)


if __name__ == "__main__":
    unittest.main()
