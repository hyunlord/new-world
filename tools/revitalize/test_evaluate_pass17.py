from __future__ import annotations

import unittest
import json
import tempfile
import subprocess
import sys
from dataclasses import replace
from pathlib import Path

import numpy as np

import evaluate_pass17_statistics as stats
import evaluate_pass17_data as data
import evaluate_pass17_stage_a as stage_a
import evaluate_pass17_read as reader
import evaluate_pass17_evidence as evidence
import evaluate_pass17_report as report


class Pass17StatisticsTests(unittest.TestCase):
    def test_average_tie_ranks_match_hand_calculation(self) -> None:
        # Given: two tied observations.
        values = [10.0, 20.0, 20.0, 40.0]
        # When: assigning average ranks.
        result = stats.rank_average(values)
        # Then: neither tie receives arbitrary order priority.
        np.testing.assert_array_equal(result, [1.0, 2.5, 2.5, 4.0])

    def test_spearman_uses_average_ranks(self) -> None:
        # Given: a monotonic sequence containing a tie.
        pairs = [(10.0, 1.0), (20.0, 2.0), (20.0, 3.0), (40.0, 4.0)]
        # When: calculating Spearman rho.
        result = stats.spearman(pairs)
        # Then: Pearson on the average ranks gives this exact value.
        self.assertAlmostEqual(result, 0.9486832980505138)

    def test_constant_behavior_is_undefined_not_zero(self) -> None:
        # Given: no observed behavior variance.
        pairs = [(0.0, 0.0), (500.0, 0.0), (1000.0, 0.0)]
        # When/Then: an explicit error prevents an invented zero estimate.
        with self.assertRaises(stats.UndefinedStatistic):
            stats.spearman(pairs)

    def test_pooled_ratio_uses_exposure_not_mean_world_ratio(self) -> None:
        # Given: one small and one large world with different exposure.
        worlds = [(1.0, 1.0), (0.0, 9.0)]
        # When: pooling the numerator and denominator.
        result = stats.pooled_ratio(worlds)
        # Then: the primary rate is 1/10, not the mean world rate 1/2.
        self.assertAlmostEqual(result, 0.1)

    def test_zero_exposure_is_an_explicit_error(self) -> None:
        # Given: an empty eligibility denominator.
        worlds = [(0.0, 0.0)]
        # When/Then: undefined data cannot pass a ratio threshold.
        with self.assertRaises(stats.UndefinedStatistic):
            stats.pooled_ratio(worlds)

    def test_world_bootstrap_matches_rng_zero_and_whole_world_draws(self) -> None:
        # Given: three differently sized worlds, never individual resampling.
        worlds = [(1.0, 1.0), (2.0, 4.0), (7.0, 10.0)]
        rng = np.random.default_rng(0)
        expected = []
        for _ in range(2000):
            selected = [worlds[int(i)] for i in rng.integers(0, 3, size=3)]
            expected.append(sum(x for x, _ in selected) / sum(y for _, y in selected))
        # When: running the registered 2,000-replicate bootstrap.
        result = stats.bootstrap(worlds, stats.pooled_ratio)
        # Then: percentile bounds and counts equal the explicit reference loop.
        self.assertEqual(result.value, 10.0 / 15.0)
        self.assertEqual(result.valid_draws, 2000)
        self.assertEqual(result.replicates, 2000)
        np.testing.assert_allclose(result.interval, np.percentile(expected, [2.5, 97.5]))


def person(person_id: int, adult_ticks: int = 14400, completed: int = 3) -> data.Person:
    return data.Person(
        agent_id=person_id, temperament=(100 * person_id, 500, 100 * person_id, 100 * person_id),
        adult_ticks=adult_ticks, social_completed=3,
        activity_minutes=(0, 0, 0, 0, 0, 0, 0), activities_started=(0, 0, 0, 0, 0, 0, 0),
        completed_activities=completed, completed_minutes=90 * completed,
        no_place_redraws=0, no_place_failures=0,
    )


