"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass7.py."""
import json
import tempfile
import unittest
from pathlib import Path

import test_evaluate_pass6 as p6
from evaluate_pass4 import EvidenceError
from evaluate_pass7 import evaluate, load_round, main, measures, winters_table, z3_clauses

TPD, OFFSET, DPS = p6.TPD, p6.OFFSET, p6.DPS
RECEIPTS = {name: True for name in ("legacy_pin", "v8_repro", "v9_repro", "v10_repro", "v11_repro", "v12_repro")}
V12 = {"winter_wood_gathered": {"median": 40.0}, "retarget_share": {"median": .5}}


def year_of(tick: int) -> int:
    return (tick + OFFSET) // TPD // (4 * DPS)


def severity_at(tick: int, seed: int) -> int:
    """Synthetic draw: severity cycles with seed + year; -1 outside a winter."""
    return (seed + year_of(tick)) % 3 if p6.season(tick) == 3 else -1


def schema7(retarget_rate: float = .25):
    def mutate(seed: int, rows: list[dict]) -> None:
        for r in rows:
            r["schema_version"] = 7
            r["winter_severity"] = severity_at(r["tick"], seed)
            r["wood_retargets_total"] = p6.cumulative(r["tick"], tuple(retarget_rate * t for t in p6.TRIPS))
            r["interventions_applied_total"] = 0
            r["fire_blocked_settlements"] = 0
            r["cold_snap_nights_left"] = 0
    return mutate


def write_round(root: Path, seeds: int, days: int, clocks: bool, extra=None, rate: float = .05) -> Path:
    base = schema7(rate)

    def mutate(seed, rows):
        base(seed, rows)
        if extra:
            extra(seed, rows)
    p6.write_round(root, seeds, days, clocks, mutate)
    summary = json.loads((root / "summary.json").read_text())
    summary["schema_version"] = 7
    (root / "summary.json").write_text(json.dumps(summary))
    return root


def winter(nights_out: int, first: int, severity: int = 1, **kw) -> dict:
    return {"nights_out": nights_out, "first_night_out": first, "severity": severity, **kw}


class EvaluatorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.count = 0

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def rounds(self, extra=None, rate=.05):
        self.count += 1
        short = load_round(write_round(self.root / f"r10_{self.count}", 64, 10, False, None, rate), 10)
        long = load_round(write_round(self.root / f"r120_{self.count}", 32, 120, True, extra, rate), 120)
        return short, long

    def failing(self, result):
        return {k: [c for c, v in t["clauses"].items() if not v["passed"]] for k, t in result.items() if not t["passed"]}

    def test_winters_table_adds_severity_and_retarget_share(self) -> None:
        _, long = self.rounds()
        winters = winters_table(long)
        self.assertEqual(len(winters), 128)
        first = winters[0]
        self.assertEqual((first["seed"], first["year"], first["severity"]), (0, 0, 0))
        self.assertEqual(winters[1]["severity"], 1)
        self.assertEqual(winters[5]["severity"], 2, "seed 1, year 1")
        self.assertAlmostEqual(first["retarget_share"], .05)
        self.assertAlmostEqual(first["wood_retargets"], .05 * first["wood_trips"])

    def test_synthetic_village_fails_only_the_spread_of_dark_nights(self) -> None:
        # Every synthetic winter has one dark night, on the seventh: winters do not differ.
        short, long = self.rounds()
        result = evaluate(short, long, RECEIPTS, p6.V11, V12, z6=True, z7=True, baseline_ms=.0012, recorder_seconds=1.0)
        failing = self.failing(result)
        self.assertEqual(failing.pop("Z3"), ["no_dark_night", "two_or_more", "first_night_spread"])
        failing.pop("Y9", None)
        self.assertEqual(failing, {})
        self.assertTrue(result["Y9"]["reported_only"])

    def test_retargets_and_gathering_floor_decide_z4(self) -> None:
        short, long = self.rounds(rate=.3)
        failing = self.failing(evaluate(short, long, RECEIPTS, p6.V11, {**V12, "winter_wood_gathered": {"median": 60.0}}))
        self.assertEqual(failing["Z4"], ["retarget_share", "normal_gathered"])
        self.assertNotIn("Z4", self.failing(evaluate(*self.rounds(), RECEIPTS, p6.V11, V12)))

    def test_z3_clauses_on_constructed_winters(self) -> None:
        weights = (300, 400, 300)
        spread = ([winter(0, 0, 0)] * 30 + [winter(1, 5, 1)] * 20 + [winter(1, 6, 1)] * 20 + [winter(1, 7, 1)] * 10
                  + [winter(3, 5, 2)] * 30)
        clauses = z3_clauses(spread, weights)
        self.assertTrue(all(c["passed"] for c in clauses.values()), clauses)
        self.assertEqual(clauses["first_night_spread"]["observed"]["distinct_first_nights"], 3)
        calendar = [winter(1, 7, s) for s in (0, 1, 2)] * 40
        failed = [k for k, c in z3_clauses(calendar, weights).items() if not c["passed"]]
        self.assertEqual(failed, ["no_dark_night", "two_or_more", "first_night_spread"])
        lopsided = [winter(0, 0, 0)] * 30 + [winter(2, 4, 2)] * 30 + [winter(1, 6, 1)] * 40
        self.assertFalse(z3_clauses(lopsided, (100, 400, 500))["severity_shares"]["passed"])

    def test_missing_receipts_baselines_and_v12_never_pass(self) -> None:
        short, long = self.rounds()
        failing = self.failing(evaluate(short, long, {**RECEIPTS, "v12_repro": False}, p6.V11, None))
        self.assertEqual(failing["Z1"], ["v12_repro"])
        self.assertEqual(failing["Z4"], ["normal_gathered"])
        self.assertEqual(failing["Z6"], ["receipt"])
        self.assertEqual(failing["Z7"], ["receipt"])
        self.assertEqual(failing["Z8"], ["day10", "day120", "recorder"])

    def test_measures_hold_the_v12_comparison_numbers(self) -> None:
        _, long = self.rounds()
        own = measures(long, (1320, 360))
        for key in ("autumn_work_1200", "winter_food_store_at_end", "winter_wood_gathered", "winter_wood_gathered_normal",
                    "retarget_share", "winter_wood_retargets"):
            self.assertIn(key, own)
        self.assertAlmostEqual(own["retarget_share"]["median"], .05)

    def test_schema_six_rounds_are_invalid_evidence(self) -> None:
        with self.assertRaises(EvidenceError):
            load_round(p6.write_round(self.root / "bad", 2, 10, False), 10)

    def test_cli_writes_reports_and_tables(self) -> None:
        write_round(self.root / "a", 64, 10, False)
        write_round(self.root / "b", 32, 120, True)
        (self.root / "v11.json").write_text(json.dumps({"measures": p6.V11}))
        (self.root / "v12.json").write_text(json.dumps({"measures": V12}))
        code = main(["--round10", str(self.root / "a"), "--round120", str(self.root / "b"), "--out", str(self.root / "out"),
                     "--v11-targets", str(self.root / "v11.json"), "--v12-targets", str(self.root / "v12.json")])
        self.assertEqual(code, 1)
        report = json.loads((self.root / "out" / "z_targets.json").read_text())
        self.assertFalse(report["passed"])
        self.assertEqual(len(report["winters"]), 128)
        self.assertEqual([row["n"] for row in report["winters_by_severity"]], [43, 43, 42])
        text = (self.root / "out" / "z_targets.md").read_text()
        for heading in ("## Work by season", "## Wood by season", "## Winters", "## Winters by severity", "Re-target share"):
            self.assertIn(heading, text)
        self.assertEqual(main(["--round10", str(self.root / "a"), "--round120", str(self.root / "b"), "--out",
                               str(self.root / "out2"), "--severity-permille", "300,300,300"]), 2)


if __name__ == "__main__":
    unittest.main()
