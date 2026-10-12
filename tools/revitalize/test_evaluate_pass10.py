"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass10.py."""
import json
import tempfile
import unittest
from pathlib import Path

import test_evaluate_pass6 as p6
import test_evaluate_pass8 as p8t
from evaluate_pass4 import EvidenceError, Round, Seed
from evaluate_pass10 import RANGES, cold_night_tick, direction, h2, h3, h6, load_round, main, mean_weights, no_worse

RULES10 = {**p8t.RULES, "keyed_draws": 1, "noise_replicate": 0}
NEUTRAL = {"starting_agents": 24, "berry_bushes": 12, "berry_capacity": 4, "deadwood_spots": 20,
           "deadwood_capacity": 6, "forest_permille": 300, "climate": 1, "food_regen_permille": 1000}


def seed_info(start=24, variety=None, weights=(350, 400, 250), placed=20, swaps=0) -> dict:
    return {"world_variety": variety, "winter_severity_permille": list(weights), "starting_agents": start,
            "deadwood_spots_requested": 20, "deadwood_spots_placed": placed, "berry_repair_swaps": swaps}


def run(seed: int, days: int, alive=None, settlements=None, winters=(), ticks=(), info=None) -> dict:
    alive = alive or [40] * days
    return {"seed": seed, "seed_info": info or seed_info(), "alive_daily": alive,
            "settlements_daily": settlements or [1] * days, "dawn_health": [[d + 1, 50.0, 0, a] for d, a in enumerate(alive)],
            "seed_winters": list(winters), "exposure_death_ticks": list(ticks), "ms_per_agent_tick": .001}


def fake_round(runs: list[dict], days: int, rows=None) -> Round:
    seeds = []
    for r in runs:
        s = Seed(r["seed"])
        for d in range(1, days + 1):
            s.days[d] = dict(rows or {"deaths_starvation": 0, "deaths_dehydration": 0})
        seeds.append(s)
    return Round(Path("."), days, seeds, {"runs": runs, "seeds": {"start": 0, "end_exclusive": len(runs)}})


def winter(deaths: int, severity: int, members: int = 40, dark: int = 1) -> dict:
    return {"first_day": 22, "exposure_deaths": deaths, "severity": severity, "members_at_start": members,
            "alive_at_start": members, "dark_nights": dark}


def write_round(root: Path, seeds: int, days: int, clocks: bool) -> Path:
    """A Pass 8 synthetic round rewritten as schema 10 with the per-seed fields."""
    p8t.write_round(root, seeds, days, clocks, None, RULES10)
    for file in root.glob("*.jsonl"):
        rows = [json.loads(line) for line in file.read_text().splitlines()]
        for r in rows:
            r["schema_version"] = 10
        file.write_text("".join(json.dumps(r) + "\n" for r in rows))
    summary = json.loads((root / "summary.json").read_text())
    summary["schema_version"] = 10
    for entry in summary["runs"]:
        entry.update({k: v for k, v in run(entry["seed"], days).items() if k not in ("seed", "ms_per_agent_tick")})
    summary["scene"] = "game_v16"
    (root / "summary.json").write_text(json.dumps(summary))
    return root


class EvaluatorTenTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_cold_night_ticks(self) -> None:
        # Offset 480: display minute = (tick + 480) % 1440; day index 21 is the first winter day.
        night = 21 * 1440 + 1320 - 480  # 22:00 of day index 21
        self.assertEqual(cold_night_tick(night, 480, 7), (True, ""))
        self.assertTrue(cold_night_tick(night + 7 * 60, 480, 7)[0])  # 05:00 of the next day: the same night
        self.assertFalse(cold_night_tick(night + 8 * 60, 480, 7)[0])  # 06:00
        self.assertFalse(cold_night_tick(night - 60, 480, 7)[0])  # 21:00
        self.assertFalse(cold_night_tick(night + 30, 480, 7)[0])  # not a fire run
        self.assertFalse(cold_night_tick(night - 7 * 1440, 480, 7)[0])  # autumn evening
        # The last winter evening (day index 27) is a winter night until 06:00 of day index 28.
        last = 27 * 1440 + 1320 - 480
        self.assertTrue(cold_night_tick(last + 7 * 60, 480, 7)[0])
        self.assertFalse(cold_night_tick(last + 1440, 480, 7)[0])

    def test_h2_floor_retention_and_settlements(self) -> None:
        good = run(0, 120)
        low = run(1, 120, alive=[24] * 30 + [9] * 90, info=seed_info(start=24))  # 9 < 0.4 x 24 = 9.6
        shrink = run(2, 120, alive=[40] * 100 + [23] * 20)  # 23 < 0.6 x 40
        split = run(3, 120, settlements=[1] * 50 + [2] + [1] * 69)
        tiny = run(4, 120, alive=[8] * 120, info=seed_info(start=18))  # floor max(8, 7.2) = 8: passes
        result, rows = h2(fake_round([good, low, shrink, split, tiny], 120))
        self.assertEqual(result["clauses"]["alive_floor"]["observed"]["failing"], {"1": 9})
        # Seed 1 also keeps only 9 / 24 of its day-21 villagers.
        self.assertEqual(list(result["clauses"]["retention"]["observed"]["failing"]), ["1", "2"])
        self.assertEqual(result["clauses"]["one_settlement"]["observed"]["failing"], [3])
        self.assertEqual(rows["4"]["floor"], 8.0)
        self.assertFalse(result["passed"])
        self.assertTrue(h2(fake_round([good, tiny], 120))[0]["passed"])

    def test_h3_bounds_and_severity_order(self) -> None:
        # 20 seed-winters: 6 with deaths (share 0.30), median 0, harsh > normal > mild.
        winters = ([winter(0, 0)] * 6 + [winter(1, 0)] + [winter(0, 1)] * 5 + [winter(1, 1)] * 2
                   + [winter(0, 2)] * 3 + [winter(2, 2), winter(3, 2), winter(1, 2)])
        runs = [run(i, 120, winters=winters[i * 4:(i + 1) * 4]) for i in range(5)]
        short = fake_round([run(i, 10) for i in range(5)], 10)
        result, tables = h3(short, fake_round(runs, 120))
        self.assertTrue(result["passed"], {k: c for k, c in result["clauses"].items() if not c["passed"]})
        self.assertAlmostEqual(result["clauses"]["share_with_death"]["observed"]["share"], 6 / 20)
        self.assertEqual(tables["by_severity"]["harsh"]["histogram"], {"0": 3, "1": 1, "2": 1, "3": 1})
        # A seed-winter losing more than 0.35 of its members fails; equal means fail the order.
        bad = winters[:-1] + [winter(15, 2, members=40)]
        result, _ = h3(short, fake_round([run(i, 120, winters=bad[i * 4:(i + 1) * 4]) for i in range(5)], 120))
        self.assertFalse(result["clauses"]["max_share_of_members"]["passed"])
        flat = [winter(1, s) for s in (0, 1, 2)] * 2 + [winter(0, s) for s in (0, 1, 2)] * 2
        result, _ = h3(short, fake_round([run(0, 120, winters=flat)], 120))
        self.assertFalse(result["clauses"]["increasing_with_severity"]["passed"])
        # Starvation: any death at day 10 fails; day 120 median <= 1 and max <= 6.
        starving = fake_round([run(0, 10)], 10, rows={"deaths_starvation": 1, "deaths_dehydration": 0})
        self.assertFalse(h3(starving, fake_round(runs, 120))[0]["clauses"]["starvation_10"]["passed"])
        long_rows = fake_round(runs, 120, rows={"deaths_starvation": 4, "deaths_dehydration": 3})
        self.assertFalse(h3(short, long_rows)[0]["clauses"]["starvation_long"]["passed"])

    def test_h6_means_climate_and_counts(self) -> None:
        centred = []
        for i in range(30):
            v = {k: (lo + hi) // 2 + (i % 3 - 1) for k, (lo, hi) in RANGES.items()}
            v["climate"] = i % 3
            centred.append(run(i, 10, info=seed_info(variety=v, placed=19 if i == 4 else 20, swaps=2 if i == 7 else 0)))
        result, detail = h6([fake_round(centred, 10)], RANGES, (333, 334, 333))
        self.assertTrue(result["passed"], result)
        self.assertEqual(detail["deadwood_shortfalls"], [{"seed": 4, "requested": 20, "placed": 19}])
        self.assertEqual(detail["berry_repair_swaps"], [{"seed": 7, "swaps": 2}])
        skewed = [run(i, 10, info=seed_info(variety={**NEUTRAL, "starting_agents": 30, "climate": 2})) for i in range(10)]
        result, _ = h6([fake_round(skewed, 10)], RANGES, (333, 334, 333))
        self.assertFalse(result["clauses"]["parameter_means"]["passed"])
        self.assertFalse(result["clauses"]["climate_shares"]["passed"])

    def test_population_escape_needs_v16_no_worse_than_v15(self) -> None:
        # Review round 1, L1.
        work = lambda m, ok: {"passed": ok, "observed": {"median": m}, "requirement": "adult work share at 12:00 median >= 0.35"}
        idle = lambda m, ok: {"passed": ok, "observed": {"median": m}, "requirement": "idle share median <= 0.60"}
        self.assertEqual((direction(work(0, 0)["requirement"]), direction(idle(0, 0)["requirement"])), (1, -1))
        self.assertTrue(no_worse(work(.34, False), work(.31, False)))
        self.assertFalse(no_worse(work(.30, False), work(.31, False)))
        self.assertTrue(no_worse(idle(.62, False), idle(.63, False)))
        self.assertFalse(no_worse(idle(.64, False), idle(.63, False)))
        self.assertIsNone(direction("median in [0.70, 0.97] and every seed >= 0.50 and <= 1"))
        nested = {"passed": False, "observed": {"autumn": {"median": 2.0}, "summer": {"median": 1.0}},
                  "requirement": "autumn wood gathered per day median >= 1.5 x summer's"}
        self.assertTrue(no_worse(nested, {**nested, "observed": {"autumn": {"median": 1.5}}}))

    def test_per_seed_weights_never_come_from_the_rules_block(self) -> None:
        r = fake_round([run(0, 10, info=seed_info(weights=(500, 350, 150))),
                        run(1, 10, info=seed_info(weights=(200, 400, 400)))], 10)
        r.summary["rules"] = {"firewood": {"winter_severity_permille": [350, 400, 250]}}
        self.assertEqual(mean_weights(r), (350, 375, 275))

    def test_cli_on_synthetic_rounds(self) -> None:
        a, b = write_round(self.root / "a", 64, 10, False), write_round(self.root / "b", 32, 120, True)
        (self.root / "v11.json").write_text(json.dumps({"measures": p6.V11}))
        base = ["--round10", str(a), "--round120", str(b), "--v11-targets", str(self.root / "v11.json"),
                "--legacy-pin", "PASS", "--v8-v15-repro", "PASS"]
        # The synthetic rounds carry no seed-winters, so H3 fails (share undefined): exit 1.
        self.assertEqual(main(base + ["--out", str(self.root / "out")]), 1)
        report = json.loads((self.root / "out" / "h_targets.json").read_text())
        self.assertFalse(report["targets"]["H3"]["passed"])
        self.assertEqual(report["probe_schema"], 10)
        self.assertTrue(report["targets"]["H1"]["passed"])
        self.assertTrue(report["targets"]["H2"]["passed"])
        self.assertTrue(report["targets"]["H4"]["passed"])
        self.assertIn("Y12_day120", report["targets"]["H5"]["clauses"])
        # Schema 9 rounds are refused.
        with self.assertRaises(EvidenceError):
            load_round(p8t.write_round(self.root / "c", 2, 10, False), 10)


if __name__ == "__main__":
    unittest.main()
