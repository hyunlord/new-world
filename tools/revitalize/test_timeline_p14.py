"""Tests of timeline_p14.py and schedules_p14.py on tiny synthetic probe runs."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import schedules_p14 as sp  # noqa: E402
import timeline_p14 as tl  # noqa: E402

D = 1440


def tick(day: int) -> int:
    """06:00 tick of one-based display day `day`."""
    return (day - 1) * D - 120


def check(day: int, members: int, food: int, sid: int = 1) -> list:
    return [tick(day), sid, members, members, food, 0, 0, tl.scarcity(members, food), 0, 0, 0, 1]


def run(seed: int, checks: list, departs: list[int] = (), cancelled: int | None = None) -> dict:
    log = [{"parent": 1, "depart_tick": t, "cancelled_tick": cancelled if i == 0 else None, "daughter": 2 + i}
           for i, t in enumerate(departs)]
    return {"seed": seed, "pass12": {"fission_checks": checks}, "pass11": {"fission_log": log}}


class Timeline(unittest.TestCase):
    def test_line_and_scarcity(self):
        self.assertEqual(tl.food_line(32), 307)  # floor(32 x 2.4 x 4) = floor(307.2)
        self.assertEqual(tl.food_line(0), 1)
        self.assertEqual(tl.scarcity(32, 307), 0)
        self.assertEqual(tl.scarcity(32, 0), 1000)
        self.assertEqual(tl.scarcity(32, 153), 1000 - 153000 // 307)
        self.assertEqual(tl.display_day(29_400), 21)
        self.assertEqual(tl.clock(29_400), "18:00")
        self.assertEqual(tl.display_day(14_280), 11)
        self.assertEqual(tl.clock(14_280), "06:00")

    def test_t_low_before_departure(self):
        # s > 0 on days 15 and 30; departure on day 20: day 15 is >= 3 days before it.
        r = run(1, [check(13, 32, 500), check(15, 32, 200), check(18, 32, 100), check(30, 32, 100)], [tick(20)])
        t = tl.seed_timeline(r, 14_280, 120)
        self.assertEqual(t["T_low"], tick(15))
        self.assertEqual(t["line_check"], [4, 4])

    def test_t_low_three_day_gap_is_inclusive(self):
        r = run(1, [check(17, 32, 100)], [tick(20)])
        self.assertEqual(tl.seed_timeline(r, 14_280, 120)["T_low"], tick(17))
        r = run(1, [check(18, 32, 100)], [tick(20)])
        self.assertIsNone(tl.seed_timeline(r, 14_280, 120)["T_low"])

    def test_t_low_after_first_departure(self):
        # Shortfall only after the first departure: first s > 0 after it and >= 3 days before the next.
        r = run(1, [check(13, 32, 500), check(25, 32, 100), check(33, 32, 100)], [tick(20), tick(30)])
        t = tl.seed_timeline(r, 14_280, 120)
        self.assertEqual(t["T_low"], tick(25))
        self.assertIn("after", t["T_low_rule"])
        r = run(1, [check(13, 32, 500), check(28, 32, 100)], [tick(20), tick(30)])
        self.assertIsNone(tl.seed_timeline(r, 14_280, 120)["T_low"])

    def test_t_low_never_departs_and_none(self):
        r = run(1, [check(13, 32, 500), check(40, 32, 100)])
        self.assertEqual(tl.seed_timeline(r, 14_280, 120)["T_low"], tick(40))
        r = run(1, [check(13, 32, 500)])
        self.assertIsNone(tl.seed_timeline(r, 14_280, 120)["T_low"])
        self.assertIsNone(tl.seed_timeline(run(1, []), 14_280, 120)["T_low"])

    def test_t_high_largest_ratio_earliest_tie_and_floor(self):
        r = run(1, [check(13, 32, 400), check(14, 32, 480), check(15, 32, 480), check(30, 32, 600)], [tick(20)])
        t = tl.seed_timeline(r, 14_280, 120)
        self.assertEqual(t["T_high"], tick(14))  # day 30 is after the departure
        self.assertAlmostEqual(t["T_high_ratio"], 480 / 307)
        r = run(1, [check(13, 32, 380)], [tick(20)])  # ratio 1.24 < 1.25
        t = tl.seed_timeline(r, 14_280, 120)
        self.assertIsNone(t["T_high"])
        self.assertAlmostEqual(t["T_high_ratio"], 380 / 307)

    def test_other_settlements_ignored_and_cancelled_counted(self):
        r = run(1, [check(13, 32, 100, sid=2), check(16, 32, 100)], [tick(20)], cancelled=tick(21))
        t = tl.seed_timeline(r, 14_280, 120)
        self.assertEqual(t["T_low"], tick(16))
        self.assertEqual(t["departures_cancelled"], 1)

    def test_times_outside_the_run_are_dropped(self):
        r = run(1, [check(118, 32, 100)])
        t = tl.seed_timeline(r, 14_280, 120)
        self.assertIsNone(t["T_low"])
        self.assertEqual(t["dropped_outside_run"], [("T_low", tick(118))])

    def test_build_shares(self):
        summary = {"scene": "game_v19", "runs": [run(2, [check(16, 32, 100)]), run(1, [])]}
        b = tl.build(summary, 14_280, 120)
        self.assertEqual(b["seeds"], [1, 2])
        self.assertEqual(b["count"]["T_low"], 1)
        self.assertEqual(b["share"]["T_low"], 0.5)
        self.assertIn("| 2 |", tl.markdown(b))


class Schedules(unittest.TestCase):
    def timeline(self):
        return {"per_seed": [
            {"seed": 7, "T_low": tick(16), "T_high": tick(14), "T_eve": 29_400},
            {"seed": 8, "T_low": None, "T_high": None, "T_eve": 29_400}]}

    def test_rows_of_every_arm(self):
        s = self.timeline()["per_seed"][0]
        self.assertEqual(sp.rows_for("sp120L", s), [{"tick": tick(16), "kind": "spoil_food", "settlement": "home", "amount": 120}])
        self.assertEqual([r["tick"] for r in sp.rows_for("gf3x80L", s)], [tick(16), tick(18), tick(20)])
        self.assertEqual(sp.rows_for("sp240H", s)[0]["tick"], tick(14))
        self.assertEqual(sp.rows_for("gf240E", s)[0]["tick"], 29_400)
        self.assertEqual(sp.rows_for("rain13W", s), [{"tick": 33_720, "kind": "rain", "settlement": "home", "amount": 13}])
        self.assertEqual(sp.rows_for("cs2W", s)[0]["kind"], "cold_snap")
        self.assertEqual(sp.rows_for("rw120E", s)[0]["amount"], 120)
        self.assertEqual(len(sp.GROUP_F), 12)
        self.assertEqual(len(sp.ARMS), 16)
        self.assertEqual(tl.display_day(33_720), 24)
        self.assertEqual(tl.clock(33_720), "18:00")
        self.assertEqual((tl.display_day(35_160), tl.clock(35_160)), (25, "18:00"))

    def test_write_counts_missing_and_checks(self):
        with tempfile.TemporaryDirectory() as d:
            m = sp.write(self.timeline(), sp.parse_arms("F"), Path(d), 14_280, 120)
            names = sorted(p.name for p in Path(d).iterdir())
            self.assertIn("s7_sp3x80L.json", names)
            self.assertNotIn("s8_sp60L.json", names)
            self.assertIn("s8_sp240E.json", names)
            self.assertEqual(len(m["files"]), 12 + 2)
            self.assertEqual(len(m["missing"]), 10)
            rows = json.loads((Path(d) / "s7_sp60L.json").read_text())
            self.assertEqual(rows[0]["tick"], tick(16))
        with self.assertRaises(AssertionError):
            sp.check_rows([{"tick": 14_279, "kind": "rain", "settlement": "home", "amount": 1}], 14_280, 120)
        with self.assertRaises(AssertionError):
            sp.check_rows([{"tick": 120 * D, "kind": "rain", "settlement": "home", "amount": 1}], 14_280, 120)
        with self.assertRaises(AssertionError):
            sp.check_rows([{"tick": 20_000, "kind": "rain", "settlement": "home", "amount": 1},
                           {"tick": 19_000, "kind": "rain", "settlement": "home", "amount": 1}], 14_280, 120)
        with self.assertRaises(SystemExit):
            sp.parse_arms("F,zz")


if __name__ == "__main__":
    unittest.main()
