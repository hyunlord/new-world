"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass5.py."""
import json
import tempfile
import unittest
from pathlib import Path

from evaluate_pass5 import EvidenceError, evaluate, load_round, main, minute_in_window, store_shares

TPD, OFFSET, DPS = 1440, 480, 7


def season(tick: int) -> int:
    return ((tick + OFFSET) // TPD // DPS) % 4


def row(tick: int, kind: str, **overrides) -> dict:
    day_index = tick // TPD
    minute = (tick + OFFSET) % TPD
    base = dict(schema_version=5, tick=tick, day=tick / TPD, minute_of_day=minute, sample_kind=kind,
                season=season(tick), days_per_season=DPS, settlements=1, births=min(day_index, 3) * 6,
                alive=40, settled_agents=40, sustainable_adults=42, deaths_starvation=0, deaths_dehydration=0,
                deaths_combat=0, deaths_old_age=1 if day_index >= 20 else 0, children=4, elders=8,
                forage_trips_total=day_index * 12, home_distance_mean=6.0, state_idle=12, sleeping_in_shelter=36,
                idle_adults=10, idle_adults_near_hearth=7, meals_from_store_total=day_index * 70,
                ground_meals_total=day_index * 30, wood_stock=250 if season(tick) == 2 else 60, fire_lit=1,
                cold_members=0, gather_seekers_critical=0, adult_work_share=.4, idle_share=.5)
    base.update(overrides)
    return base


def write_round(root: Path, seeds: int, days: int, clocks: bool, mutate=None) -> Path:
    root.mkdir(parents=True)
    for seed in range(seeds):
        rows = []
        for tick in range(0, days * TPD + 1, 120):
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
        with (root / f"game_v11_s{seed}.jsonl").open("w") as stream:
            stream.writelines(json.dumps(r) + "\n" for r in rows)
    summary = {"schema_version": 5, "scene": "game_v11", "seeds": {"start": 0, "end_exclusive": seeds},
               "determinism_checked": True,
               "days": {str(d): {"idle_breach_actionable_share": {"median": .004, "max": .01}} for d in range(1, days + 1)},
               "runs": [{"seed": s, "social_completed_per_agent_day": .6, "ms_per_agent_tick": .0012} for s in range(seeds)]}
    (root / "summary.json").write_text(json.dumps(summary))
    return root


class EvaluatorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.receipts = {"legacy_pin": True, "v8_repro": True, "v9_repro": True, "v10_repro": True}
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

    def test_synthetic_healthy_village_passes_every_target(self) -> None:
        self.assertEqual(self.failing(evaluate(*self.rounds(), .001, self.receipts)), {})

    def test_cold_window_wraps_midnight(self) -> None:
        self.assertTrue(minute_in_window(1320, 1320, 360))
        self.assertTrue(minute_in_window(0, 1320, 360))
        self.assertFalse(minute_in_window(360, 1320, 360))
        self.assertFalse(minute_in_window(720, 1320, 360))

    def test_store_share_skips_the_first_two_days(self) -> None:
        _, long = self.rounds()
        shares, store, total = store_shares(long.seeds[0], 120)
        self.assertEqual(len(shares), 118)
        self.assertAlmostEqual(store / total, .7)

    def test_ground_eating_village_fails_x4(self) -> None:
        def ground(seed, rows):
            for r in rows:
                r["meals_from_store_total"] = r["tick"] // TPD * 10
        result = evaluate(*self.rounds(ground), .001, self.receipts)
        self.assertEqual(self.failing(result), {"X4": ["store_share", "every_seed", "winter"]})

    def test_cold_winter_nights_fail_x7_and_unlit_fire_fails_x6(self) -> None:
        def cold(seed, rows):
            for r in rows:
                if r["season"] == 3 and minute_in_window(r["minute_of_day"], 1320, 360):
                    r["cold_members"] = 20
                if r["minute_of_day"] == 120 and seed == 0:
                    r["fire_lit"] = 0
        failing = self.failing(evaluate(*self.rounds(cold), .001, self.receipts))
        self.assertEqual(failing, {"X6": ["lit_at_0200"], "X7": ["winter_cold"]})

    def test_low_autumn_wood_fails_x6(self) -> None:
        def short_wood(seed, rows):
            for r in rows:
                r["wood_stock"] = 100
        failing = self.failing(evaluate(*self.rounds(short_wood), .001, self.receipts))
        self.assertEqual(failing, {"X6": ["autumn_wood"]})

    def test_idle_noon_and_critical_gatherer_fail(self) -> None:
        def idle(seed, rows):
            for r in rows:
                r["adult_work_share"], r["idle_share"] = .1, .8
        def critical(seed, rows):
            rows[5]["gather_seekers_critical"] = 1
        failing = self.failing(evaluate(*self.rounds(idle, critical), .001, self.receipts))
        self.assertEqual(failing, {"X8": ["work_1200", "work_1500", "idle_1200"], "X11": ["day10"]})

    def test_missing_baseline_or_receipt_never_passes(self) -> None:
        short, long = self.rounds()
        result = evaluate(short, long, None, {**self.receipts, "v10_repro": False})
        self.assertFalse(result["X12"]["passed"])
        self.assertFalse(result["X1"]["passed"])
        self.assertFalse(evaluate(short, long, .0007, self.receipts)["X12"]["passed"])

    def test_schema_four_rounds_are_invalid_evidence(self) -> None:
        def old(seed, rows):
            for r in rows:
                r["schema_version"] = 4
        with self.assertRaises(EvidenceError):
            load_round(write_round(self.root / "bad", 2, 10, False, old), 10)

    def test_cli_writes_reports_and_returns_status(self) -> None:
        write_round(self.root / "a", 64, 10, False)
        write_round(self.root / "b", 32, 120, True)
        code = main(["--round10", str(self.root / "a"), "--round120", str(self.root / "b"), "--out", str(self.root / "out"),
                     "--baseline-ms", ".001", "--legacy-pin", "PASS", "--v8-repro", "PASS", "--v9-repro", "PASS",
                     "--v10-repro", "PASS"])
        self.assertEqual(code, 0)
        self.assertTrue(json.loads((self.root / "out" / "x_targets.json").read_text())["passed"])


if __name__ == "__main__":
    unittest.main()
