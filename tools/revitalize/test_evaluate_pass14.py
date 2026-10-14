"""Tests of evaluate_pass14.py (the 4.3 amendments) on synthetic Pass 13 reports."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import evaluate_pass14 as ev  # noqa: E402


def clause(mine: float, ref: float, passed: bool = False) -> dict:
    return {"passed": passed, "observed": {"v19": {"median": mine}, "game_v16": {"median": ref}}}


def report(work1500: float, idle: float, other_passed: bool = True) -> dict:
    return {"seeds": {}, "targets": {"J6": {"passed": False, "clauses": {
        "Y9_work_1200": clause(0.36, 0.37, True), "Y9_work_1500": clause(work1500, 0.24),
        "Y9_idle_1200": clause(idle, 0.62), "Y4_store_share": {"passed": other_passed}}}}}


class J6(unittest.TestCase):
    def test_within_tolerance_passes_and_pass13_kept(self):
        r = ev.rescore_j6(report(0.211, 0.649))
        self.assertTrue(r["passed"])
        self.assertFalse(r["pass13_passed"])
        self.assertAlmostEqual(r["clauses"]["Y9_work_1500"]["worse_by"], 0.029)
        self.assertAlmostEqual(r["clauses"]["Y9_idle_1200"]["worse_by"], 0.029)
        self.assertFalse(r["clauses"]["Y9_work_1500"]["absolute_bound_reported"]["v19n_meets"])

    def test_beyond_tolerance_fails_in_either_direction(self):
        self.assertFalse(ev.rescore_j6(report(0.205, 0.62))["passed"])
        self.assertFalse(ev.rescore_j6(report(0.24, 0.655))["passed"])
        self.assertTrue(ev.rescore_j6(report(0.30, 0.50))["passed"])  # better than v16 passes

    def test_other_clauses_keep_their_verdict(self):
        r = ev.rescore_j6(report(0.24, 0.62, other_passed=False))
        self.assertFalse(r["passed"])
        self.assertEqual(r["failing_clauses_amended"], ["Y4_store_share"])


class J7(unittest.TestCase):
    def test_b_reported_only(self):
        ps = lambda m: {"PS_mean": m, "ci90": {"PS": {"mean": [m - .01, m + .01]}}}
        paths = {"scenes": {"game_v19": {"vars": {"alive": ps(0.02), "food_stock": ps(0.03)}}},
                 "diffs": {"game_v19 - game_v18": {"vars": {"alive": ps(-0.25)}}}}
        div = {"scenes": {"game_v19": {"j7": {"passed": True, "clauses": {}}}}}
        r = ev.j7(paths, div)
        self.assertTrue(r["passed"])
        self.assertFalse(r["b_reported"]["band_met"]["alive"])
        self.assertTrue(r["b_reported"]["v19_below_v18"])


if __name__ == "__main__":
    unittest.main()
