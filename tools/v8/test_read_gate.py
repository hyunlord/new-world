"""Unit tests for the M-READ gate verifiers (read_gate_checks.py).
Each rule must be able to fail."""
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import read_gate_checks  # noqa: E402


class LoadAdmissionTest(unittest.TestCase):
    def test_returns_exact_admitted_triplet_without_second_read(self):
        from unittest.mock import patch
        with patch.object(read_gate_checks.os, "getloadavg", side_effect=[(7.9, 9.0, 10.0), (8.1, 9.0, 10.0)]) as load:
            self.assertEqual(read_gate_checks.wait_for_load(), (7.9, 9.0, 10.0))
            self.assertEqual(load.call_count, 1)

    def test_each_interleaved_repeat_uses_one_admitted_sample(self):
        import tempfile
        from unittest.mock import patch
        report = {"scenes": [{"label": label, "tick_mean_us": 1.0, "tick_p99_us": 2.0}
                              for label in read_gate_checks.BENCH_SCENES],
                  "ceilings": {key: "PASS" for key in read_gate_checks.BENCH_CEILINGS}}
        with (
            tempfile.TemporaryDirectory() as folder,
            patch.object(read_gate_checks, "load_config", return_value={"base_perf": "/base", "base_perf_sha256": "a" * 64, "base_commit": "a" * 40}),
            patch.object(read_gate_checks, "verify_binary"),
            patch.object(read_gate_checks.Path, "exists", return_value=True),
            patch.object(read_gate_checks, "load_json", side_effect=[dict(report, commit=sha) for sha in ["a" * 40, "b" * 40] * 3]),
            patch.object(read_gate_checks.subprocess, "check_output", return_value="b" * 40 + "\n"),
            patch.object(read_gate_checks.os, "getloadavg", side_effect=[(7.0, 4.0, 3.0)] * 6) as loads,
            patch.object(read_gate_checks.subprocess, "check_call") as run,
        ):
            read_gate_checks.bench("unused", folder)
            self.assertEqual(loads.call_count, 6)
            self.assertEqual([call.args[0][0] for call in run.call_args_list],
                             ["/base", str(read_gate_checks.ROOT / "rust/target/release/worldsim_perf")] * 3)
            for call, sha in zip(run.call_args_list, ["a" * 40, "b" * 40] * 3):
                self.assertEqual(call.kwargs["env"]["WORLDSIM_GIT_SHA"], sha)
                self.assertEqual(call.args[0][-2:], ["--commit", sha])

    def test_waits_at_boundary_and_rejects_invalid_samples(self):
        from unittest.mock import patch
        with patch.object(read_gate_checks.os, "getloadavg", side_effect=[(8.0, 4.0, 3.0), (7.0, 4.0, 3.0)]), patch.object(read_gate_checks.time, "sleep") as sleep:
            self.assertEqual(read_gate_checks.wait_for_load(), (7.0, 4.0, 3.0))
            sleep.assert_called_once_with(30)
        for sample in ((True, 0, 0), (float("nan"), 0, 0), (1, -1, 0)):
            with self.subTest(sample=sample), patch.object(read_gate_checks.os, "getloadavg", return_value=sample):
                with self.assertRaises(ValueError):
                    read_gate_checks.wait_for_load()


