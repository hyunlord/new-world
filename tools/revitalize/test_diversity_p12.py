"""Synthetic checks of diversity_p12 (PASS12_GOAL.md 5.2): static-only linear worlds give UBS ~ 0, worlds with an
independent random walk after day 10 give UBS > 0.3, a nonlinear static map is (by design) partly unexplained."""
import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import diversity_p12 as d12  # noqa: E402

T = 2881  # 120 days, every 60 minutes


class Ep:
    def __init__(self, seed, v, variety):
        self.seed, self.v, self.variety = seed, v, variety


def worlds(kind: str, n: int = 112, seed: int = 1) -> list:
    rng = np.random.default_rng(seed)
    days = d12.cc.display_day(np.arange(T))
    eps = []
    for e in range(n):
        p1, p2 = rng.normal(), rng.normal()
        climate = int(rng.integers(0, 3))
        level = 5.0 * p1 - 3.0 * p2 + 2.0 * climate
        v = np.zeros((T, 8))
        for j in range(8):
            base = 50.0 + level * (1 + 0.1 * j)
            series = np.full(T, base) + rng.normal(0, 0.5, T)
            if kind == "nonlinear":
                # A static effect that only appears after day 10 (so the early level cannot carry it) and is
                # quadratic in a hidden parameter: a linear ridge leaves it unexplained.
                series = series + np.where(days > 10, 10.0 * p1 * p1, 0.0)
            if kind == "walk":
                steps = rng.normal(0, 3.0, 121)
                walk = np.concatenate([[0.0], np.cumsum(steps)])
                series = series + np.where(days > 10, walk[np.minimum(days, 120)] - walk[10], 0.0)
            v[:, j] = series
        eps.append(Ep(1000 + e, v, {"a": p1, "b": p2, "constant": 7, "climate": climate}))
    return eps


class UbsTest(unittest.TestCase):
    def test_day_rows_are_one_based_display_days(self):
        v = np.arange(T, dtype=float)[:, None].repeat(8, axis=1)
        m = d12.day_means(v, (1, 30))
        rows1 = np.flatnonzero(d12.cc.display_day(np.arange(T)) == 1)
        self.assertEqual(rows1[0], 0)
        self.assertEqual(rows1[-1], 15)  # 08:00 + 16 h = 23:00 of day 1
        self.assertAlmostEqual(m[0, 0], rows1.mean())
        rows30 = [r for r in range(T) if (60 * r + 480) // 1440 + 1 == 30]
        self.assertAlmostEqual(m[1, 0], float(np.mean(rows30)))

    def test_variety_features_drop_climate_and_one_hot_it(self):
        x, names = d12.variety_matrix([{"a": 1, "climate": 2}, {"a": 3, "climate": 0}])
        self.assertEqual(names, ["a", "climate_0", "climate_1", "climate_2"])
        self.assertEqual(x.tolist(), [[1, 0, 0, 1], [3, 1, 0, 0]])

    def test_loo_matches_explicit_refit(self):
        rng = np.random.default_rng(3)
        x = rng.normal(size=(30, 3))
        y = x @ np.array([1.0, -2.0, 0.5]) + rng.normal(size=30)
        fast = d12.loo_predict(x, y, 10.0)
        z = (x - x.mean(axis=0)) / x.std(axis=0)
        xa = np.hstack([np.ones((30, 1)), z])
        pen = 10.0 * np.eye(4)
        pen[0, 0] = 0
        for i in range(30):
            keep = np.arange(30) != i
            b = np.linalg.solve(xa[keep].T @ xa[keep] + pen, xa[keep].T @ y[keep])
            self.assertAlmostEqual(fast[i], xa[i] @ b, places=9)

    def test_static_linear_worlds_have_ubs_near_zero(self):
        block = d12.score_scene(d12.Scene(worlds("static")))
        for name in d12.VARS:
            self.assertLess(block["ubs_mean"][name], 0.05, name)
            self.assertGreater(block["r2_static_mean"][name], 0.99, name)

    def test_random_walk_after_day_ten_gives_ubs_above_0_3(self):
        block = d12.score_scene(d12.Scene(worlds("walk")))
        for name in d12.VARS:
            self.assertGreater(block["ubs_mean"][name], 0.3, name)

    def test_nonlinear_static_map_is_partly_unexplained(self):
        block = d12.score_scene(d12.Scene(worlds("nonlinear")))
        ubs = block["ubs_mean"]["alive"]
        print(f"\nnonlinear static map (+10 p1^2 after day 10): UBS(alive) = {ubs:.3f}, R2_static = {block['r2_static_mean']['alive']:.3f}")
        self.assertGreater(ubs, 0.2)

    def test_paired_bootstrap_separates_walk_from_static(self):
        a, b = d12.Scene(worlds("static", 40)), d12.Scene(worlds("walk", 40))
        ba, bb = d12.score_scene(a), d12.score_scene(b)
        bt = d12.paired_bootstrap(a, ba, b, bb, n_boot=40)
        self.assertTrue(bt["above_zero"]["alive"])
        same = d12.paired_bootstrap(a, ba, a, ba, n_boot=20)
        self.assertEqual(same["ci90"]["alive"], [0.0, 0.0])


if __name__ == "__main__":
    unittest.main()
