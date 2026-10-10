"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass4.py."""
import json
import tempfile
import unittest
from pathlib import Path

from evaluate_pass4 import EvidenceError, autumn_closings, evaluate, load_round, longest_stagnation, main

TPD, OFFSET, DPS = 1440, 480, 7


def season(tick: int) -> int:
    return ((tick + OFFSET) // TPD // DPS) % 4


def row(tick: int, kind: str, **overrides) -> dict:
    day_index = tick // TPD
    base = dict(schema_version=4, tick=tick, day=tick / TPD, minute_of_day=(tick + OFFSET) % TPD, sample_kind=kind,
                season=season(tick), days_per_season=DPS, settlements=1, births=min(day_index, 3) * 6,
                alive=40, settled_agents=40, sustainable_adults=42, deaths_starvation=0, deaths_dehydration=0,
                deaths_combat=0, deaths_old_age=1 if day_index >= 20 else 0, children=4, elders=8,
                stockpile_food=200, stock_target=150.0, food_need_per_day=100.8, food_supply_winter_per_day=72.0,
                forage_trips_total=day_index * 12, home_distance_mean=5.0, state_idle=12, sleeping_in_shelter=36,
                idle_adults=10, idle_adults_near_hearth=7)
    base.update(overrides)
    return base


def write_round(root: Path, seeds: int, days: int, clocks: bool, mutate=None) -> Path:
    root.mkdir(parents=True)
    for seed in range(seeds):
        rows = []
        for tick in range(0, days * TPD + 1, 360):
            rows.append(row(tick, "periodic"))
            if tick and tick % TPD == 0:
                rows.append(row(tick, "day"))
        if clocks:
            for display_day in range(2, days + 1):
                for minute in (120, 720, 1170):
                    tick = (display_day - 1) * TPD + minute - OFFSET
                    if 0 <= tick <= days * TPD:
                        rows.append(row(tick, "clock"))
        if mutate:
            mutate(seed, rows)
        with (root / f"game_v10_s{seed}.jsonl").open("w") as stream:
            stream.writelines(json.dumps(r) + "\n" for r in rows)
    summary = {"schema_version": 4, "scene": "game_v10", "seeds": {"start": 0, "end_exclusive": seeds},
               "determinism_checked": True,
               "days": {str(d): {"idle_breach_actionable_share": {"median": .004, "max": .01}} for d in range(1, days + 1)},
               "runs": [{"seed": s, "social_completed_per_agent_day": .6, "ms_per_agent_tick": .0009} for s in range(seeds)]}
    (root / "summary.json").write_text(json.dumps(summary))
    return root


class EvaluatorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.receipts = {"legacy_pin": True, "v8_repro": True, "v9_repro": True}

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def rounds(self, mutate_long=None):
        short = load_round(write_round(self.root / "r10", 64, 10, False), 10)
        long = load_round(write_round(self.root / "r120", 32, 120, True, mutate_long), 120)
        return short, long

    def test_synthetic_healthy_village_passes_every_target(self) -> None:
        result = evaluate(*self.rounds(), .0008, self.receipts)
        failing = {k: [c for c, v in t["clauses"].items() if not v["passed"]] for k, t in result.items() if not t["passed"]}
        self.assertEqual(failing, {})

    def test_stagnation_counts_consecutive_birthless_days_below_capacity(self) -> None:
        def stall(seed, rows):
            for r in rows:
                if 40 * TPD <= r["tick"] <= 46 * TPD:
                    r["settled_agents"] = 30
        _, long = self.rounds(stall)
        self.assertEqual(longest_stagnation(long.seeds[0], 120), 7)
        self.assertFalse(evaluate(*self.rounds_again(stall), .0008, self.receipts)["W5"]["passed"])

    def rounds_again(self, mutate_long):
        self.tmp.cleanup()
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        return self.rounds(mutate_long)

    def test_autumn_closing_is_the_last_autumn_sample_before_winter(self) -> None:
        _, long = self.rounds()
        closings = autumn_closings(long.seeds[0])
        self.assertEqual(len(closings), 4)  # 120 display days cover four autumns
        for closing in closings:
            self.assertEqual(closing["season"], 2)
            self.assertEqual(season(closing["tick"] + 360), 3)

    def test_low_autumn_store_and_hoarding_fail_w7(self) -> None:
        def starve(seed, rows):
            for r in rows:
                r["stockpile_food"] = 10 if r["season"] == 2 else 1000
        result = evaluate(*self.rounds(starve), .0008, self.receipts)
        self.assertFalse(result["W7"]["clauses"]["autumn_store"]["passed"])
        self.assertFalse(result["W7"]["clauses"]["hoard_cap"]["passed"])

    def test_missing_baseline_or_receipt_never_passes(self) -> None:
        short, long = self.rounds()
        result = evaluate(short, long, None, {**self.receipts, "v9_repro": False})
        self.assertFalse(result["W11"]["passed"])
        self.assertFalse(result["W1"]["passed"])

    def test_incomplete_day_checkpoints_are_invalid_evidence(self) -> None:
        def drop(seed, rows):
            rows[:] = [r for r in rows if not (r["sample_kind"] == "day" and r["tick"] == 5 * TPD)]
        with self.assertRaises(EvidenceError):
            load_round(write_round(self.root / "bad", 2, 10, False, drop), 10)

    def test_cli_writes_reports_and_returns_status(self) -> None:
        write_round(self.root / "a", 64, 10, False)
        write_round(self.root / "b", 32, 120, True)
        code = main(["--round10", str(self.root / "a"), "--round120", str(self.root / "b"), "--out", str(self.root / "out"),
                     "--baseline-ms", ".0008", "--legacy-pin", "PASS", "--v8-repro", "PASS", "--v9-repro", "PASS"])
        self.assertEqual(code, 0)
        self.assertTrue(json.loads((self.root / "out" / "w_targets.json").read_text())["passed"])


if __name__ == "__main__":
    unittest.main()
