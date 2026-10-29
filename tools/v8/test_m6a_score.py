"""Fail-closed evidence validation tests; no simulation execution."""
import json
import tempfile
import unittest
from dataclasses import fields
from pathlib import Path
from unittest.mock import patch

from m6a_evidence import EvidenceError, Node, reference_json
from m6a_score import WorldScore, audit_flows, calibration, conservation, expiry_pair, receipt, store_pass


class EvidenceTests(unittest.TestCase):
    def calibration_receipt(self, folder, purpose="calibration", seeds=None):
        seeds = list(range(8)) if seeds is None else seeds
        proof = Path(folder) / "proof.txt"
        proof.write_text("synthetic unit-test reference, not observed evidence")
        ref = reference_json(proof)
        payload = {"schema_version": 1, "kind": "m6a_cohort_v1", "arm": "candidate",
                   "purpose": purpose, "days": 72, "exit_code": 0, "source_unchanged": True,
                   "binary": ref, "observer": ref, "scorer": ref, "seeds": seeds,
                   "worlds": [{"seed": seed, "exit_code": 0, "log": ref, "files": []} for seed in seeds]}
        path = Path(folder) / "receipt.json"
        path.write_text(json.dumps(payload))
        return path

    def test_calibration_rejects_wrong_purpose_or_seed_set(self):
        for purpose, seeds in (("fixture", list(range(8))), ("heldout", list(range(8))),
                               ("calibration", [0, 1]), ("calibration", list(range(1296, 1304))),
                               ("calibration", list(reversed(range(8))))):
            with self.subTest(purpose=purpose, seeds=seeds), tempfile.TemporaryDirectory() as folder:
                path = self.calibration_receipt(folder, purpose, seeds)
                with self.assertRaises(EvidenceError):
                    receipt(path, "candidate", "calibration")

    def test_calibration_reports_all_eight_worlds_without_heldout_acceptance(self):
        def observed(world):
            values = {field.name: 0 for field in fields(WorldScore)}
            values.update(seed=world.get("seed").integer(), summer_daily_medians={}, causal={})
            for name in ("alive", "hunger_free_winters", "stores", "cooked_share", "diversity",
                         "spoilage", "ledger", "stockpile_day1", "beds_day10", "wolf_deaths_le1",
                         "winter42_alive", "winter66_alive", "winter42_store", "winter66_store"):
                values[name] = world.get("seed").integer() < 5
            return WorldScore(**values)
        with tempfile.TemporaryDirectory() as folder:
            path = self.calibration_receipt(folder)
            with patch("m6a_score.score_world", side_effect=observed) as score:
                report = calibration(path)
            self.assertEqual(score.call_count, 8)
            self.assertEqual(report["status"], "OBSERVED")
            self.assertEqual(report["acceptance_status"], "NOT_EVALUATED")
            self.assertEqual(report["denominator"], 8)
            self.assertEqual([world["seed"] for world in report["worlds"]], list(range(8)))
            self.assertEqual(report["counts"]["alive"], {"passed_worlds": 5, "observed_worlds": 8})
            self.assertEqual(report["winter_conditional"]["42"], {"passed": 5, "alive": 5})
            self.assertNotIn("bars", report)
            self.assertTrue(all("required" not in row for row in report["counts"].values()))

    def test_food_flows_reconcile_with_authoritative_boundary_and_final_tallies(self):
        initial = Node({"harvested_units": 0, "spoiled_units": 0})
        harvested = {"harvested_units": 100, "spoiled_units": 0}
        spoiled = {"harvested_units": 100, "spoiled_units": 40}
        flows = [Node({"tick": 1, "tallies": harvested}), Node({"tick": 9, "tallies": spoiled})]
        boundaries = {72: Node({"tick": 10, "tallies": spoiled})}
        audit_flows(flows, initial, boundaries, Node(spoiled), 11)
        for corrupted in (flows[:1], flows[::-1], flows + flows[-1:]):
            with self.subTest(corrupted=corrupted), self.assertRaises(EvidenceError):
                audit_flows(corrupted, initial, boundaries, Node(spoiled), 11)
        # Day72 expiry is outside the spoilage window, but final flow coverage is audited.
        final = {"harvested_units": 100, "spoiled_units": 60}
        audit_flows(flows + [Node({"tick": 10, "tallies": final})], initial, boundaries, Node(final), 11)
        with self.assertRaises(EvidenceError):
            audit_flows(flows, initial, boundaries, Node(final), 11)

    def test_reject_nonfinite_and_duplicate_json(self):
        for text in ('1e999', '-1e999', 'NaN', 'Infinity', '{"a":1,"a":2}'):
            with self.subTest(text=text), self.assertRaises(EvidenceError):
                Node.parse(text)

    def snapshot(self, count=3, spoiled=0):
        return Node({"day": 4, "physical": [{"owner": {"Ground": [1, 2]}, "pieces": count,
                     "units_per_piece": 2, "born_day": 0, "lifetime_days": 4}],
                     "physical_food_units": count * 2,
                     "tallies": {"initial_units": 6, "harvested_units": 0, "craft_output_units": 0,
                                "traded_in_units": 0, "eaten_units": 0, "spoiled_units": spoiled,
                                "lost_units": 0, "craft_input_units": 0, "traded_out_units": 0}})

    def test_independent_conservation_rejects_unbooked_loss(self):
        conservation(self.snapshot())
        with self.assertRaises(EvidenceError):
            conservation(self.snapshot(count=2))

    def test_expiry_requires_storage_lifetime_units_and_removal(self):
        before = self.snapshot()
        after = self.snapshot(count=0, spoiled=6)
        after.value["physical"] = []
        conservation(after)
        expiry_pair(before, after)
        with self.assertRaises(EvidenceError):
            expiry_pair(before, before)

    def test_stores_allocate_once_and_round_per_settlement(self):
        snapshot = Node({"alive": 2, "adults": [{"id": 1}, {"id": 2}],
                         "settlements": {"1": [1], "2": [2]},
                         "settlement_stores": [
                             {"id": 1, "home": [0, 0], "region": 1, "runtime_units": 30},
                             {"id": 2, "home": [10, 0], "region": 1, "runtime_units": 0}],
                         "stored": [{"tile": [5, 0], "region": 1, "pieces": 30,
                                     "units_per_piece": 1, "remaining_days": 4,
                                     "reachable_adults": [1, 2]}]})
        self.assertFalse(store_pass(snapshot))
        snapshot.value["settlement_stores"][1]["runtime_units"] = 30
        with self.assertRaises(EvidenceError):
            store_pass(snapshot)


if __name__ == "__main__":
    unittest.main()
