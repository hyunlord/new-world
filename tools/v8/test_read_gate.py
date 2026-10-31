"""Unit tests for the M-READ gate verifiers (read_gate_checks.py).
Each rule must be able to fail."""
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import read_gate_checks  # noqa: E402


class BenchRatioTest(unittest.TestCase):
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

    def test_zero_base_returns_zero(self):
        ok, ratio = read_gate_checks.check_bench_ratio(0.0, 50.0)
        self.assertTrue(ok)  # 0 ratio = no regression
        self.assertEqual(ratio, 0)


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
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(report, f)
            path = f.name
        with self.assertRaises(SystemExit):
            read_gate_checks.stall_waiver(path)
        Path(path).unlink()

    def test_empty_passes(self):
        ok, count, mx = read_gate_checks.check_stall_frames([])
        self.assertTrue(ok)
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

    def test_excluded_keys_ignored(self):
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
        self.assertEqual(result.returncode, 0, f"excluded key 'why_not' must be ignored: {result.stdout}")
        Path(a_path).unlink()
        Path(b_path).unlink()


if __name__ == "__main__":
    unittest.main()
