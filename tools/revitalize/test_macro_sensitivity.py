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


if __name__ == "__main__":
    unittest.main()
