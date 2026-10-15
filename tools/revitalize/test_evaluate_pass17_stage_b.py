from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from dataclasses import replace
import subprocess
import sys

import numpy as np

import evaluate_pass17 as cli
import evaluate_pass17_evidence as evidence
from evaluate_pass17_family_data import FamilyBirth, FamilyFission, FamilyMember, FamilyPerson, FamilySnapshot, FamilyWorld, LifeStage
import evaluate_pass17_statistics as statistics


def family_person(agent_id: int, value: int, mother: int | None = None,
                  father: int | None = None) -> FamilyPerson:
    return FamilyPerson(agent_id, (value,) * 4, agent_id % 2 == 0, mother, father)


def member(agent_id: int, stage: LifeStage = "child", settlement: int = 0) -> FamilyMember:
    return FamilyMember(agent_id, settlement, stage, None, None, agent_id, None)


class StageBPairTests(unittest.TestCase):
    def test_shared_parent_in_opposite_slots_is_close_kin(self) -> None:
        from evaluate_pass17_family_pairs import close_kin, full_siblings
        first = family_person(0, 100, 8, 9)
        second = family_person(1, 200, 7, 8)
        self.assertTrue(close_kin(first, second))
        self.assertFalse(full_siblings(first, second))

    def test_unassigned_living_people_remain_in_f1_but_never_form_same_settlement_pairs(self) -> None:
        from evaluate_pass17_family_pairs import collect_pairs
        from evaluate_pass17_family_rows import parse_members
        from evaluate_pass17_stage_b import partner_exposure
        rows = [{"id": i, "settlement": None, "life_stage": "adult", "partner": i ^ 1,
            "pairing_tick": 0, "household_key": 0, "bed": None} for i in range(2)]
        snapshot = FamilySnapshot(60, 1, 60, parse_members(rows, 60))
        people = tuple(family_person(i, i * 100, 8, 9) for i in range(2))
        result = collect_pairs(FamilyWorld(0, people, (snapshot,), snapshot, (), ()))
        self.assertEqual(partner_exposure(snapshot), (2, 2))
        self.assertEqual(result.siblings, ())
        self.assertEqual(result.unrelated, ())

    def test_full_siblings_require_both_known_parents_and_controls_exclude_close_kin(self) -> None:
        from evaluate_pass17_family_pairs import collect_pairs
        people = (family_person(0, 100), family_person(1, 200),
            family_person(2, 300, 0, 1), family_person(3, 350, 0, 1),
            family_person(4, 600, 0, 9), family_person(5, 800), family_person(6, 900))
        snapshots = (FamilySnapshot(0, 1, 0, tuple(member(i) for i in range(7))),)
        result = collect_pairs(FamilyWorld(0, people, snapshots, snapshots[0], (), ()))
        self.assertEqual([(p.first.agent_id, p.second.agent_id) for p in result.siblings], [(2, 3)])
        unrelated = {(p.first.agent_id, p.second.agent_id) for p in result.unrelated}
        self.assertNotIn((0, 2), unrelated)
        self.assertNotIn((1, 3), unrelated)
        self.assertNotIn((2, 4), unrelated)
        self.assertIn((5, 6), unrelated)
        self.assertEqual(result.excluded_kin, 7)

    def test_first_qualifying_shared_snapshot_controls_stage_categories_and_deduplicates(self) -> None:
        from evaluate_pass17_family_pairs import collect_pairs
        people = (family_person(0, 100, 10, 11), family_person(1, 200, 10, 11),
            family_person(2, 300), family_person(3, 400))
        snapshots = (FamilySnapshot(0, 1, 0, (member(0), member(1, settlement=1), member(2, "adult"), member(3, "adult"))),
            FamilySnapshot(60, 1, 60, (member(0, "adult"), member(1, "adult"), member(2), member(3))),
            FamilySnapshot(120, 1, 120, tuple(member(i, "adult") for i in range(4))),
            FamilySnapshot(180, 1, 180, tuple(member(i, "adult") for i in range(4))))
        result = collect_pairs(FamilyWorld(0, people, snapshots, snapshots[0], (), ()))
        self.assertEqual(result.siblings[0].tick, 60)
        self.assertEqual(result.categories, (("adult", "adult"),))
        pair = next(p for p in result.unrelated if (p.first.agent_id, p.second.agent_id) == (2, 3))
        self.assertEqual(pair.tick, 0)
        self.assertEqual(len(result.unrelated), 5)

    def test_symmetric_pearson_matches_explicit_both_orientations(self) -> None:
        from evaluate_pass17_family_pairs import axis_correlation, moments
        pairs = [(10, 30), (20, 80), (70, 60)]
        observed = axis_correlation([moments(pairs)])
        expected = statistics.pearson([(float(a), float(b)) for a, b in pairs] + [(float(b), float(a)) for a, b in pairs])
        self.assertAlmostEqual(observed, expected)
        self.assertNotAlmostEqual(observed, statistics.pearson([(float(a), float(b)) for a, b in pairs]))

    def test_f3_world_bootstrap_matches_explicit_pairs_and_requires_positive_interval(self) -> None:
        from evaluate_pass17_family_pairs import PairMoments, moments, resemblance_difference
        from evaluate_pass17_stage_a import measure
        from evaluate_pass17_data import Criterion
        high = [(10, 11), (90, 91)]
        low = [(10, 90), (40, 50)]
        raw = [(high, low), (low, high), (high * 2, low * 2)]
        worlds = [PairMoments((moments(a),) * 4, (moments(b),) * 4) for a, b in raw]
        rng = np.random.default_rng(0)
        expected = []
        for _ in range(2000):
            selected = [raw[int(i)] for i in rng.integers(0, 3, size=3)]
            oriented = [[pair for world in selected for a, b in world[kind] for pair in ((a, b), (b, a))] for kind in (0, 1)]
            expected.append(statistics.pearson(oriented[0]) - statistics.pearson(oriented[1]))
        result = measure(worlds, resemblance_difference, Criterion(0.06, None, ">=.06; lower CI>0", "rho", True))
        np.testing.assert_allclose(result.ci95, np.percentile(expected, [2.5, 97.5]))
        self.assertGreater(result.value, 0.06)
        self.assertLess(result.ci95[0], 0)
        self.assertEqual(result.verdict, "FAIL")
        self.assertEqual(result.bootstrap_valid, 2000)


