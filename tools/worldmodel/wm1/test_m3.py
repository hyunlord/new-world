"""M3 serialization tests (numpy only; the LoRA training itself runs on the DGX).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import unittest

import numpy as np

import wm1  # noqa: F401
from wm1 import evaluate as ev
from wm1 import m3_text as m
from wm1.test_data import sample_episode


class M3SerializationTests(unittest.TestCase):
    def test_day_lines_round_trip_and_cover_the_episode(self) -> None:
        ep = sample_episode(0, interventions=[{"tick": ev.row_of(4, 600) * 60, "kind": "rain", "settlement": 1,
                                               "amount": 6, "applied": 6}])
        days = m.days_of(ep)
        self.assertEqual(days[0], 2)
        self.assertTrue(ev.row_of(days[-1] + 1, 360) < ep.rows)
        line = m.day_line(ep, 3)
        rec = json.loads(line)
        self.assertEqual(rec["day"], 3)
        np.testing.assert_allclose(m.parse(line), np.round(ep.bench.v[ev.row_of(3, 360)]))
        # The rain requested at day 4 10:00 belongs to day 4's line (06:00 day 4 .. 06:00 day 5).
        self.assertEqual(json.loads(m.day_line(ep, 4))["iv"], [["rain", 6]])
        self.assertEqual(json.loads(m.day_line(ep, 3))["iv"], [])
        # Day numbers arriving as numpy integers (bench start rows) still serialize.
        self.assertEqual(json.loads(m.day_line(ep, np.int64(3))), rec)
        self.assertIsNone(m.parse("not json"))
        self.assertIsNone(m.parse('{"food": 1}'))


if __name__ == "__main__":
    unittest.main()
