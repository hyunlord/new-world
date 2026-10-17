"""WM-1.1 diversity tests (GOAL_1_1.md 7, 11.4). numpy only, synthetic arrays.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import importlib.util
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
from wm1 import calcore as cc

CALCORE = Path(__file__).resolve().parent / "calcore.py"


class DiversityTests(unittest.TestCase):
    def test_eta2_is_zero_for_identical_episodes(self) -> None:
        base = np.random.default_rng(0).standard_normal((300, 8))
        v = np.repeat(base[None], 5, axis=0)
        np.testing.assert_allclose(cc.eta2(v), 0.0, atol=1e-12)
        self.assertEqual(cc.divergence_rows(v[0], v[1])[0], -1)

    def test_eta2_for_independent_noise_around_a_constant(self) -> None:
        e, t = 50, 1000
        v = 3.0 + np.random.default_rng(1).standard_normal((e, t, 8))
        np.testing.assert_allclose(cc.eta2(v, first=0), (e - 1) / e, atol=0.02)
        np.testing.assert_allclose(cc.eta2(v), (e - 1) / e, atol=0.02)

    def test_divergence_day_of_a_pair(self) -> None:
        a = np.zeros((500, 8))
        b = a.copy()
        b[100:, 3] = 1.0
        first, per = cc.divergence_rows(a, b)
        self.assertEqual(first, 100)
        self.assertEqual(per[3], 100)
        self.assertEqual(per[0], -1)
        summ = cc.divergence_summary([a, b, a.copy()], tuple("abcdefgh"))
        self.assertEqual(summ["pairs"], 3)
        self.assertEqual(summ["never"], 1)
        self.assertEqual(summ["min_day"], int(cc.display_day(100)))
        self.assertEqual(int(cc.display_day(100)), 5)   # row 100 = tick 6000 -> day 5 (08:00 + 100 h)

    def test_r2_of_the_reference_itself_is_one(self) -> None:
        m = np.random.default_rng(2).standard_normal((300, 8))
        v = np.repeat(m[None], 3, axis=0)
        np.testing.assert_allclose(cc.r2(v, m), 1.0)

    def test_eta2_within_equal_severity_groups(self) -> None:
        rng = np.random.default_rng(3)
        t = 400
        shift = np.array([0.0, 5.0])
        v = np.stack([shift[g] + 0.01 * rng.standard_normal((t, 8)) for g in (0, 0, 1, 1)])
        e2w, used = cc.eta2_within_groups(v, [0, 0, 1, 1])
        self.assertEqual(used, 4)
        self.assertLess(float(e2w.max()), 0.01)
        self.assertGreater(float(cc.eta2(v).min()), 0.9)

    def test_calcore_loads_by_file_path(self) -> None:
        """The v1 entry point loads calcore by path (diversity_v1.py is never imported from a unittest process)."""
        spec = importlib.util.spec_from_file_location("calcore_by_path", CALCORE)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        v = np.repeat(np.arange(300.0)[None, :, None], 4, axis=0) * np.ones((1, 1, 2))
        v[1] += 1.0
        v[3] += 1.0
        # Two pairs of identical episodes at offsets 0 and 1: per-row variance 0.25, total variance 0.25 + var(rows).
        rows = np.arange(24, 300.0)
        expected = 0.25 / (0.25 + rows.var())
        np.testing.assert_allclose(mod.eta2(v), expected, rtol=1e-12)
        self.assertEqual(set(mod.__dict__) & {"wm1", "wm_data", "bench_v0"}, set())


if __name__ == "__main__":
    unittest.main()
