"""Unit tests of pilot_p13's per-episode helpers on synthetic settlement arrays."""
import unittest

import numpy as np

import pilot_p13 as p

NAMES = ["members", "food_stock", "fission_threshold", "depart_tick"]


class FakeEp:
    def __init__(self, data):
        self.meta = {"fields": {"settlements.npy": [{"name": n} for n in NAMES]}}
        self.arrays = {"settlements.npy": data}

    def column(self, array, name):
        return self.arrays[array][..., NAMES.index(name)]


def ep(rows=2880, slots=2):
    data = np.full((rows, slots, len(NAMES)), np.nan)
    data[:, 0, 0] = 30.0      # members
    data[:, 0, 1] = 144.0     # food: half of 30 x 2.4 x 4 = 288 -> s = 500
    data[:, 0, 2] = 0.0
    data[5:, 0, 2] = 900.0    # theta drawn when the camp formed
    data[:, 0, 3] = -1.0
    return data


class PilotTests(unittest.TestCase):
    def test_first_fission_and_censoring(self):
        d = ep()
        self.assertIsNone(p.first_fission_day(FakeEp(d)))
        d[2000:, 1, 3] = 40 * 1440 - 480 + 360  # departed 06:00 of zero-based day 40
        d[2000:, 1, 0] = 10.0
        self.assertEqual(p.first_fission_day(FakeEp(d)), 41)

    def test_camp_theta_and_early_scarcity(self):
        d = ep()
        self.assertEqual(p.camp_theta(FakeEp(d)), 900.0)
        self.assertAlmostEqual(p.early_scarcity(FakeEp(d)), 500.0)
        d[:, 0, 0] = 20.0  # below min_members: no eligible day
        self.assertIsNone(p.early_scarcity(FakeEp(d)))

    def test_pearson(self):
        self.assertAlmostEqual(p.pearson([1, 2, 3], [3, 2, 1]), -1.0)
        self.assertIsNone(p.pearson([1, 1, 1], [1, 2, 3]))


if __name__ == "__main__":
    unittest.main()
