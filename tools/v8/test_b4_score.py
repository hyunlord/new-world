"""Synthetic B4 scorer boundary tests; no simulation or held-out evidence."""
from __future__ import annotations

import copy
import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import b4_score as score


def arithmetic():
    return {
        "schema_version": 1,
        "xp_table": [15 * level * (level + 1) for level in range(21)],
        "quality_bonus_table": [27 - abs(2 * value - 5) for value in range(-10, 16)],
        "qual_cap": [0, 20, 35, 45, 55, 65, 75, 85, 92, 97, 100],
        "quality_base": 30, "quality_per_level": 3,
        "stream_quality": 0x5155414C4954595F, "stream_apparel": 0x4150504152454C21,
        "stream_trade": 0x5452414445525321,
        "rng": {"algorithm": "splitmix64-chain-v1", "below": "multiply-high-u64",
                "add": 0x9E3779B97F4A7C15, "mul1": 0xBF58476D1CE4E5B9,
                "mul2": 0x94D049BB133111EB},
        "primary_recipe": {"code": 0, "identity": "knap_blade", "output_item_code": 5,
                           "output_count": 1, "input_item_code": 3, "input_count": 1,
                           "work_ticks": 45, "skill_code": 2, "quality_add": 0,
                           "material_term": {"kind": "hardness", "offset": 50, "divisor": 5},
                           "gate_mask": 8, "material_id": 4, "material_hardness": 700},
        "garments": [{"garment_arm": "hide_wrap", "item_code": 20, "cold_permille": 700},
                     {"garment_arm": "fur_cloak", "item_code": 21, "cold_permille": 500}],
        "knowledge_nodes": [{"node_id": 3, "threshold": 300, "era": "paleolithic"}],
        "trade": {"goods": [{"authored_index": 0, "item_code": 5, "made_by": 3,
                              "tag": "lacked", "count_min": 1, "count_max": 2},
                             {"authored_index": 1, "item_code": 3, "made_by": None,
                              "tag": "staple", "count_min": 2, "count_max": 3}],
                  "stacks_min": 2, "stacks_max": 3, "quality_min": 20,
                  "quality_max": 80, "practice_days": 6},
        "sources": [],
    }


HASH = "a" * 64
META = {"schema_version": 1, "arm": "B0", "arithmetic_manifest_sha256": HASH,
        "catalog_revision": "synthetic", "policy": "old_3_1"}


def event(seed=0, tick=100, level=1, proficiency=1000, alive=True):
    row = {
        "seed": seed, "agent_id": 1, "executed_tick": tick, "workshop_id": 1,
        "bill_id": 1, "recipe_code": 0, "recipe_identity": "knap_blade",
        "catalog_revision": "synthetic", "output_item_code": 5, "output_count": 1,
        "at": [3, 4], "job_stage": "working", "ticks_left": 1, "skill_code": 2,
        "skill_present": True, "xp_before": 15 * level * (level + 1), "level_before": level,
        "material_id": 4, "material_hardness": 700, "hardness_percent": 70,
        "temperament_present": True, "persistence": 500, "knowledge_enabled": True,
        "gate_mask": 8, "gating_nodes": [{"node_id": 3, "entry_present": True,
            "proficiency": proficiency, "last_used": 0, "admission_threshold": 300}],
        "recipe_quality_add": 0, "material_term": {"kind": "hardness", "offset": 50, "divisor": 5},
        "tally_before_count": 0, "tally_after_count": 1, "tally_before_quality_sum": 0,
        "tally_after_quality_sum": 0, "completion_count_delta": 1, "actual_quality": 0,
        "alive_after_tick": alive, "arithmetic_manifest_sha256": HASH,
    }
    row["actual_quality"] = score.craft_formula(row, arithmetic())["quality"]
    row["tally_after_quality_sum"] = row["actual_quality"]
    return row


def tally(row):
    return {"agent_id": row["agent_id"], "recipe_code": row["recipe_code"],
            "completed_count": row["tally_after_count"], "quality_sum": row["tally_after_quality_sum"]}