class StageBFractionTests(unittest.TestCase):
    def test_day60_pools_adults_and_ignores_children_elders_and_dead_partners(self) -> None:
        from evaluate_pass17_stage_b import partner_exposure
        members = (replace(member(0, "adult"), partner=1), replace(member(1, "adult"), partner=0),
            replace(member(2, "adult"), partner=99), member(3, "child"), member(4, "elder"))
        self.assertEqual(partner_exposure(FamilySnapshot(86399, 60, 419, members)), (2.0, 3.0))

    def test_day20_inclusive_births_use_fallback_flag_and_keep_early_daily_counts(self) -> None:
        from evaluate_pass17_stage_b import birth_exposure, births_by_day
        births = (FamilyBirth(19 * 1440 - 1, 19, 0, 5, 6, False),
            FamilyBirth(19 * 1440, 20, 1, 5, 6, True), FamilyBirth(20 * 1440, 21, 2, 5, 6, False))
        snapshot = FamilySnapshot(86399, 60, 419, ())
        world = FamilyWorld(0, (), (), snapshot, births, ())
        self.assertEqual(birth_exposure(world), (1.0, 2.0))
        daily = births_by_day((world,), 60)
        self.assertEqual(daily[18], {"day": 19, "births": 1, "couple_births": 1, "fallback_births": 0})
        self.assertEqual(daily[19], {"day": 20, "births": 1, "couple_births": 0, "fallback_births": 1})
        self.assertEqual(len(daily), 60)

    def test_couple_nights_count_once_only_at_20_and_both_bedded(self) -> None:
        from evaluate_pass17_stage_b import bed_exposure
        members = tuple(replace(member(i, "adult"), partner=i ^ 1,
            shelter=(1, 2) if i < 2 else (i, 2)) for i in range(4))
        snapshot = FamilySnapshot(780, 1, 1200, members)
        bedless = replace(snapshot, tick=2220, day=2,
            members=tuple(replace(m, shelter=None) if m.agent_id == 3 else m for m in members))
        world = FamilyWorld(0, (), (replace(snapshot, minute=1140), snapshot, bedless), snapshot, (), ())
        self.assertEqual(bed_exposure(world), (2.0, 3.0))

    def test_fissions_only_completed_and_zero_denominator_is_error(self) -> None:
        from evaluate_pass17_stage_b import split_exposure
        from evaluate_pass17_stage_a import measure
        from evaluate_pass17_data import Criterion
        snapshot = FamilySnapshot(86399, 60, 419, ())
        world = FamilyWorld(0, (), (), snapshot, (), (
            FamilyFission(10, 0, 10, 1, 4, 1), FamilyFission(11, 0, 10, 9, 0, 5, "cancelled")))
        self.assertEqual(split_exposure(world), (1.0, 10.0))
        empty = measure([(0.0, 0.0)], statistics.pooled_ratio, Criterion(None, 0.10, "<=0.10", "share"))
        self.assertEqual(empty.verdict, "ERROR")
        self.assertIsNone(empty.value)