class Pass17EligibilityTests(unittest.TestCase):
    def test_zero_activity_people_remain_in_ns_and_rd(self) -> None:
        people = (person(1), person(2, adult_ticks=14399), person(3))
        result = stage_a.routine_pairs(people, "ns")
        self.assertEqual(result, [(100.0, 0.0), (300.0, 0.0)])
        self.assertEqual(stage_a.routine_pairs(people, "rd"), result)

    def test_persistence_requires_three_completed_activities(self) -> None:
        people = (person(1), person(2, completed=2), person(3, adult_ticks=14399))
        result = stage_a.routine_pairs(people, "p")
        self.assertEqual(result, [(100.0, 90.0)])

    def test_r1_pools_adult_observations_in_exact_hour_window(self) -> None:
        hours = (
            data.Hour(1, 420, ((90, 0, 0, 0, 10), (0,) * 5, (0,) * 5), 20, 20),
            data.Hour(1, 480, ((1, 0, 0, 0, 1), (99, 0, 0, 0, 0), (99, 0, 0, 0, 0)), 20, 20),
            data.Hour(1, 1020, ((1, 0, 0, 0, 7), (0,) * 5, (0,) * 5), 20, 20),
            data.Hour(1, 1080, ((90, 0, 0, 0, 10), (0,) * 5, (0,) * 5), 20, 20),
        )
        result = stage_a.idle_exposure(hours)
        self.assertEqual(result, (2.0, 10.0))


def write_cohort(root: Path, scene: str) -> Path:
    root.mkdir(parents=True)
    runs = []
    for seed in range(3):
        stem = root / f"{scene}_s{seed}"
        agents = []
        for i in range(6):
            enabled = scene == "game_v21"
            agents.append({"id": i, "ever_adult": True, "temperament": {axis: 200 * i for axis in data.AXES},
                "adult_ticks": 14400, "adult_days": 10.0, "social_completed": 3,
                "routine": {"activities_started": [2] * 7 if enabled else [0] * 7,
                    "activity_minutes": [0, 600 - 100 * i, 0, 50 * i, 50 * i, 0, 0] if enabled else [0] * 7,
                    "completed_activities": 3 if enabled else 0,
                    "completed_minutes": 3 * (30 + 30 * i) if enabled else 0,
                    "no_place_redraws": 0, "no_place_failures": 0}})
        Path(f"{stem}_agents.jsonl").write_text("".join(json.dumps(row) + "\n" for row in agents))
        Path(f"{stem}_people.jsonl").write_text("".join(json.dumps(row) + "\n" for row in agents))
        hours = [{"tick": (day - 1) * 1440 + minute, "day": day, "minute": minute,
            "states": {"adult": [2, 0, 0, 0, 4] if scene == "game_v21" else [6, 0, 0, 0, 0],
                "child": [0] * 5, "elder": [0] * 5}, "stockpile_food": 100, "stockpile_wood": 80}
            for day in range(1, 13) for minute in range(0, 1440, 60)]
        Path(f"{stem}_hours.jsonl").write_text("".join(json.dumps(row) + "\n" for row in hours))
        Path(f"{stem}_trace_meta.json").write_text(json.dumps({"end_tick": 12 * 1440, "people": 6, "traced_adults": 6,
            "pass17": {"schema_version": 17, "shelters_completed": 5, "no_place_redraws": 0,
                "no_place_failures": 0, "observation_misses": 0, "activity_kinds": data.KINDS}}))
        runs.append({"seed": seed, "alive_daily": [6] * 12,
            "death_census": {"total": 3, "counts_by_cause": {"old_age": 2, "starvation": 1}}})
    (root / "summary.json").write_text(json.dumps({"scene": scene,
        "seeds": {"start": 0, "end_exclusive": 3}, "days": {str(d): {} for d in range(1, 13)},
        "determinism_checked": True, "runs": runs}))
    return root