class BenchRatioTest(unittest.TestCase):
    def test_invalid_base_identity_stops_before_measurement(self):
        from unittest.mock import patch
        for value in (None, "unknown", "a" * 8, True):
            with (
                self.subTest(value=value),
                patch.object(read_gate_checks, "load_config", return_value={"base_perf": "/base", "base_perf_sha256": "a" * 64, "base_commit": value}),
                patch.object(read_gate_checks, "verify_binary"),
                patch.object(read_gate_checks.subprocess, "check_output", return_value="b" * 40),
                patch.object(read_gate_checks.subprocess, "check_call") as run,
            ):
                with self.assertRaisesRegex(ValueError, "full base_commit"):
                    read_gate_checks.bench("unused", "unused")
                run.assert_not_called()

    def test_wrong_arm_summary_identity_is_rejected(self):
        import tempfile
        from unittest.mock import patch
        with (
            tempfile.TemporaryDirectory() as folder,
            patch.object(read_gate_checks, "load_config", return_value={"base_perf": "/base", "base_perf_sha256": "a" * 64, "base_commit": "a" * 40}),
            patch.object(read_gate_checks, "verify_binary"),
            patch.object(read_gate_checks.Path, "exists", return_value=True),
            patch.object(read_gate_checks, "wait_for_load", return_value=(1.0, 2.0, 3.0)),
            patch.object(read_gate_checks.subprocess, "check_output", return_value="b" * 40),
            patch.object(read_gate_checks.subprocess, "check_call"),
            patch.object(read_gate_checks, "load_json", return_value={"commit": "b" * 40}),
        ):
            with self.assertRaisesRegex(ValueError, "summary commit does not match"):
                read_gate_checks.bench("unused", folder)

    def test_pass_within_limit(self):
        ok, ratio = read_gate_checks.check_bench_ratio(100.0, 108.0)
        self.assertTrue(ok)
        self.assertAlmostEqual(ratio, 1.08, places=2)

    def test_fail_over_limit(self):
        ok, ratio = read_gate_checks.check_bench_ratio(100.0, 115.0)
        self.assertFalse(ok, "ratio 1.15 must fail the 1.10 limit")
        self.assertAlmostEqual(ratio, 1.15, places=2)

    def test_exact_boundary(self):
        ok, _ = read_gate_checks.check_bench_ratio(100.0, 110.0)
        self.assertTrue(ok, "exactly 1.10 must pass")

    def test_improvement_passes(self):
        ok, ratio = read_gate_checks.check_bench_ratio(100.0, 90.0)
        self.assertTrue(ok)
        self.assertAlmostEqual(ratio, 0.90, places=2)

    def test_zero_base_is_invalid(self):
        with self.assertRaises(ValueError):
            read_gate_checks.check_bench_ratio(0.0, 50.0)


class StallWaiverTest(unittest.TestCase):
    def test_all_under_50_passes(self):
        ok, count, mx = read_gate_checks.check_stall_frames([30.0, 40.0, 49.9])
        self.assertTrue(ok)
        self.assertEqual(count, 0)

    def test_two_over_50_passes(self):
        ok, count, mx = read_gate_checks.check_stall_frames([52.0, 55.0, 30.0])
        self.assertTrue(ok)
        self.assertEqual(count, 2)

    def test_three_over_50_fails(self):
        ok, count, mx = read_gate_checks.check_stall_frames([51.0, 52.0, 53.0])
        self.assertFalse(ok, "3 frames over 50 must fail")
        self.assertEqual(count, 3)

    def test_one_over_60_fails(self):
        ok, count, mx = read_gate_checks.check_stall_frames([61.0])
        self.assertFalse(ok, "a frame over 60 must fail")

    def test_stall_waiver_accepts_report_with_one_52ms(self):
        """The stall waiver must accept a report with one 52 ms post-warmup frame."""
        import tempfile
        report = {
            "complete": True,
            "arms": [
                {"zoom": 1.0, "capture_complete": True, "samples": [
                    {"interval_ms": 3000.0, "telemetry": {"buildings": {"render_warmup": {"complete": False}}}},
                    {"interval_ms": 12.0, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}},
                    {"interval_ms": 52.8, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}},
                    {"interval_ms": 30.0, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}},
                ]},
                {"zoom": 0.5, "capture_complete": True, "samples": [
                    {"interval_ms": 4000.0, "telemetry": {"buildings": {"render_warmup": {"complete": False}}}},
                    {"interval_ms": 11.0, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}},
                    {"interval_ms": 48.9, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}},
                ]},
            ],
        }
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(report, f)
            path = f.name
        read_gate_checks.stall_waiver(path)  # must not raise
        Path(path).unlink()

    def test_stall_waiver_rejects_over_60(self):
        """The stall waiver must reject a post-warmup frame > 60 ms."""
        import tempfile
        report = {
            "complete": True,
            "arms": [{"zoom": 1.0, "capture_complete": True, "samples": [
                {"interval_ms": 10.0, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}},
                {"interval_ms": 61.0, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}},
            ]}],
        }
        report["arms"].append({"zoom": 0.5, "capture_complete": True, "samples": [
            {"interval_ms": 10.0, "telemetry": {"buildings": {"render_warmup": {"complete": True}}}}]})
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(report, f)
            path = f.name
        with self.assertRaises(SystemExit):
            read_gate_checks.stall_waiver(path)
        Path(path).unlink()

    def test_empty_fails(self):
        ok, count, mx = read_gate_checks.check_stall_frames([])
        self.assertFalse(ok)
        self.assertEqual(mx, 0)