def natural(seed=0, events=None):
    events = [event(seed=seed)] if events is None else events
    days = []
    for day in range(1, 49):
        done = [r for r in events if r["executed_tick"] < day * 1440]
        days.append({"day": day, "executed_tick": day * 1440 - 1, "population": 8,
                     "insulating_wearers": 5,
                     "garment_counts": [{"item_code": 20, "count": 5, "cold_permille": 700}],
                     "craft_tallies": [tally(done[-1])] if done else [],
                     "cold": {"clothed_cold": 0, "clothed_eligible": 0,
                              "unclothed_cold": 0, "unclothed_eligible": 0},
                     "deaths_total": 0, "jobs_done_total": len(done), "cancel_totals": [],
                     "meat": {"dropped": 0, "eaten": 0, "spoiled": 0, "hauls": 0},
                     "ledger_violation_ticks": 0, "reservation_violation_ticks": 0})
    return {"schema_version": 1, "manifest_sha256": HASH, "arm": "B0", "seed": seed,
            "ticks_per_day": 1440, "horizon_ticks": 69120, "status": "VALID", "invalid_reasons": [],
            "days": days, "craft_events": events, "tally_initial": [],
            "tally_final": [tally(events[-1])] if events else [],
            "command_receipts": [], "wave_transitions": [], "caravan_generations": [],
            "purchase_receipts": [], "haul_starts": [], "haul_terminals": [],
            "conservation": {"ledger_violation_ticks": 0, "reservation_violation_ticks": 0,
                             "unresolved_start_count": 0, "unresolved_terminal_count": 0}}


class StrictInputTests(unittest.TestCase):
    def test_duplicate_json_keys_are_invalid(self):
        with self.assertRaises(score.Invalid):
            score.parse_json('{"x":1,"x":2}')

    def test_float_and_nonfinite_tokens_are_invalid(self):
        for text in ['{"x":1.0}', '{"x":NaN}', '{"x":Infinity}']:
            with self.subTest(text=text), self.assertRaises(score.Invalid):
                score.parse_json(text)

    def test_bool_is_not_an_integer_and_widths_are_exact(self):
        for value in [True, -1, 2**64]:
            with self.subTest(value=value), self.assertRaises(score.Invalid):
                score.validate(value, "u64", "counter")
        score.validate(2**64 - 1, "u64", "counter")

    def test_hash_tampering_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.json"
            path.write_text("{}")
            ref = {"path": path.name, "sha256": hashlib.sha256(b"{}").hexdigest()}
            self.assertEqual(score.read_ref(Path(directory), ref), {})
            path.write_text('{"changed":1}')
            with self.assertRaises(score.Invalid):
                score.read_ref(Path(directory), ref)

    def test_unknown_event_field_is_invalid(self):
        row = event()
        row["made_up"] = 1
        with self.assertRaises(score.Invalid):
            score.check_craft(row, arithmetic(), META)


class ArithmeticTests(unittest.TestCase):
    def test_splitmix_reference_wraps_u64(self):
        self.assertEqual(score.mix(0), 0xE220A8397B1DCDAF)
        self.assertEqual(score.mix(2**64 - 1), 0xE4D971771B652C20)

    def test_below_is_multiply_high_not_modulo(self):
        draw = score.keyed_u64(1, 2, 3, 4, 5, 6)
        self.assertEqual(score.keyed_below(1, 2, 3, 4, 5, 6, 1000), (draw * 1000) >> 64)

    def test_negative_material_divides_toward_zero(self):
        row = event()
        row["hardness_percent"] = 48
        self.assertEqual(score.craft_formula(row, arithmetic())["material"], 0)
        row["hardness_percent"] = 44
        self.assertEqual(score.craft_formula(row, arithmetic())["material"], -1)

    def test_capped_pair_does_not_demand_fifteen(self):
        low, high = event(level=1, proficiency=300), event(level=6, proficiency=300)
        q1 = score.check_craft(low, arithmetic(), META)
        q6 = score.check_craft(high, arithmetic(), META)
        self.assertEqual(q6["quality"], min(q6["base"], 45))
        self.assertLessEqual(q6["quality"] - q1["quality"], 15)

    def test_multiple_or_negative_tally_delta_is_invalid(self):
        for count in [2, 0]:
            row = event()
            row["tally_after_count"] = count
            row["completion_count_delta"] = count
            if count == 0:
                row["tally_before_count"] = 1
            with self.subTest(count=count), self.assertRaises(score.Invalid):
                score.check_craft(row, arithmetic(), META)

    def test_xp_level_disagreement_is_invalid(self):
        row = event()
        row["level_before"] = 6
        with self.assertRaises(score.Invalid):
            score.check_craft(row, arithmetic(), META)

    def test_zero_denominator_is_not_measured(self):
        self.assertEqual(score.rate(0, 0)["status"], "NOT_MEASURED")
        self.assertEqual(score.rate(0, 8)["status"], "VALID")