class Pass17InputTests(unittest.TestCase):
    def test_activity_started_as_child_and_completed_as_adult_loads(self) -> None:
        row: dict[str, reader.JsonValue] = {
            "id": 7, "temperament": {axis: 500 for axis in data.AXES},
            "adult_ticks": 30, "adult_days": 30 / 1440, "social_completed": 0,
            "routine": {"activities_started": [0] * 7, "activity_minutes": [0, 0, 0, 0, 0, 30, 0],
                "completed_activities": 1, "completed_minutes": 30,
                "no_place_redraws": 0, "no_place_failures": 0},
        }
        parsed = reader.parse_person(row)
        self.assertEqual(parsed.completed_activities, 1)
        self.assertEqual(sum(parsed.activities_started), 0)
        self.assertEqual(parsed.completed_minutes, 30)
        self.assertEqual(stage_a.routine_pairs((parsed,), "p"), [])
        for minutes, completed, duration in ((31, 1, 30), (30, 1, 31), (30, 0, 30)):
            row["routine"] = {"activities_started": [0] * 7, "activity_minutes": [0, 0, 0, 0, 0, minutes, 0],
                "completed_activities": completed, "completed_minutes": duration,
                "no_place_redraws": 0, "no_place_failures": 0}
            with self.subTest(minutes=minutes, completed=completed, duration=duration):
                with self.assertRaises(data.InputError):
                    reader.parse_person(row)

    def test_complete_synthetic_cohort_loads_exact_numeric_values(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = write_cohort(Path(temporary) / "v21", "game_v21")
            cohort = reader.load_cohort(root, data.CohortSpec("game_v21", (0, 1, 2), 12))
            self.assertEqual(len(cohort.worlds), 3)
            self.assertEqual(cohort.worlds[0].non_old_age_deaths, 1)
            self.assertEqual(cohort.worlds[0].shelters_completed, 5)

    def test_missing_activity_metric_is_error_not_zero(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = write_cohort(Path(temporary) / "v21", "game_v21")
            path = root / "game_v21_s0_agents.jsonl"
            rows = [json.loads(line) for line in path.read_text().splitlines()]
            del rows[0]["routine"]["activity_minutes"]
            path.write_text("".join(json.dumps(row) + "\n" for row in rows))
            with self.assertRaisesRegex(data.InputError, "activity_minutes"):
                reader.load_cohort(root, data.CohortSpec("game_v21", (0, 1, 2), 12))

    def test_missing_ever_adult_person_is_error_not_cohort_selection(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = write_cohort(Path(temporary) / "v21", "game_v21")
            path = root / "game_v21_s0_agents.jsonl"
            path.write_text("\n".join(path.read_text().splitlines()[1:]) + "\n")
            with self.assertRaisesRegex(data.InputError, "ever-adult inventory"):
                reader.load_cohort(root, data.CohortSpec("game_v21", (0, 1, 2), 12))


class Pass17EvaluationTests(unittest.TestCase):
    def test_complete_stage_a_has_numeric_r1_r2_r3_and_failing_bar(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            candidate = reader.load_cohort(write_cohort(root / "v21", "game_v21"), data.CohortSpec("game_v21", (0, 1, 2), 12))
            baseline = reader.load_cohort(write_cohort(root / "v20", "game_v20"), data.CohortSpec("game_v20", (0, 1, 2), 12))
            result = stage_a.evaluate(candidate, baseline)
            self.assertAlmostEqual(result.metrics["R1"].value, 1 / 3)
            self.assertEqual(result.verdicts["R2"], "PASS")
            self.assertEqual(result.verdicts["R3-P"], "PASS")
            self.assertEqual(result.verdicts["R3-NS"], "PASS")
            self.assertEqual(result.verdicts["R3-RD"], "PASS")
            self.assertEqual(result.metrics["R2-social"].candidate, 0.3)
            self.assertEqual(result.metrics["R2-social"].value, 1.0)

    def test_relative_comparison_rejects_zero_baseline(self) -> None:
        with self.assertRaises(stats.UndefinedStatistic):
            stage_a.relative_change([(1.0, 0.0), (2.0, 0.0)])

    def test_relative_comparison_preserves_inclusive_decimal_boundaries(self) -> None:
        self.assertEqual(stage_a.relative_change([(11.0, 10.0)]), 0.1)
        self.assertEqual(stage_a.relative_change([(17.0, 20.0)]), -0.15)

    def test_failed_bar_retains_value_and_interval(self) -> None:
        criterion = data.Criterion(0.1, 0.45, "0.10 <= share <= 0.45", "share")
        result = stage_a.measure([(9.0, 10.0)] * 3, stats.pooled_ratio, criterion)
        self.assertEqual(result.verdict, "FAIL")
        self.assertEqual(result.value, 0.9)
        self.assertEqual(result.ci95, (0.9, 0.9))

    def test_missing_hour_is_error_even_when_noon_still_exists(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = write_cohort(Path(temporary) / "v21", "game_v21")
            path = root / "game_v21_s0_hours.jsonl"
            path.write_text("\n".join(path.read_text().splitlines()[1:]) + "\n")
            with self.assertRaisesRegex(data.InputError, "hour"):
                reader.load_cohort(root, data.CohortSpec("game_v21", (0, 1, 2), 12))

    def test_social_exposure_pools_worlds_before_comparing(self) -> None:
        world = data.World(0, 1, 0, 0, 1, (person(1),), (), 0, 0, (0,) * 7, (0,) * 7)
        small = replace(world, people=(replace(person(1), adult_ticks=1440, social_completed=1),))
        large = replace(world, seed=1, people=(replace(person(1), adult_ticks=12960, social_completed=0),))
        baseline = replace(world, people=(replace(person(1), adult_ticks=1440, social_completed=1),))
        self.assertEqual(stage_a.social_ratio([(small, baseline), (large, baseline)]), 0.1)


class Pass17EvidenceTests(unittest.TestCase):
    def test_missing_proofs_are_pending_only_in_tuning(self) -> None:
        for kind in ("identity", "performance"):
            self.assertEqual(evidence.load_evidence(None, kind, True).verdict, "PENDING")
            self.assertEqual(evidence.load_evidence(None, kind, False).verdict, "ERROR")

    def test_identity_requires_every_check_and_hash_bound_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            artifact = root / "proof.log"
            artifact.write_text("verified synthetic proof\n")
            check = {"passed": True, "evidence": [{"path": "proof.log", "sha256": evidence.digest(artifact)}]}
            payload = {"schema_version": 1, "stage": "a", "verdict": "PASS", "passed": True,
                "errors": [], "checks": {name: check for name in ("I1", "I2", "I3", "I4", "v19_pins", "v20_pins")}}
            path = root / "identity.json"
            path.write_text(json.dumps(payload))
            self.assertEqual(evidence.load_evidence(path, "identity", False).verdict, "PASS")
            artifact.write_text("changed\n")
            self.assertEqual(evidence.load_evidence(path, "identity", False).verdict, "ERROR")

    def test_nonfinite_json_is_rejected_before_reporting(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "bad.json"
            path.write_text('{"unused": NaN}')
            with self.assertRaises(data.InputError):
                reader.read_json(path)

    def test_perf_numeric_result_cannot_claim_pass_with_changed_limit(self) -> None:
        payload = {"checks": {name: {"value": 0, "limit": limit, "operator": op, "passed": True}
            for name, limit, op in (("mean_relative", 0.2, "<="), ("mean_absolute", 4, "<="),
                ("p999_relative", 0.35, "<="), ("slope", 1.7, "<"))}}
        with self.assertRaisesRegex(data.InputError, "threshold"):
            evidence.verify_performance(payload)


class Pass17ReportTests(unittest.TestCase):
    def test_report_contains_all_rules_hourly_life_stages_and_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            candidate = reader.load_cohort(write_cohort(root / "v21", "game_v21"), data.CohortSpec("game_v21", (0, 1, 2), 12))
            baseline = reader.load_cohort(write_cohort(root / "v20", "game_v20"), data.CohortSpec("game_v20", (0, 1, 2), 12))
            pending = evidence.load_evidence(None, "identity", True)
            result = report.build_report(candidate, baseline, stage_a.evaluate(candidate, baseline), pending, pending, True)
            report.write_report(result, root / "out.json", root / "out.md")
            loaded = json.loads((root / "out.json").read_text())
            self.assertEqual(loaded["verdict"], "PENDING")
            self.assertEqual(len(loaded["candidate"]["hourly_states"]), 72)
            self.assertEqual(len(loaded["candidate"]["inputs_sha256"]), 13)
            self.assertEqual(len(loaded["candidate"]["activity_minutes"]), 7)
            self.assertIn("daughter-camp", (root / "out.md").read_text())

    def test_cli_missing_inputs_writes_error_artifacts_and_exits_two(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            result = subprocess.run([sys.executable, str(Path(__file__).with_name("evaluate_pass17.py")),
                "--stage", "a", "--candidate", str(root / "missing"), "--baseline", str(root / "missing"),
                "--out-json", str(root / "result.json"), "--out-md", str(root / "result.md")],
                capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertEqual(json.loads((root / "result.json").read_text())["verdict"], "ERROR")
            self.assertIn("ERROR", (root / "result.md").read_text())


if __name__ == "__main__":
    unittest.main()
