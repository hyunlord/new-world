"""Unit tests of macro_sensitivity's shape comparison and row arithmetic."""
import unittest

import macro_sensitivity as m


class MacroTests(unittest.TestCase):
    def test_row_of_matches_the_recorder_clock(self):
        # Row r is tick 60 r; display day d 08:00 is tick (d - 1) * 1440 + 480 - 480.
        self.assertEqual(m.row_of(1, 480), 0)
        self.assertEqual(m.row_of(60, 480), 59 * 24)

    def test_shapes_differ_on_counts_or_first_fission_day(self):
        a = {"settlements": [2, 3, 3], "first_fission_day": 29, "alive_118": 100.0}
        self.assertFalse(m.differs(a, dict(a, alive_118=90.0)))
        self.assertTrue(m.differs(a, dict(a, settlements=[2, 3, 4])))
        self.assertTrue(m.differs(a, dict(a, first_fission_day=57)))
        self.assertTrue(m.differs(a, dict(a, first_fission_day=None)))

    def test_post_fork_shapes_and_delta_days(self):
        a = {"settlements": [2, 2, 2], "post_fork_first_fission_day": 40, "first_fission_day": 30, "alive_118": 80.0}
        self.assertFalse(m.differs_post(a, dict(a, first_fission_day=29)))
        self.assertTrue(m.differs_post(a, dict(a, post_fork_first_fission_day=55)))
        d = m.delta_day([(a, dict(a, post_fork_first_fission_day=55)), (a, dict(a, post_fork_first_fission_day=None)), (a, a)])
        self.assertEqual((d["n_both_split_after_fork"], d["mean_abs_delta_day"]), (2, 7.5))
        self.assertAlmostEqual(d["share_with_post_fork_day_differing"], 2 / 3)

    def test_j11_day_rule_and_signed_shift(self):
        # Pass 13: none in both equal; one and none differs; more than 3 days differs.
        self.assertFalse(m.day_differs(None, None))
        self.assertTrue(m.day_differs(40, None))
        self.assertFalse(m.day_differs(40, 43))
        self.assertTrue(m.day_differs(40, 44))
        shape = lambda day, n=2, alive=80.0: {"settlements_118": n, "alive_118": alive, "post_fork_day": day}
        self.assertEqual(m.signed_shifts([(shape(35), shape(40)), (shape(None), shape(40)), (shape(45), shape(40))]), [-5, 5])
        self.assertTrue(m.j11_pair_differs(shape(40, n=3), shape(40)))
        self.assertFalse(m.j11_pair_differs(shape(41), shape(40)))

    def test_j11_verdicts(self):
        seeds = range(0, 20)
        shape = lambda day, n=2, alive=80.0: {"settlements_118": n, "alive_118": alive, "post_fork_day": day}
        shapes = {}
        for s in seeds:
            day = 40 + s
            for k in range(2):
                shapes[(s, "none", k)] = shape(day + k)          # noise moves the day by 1
                shapes[(s, "spoil_food", k)] = shape(day + k - 6, n=3)  # earlier by 6, one more village
                shapes[(s, "gift_food", k)] = shape(day + k + 6, n=1)   # later by 6
                shapes[(s, "rot_wood", k)] = shape(day + k)       # no effect
        r = m.build_j11(shapes, "game_v19", seeds, ["rot_wood", "spoil_food", "gift_food"], 2, 28800)
        j = r["j11"]
        self.assertEqual(j["a"]["share_day_differs_gt3"]["value"], 0.0)
        self.assertEqual(j["a"]["mean_abs_delta_day"]["value"], 1.0)
        self.assertEqual(r["variants"]["spoil_food"]["signed_shift_mean"], -6.0)
        self.assertTrue(j["b"]["spoil_food_earlier"]["passed"] and j["b"]["gift_food_later"]["passed"])
        self.assertEqual(j["c"]["variants_over"], ["spoil_food", "gift_food"])
        self.assertTrue(j["passed"])
        # Fewer than 12 pairs where both split: the mean is "not measured" and J11 does not pass.
        small = {key: v for key, v in shapes.items() if key[0] < 5}
        r = m.build_j11(small, "game_v19", range(0, 5), ["rot_wood", "spoil_food", "gift_food"], 2, 28800)
        self.assertIsNone(r["j11"]["a"]["mean_abs_delta_day"]["value"])
        self.assertIn("mean_abs_delta_day", r["j11"]["not_measured"])
        self.assertFalse(r["j11"]["passed"])


if __name__ == "__main__":
    unittest.main()