class NaturalTests(unittest.TestCase):
    def check(self, report):
        return score.check_natural(report, arithmetic(), META, HASH, require_steward=False)

    def test_capacity_blocked_counts_only_rejected_accept_migrants(self):
        report = natural()
        cases = [({"AcceptMigrants": {"wave": 1}}, False, "blocked"),
                 ({"AcceptMigrants": {"wave": 2}}, False, "no_visitor"),
                 ({"AcceptMigrants": {"wave": 3}}, True, None),
                 ({"DeclineMigrants": {"wave": 4}}, False, "blocked")]
        report["command_receipts"] = [
            {"ordinal": ordinal, "upcoming_tick": 10, "command": command,
             "result": {"accepted": accepted, "count": int(accepted), "reason": reason},
             "population_before": 39, "population_after": 39,
             "wave_id": ordinal + 1, "wave_size": 2, "wave_before": None, "wave_after": None}
            for ordinal, (command, accepted, reason) in enumerate(cases)]
        observations = self.check(report)["population_observations"]
        self.assertEqual(observations["capacity_blocked"], 1)
        self.assertEqual(observations["accept_command_attempts"], 3)
        self.assertEqual(observations["accept_command_rejected"], 2)
        self.assertEqual(observations["arrival_acceptance_cleared"], 0)

    def test_deceased_crafter_retains_completion(self):
        result = self.check(natural(events=[event(alive=False)]))
        self.assertEqual(result["craft_count"], 1)
        self.assertTrue(result["arithmetic_pass"])

    def test_missing_and_duplicate_days_are_invalid(self):
        for change in ["missing", "duplicate"]:
            report = natural()
            if change == "missing":
                report["days"].pop()
            else:
                report["days"][1] = copy.deepcopy(report["days"][0])
            with self.subTest(change=change), self.assertRaises(score.Invalid):
                self.check(report)

    def test_missing_event_does_not_pass_tally_reconciliation(self):
        report = natural()
        report["craft_events"] = []
        with self.assertRaises(score.Invalid):
            self.check(report)

    def test_no_crafts_cannot_earn_arithmetic_pass(self):
        self.assertFalse(self.check(natural(events=[]))["arithmetic_pass"])

    def test_day_boundary_cannot_shift_event(self):
        report = natural(events=[event(tick=1440)])
        report["days"][0]["craft_tallies"] = copy.deepcopy(report["tally_final"])
        with self.assertRaises(score.Invalid):
            self.check(report)

    def test_missing_and_duplicate_seeds_are_invalid(self):
        for observed in [[0], [0, 0]]:
            with self.subTest(observed=observed), self.assertRaises(score.Invalid):
                score.check_coverage(observed, [0, 1], "seed")


def cold_fixture(seeds=(0,)):
    rows = []
    for seed in seeds:
        for agent in range(1, 9):
            for tick in score.COLD_TICKS:
                for garment, item, p in [("none", None, 1000), ("hide_wrap", 20, 700), ("fur_cloak", 21, 500)]:
                    rows.append({"seed": seed, "agent_id": agent, "lineage_key": agent, "executed_tick": tick,
                        "garment_arm": garment, "item_code": item, "cold_permille": p,
                        "observed_cold": score.keyed_below(seed, 0x4150504152454C21, agent, tick, 0, 0, 1000) < p,
                        "member": True, "season": 3, "in_cold_window": True, "burn_rate": 1,
                        "no_roof": True, "wood_zero": True, "excluded_markers_absent": True, "damage_disabled": True})
    return {"schema_version": 1, "manifest_sha256": HASH, "arm": "B0", "fixture_kind": "cold_filter",
            "status": "VALID", "invalid_reasons": [], "rows": rows}


