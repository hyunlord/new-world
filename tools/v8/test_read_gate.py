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



class BenchJustifiedTest(unittest.TestCase):
    def dispatch(self, declaration, ratio=1.15, metric="tick_mean_us", scene="n2000", ceiling="PASS", feature_first=False):
        import io
        import tempfile
        from contextlib import redirect_stdout
        from unittest.mock import patch
        config = {"base_perf": "/base", "base_perf_sha256": "a" * 64, "base_commit": "a" * 40}
        if feature_first:
            config["evaluation_mode"] = "feature_first"
        if declaration != "ABSENT":
            config["bench_justified"] = declaration
        reports = []
        for arm in ("base", "new") * 3:
            report = {"commit": ("a" if arm == "base" else "b") * 40,
                      "scenes": [{"label": name, "tick_mean_us": 100.0, "tick_p99_us": 100.0}
                                 for name in sorted(read_gate_checks.BENCH_SCENES)],
                      "ceilings": {key: ceiling for key in read_gate_checks.BENCH_CEILINGS}}
            if arm == "new":
                next(row for row in report["scenes"] if row["label"] == scene)[metric] = round(100.0 * ratio, 9)
            reports.append(report)
        output = io.StringIO()
        with (tempfile.TemporaryDirectory() as folder,
              patch.object(read_gate_checks, "ROOT", Path(folder)),
              patch.object(read_gate_checks, "load_config", return_value=config),
              patch.object(read_gate_checks, "verify_binary"),
              patch.object(read_gate_checks.Path, "exists", return_value=True),
              patch.object(read_gate_checks, "wait_for_load", return_value=(1.0, 2.0, 3.0)),
              patch.object(read_gate_checks.subprocess, "check_output", return_value="b" * 40),
              patch.object(read_gate_checks.subprocess, "check_call") as run,
              patch.object(read_gate_checks, "load_json", side_effect=reports),
              redirect_stdout(output)):
            (Path(folder) / "CHANGELOG.md").write_text("Measured survival policy remainder.\n")
            try:
                read_gate_checks.bench("unused", folder)
            except ValueError:
                if ceiling != "FAIL":
                    run.assert_not_called()
                raise
            self.assertEqual(run.call_count, 6)
        return output.getvalue()

    def declaration(self, **changes):
        return {"scene": "n2000", "metric": "tick_mean_us",
                "quote": "Measured survival policy remainder.", **changes}

    def test_default_and_single_metric_limits_through_real_dispatch(self):
        for declaration, ratio, metric, scene in (
            ("ABSENT", 1.15, "tick_mean_us", "n2000"),
            (self.declaration(), 1.150001, "tick_mean_us", "n2000"),
            (self.declaration(), 1.11, "tick_p99_us", "n2000"),
            (self.declaration(), 1.11, "tick_mean_us", "n200"),
        ):
            with self.subTest(declaration=declaration, ratio=ratio, metric=metric, scene=scene):
                with self.assertRaises(SystemExit):
                    self.dispatch(declaration, ratio, metric, scene)
        self.assertIn("JUSTIFIED n2000/tick_mean_us", self.dispatch(self.declaration()))
        self.assertNotIn("JUSTIFIED", self.dispatch(self.declaration(), 1.10))
        self.assertIn("READ_BENCH PASS", self.dispatch("ABSENT", 1.10))
        self.assertIn("JUSTIFIED n2000/tick_p99_us",
                      self.dispatch(self.declaration(metric="tick_p99_us"), metric="tick_p99_us"))

    def test_invalid_declaration_rejects_before_any_measurement(self):
        for declaration in (None, False, [], {}, self.declaration(extra="x"),
                            self.declaration(scene="unknown"), self.declaration(metric="rss"),
                            self.declaration(quote=""), self.declaration(quote="  "),
                            self.declaration(quote="absent quote"), self.declaration(quote=1),
                            self.declaration(scene=[]), self.declaration(metric=True)):
            with self.subTest(declaration=declaration), self.assertRaises(ValueError):
                self.dispatch(declaration)

    def test_justification_does_not_override_absolute_or_memory_ceilings(self):
        with self.assertRaisesRegex(ValueError, "bench ceiling"):
            self.dispatch(self.declaration(), ceiling="FAIL")


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
    def run_soak(self, paired, *, idle=False, extra_loss=False, deaths=True, feature_first=False):
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
                "base_perf": "unused", "base_perf_sha256": "unused",
                "evaluation_mode": "feature_first" if feature_first else "legacy"}))
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

    def test_feature_first_soak_reports_losses_but_rejects_idle_colony(self):
        run = self.run_soak(True, extra_loss=True, feature_first=True)
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn("safety in feature smoke", run.stdout)
        run = self.run_soak(True, idle=True, feature_first=True)
        self.assertNotEqual(run.returncode, 0, run.stdout + run.stderr)

    def test_paired_path_still_rejects_idle_colony_and_extra_loss(self):
        for flags in ({"idle": True}, {"extra_loss": True}):
            run = self.run_soak(True, **flags)
            self.assertNotEqual(run.returncode, 0, run.stdout + run.stderr)


