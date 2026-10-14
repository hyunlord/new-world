"""Unit tests of evaluate_pass13's J helpers on synthetic per-seed blocks."""
import unittest
from types import SimpleNamespace

import evaluate_pass13 as e


def round_of(runs, duration=120):
    return SimpleNamespace(summary={"runs": runs}, duration=duration)


def run(seed, first=None, checks=(), settlements=(), half=(), splits=()):
    return {"seed": seed, "seed_info": {"fission": {"max_settlements": 4}, "world_variety": {"starting_agents": seed}},
            "settlements_alive_daily": [1] * 120,
            "pass11": {"fissions": int(first is not None), "fission_attempts": int(first is not None), "fissions_cancelled": 0,
                       "fission_blocked_no_site": 0, "first_fission_day": first, "remnant_merges": 0,
                       "exclusivity_moves": 0, "exposure_by_winter": [], "exposure_settlement_nights": [],
                       "settlements": list(settlements), "max_hours_with_1_to_2_members": 0},
            "pass12": {"fission_checks": [], "settlement_winters": [], "region_fill_bushes": [0, 0, 0, 0], "region_fill_deadwood": 0},
            "pass13": {"pressure_checks": [list(c) for c in checks], "half_threshold_day": [list(h) for h in half],
                       "splits": [list(s) for s in splits], "area_floor_placed": 0, "area_floor_lost_at_max": 0}}


class Pass13Tests(unittest.TestCase):
    def test_first_winter_end(self):
        # Winter = display days 21..27, 49..55, ...
        self.assertEqual(e.first_winter_end(0), 28)
        self.assertEqual(e.first_winter_end(22), 28)
        self.assertEqual(e.first_winter_end(28), 56)
        self.assertEqual(e.display_day(1440 * 5 - 480), 5)

    def test_j4_upper_by_season(self):
        # Spring founding (day 28) dissolved on day 50 (inside its first winter 49..55);
        # autumn founding (day 14) dissolved on day 40 (after its first winter 21..27);
        # summer founding (day 35) alive.
        tick = lambda day: day * 1440 - 480 + 600
        daughters = [{"settlement_id": 2, "parent": 1, "founders": 8, "founded_day": 28, "alive_28_days_after_founding": True, "dissolved_tick": tick(50)},
                     {"settlement_id": 3, "parent": 1, "founders": 8, "founded_day": 14, "alive_28_days_after_founding": True, "dissolved_tick": tick(40)},
                     {"settlement_id": 4, "parent": 1, "founders": 8, "founded_day": 35, "alive_28_days_after_founding": True, "dissolved_tick": None}]
        up = e.j4_upper(round_of([run(1, settlements=daughters)]))
        self.assertEqual((up["spring"]["dissolved_or_merged"], up["spring"]["dissolved_in_first_winter"]), (1, 1))
        self.assertEqual((up["autumn"]["dissolved_or_merged"], up["autumn"]["dissolved_in_first_winter"]), (1, 0))
        self.assertEqual((up["summer"]["dissolved_or_merged"], up["all"]["share"]), (0, 2 / 3))

    def test_j3_pressure_correlations(self):
        # Camp θ rises with the first-fission day; early scarcity falls with it.
        runs = []
        for seed, (theta, s, day) in enumerate([(300, 900, 20), (600, 600, 40), (900, 300, 60), (1200, 0, 80)], 1):
            checks = [(t, 1, 10 * t, theta, 50, 500, 0, s) for t in range(8)] + [(0, 2, 5, 77, 1, 1, 1, 1000)]
            runs.append(run(seed, first=day - 1, checks=checks, half=[(1, day - 5)], splits=[(1, 1, 0, theta + 10, theta, 2, 10, 0)]))
        out = e.j3_pressure(round_of(runs))
        self.assertAlmostEqual(out["first_fission_day_correlation"]["camp_theta"]["r"], 1.0)
        self.assertAlmostEqual(out["first_fission_day_correlation"]["camp_early_scarcity"]["r"], -1.0)
        self.assertEqual(out["rows"][0]["camp_eligible_checks"], 8)
        self.assertEqual(out["splits"], 4)
        self.assertGreater(out["pressure_over_threshold_at_split"]["median"], 1.0)


if __name__ == "__main__":
    unittest.main()
