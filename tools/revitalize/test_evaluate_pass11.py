"""Unit tests of evaluate_pass11's J helpers on synthetic per-seed blocks."""
import unittest
from types import SimpleNamespace

import evaluate_pass11 as e


def round_of(runs, duration=120):
    return SimpleNamespace(summary={"runs": runs}, duration=duration)


def run(seed, fissions=0, attempts=0, cancelled=0, first=None, alive=None, winters=(), nights=(), settlements=()):
    return {"seed": seed, "seed_info": {"fission": {"max_settlements": 4}},
            "settlements_alive_daily": alive or [1] * 120,
            "pass11": {"fissions": fissions, "fission_attempts": attempts, "fissions_cancelled": cancelled,
                       "fission_blocked_no_site": 0, "first_fission_day": first, "remnant_merges": 0,
                       "exclusivity_moves": 0, "exposure_by_winter": list(winters),
                       "exposure_settlement_nights": list(nights), "settlements": list(settlements),
                       "max_hours_with_1_to_2_members": 0}}


class Pass11Tests(unittest.TestCase):
    def test_percentile_interpolates(self):
        self.assertEqual(e.percentile([30, 58, 86], .5), 58)
        self.assertEqual(e.percentile([28, 56], .25), 35)

    def test_stage_pool_counts_only_winters_with_a_death(self):
        winters = [{"child": 2, "adult": 1, "elder": 0, "elders_at_start": 0},
                   {"child": 0, "adult": 0, "elder": 0, "elders_at_start": 3},
                   {"child": 0, "adult": 1, "elder": 1, "elders_at_start": 2}]
        pool = e.stage_pool([round_of([run(1, winters=winters)])])
        self.assertEqual(pool["seed_winters_with_death"], 2)
        self.assertEqual(pool["total"], 5)
        self.assertAlmostEqual(pool["share"]["adult"], 0.4)
        self.assertAlmostEqual(pool["elder_share_where_elders_present"], 0.5)

    def test_j3_judges_fission_share_timing_and_cancellations(self):
        alive2 = [1] * 60 + [2] * 60
        runs = [run(1, 1, 1, 0, 30, alive2), run(2, 1, 2, 1, 58, alive2), run(3, 0, 0, 0, None), run(4, 1, 1, 0, 86, alive2)]
        targets, rows = e.j3(round_of(runs))
        c = targets["clauses"]
        self.assertTrue(c["share_with_fission"]["passed"])  # 0.75
        self.assertTrue(c["first_fission_median"]["passed"])  # 58
        self.assertTrue(c["first_fission_iqr"]["passed"])  # 72 - 44 = 28
        self.assertTrue(c["two_alive_day_end"]["passed"])  # 0.75
        self.assertTrue(c["cancellations"]["passed"])  # 1 of 4 attempts = 0.25
        self.assertEqual(len(rows), 4)

    def test_j4_counts_daughters_alive_after_28_days(self):
        daughters = [{"settlement_id": 2, "parent": 1, "founders": 10, "alive_28_days_after_founding": True, "dissolved_tick": None},
                     {"settlement_id": 3, "parent": 1, "founders": 8, "alive_28_days_after_founding": False, "dissolved_tick": 9},
                     {"settlement_id": 4, "parent": 2, "founders": 9, "alive_28_days_after_founding": None, "dissolved_tick": None},
                     {"settlement_id": 1, "parent": None, "founders": None, "alive_28_days_after_founding": True, "dissolved_tick": None}]
        targets, detail = e.j4(round_of([run(1, settlements=daughters)]))
        self.assertFalse(targets["clauses"]["daughters_viable"]["passed"])  # 1 of 2 judged
        self.assertEqual((detail["daughters"], detail["judged_28d"], detail["dissolved_or_merged"]), (3, 2, 1))


if __name__ == "__main__":
    unittest.main()