class NormalisedEquivalenceTest(unittest.TestCase):
    def test_identical_files_match(self):
        import tempfile, json
        data = {"seed": 0, "values": [1, 2, 3]}
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as a, \
             tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as b:
            json.dump(data, a)
            json.dump(data, b)
            a_path, b_path = a.name, b.name
        # read_normalise.py returns 0 for identical files
        import subprocess
        result = subprocess.run(
            [sys.executable, str(Path(__file__).parent / "read_normalise.py"), a_path, b_path],
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        Path(a_path).unlink()
        Path(b_path).unlink()

    def test_different_files_fail(self):
        import tempfile, json
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as a, \
             tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as b:
            json.dump({"x": 1}, a)
            json.dump({"x": 2}, b)
            a_path, b_path = a.name, b.name
        import subprocess
        result = subprocess.run(
            [sys.executable, str(Path(__file__).parent / "read_normalise.py"), a_path, b_path],
            capture_output=True, text=True,
        )
        self.assertNotEqual(result.returncode, 0, "different files must fail")
        Path(a_path).unlink()
        Path(b_path).unlink()

    def test_unsupported_keys_preserved(self):
        import tempfile, json
        data_a = {"x": 1, "why_not": {"a": 1}}
        data_b = {"x": 1, "why_not": {"b": 2}}
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as a, \
             tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as b:
            json.dump(data_a, a)
            json.dump(data_b, b)
            a_path, b_path = a.name, b.name
        import subprocess
        result = subprocess.run(
            [sys.executable, str(Path(__file__).parent / "read_normalise.py"), a_path, b_path],
            capture_output=True, text=True,
        )
        self.assertNotEqual(result.returncode, 0, "unsupported why_not path must remain comparable")
        Path(a_path).unlink()
        Path(b_path).unlink()


class AdmissionCoverageTest(unittest.TestCase):
    def report(self):
        return {"scenes": [{"label": label, "tick_mean_us": 1.0, "tick_p99_us": 2.0}
                           for label in sorted(read_gate_checks.BENCH_SCENES)],
                "ceilings": dict.fromkeys(read_gate_checks.BENCH_CEILINGS, "PASS")}

    def test_literal_pass_and_complete_scenes(self):
        read_gate_checks.validate_bench_report(self.report())
        for verdict in ("MISS", "n/a", True, None):
            report = self.report()
            report["ceilings"]["n1000_mean_le_5ms"] = verdict
            with self.assertRaises(ValueError):
                read_gate_checks.validate_bench_report(report)

    def test_missing_ceiling_scene_or_invalid_timing_fails(self):
        for defect in ("ceiling", "scene", "timing", "duplicate"):
            report = self.report()
            if defect == "ceiling":
                report["ceilings"] = {}
            elif defect == "scene":
                report["scenes"].pop()
            elif defect == "duplicate":
                report["scenes"][0] = report["scenes"][1]
            else:
                report["scenes"][0]["tick_mean_us"] = float("nan")
            with self.assertRaises(ValueError):
                read_gate_checks.validate_bench_report(report)

    def test_empty_arms_and_missing_zoom_fail(self):
        import tempfile
        for arms in ([], [{"zoom": 1.0}], [{"zoom": 1.0}, {"zoom": 1.0}]):
            with tempfile.TemporaryDirectory() as folder:
                path = Path(folder) / "stall.json"
                path.write_text(json.dumps({"complete": True, "arms": arms}))
                with self.assertRaises(SystemExit):
                    read_gate_checks.stall_waiver(path)

    def test_invalid_and_empty_frames_fail(self):
        for frames in ([], [float("nan")], [True], [0], [float("inf")]):
            self.assertFalse(read_gate_checks.check_stall_frames(frames)[0])



class RelativeColonyFixtureTest(unittest.TestCase):
    @staticmethod
    def summary(name="story", alive=10, losses=2):
        days, count = {"story": (48, 2), "colony_u": (10, 4),
                       "knowledge": (4, 8)}[name]
        return {"scene": "colony", "days": days, "seeds": [
            {"seed": seed, "days": days, "alive_end": alive,
             "ledger_violation_ticks": 0, "reservation_violation_ticks": 0,
             "m4": {"start": {"adults": 10},
                    "lost": [[100 + n, "node", 1] for n in range(losses)]}}
            for seed in range(count)]}

    def test_base_losses_are_relative_and_survival_boundary_is_inclusive(self):
        for base_alive, candidate_alive, passes in ((10, 8, True), (10, 7, False),
                                                    (5, 3, True), (5, 2, False),
                                                    (18, 15, False), (18, 16, True), (0, 0, True)):
            with self.subTest(base=base_alive, candidate=candidate_alive):
                base = self.summary(alive=base_alive)
                candidate = self.summary(alive=candidate_alive)
                if passes:
                    read_gate_checks.compare_fixture("story", base, candidate, None, {"story"})
                else:
                    with self.assertRaisesRegex(ValueError, "survivors"):
                        read_gate_checks.compare_fixture("story", base, candidate, None, {"story"})

    def test_extra_loss_fails_per_seed_not_just_total(self):
        base, candidate = self.summary(), self.summary()
        candidate["seeds"][0]["m4"]["lost"].append([200, "extra", 1])
        candidate["seeds"][1]["m4"]["lost"] = []
        with self.assertRaisesRegex(ValueError, "KnowledgeLost"):
            read_gate_checks.compare_fixture("story", base, candidate, None, {"story"})

    def test_bad_seed_horizon_start_and_required_fields_fail(self):
        changes = [
            lambda s: s["seeds"].pop(),
            lambda s: s["seeds"].append(s["seeds"][0]),
            lambda s: s["seeds"][0].update(seed=99),
            lambda s: s.update(days=47),
            lambda s: s["seeds"][0].update(days=47),
            lambda s: s["seeds"][0].update(alive_end=True),
            lambda s: s["seeds"][0]["m4"]["start"].update(adults=9),
            lambda s: s["seeds"][0]["m4"].pop("lost"),
            lambda s: s["seeds"][0]["m4"].update(lost=""),
            lambda s: s["seeds"][0]["m4"].update(lost=[[999999, "node", 1]]),
            lambda s: s["seeds"][0].pop("ledger_violation_ticks"),
            lambda s: s["seeds"][0].update(reservation_violation_ticks=1),
        ]
        for change in changes:
            with self.subTest(change=change):
                base, candidate = self.summary(), self.summary()
                change(candidate)
                with self.assertRaises(ValueError):
                    read_gate_checks.compare_fixture("story", base, candidate, None, {"story"})

    def test_baseline_and_undeclared_colony_remain_strict(self):
        for name, declared in (("baseline", {"baseline"}), ("story", set())):
            with self.subTest(name=name):
                base, candidate = self.summary(), self.summary(alive=9)
                with self.assertRaisesRegex(ValueError, "MISMATCH"):
                    read_gate_checks.compare_fixture(name, base, candidate, None, declared)

    def test_core_and_knowledge_require_explicit_declaration(self):
        for name in ("colony_u", "knowledge"):
            with self.subTest(name=name):
                base, candidate = self.summary(name), self.summary(name, alive=8)
                read_gate_checks.compare_fixture(name, base, candidate, None, {name})
                with self.assertRaisesRegex(ValueError, "MISMATCH"):
                    read_gate_checks.compare_fixture(name, base, candidate, None, set())

    def test_cli_reads_paired_files_and_fails_extra_loss(self):
        import os
        import subprocess
        import tempfile
        with tempfile.TemporaryDirectory() as folder:
            base_path, head_path = Path(folder) / "base.json", Path(folder) / "head.json"
            base_path.write_text(json.dumps(self.summary()))
            env = dict(os.environ, READ_CHANGED_FIXTURES="story")
            env.pop("READ_GATE_CONFIG", None)
            for losses, expected in ((2, 0), (3, 1)):
                head_path.write_text(json.dumps(self.summary(losses=losses)))
                run = subprocess.run(
                    [sys.executable, str(Path(__file__).with_name("read_gate_checks.py")),
                     "fixture_pair", "story", str(base_path), str(head_path)],
                    env=env, capture_output=True, text=True, check=False)
                self.assertEqual(run.returncode, expected, run.stdout + run.stderr)

    def test_malformed_base_cannot_lower_acceptance_bar(self):
        base, candidate = self.summary(), self.summary(alive=0)
        base["seeds"][0]["alive_end"] = -1
        with self.assertRaisesRegex(ValueError, "alive_end"):
            read_gate_checks.compare_fixture("story", base, candidate, None, {"story"})

    def test_seed_order_does_not_change_relative_pairing(self):
        base, candidate = self.summary(), self.summary()
        candidate["seeds"].reverse()
        read_gate_checks.compare_fixture("story", base, candidate, None, {"story"})


class PairedEconomySoakTest(unittest.TestCase):
    def run_soak(self, paired, *, idle=False, extra_loss=False, deaths=True):
        import hashlib
        import os
        import subprocess
        import tempfile
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            probes = []
            for label in ("base", "head"):
                summary = RelativeColonyFixtureTest.summary("colony_u", alive=9, losses=1)
                summary["days"] = 20
                for row in summary["seeds"]:
                    row.update(days=20, workshops_built=2, crafts_done=1, trees_felled=1,
                               first_violation=None, deaths={"combat": int(deaths)})
                    if idle and label == "head":
                        row["crafts_done"] = 0
                    if extra_loss and label == "head":
                        row["m4"]["lost"].append([200, "extra", 1])
                source = root / f"{label}.json"
                source.write_text(json.dumps(summary))
                probe = root / label
                probe.write_text("#!/usr/bin/env python3\nimport json,sys,pathlib\n"
                                 "args=sys.argv[1:]\n"
                                 "assert args[args.index('--story')+1]=='no-beasts'\n"
                                 "assert '--check-determinism' in args\n"
                                 "assert args[args.index('--days')+1]=='20'\n"
                                 "assert args[args.index('--seeds')+1]=='0..4'\n"
                                 "out=pathlib.Path(args[args.index('--out')+1]);out.mkdir(parents=True,exist_ok=True)\n"
                                 "(out/'summary.json').write_text(pathlib.Path(__file__+'.json').read_text())\n"
                                 "print('DETERMINISM PASS')\n")
                probe.chmod(0o755)
                probes.append(probe)
            config = root / "config.json"
            config.write_text(json.dumps({"schema_version": 1, "base_probe": str(probes[0]),
                "base_probe_sha256": hashlib.sha256(probes[0].read_bytes()).hexdigest(),
                "base_perf": "unused", "base_perf_sha256": "unused"}))
            env = dict(os.environ, READ_GATE_CONFIG=str(config),
                       READ_CHANGED_FIXTURES="soak_economy" if paired else "")
            command = ["bash", str(Path(__file__).with_name("soak.sh")),
                       str(root / "out"), str(probes[1])]
            if paired:
                command.append(str(probes[0]))
            return subprocess.run(command, env=env, capture_output=True, text=True, check=False)

    def test_default_keeps_absolute_death_and_loss_rejections(self):
        for deaths in (True, False):
            run = self.run_soak(False, deaths=deaths)
            self.assertNotEqual(run.returncode, 0, run.stdout + run.stderr)
            self.assertIn("KnowledgeLost", run.stdout)
            if deaths:
                self.assertIn("1 deaths", run.stdout)

    def test_paired_relative_losses_and_deaths_pass(self):
        run = self.run_soak(True)
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn("BASE-relative", run.stdout)

    def test_paired_path_still_rejects_idle_colony_and_extra_loss(self):
        for flags in ({"idle": True}, {"extra_loss": True}):
            run = self.run_soak(True, **flags)
            self.assertNotEqual(run.returncode, 0, run.stdout + run.stderr)

if __name__ == "__main__":
    unittest.main()
