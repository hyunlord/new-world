#!/usr/bin/env python3
"""Synthetic scorer counterexamples; never executes simulation or consumes held-out data."""
import copy
import unittest
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
from unittest.mock import patch
import mux2_score as scorer


def fixture(phase="heldout", defence=True):
    seeds = list(range(1200, 1232)) if phase == "heldout" else list(range(8))
    receipt = {"phase":phase,"defence":defence,"seeds":seeds,"days":48,"round":None if phase == "heldout" else 1,
               "policy_frozen_before_run":True,"source_sha":"a"*40,"policy_sha256":"b"*64,"binary_sha256":"c"*64}
    inputs = {"steward":True,"defence":defence,"defence_enabled_end":defence,"player_script":False,
              "story":"on","neighbours":"on","source_sha":"a"*40,"policy_sha256":"b"*64}
    old = {"ticks_per_day":1440,"stockpile_first_tick":1440,"first_winter_observed":True,
           "first_winter_complete":True,"first_winter_cold_deaths":0,"cold_snap_observed_ticks":10,
           "cold_snap_firewood_zero_ticks":0,"alive_end":10,"alive_at_day48":True}
    observed = {"schema_version":1,"start_tick":0,"end_tick":69120,"observed_ticks":69120,
                "observer_integrity_errors":[],"inputs":inputs,"walled_beds_for_all_first_tick":14400,
                "wolf_deaths":0,"wolf_death_events":[],"fatal_animal_events":0,"combat_deaths":0}
    rows = [{"seed":seed,"days":48,"mux":copy.deepcopy(old),"mux2":copy.deepcopy(observed),
             "alive_end":10,"deaths":{"combat":0},"ledger_violation_ticks":0,"reservation_violation_ticks":0} for seed in seeds]
    return {"scene":"colony","days":48,"seeds":rows}, receipt


