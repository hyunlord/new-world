"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass6.py."""
import json
import tempfile
import unittest
from pathlib import Path

from evaluate_pass6 import (EvidenceError, day_index, evaluate, load_round, main, measures, night_season,
                            system_day, winters_table)

TPD, OFFSET, DPS, STEP = 1440, 480, 7, 120
# Per-day rates by season (spring, summer, autumn, winter).
GATHERED, TRIPS = (20, 10, 40, 6), (10, 5, 20, 5)


def season(tick: int) -> int:
    return ((tick + OFFSET) // TPD // DPS) % 4


_PREFIX: dict = {}


def cumulative(tick: int, per_day) -> float:
    """Sum of per-day rates over the 120-tick steps before `tick` (each step 1/12 of a day)."""
    prefix = _PREFIX.setdefault(tuple(per_day), [0.0])
    while len(prefix) <= tick // STEP:
        t = (len(prefix) - 1) * STEP
        prefix.append(prefix[-1] + per_day[season(t)] / 12)
    return prefix[tick // STEP]


def evening_out(tick: int) -> bool:
    """The fire is out on the last winter evening's night (display days 27, 55, ...)."""
    day = (tick + OFFSET) // TPD
    minute = (tick + OFFSET) % TPD
    evening = day - 1 if minute < 420 else day
    return minute < 420 or minute >= 1080 and ((evening // DPS) % 4 == 3 and evening % DPS == DPS - 1)


def row(tick: int, kind: str, **overrides) -> dict:
    day_index_ = tick // TPD
    minute = (tick + OFFSET) % TPD
    dark = evening_out(tick) and ((((tick + OFFSET) // TPD - (1 if minute < 420 else 0)) // DPS) % 4 == 3) \
        and (((tick + OFFSET) // TPD - (1 if minute < 420 else 0)) % DPS == DPS - 1)
    cold_window = minute >= 1320 or minute < 360
    base = dict(schema_version=6, tick=tick, day=tick / TPD, minute_of_day=minute, sample_kind=kind,
                season=season(tick), days_per_season=DPS, settlements=1, births=min(day_index_, 3) * 6,
                alive=40, settled_agents=40, sustainable_adults=42, deaths_starvation=0, deaths_dehydration=0,
                deaths_combat=0, deaths_old_age=1 if day_index_ >= 20 else 0, children=4, elders=8,
                forage_trips_total=day_index_ * 12, home_distance_mean=6.0, state_idle=12, sleeping_in_shelter=36,
                idle_adults=10, idle_adults_near_hearth=7, meals_from_store_total=day_index_ * 70,
                ground_meals_total=day_index_ * 30, wood_stock=150 if season(tick) == 2 else 40, wood_target=156,
                wood_in_hand=150, fire_lit=0 if dark else 1, cold_members=10 if dark and cold_window else 0,
                gather_seekers_critical=0, adult_work_share=.4, idle_share=.5,
                wood_gathered_total=cumulative(tick, GATHERED), wood_trips_total=cumulative(tick, TRIPS),
                urgent_wood_trips_total=cumulative(tick, (0, 0, 0, 2)), cold_member_hours_total=0,
                stockpile_food=200, wood_gatherers_now=1, urgent_wood=0, winter_wood_lost_total=0,
                idle_elders=2, idle_elders_near_hearth=2)
    base.update(overrides)
    return base


def write_round(root: Path, seeds: int, days: int, clocks: bool, mutate=None) -> Path:
    root.mkdir(parents=True)
    for seed in range(seeds):
        rows = []
        for tick in range(0, days * TPD + 1, STEP):
            rows.append(row(tick, "periodic"))
            if tick and tick % TPD == 0:
                rows.append(row(tick, "day"))
        if clocks:
            for display_day in range(2, days + 1):
                for minute in (120, 720, 900, 1170):
                    tick = (display_day - 1) * TPD + minute - OFFSET
                    if 0 <= tick <= days * TPD:
                        rows.append(row(tick, "clock"))
        if mutate:
            mutate(seed, rows)
        with (root / f"game_v12_s{seed}.jsonl").open("w") as stream:
            stream.writelines(json.dumps(r) + "\n" for r in rows)
    summary = {"schema_version": 6, "scene": "game_v12", "seeds": {"start": 0, "end_exclusive": seeds},
               "determinism_checked": True,
               "days": {str(d): {"idle_breach_actionable_share": {"median": .004, "max": .01}} for d in range(1, days + 1)},
               "runs": [{"seed": s, "social_completed_per_agent_day": .6, "ms_per_agent_tick": .0012} for s in range(seeds)]}
    (root / "summary.json").write_text(json.dumps(summary))
    return root


V11 = {"autumn_work_1200": {"median": .42}, "winter_food_store_at_end": {"median": 220.0}}
RECEIPTS = {"legacy_pin": True, "v8_repro": True, "v9_repro": True, "v10_repro": True, "v11_repro": True}


class EvaluatorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.count = 0

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def rounds(self, mutate_long=None, mutate_short=None):
        self.count += 1
        short = load_round(write_round(self.root / f"r10_{self.count}", 64, 10, False, mutate_short), 10)
        long = load_round(write_round(self.root / f"r120_{self.count}", 32, 120, True, mutate_long), 120)
        return short, long

    def failing(self, result):
        return {k: [c for c, v in t["clauses"].items() if not v["passed"]] for k, t in result.items() if not t["passed"]}

    def test_synthetic_v12_village_passes_every_target(self) -> None:
        self.assertEqual(self.failing(evaluate(*self.rounds(), .001, RECEIPTS, v11=V11)), {})

    def test_night_after_the_last_winter_evening_is_booked_to_winter(self) -> None:
        # 02:00 on display day 28 (first spring day of year two) closes the night of day 27.
        r = row(28 * TPD + 120 - OFFSET, "clock")
        self.assertEqual((day_index(r), r["season"]), (28, 0))
        self.assertEqual(night_season(r, 720), 3)
        self.assertEqual(night_season(row(27 * TPD + 1320 - OFFSET, "periodic"), 360), 3)
        self.assertEqual(night_season(row(21 * TPD + 120 - OFFSET, "periodic"), 360), 2)

    def test_rows_are_booked_to_the_tick_the_systems_saw(self) -> None:
        # The probe writes a row after a completed tick: midnight's row still saw the last winter day.
        midnight = {"tick": 28 * TPD - OFFSET, "minute_of_day": 0}
        self.assertEqual((day_index(midnight), system_day(midnight)), (28, 27))
        self.assertEqual(system_day({"tick": 0, "minute_of_day": 480}), 0)

    def test_winters_table_counts_one_dark_night_per_winter(self) -> None:
        _, long = self.rounds()
        winters = winters_table(long, 360)
        self.assertEqual(len(winters), 32 * 4)
        first = winters[0]
        self.assertEqual((first["year"], first["first_display_day"], first["nights"]), (0, 21, 7))
        self.assertEqual((first["nights_out"], first["first_night_out"], first["trip_days"]), (1, 7, 7))
        # Elapsed days run 08:00 to 08:00: the last winter window holds 16 winter
        # and 8 spring hours (rates 5 and 10 trips a day; urgent 2 and 0).
        self.assertAlmostEqual(first["wood_trips"], 6 * 5 + 5 * 16 / 24 + 10 * 8 / 24)
        self.assertAlmostEqual(first["urgent_trips"], 6 * 2 + 2 * 16 / 24)
        self.assertEqual(first["food_store_at_end"], 200)
        self.assertEqual(measures(long, (1320, 360))["winter_food_store_at_end"]["median"], 200)

    def test_every_winter_night_lit_fails_y7_and_no_winter_trips_fail_y6(self) -> None:
        def mild(seed, rows):
            for r in rows:
                r["fire_lit"], r["cold_members"] = 1, 0
                if r["season"] == 3:
                    r["wood_trips_total"] = cumulative(r["tick"] - r["tick"] % TPD, (10, 5, 20, 0))
        failing = self.failing(evaluate(*self.rounds(mild), .001, RECEIPTS, v11=V11))
        self.assertEqual(failing.get("Y7"), ["winter_lit"])
        self.assertIn("trip_days", failing.get("Y6", []))

    def test_summer_like_autumn_and_hungry_winter_fail(self) -> None:
        def flat(seed, rows):
            for r in rows:
                r["wood_gathered_total"] = cumulative(r["tick"], (20, 30, 30, 6))
                r["stockpile_food"] = 100
                if r["season"] == 3 and (r["minute_of_day"] >= 1320 or r["minute_of_day"] < 360):
                    r["cold_members"] = 30
        failing = self.failing(evaluate(*self.rounds(flat), .001, RECEIPTS, v11=V11))
        self.assertEqual(failing, {"Y5": ["autumn_wood"], "Y8": ["winter_cold", "winter_food"]})

    def test_missing_baselines_and_receipts_never_pass(self) -> None:
        short, long = self.rounds()
        result = evaluate(short, long, None, {**RECEIPTS, "v11_repro": False})
        failing = self.failing(result)
        self.assertEqual(failing["Y1"], ["v11_repro"])
        self.assertEqual(failing["Y5"], ["autumn_work"])
        self.assertEqual(failing["Y8"], ["winter_food"])
        self.assertEqual(failing["Y13"], ["day10", "day120"])
        self.assertFalse(evaluate(short, long, .0009, RECEIPTS, v11=V11)["Y13"]["passed"])

    def test_schema_five_rounds_are_invalid_evidence(self) -> None:
        def old(seed, rows):
            for r in rows:
                r["schema_version"] = 5
        with self.assertRaises(EvidenceError):
            load_round(write_round(self.root / "bad", 2, 10, False, old), 10)

    def test_cli_writes_reports_and_tables(self) -> None:
        write_round(self.root / "a", 64, 10, False)
        write_round(self.root / "b", 32, 120, True)
        (self.root / "v11.json").write_text(json.dumps({"measures": V11}))
        code = main(["--round10", str(self.root / "a"), "--round120", str(self.root / "b"), "--out", str(self.root / "out"),
                     "--baseline-ms", ".001", "--v11-targets", str(self.root / "v11.json"),
                     *[arg for name in RECEIPTS for arg in (f"--{name.replace('_', '-')}", "PASS")]])
        self.assertEqual(code, 0)
        report = json.loads((self.root / "out" / "y_targets.json").read_text())
        self.assertTrue(report["passed"])
        self.assertEqual(len(report["winters"]), 128)
        text = (self.root / "out" / "y_targets.md").read_text()
        for heading in ("## Work by season", "## Wood by season", "## Winters"):
            self.assertIn(heading, text)


if __name__ == "__main__":
    unittest.main()