def craft_fixture(seeds=(0,)):
    rows = []
    def ledger(code, created=0, consumed=0, ground=0):
        return {"item_code": code, "created": created, "consumed": consumed, "ground": ground,
                "traded_in": 0, "traded_out": 0, "carried": 0, "held": 0, "delivered": 0, "shelved": 0}
    for seed in seeds:
        for level in (1, 6):
            for cap, proficiency in [("unrestricted", 1000), ("restricted", 300)]:
                row = event(seed=seed, tick=27360, level=level, proficiency=proficiency)
                row["ticks_left"] = 45
                initial = {"agent_id": 1, "recipe_code": 0, "completed_count": 0, "quality_sum": 0}
                row.update({"fixture_id": f"{seed}/{level}/{cap}", "cap_arm": cap, "target_level": level,
                    "target_proficiency": proficiency, "admission_eligible": True, "inputs_consumed": True,
                    "ledger_before": [ledger(3, created=1, ground=1), ledger(5)],
                    "ledger_after": [ledger(3, created=1, consumed=1), ledger(5, created=1, ground=1)],
                    "output_delta": 1, "bill_done_before": 0, "bill_done_after": 1, "setup_finished_tick": 50,
                    "add_bill_result": {"accepted": True, "count": 1, "reason": None},
                    "paid_working_observed": True, "ticks_left_at_paid_boundary": 45,
                    "primary_tally_before_payment": initial.copy(), "primary_tally_at_paid_boundary": initial.copy(),
                    "tally_key_present_before_payment": False, "cleanup_complete": True,
                    "ledger_balanced": True, "reservation_check_passed": True})
                rows.append(row)
    return {"schema_version": 1, "manifest_sha256": HASH, "arm": "B0", "fixture_kind": "craft_finish",
            "status": "VALID", "invalid_reasons": [], "rows": rows}


class FixtureTests(unittest.TestCase):
    def test_cold_exact_grid_including_two_distinct_absolute_days(self):
        result = score.check_cold_fixture(cold_fixture(), arithmetic(), META, HASH, [0])
        self.assertEqual(result["garments"]["none"]["N"], 96)
        self.assertEqual(result["garments"]["none"]["O"], 96)
        self.assertEqual(score.COLD_TICKS[0], 27360)
        self.assertEqual(score.COLD_TICKS[6], 28800)

    def test_cold_missing_duplicate_and_wrong_day_rejected(self):
        for mutation in ("missing", "duplicate", "wrong_day"):
            fixture = cold_fixture()
            if mutation == "missing":
                fixture["rows"].pop()
            elif mutation == "duplicate":
                fixture["rows"][1] = copy.deepcopy(fixture["rows"][0])
            else:
                fixture["rows"][0]["executed_tick"] += 1440
            with self.subTest(mutation=mutation), self.assertRaises(score.Invalid):
                score.check_cold_fixture(fixture, arithmetic(), META, HASH, [0])

    def test_cold_failed_eligibility_and_keyed_outcome_rejected(self):
        for field in ("member", "no_roof", "damage_disabled", "observed_cold"):
            fixture = cold_fixture()
            fixture["rows"][0][field] = False
            with self.subTest(field=field), self.assertRaises(score.Invalid):
                score.check_cold_fixture(fixture, arithmetic(), META, HASH, [0])

    def test_controlled_paid_boundary_is_not_natural_last_tick(self):
        fixture = craft_fixture()
        result = score.check_craft_fixture(fixture, arithmetic(), META, HASH, [0])
        self.assertTrue(result["pass"])
        self.assertEqual(result["completions"], 4)
        raw = {key: fixture["rows"][0][key] for key in score.CRAFT}
        with self.assertRaises(score.Invalid):
            score.check_craft(raw, arithmetic(), META)

    def test_missing_or_duplicate_cap_level_cell_rejected(self):
        for duplicate in (False, True):
            fixture = craft_fixture()
            if duplicate:
                fixture["rows"][1] = copy.deepcopy(fixture["rows"][0])
            else:
                fixture["rows"].pop()
            with self.subTest(duplicate=duplicate), self.assertRaises(score.Invalid):
                score.check_craft_fixture(fixture, arithmetic(), META, HASH, [0])

    def test_fake_payment_or_unbalanced_output_rejected(self):
        for field in ("inputs_consumed", "cleanup_complete", "ledger"):
            fixture = craft_fixture()
            if field == "ledger":
                fixture["rows"][0]["ledger_after"][1]["ground"] = 0
            else:
                fixture["rows"][0][field] = False
            with self.subTest(field=field), self.assertRaises(score.Invalid):
                score.check_craft_fixture(fixture, arithmetic(), META, HASH, [0])