class Mux2ScoreTests(unittest.TestCase):
    def test_exact_seven_thresholds_both_arms(self):
        for arm in (True, False):
            report, receipt = fixture(defence=arm)
            result = scorer.score(report, receipt, phase="heldout", defence=arm)
            self.assertTrue(result["passed"])
            self.assertEqual({key:value["required"] for key,value in result["bars"].items()}, scorer.TARGETS)

    def test_every_threshold_boundary_without_changing_denominator(self):
        for bar, required in scorer.TARGETS.items():
            report, receipt = fixture()
            def fail(row):
                if bar == "stockpile_day1": row["mux"]["stockpile_first_tick"] = 1441
                elif bar == "walled_beds_day10": row["mux2"]["walled_beds_for_all_first_tick"] = 14401
                elif bar == "zero_cold_deaths": row["mux"]["first_winter_cold_deaths"] = 1
                elif bar == "firewood_during_snap": row["mux"]["cold_snap_firewood_zero_ticks"] = 1
                elif bar == "alive_day48": row["mux"].update(alive_end=0,alive_at_day48=False); row["alive_end"] = 0
                elif bar == "wolf_deaths_at_most_one":
                    row["mux2"].update(wolf_deaths=2,fatal_animal_events=2,combat_deaths=2,
                        wolf_death_events=[{"tick":0,"victim":i,"attacker":1,"kind":1,"mechanic":"predator"} for i in (1,2)])
                    row["deaths"]["combat"] = 2
                else: row["ledger_violation_ticks"] = 1
            for row in report["seeds"][:32-required]: fail(row)
            self.assertEqual(scorer.score(report,receipt,phase="heldout",defence=True)["bars"][bar]["passed"], required)
            fail(report["seeds"][32-required])
            result = scorer.score(report,receipt,phase="heldout",defence=True)
            self.assertEqual(result["bars"][bar]["passed"], required-1)
            self.assertFalse(result["passed"])

    def test_missing_duplicate_wrong_seeds_invalid(self):
        for change in (lambda r:r["seeds"].pop(), lambda r:r["seeds"].append(r["seeds"][0]),
                       lambda r:r["seeds"][0].update(seed=1000), lambda r:r["seeds"][0].update(seed=1201)):
            report,receipt = fixture(); change(report)
            with self.assertRaises(ValueError): scorer.score(report,receipt,phase="heldout",defence=True)

    def test_corrupted_metrics_never_become_pass_or_silently_disappear(self):
        for change in (lambda r:r["mux2"].update(observed_ticks=69119),
                       lambda r:r["mux2"].update(wolf_deaths=1),
                       lambda r:r["mux2"].update(observer_integrity_errors=["bad"]),
                       lambda r:r["mux2"].update(walled_beds_for_all_first_tick=-1),
                       lambda r:r["mux"].update(ticks_per_day=True),
                       lambda r:r["mux"].update(cold_snap_firewood_zero_ticks=11),
                       lambda r:r["mux2"]["inputs"].update(defence=False),
                       lambda r:r["mux2"]["inputs"].update(player_script=True),
                       lambda r:r["mux2"]["inputs"].update(policy_sha256="d"*64),
                       lambda r:r["mux2"].update(schema_version=True)):
            report,receipt = fixture(); change(report["seeds"][0])
            with self.assertRaises(ValueError): scorer.score(report,receipt,phase="heldout",defence=True)

    def test_no_snap_no_winter_or_zero_population_are_not_vacuous_passes(self):
        report,receipt = fixture()
        row=report["seeds"][0]
        row["mux"].update(cold_snap_observed_ticks=0,first_winter_observed=False,first_winter_complete=False,
                          alive_end=0,alive_at_day48=False)
        row["alive_end"]=0
        result=scorer.score(report,receipt,phase="heldout",defence=True)
        for bar in ("firewood_during_snap","zero_cold_deaths","alive_day48"):
            self.assertFalse(result["seeds"][0]["bars"][bar])
            self.assertEqual(result["bars"][bar]["total"],32)

    def test_calibration_requires_seeds_zero_to_seven_and_at_most_four_rounds(self):
        report,receipt=fixture("calibration")
        for round_number in range(1,5):
            receipt["round"]=round_number
            result=scorer.score(report,receipt,phase="calibration",defence=True)
            self.assertFalse(result["acceptance_claim"])
        receipt["round"]=5
        with self.assertRaises(ValueError): scorer.score(report,receipt,phase="calibration",defence=True)
        report,receipt=fixture()
        with self.assertRaises(ValueError): scorer.score(report,receipt,phase="calibration",defence=True)