class FeatureFirstTest(unittest.TestCase):
    @staticmethod
    def smoke(alive=10, losses=0):
        rows = RelativeColonyFixtureTest.summary("knowledge", alive, losses)
        rows["days"] = 48
        for row in rows["seeds"]:
            row["days"] = 48
        return rows

    def test_explicit_mode_rejects_unknown_values(self):
        import tempfile
        config = {"schema_version": 1, "base_probe": "base", "base_perf": "perf",
                  "base_probe_sha256": "hash", "base_perf_sha256": "hash"}
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "config.json"
            for mode in (None, True, {}, [], "feature-first", ""):
                with self.subTest(mode=mode), self.assertRaises(ValueError):
                    path.write_text(json.dumps(dict(config, evaluation_mode=mode)))
                    read_gate_checks.load_config(path)
            for mode in ("legacy", "feature_first"):
                path.write_text(json.dumps(dict(config, evaluation_mode=mode)))
                self.assertEqual(read_gate_checks.load_config(path)["evaluation_mode"], mode)

    def test_ratios_are_advisory_but_ceilings_still_block(self):
        helper = BenchJustifiedTest()
        self.assertIn("REPORT_ONLY", helper.dispatch("ABSENT", ratio=3, feature_first=True))
        with self.assertRaisesRegex(ValueError, "bench ceiling"):
            helper.dispatch("ABSENT", ratio=3, ceiling="FAIL", feature_first=True)

    def test_smoke_extinction_and_total_loss_boundaries(self):
        for base_alive, new_alive, accepted in ((1, 0, False), (0, 0, True), (10, 1, True)):
            base, candidate = self.smoke(base_alive), self.smoke(new_alive)
            with self.subTest(base=base_alive, new=new_alive):
                if accepted:
                    read_gate_checks.smoke_pair(base, candidate)
                else:
                    with self.assertRaisesRegex(ValueError, "extinction"):
                        read_gate_checks.smoke_pair(base, candidate)
        base, candidate = self.smoke(losses=1), self.smoke()
        candidate["seeds"][0]["m4"]["lost"] = [[i, "node", 1] for i in range(18)]
        read_gate_checks.smoke_pair(base, candidate)  # 18 == 2 * 8 + 2, concentrated in one seed.
        candidate["seeds"][0]["m4"]["lost"].append([20, "node", 1])
        with self.assertRaisesRegex(ValueError, "KnowledgeLost"):
            read_gate_checks.smoke_pair(base, candidate)

    def test_zero_base_loss_allows_two_total_not_two_per_seed(self):
        base, candidate = self.smoke(), self.smoke()
        for row in candidate["seeds"][:2]:
            row["m4"]["lost"] = [[1, "node", 1]]
        read_gate_checks.smoke_pair(base, candidate)
        candidate["seeds"][2]["m4"]["lost"] = [[1, "node", 1]]
        with self.assertRaisesRegex(ValueError, "KnowledgeLost"):
            read_gate_checks.smoke_pair(base, candidate)

    def test_smoke_rejects_incomplete_or_corrupt_evidence(self):
        changes = [lambda s: s["seeds"].pop(),
                   lambda s: s["seeds"].append(s["seeds"][0]),
                   lambda s: s["seeds"][0].update(seed=8),
                   lambda s: s.update(days=47),
                   lambda s: s["seeds"][0].update(days=47),
                   lambda s: s["seeds"][0].update(alive_end=True),
                   lambda s: s["seeds"][0]["m4"].update(lost=[[999999, "node", 1]]),
                   lambda s: s["seeds"][0].update(ledger_violation_ticks=1),
                   lambda s: s["seeds"][0].update(reservation_violation_ticks=1)]
        for change in changes:
            for arm in (0, 1):
                pair = [self.smoke(), self.smoke()]
                change(pair[arm])
                with self.subTest(change=change, arm=arm), self.assertRaises(ValueError):
                    read_gate_checks.smoke_pair(*pair)

    def test_only_declared_colony_outcomes_are_nonblocking(self):
        base = RelativeColonyFixtureTest.summary()
        candidate = RelativeColonyFixtureTest.summary(alive=1, losses=20)
        read_gate_checks.compare_fixture("story", base, candidate, None, {"story"}, feature_first=True)
        for name, declared in (("baseline", {"baseline"}), ("story", set())):
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "MISMATCH"):
                read_gate_checks.compare_fixture(name, base, candidate, None, declared, feature_first=True)
        candidate["seeds"][0]["ledger_violation_ticks"] = 1
        with self.assertRaisesRegex(ValueError, "ledger"):
            read_gate_checks.compare_fixture("story", base, candidate, None, {"story"}, feature_first=True)

    def test_smoke_runner_uses_fixed_on_no_orders_pair(self):
        import tempfile
        from unittest.mock import patch
        config = {"base_probe": "/base", "base_probe_sha256": "hash", "evaluation_mode": "feature_first"}
        with (tempfile.TemporaryDirectory() as folder,
              patch.object(read_gate_checks, "load_config", return_value=config),
              patch.object(read_gate_checks, "verify_binary"),
              patch.object(read_gate_checks.Path, "exists", return_value=True),
              patch.object(read_gate_checks.subprocess, "check_call") as run,
              patch.object(read_gate_checks, "load_json", return_value=self.smoke()),
              patch.object(read_gate_checks, "compare_fixture"),
              patch.object(read_gate_checks, "smoke_pair") as compare,
              patch.object(read_gate_checks.Path, "read_text", return_value="DETERMINISM PASS")):
            read_gate_checks.core_fixture("unused", folder)
            smoke_calls = [c.args[0] for c in run.call_args_list if "0..8" in c.args[0]]
            self.assertEqual(len(smoke_calls), 2)
            self.assertEqual([c[0] for c in smoke_calls], ["/base", str(read_gate_checks.ROOT / "rust/target/release/worldsim_probe")])
            for command in smoke_calls:
                self.assertEqual(command[command.index("--days") + 1], "48")
                self.assertEqual(command[command.index("--steward") + 1], "on")
                self.assertNotIn("--commands", command)
            compare.assert_called_once()

    def test_neighbours_keeps_functional_and_accounting_checks(self):
        import importlib.util
        import tempfile
        source = read_gate_checks.ROOT / "docs/v8/m5_5/m55_score.py"
        spec = importlib.util.spec_from_file_location("m55_score", source)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "summary.json"
            summary = RelativeColonyFixtureTest.summary(alive=1, losses=2)
            for row in summary["seeds"]:
                row.update(deaths={}, first_violation=None, m55={"population": {"start": 10}, "neighbours": {
                    "list": [{"edge": 0}, {"edge": 1}], "traded_out": {}, "tallies": {"trades": 1},
                    "waves": [{"id": 1, "arrived": 9 * 1440, "answer": "Accepted"},
                              {"id": 2, "arrived": 22 * 1440, "answer": "Accepted"}]}})
            path.write_text(json.dumps(summary))
            self.assertEqual(module.main(["--neighbours", str(path), "--feature-first"]), 0)
            self.assertEqual(module.main(["--neighbours", str(path)]), 1)
            summary["seeds"][0]["m55"]["neighbours"]["tallies"]["trades"] = 0
            path.write_text(json.dumps(summary))
            self.assertEqual(module.main(["--neighbours", str(path), "--feature-first"]), 1)
            summary["seeds"][0]["m55"]["neighbours"]["tallies"]["trades"] = 1
            summary["seeds"][0]["ledger_violation_ticks"] = 1
            path.write_text(json.dumps(summary))
            self.assertEqual(module.main(["--neighbours", str(path), "--feature-first"]), 1)

if __name__ == "__main__":
    unittest.main()
