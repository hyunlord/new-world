"""WM-1.1 X-rule tests (METRICS_1_1.md 3): KEEP / WEAK / NONE and skill. numpy only.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest

import wm1  # noqa: F401
from wm1 import judge_1_1 as j


class VerdictTests(unittest.TestCase):
    def test_keep_weak_none(self) -> None:
        keep = {"a": ([0.7, 0.75, 0.8], 1.0), "b": ([0.1, 0.1, 0.1], 0.2)}
        self.assertEqual(j.verdict(keep), "KEEP")
        # Mean <= 0.8 x cal but one seed not below cal -> WEAK.
        self.assertEqual(j.verdict({"a": ([0.3, 0.3, 1.0], 1.0)}), "WEAK")
        # Mean below cal in every cell but above 0.8 x cal in one -> WEAK.
        self.assertEqual(j.verdict({"a": ([0.7, 0.7, 0.7], 1.0), "b": ([0.9, 0.9, 0.9], 1.0)}), "WEAK")
        # One cell's mean at or above cal -> NONE.
        self.assertEqual(j.verdict({"a": ([0.7, 0.7, 0.7], 1.0), "b": ([1.0, 1.0, 1.0], 1.0)}), "NONE")
        # A single model (ridge): its value is the mean and every seed.
        self.assertEqual(j.verdict({"a": ([0.8], 1.0)}), "KEEP")
        self.assertEqual(j.verdict({}), "n/a")

    def test_skill(self) -> None:
        self.assertAlmostEqual(j.skill(0.5, 1.0), 0.5)
        self.assertAlmostEqual(j.skill(1.5, 1.0), -0.5)
        self.assertIsNone(j.skill(0.5, 0.0))
        self.assertIsNone(j.skill(0.5, float("nan")))


if __name__ == "__main__":
    unittest.main()