class Mux2PairTests(unittest.TestCase):
    def test_complete_complementary_pair_binds_all_hashes_and_64_worlds(self):
        on, on_receipt = fixture(defence=True)
        off, off_receipt = fixture(defence=False)
        result = scorer.score_pair(on, on_receipt, off, off_receipt)
        self.assertEqual(result["worlds"], 64)
        self.assertTrue(result["comparison_valid"])
        self.assertTrue(result["acceptance_claim"])
        self.assertEqual(set(result["arms"]), {"on", "off"})
        self.assertEqual(result["binary_sha256"], on_receipt["binary_sha256"])
        self.assertFalse(result["arms"]["on"]["acceptance_claim"])

    def test_individually_valid_but_mismatched_hash_pairs_rejected(self):
        for key, replacement in [("source_sha", "d"*40), ("policy_sha256", "d"*64), ("binary_sha256", "d"*64)]:
            on, on_receipt = fixture(defence=True)
            off, off_receipt = fixture(defence=False)
            off_receipt[key] = replacement
            if key != "binary_sha256":
                for row in off["seeds"]: row["mux2"]["inputs"][key] = replacement
            self.assertEqual(scorer.score(off,off_receipt,phase="heldout",defence=False)["status"], "VALID")
            with self.assertRaisesRegex(ValueError, "Pair .* mismatch"):
                scorer.score_pair(on,on_receipt,off,off_receipt)

    def test_wrong_or_missing_arm_seed_or_duration_cannot_form_pair(self):
        for change in [lambda r,m:m.update(defence=True), lambda r,m:r["seeds"].pop(),
                       lambda r,m:r["seeds"][0].update(seed=1199), lambda r,m:m.update(days=47),
                       lambda r,m:m.update(phase="calibration")]:
            on,on_receipt=fixture(defence=True); off,off_receipt=fixture(defence=False)
            change(off,off_receipt)
            with self.assertRaises(ValueError): scorer.score_pair(on,on_receipt,off,off_receipt)

    def test_control_arm_outcome_failure_is_reported_not_used_as_intervention_bar(self):
        on,on_receipt=fixture(defence=True); off,off_receipt=fixture(defence=False)
        for row in off["seeds"]:
            row["mux"].update(alive_end=0,alive_at_day48=False); row["alive_end"]=0
        result=scorer.score_pair(on,on_receipt,off,off_receipt)
        self.assertTrue(result["comparison_valid"])
        self.assertTrue(result["passed"])
        self.assertFalse(result["arms"]["off"]["passed"])
        self.assertEqual(result["arms"]["off"]["bars"]["alive_day48"]["passed"],0)

    def test_single_arm_never_claims_final_paired_acceptance(self):
        report,receipt=fixture()
        self.assertFalse(scorer.score(report,receipt,phase="heldout",defence=True)["acceptance_claim"])

    def test_on_failure_denies_acceptance_even_with_valid_control(self):
        on,on_receipt=fixture(defence=True); off,off_receipt=fixture(defence=False)
        on["seeds"][0]["ledger_violation_ticks"]=1
        result=scorer.score_pair(on,on_receipt,off,off_receipt)
        self.assertTrue(result["comparison_valid"])
        self.assertFalse(result["passed"])
        self.assertFalse(result["acceptance_claim"])

    def test_matching_days_with_different_tick_lengths_is_not_equivalent(self):
        on,on_receipt=fixture(defence=True); off,off_receipt=fixture(defence=False)
        for row in off["seeds"]:
            row["mux"]["ticks_per_day"]=720
            row["mux2"].update(end_tick=34560,observed_ticks=34560)
        self.assertEqual(scorer.score(off,off_receipt,phase="heldout",defence=False)["status"],"VALID")
        with self.assertRaisesRegex(ValueError,"tick duration mismatch"):
            scorer.score_pair(on,on_receipt,off,off_receipt)

    def test_cli_binds_both_raw_inputs_and_off_failures_exit_zero(self):
        with tempfile.TemporaryDirectory(prefix="mux2-score-test-") as directory:
            root=Path(directory)
            for arm in ("on","off"):
                report,receipt=fixture(defence=arm=="on")
                if arm=="off":
                    for row in report["seeds"][:10]:
                        row["mux"].update(alive_end=0,alive_at_day48=False); row["alive_end"]=0
                raw=json.dumps(report).encode()
                receipt["summary_sha256"]=hashlib.sha256(raw).hexdigest()
                (root/f"{arm}.json").write_bytes(raw)
                (root/f"{arm}-receipt.json").write_text(json.dumps(receipt))
            def invoke(arm, extra=()):
                argv=["mux2_score",str(root/f"{arm}.json"),"--receipt",str(root/f"{arm}-receipt.json"),
                      "--phase","heldout","--defence",arm,"--output",str(root/"score.json"),*extra]
                with patch("sys.argv",argv), contextlib.redirect_stdout(io.StringIO()):
                    return scorer.main()
            self.assertEqual(invoke("off"),0)
            single=json.loads((root/"score.json").read_text())
            self.assertFalse(single["passed"])
            self.assertFalse(single["acceptance_claim"])
            extra=["--off-summary",str(root/"off.json"),"--off-receipt",str(root/"off-receipt.json")]
            self.assertEqual(invoke("on",extra),0)
            paired=json.loads((root/"score.json").read_text())
            self.assertEqual(set(paired["input_hashes"]),{"on","off"})
            self.assertTrue(paired["acceptance_claim"])
            self.assertEqual(paired["arms"]["off"]["bars"]["alive_day48"]["passed"],22)
            (root/"off.json").write_text((root/"off.json").read_text()+" ")
            with self.assertRaisesRegex(ValueError,"Summary bytes not bound"):
                invoke("on",extra)


if __name__ == "__main__":
    unittest.main()