class StageBEvidenceTests(unittest.TestCase):
    def test_stage_b_performance_valid_fail_survives_raw_proof_reverification(self) -> None:
        import test_evaluate_perf_pass17 as performance_fixture
        import evaluate_perf_pass17 as performance
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            fixture = performance_fixture.PerformanceEvaluation()
            fixture.root = root
            fixture.make_fixture("b", old_mean=1.0, new_mean=1.3)
            result = performance.evaluate(root, "b")
            self.assertEqual(result["verdict"], "FAIL", result["errors"])
            path = root / "perf.json"
            path.write_text(json.dumps(result))
            self.assertEqual(evidence.load_evidence(path, "performance", False, "b").verdict, "FAIL")
            self.assertEqual(evidence.load_evidence(path, "performance", False, "a").verdict, "ERROR")
            (root / "final_binary").write_text("tampered")
            self.assertEqual(evidence.load_evidence(path, "performance", False, "b").verdict, "ERROR")

    def test_stage_b_cli_missing_inputs_writes_error_with_stage_b_title(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            result = subprocess.run([sys.executable, str(Path(__file__).with_name("evaluate_pass17.py")),
                "--stage", "b", "--candidate", str(root / "missing"), "--baseline", str(root / "missing"),
                "--out-json", str(root / "result.json"), "--out-md", str(root / "result.md")],
                capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertEqual(json.loads((root / "result.json").read_text())["stage"], "b")
            self.assertIn("Stage B", (root / "result.md").read_text())

    def test_stage_b_requires_i5_and_rejects_stage_a_manifest(self) -> None:
        # Given: a hash-bound seven-check Stage B identity manifest.
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            artifact = root / "proof.log"
            artifact.write_text("synthetic identity proof\n")
            check = {"passed": True, "evidence": [{"path": "proof.log", "sha256": evidence.digest(artifact)}]}
            payload = {"schema_version": 1, "stage": "b", "verdict": "PASS", "passed": True,
                "errors": [], "checks": {name: check for name in ("I1", "I2", "I3", "I4", "I5", "v19_pins", "v20_pins")}}
            path = root / "identity.json"
            path.write_text(json.dumps(payload))
            # When/Then: Stage B accepts all seven, but not the six-check variant or wrong stage.
            self.assertEqual(evidence.load_evidence(path, "identity", False, "b").verdict, "PASS")
            del payload["checks"]["I5"]
            path.write_text(json.dumps(payload))
            self.assertEqual(evidence.load_evidence(path, "identity", False, "b").verdict, "ERROR")
            payload["stage"] = "a"
            path.write_text(json.dumps(payload))
            self.assertEqual(evidence.load_evidence(path, "identity", False, "b").verdict, "ERROR")

    def test_stage_b_cli_registration_uses_disjoint_exact_cohorts(self) -> None:
        # Given/When: selecting the registered Stage B cohorts, without running seeds.
        candidate, baseline = cli.cohort_specs("b", False)
        tuning, tuning_baseline = cli.cohort_specs("b", True)
        # Then: scenes, complete seed bounds and durations match the brief.
        self.assertEqual((candidate.scene, baseline.scene), ("game_v22", "game_v21"))
        self.assertEqual(candidate.seeds, tuple(range(18000, 18032)))
        self.assertEqual(candidate.days, 120)
        self.assertEqual(tuning.seeds, tuple(range(18100, 18116)))
        self.assertEqual(tuning.days, 60)
        self.assertEqual(tuning_baseline.seeds, tuning.seeds)


if __name__ == "__main__":
    unittest.main()
