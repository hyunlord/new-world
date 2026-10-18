"""Unit tests of evaluate_pass12's J helpers on synthetic per-seed blocks."""
import unittest
from types import SimpleNamespace

import evaluate_pass12 as e


def round_of(runs, duration=120):
    return SimpleNamespace(summary={"runs": runs}, duration=duration)


def run(seed, fissions=0, attempts=0, first=None, alive=None, checks=(), winters=(), settlements=(), variety=None):
    return {"seed": seed, "seed_info": {"fission": {"max_settlements": 4}, "world_variety": variety or {"starting_agents": seed}},
            "settlements_alive_daily": alive or [1] * 120,
            "pass11": {"fissions": fissions, "fission_attempts": attempts, "fissions_cancelled": 0,
                       "fission_blocked_no_site": 0, "first_fission_day": first, "remnant_merges": 0,
                       "exclusivity_moves": 0, "exposure_by_winter": [], "exposure_settlement_nights": [],
                       "settlements": list(settlements), "max_hours_with_1_to_2_members": 0},
            "pass12": {"fission_checks": [list(c) for c in checks], "settlement_winters": list(winters),
                       "region_fill_bushes": [0, 0, 0, 0], "region_fill_deadwood": 0}}


def check(h, split):
    return (0, 1, 30, 30, 0, 0, 0, 0, h, 0, split, 1)


class Pass12Tests(unittest.TestCase):
    def test_j3_converts_the_zero_based_day_and_judges_the_peak(self):
        alive2 = [1] * 60 + [2] * 60
        # Zero-based 28 x 3 -> display day 29 holds 3 of 4 first fissions: the peak clause fails.
        runs = [run(1, 1, 1, 28, alive2), run(2, 1, 1, 28, alive2), run(3, 1, 1, 28, alive2), run(4, 1, 1, 60, alive2), run(5)]
        targets, extra = e.j3(round_of(runs))
        c = targets["clauses"]
        self.assertEqual(c["first_fission_median"]["observed"]["median"], 29.0)
        self.assertFalse(c["first_fission_no_peak"]["passed"])
        self.assertEqual(c["first_fission_no_peak"]["observed"]["day"], 29.0)
        self.assertFalse(c["first_fission_iqr"]["passed"])
        self.assertEqual(extra["first_fission_day_histogram"], {"29": 3, "61": 1})
        # Spread days pass the peak and IQR clauses.
        runs = [run(i, 1, 1, d, alive2) for i, d in enumerate([20, 35, 50, 65], 1)] + [run(9)]
        c = e.j3(round_of(runs))[0]["clauses"]
        self.assertTrue(c["first_fission_no_peak"]["passed"] and c["first_fission_iqr"]["passed"] and c["first_fission_median"]["passed"])

    def test_hazard_terciles_split_rate(self):
        checks = [check(h, int(h >= 100)) for h in (0, 0, 10, 50, 60, 70, 100, 110, 120)]
        t = e.hazard_terciles(round_of([run(1, checks=checks)]))
        self.assertEqual((t["low"]["split_rate"], t["mid"]["split_rate"], t["high"]["split_rate"]), (0.0, 0.0, 1.0))
        self.assertEqual(t["n"], 9)

    def test_terciles_do_not_sort_ties_by_outcome(self):
        # Six checks at h = 10, three of them splits, ordered so the splits come first in tick order.
        checks = [(t, 1, 30, 30, 0, 0, 0, 0, 10, 0, int(t < 3), 1) for t in range(6)]
        t = e.hazard_terciles(round_of([run(1, checks=checks)]))
        self.assertEqual(t["low"]["split_rate"], 1.0)  # ties keep tick order, not split order

    def test_pearson(self):
        self.assertAlmostEqual(e.pearson([1, 2, 3, 4], [2, 4, 6, 8]), 1.0)
        self.assertIsNone(e.pearson([1, 1, 1], [1, 2, 3]))

    def test_settlement_winter_bound(self):
        winters = [{"first_day": 22, "settlements": [
            {"settlement": 1, "members_at_winter_start": 30, "exposure_deaths": 3, "founded_inside": False},
            {"settlement": 2, "members_at_winter_start": 10, "exposure_deaths": 4, "founded_inside": False}]}]
        sw = e.settlement_winters([round_of([run(1, winters=winters)])])
        self.assertEqual([round(w["ratio"], 2) for w in sw], [0.1, 0.4])

    def test_j4_by_founding_season(self):
        daughters = [{"settlement_id": 2, "parent": 1, "founders": 10, "founded_day": 28, "alive_28_days_after_founding": True, "dissolved_tick": None},
                     {"settlement_id": 3, "parent": 1, "founders": 8, "founded_day": 14, "alive_28_days_after_founding": False, "dissolved_tick": 9},
                     {"settlement_id": 4, "parent": 1, "founders": 8, "founded_day": 16, "alive_28_days_after_founding": True, "dissolved_tick": None}]
        targets, detail = e.j4(round_of([run(1, settlements=daughters)]))
        self.assertFalse(targets["clauses"]["daughters_viable"]["passed"])  # 2 of 3 < 0.70
        self.assertEqual(detail["by_founding_season"]["autumn"]["daughters"], 2)
        self.assertEqual(detail["by_founding_season"]["autumn"]["dissolved_or_merged"], 1)
        self.assertEqual(detail["by_founding_season"]["spring"]["alive_28d_share"], 1.0)


if __name__ == "__main__":
    unittest.main()