class ExtendedBoundaries(unittest.TestCase):
    def test_full_evaluation_never_accepts_calibration_subset(self):
        index = {"schema_version": 1, "mode": "evaluation", "seeds": [0], "arms": []}
        with self.assertRaises(score.Invalid):
            score.score_index(index, Path("."))

    def test_calibration_requires_explicit_flag_and_label(self):
        index = {"schema_version": 1, "mode": "calibration", "seeds": [0], "arms": []}
        with self.assertRaises(score.Invalid):
            score.score_index(index, Path("."))

    def test_modified_arithmetic_table_is_not_redefined_contract(self):
        data = arithmetic()
        data["qual_cap"][3] = 46
        with self.assertRaisesRegex(score.Invalid, "constant"):
            score.check_arithmetic(data, Path("."))

    def test_duplicate_full_width_agent_events_rejected(self):
        row = event()
        row["agent_id"] = 2**64 - 1
        row["actual_quality"] = score.craft_formula(row, arithmetic())["quality"]
        row["tally_after_quality_sum"] = row["actual_quality"]
        score.check_craft(row, arithmetic(), META)
        report = natural(events=[row, copy.deepcopy(row)])
        with self.assertRaises(score.Invalid):
            score.check_natural(report, arithmetic(), META, HASH, require_steward=False)

    def test_deceased_tally_removal_is_invalid(self):
        report = natural(events=[event(alive=False)])
        report["days"][-1]["craft_tallies"] = []
        report["tally_final"] = []
        with self.assertRaises(score.Invalid):
            score.check_natural(report, arithmetic(), META, HASH, require_steward=False)

    def test_nonprimary_count_cannot_replace_primary_coverage(self):
        report = natural()
        report["craft_events"][0]["recipe_code"] = 2
        with self.assertRaises(score.Invalid):
            score.check_natural(report, arithmetic(), META, HASH, require_steward=False)

    def test_lossless_u64_and_legacy_float_isolation(self):
        value = score.parse_json('{"counter":18446744073709551615,"legacy":0.5}', legacy_floats=True)
        self.assertEqual(value["counter"], 2**64 - 1)
        with self.assertRaises(score.Invalid):
            score.validate(value["legacy"], "u64")
        with self.assertRaises(score.Invalid):
            score.parse_json('{"legacy":1e999}', legacy_floats=True)

    def test_command_tag_fields_and_trade_widths(self):
        score.validate({"SetSteward": {"on": False, "scope": "All"}}, "command")
        for command in [{"SetSteward": {"on": 0, "scope": "All"}},
                        {"Trade": {"caravan": 1, "give": [[0, 0, 65536]], "take": []}},
                        {"Unknown": {}}, {"AddBill": {"workshop": 1, "recipe": "KnapBlade", "mode": {"Make": True}}}]:
            with self.subTest(command=command), self.assertRaises(score.Invalid):
                score.validate(command, "command")

    def test_natural_no_denominator_cannot_be_protection_pass(self):
        result = score.check_natural(natural(), arithmetic(), META, HASH, require_steward=False)
        self.assertEqual(result["natural_cold"]["status"], "NOT_MEASURED")
        self.assertIsNone(result["natural_cold"]["ratio"])

    def test_negative_quality_delta_is_invalid(self):
        row = event()
        row["tally_before_quality_sum"] = row["tally_after_quality_sum"] + 1
        with self.assertRaises(score.Invalid):
            score.check_craft(row, arithmetic(), META)

    def test_pair_means_and_medians_remain_exact(self):
        result = score.mean_median([1, 2])
        self.assertEqual(result["mean"], {"numerator": 3, "denominator": 2})
        self.assertEqual(result["median"], result["mean"])


if __name__ == "__main__":
    unittest.main()
