"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass8.py."""
import json
import tempfile
import unittest
from pathlib import Path

import test_evaluate_pass6 as p6
import test_evaluate_pass7 as p7
from evaluate_pass4 import EvidenceError
from evaluate_pass8 import evaluate, load_round, main, per_seed_clause, r_squared, rules_weights, winters_table, x3_clauses

TPD, OFFSET = p6.TPD, p6.OFFSET
RECEIPTS = {name: True for name in ("legacy_pin", "v8_repro", "v9_repro", "v10_repro", "v11_repro", "v12_repro", "v13_repro")}
RULES = {"firewood": {"winter_severity_permille": [350, 400, 250], "winter_severity_burn_per_hour": [2, 3, 4],
                      "winter_severity_regen_units_per_day": [8, 6, 4], "winter_night_burn_permille": [250, 500, 250],
                      "winter_night_burn_step": 2, "winter_fall_spread": 2, "windfall_over_snow": True, "burn_hours": 13}}


def schema8(extra=None):
    base = p7.schema7(.05)

    def mutate(seed: int, rows: list[dict]) -> None:
        base(seed, rows)
        for r in rows:
            r["schema_version"] = 8
            winter = p6.season(r["tick"]) == 3
            minute = r["minute_of_day"]
            # The night belongs to its evening (booking as the simulation).
            evening = r["tick"] - TPD if minute < 420 else r["tick"]
            night = p6.season(max(evening, 0)) == 3 and (minute >= 1080 or minute < 420)
            r["night_burn_per_hour"] = (3 + seed % 2) if night else 0
            r["winter_fall_today"] = 6 + (r["tick"] // TPD) % 3 - 1 if winter else -1
        if extra:
            extra(seed, rows)
    return mutate


def write_round(root: Path, seeds: int, days: int, clocks: bool, extra=None, rules=RULES) -> Path:
    p6.write_round(root, seeds, days, clocks, schema8(extra))
    summary = json.loads((root / "summary.json").read_text())
    summary["schema_version"] = 8
    summary["rules"] = rules
    (root / "summary.json").write_text(json.dumps(summary))
    return root


def winter(nights_out: int, first: int, severity: int) -> dict:
    return {"nights_out": nights_out, "first_night_out": first, "severity": severity}


def failing_replacement(seeds: tuple[int, ...]):
    """Four extra old-age deaths by day 120 in `seeds`: replacement margin -4."""
    def extra(seed: int, rows: list[dict]) -> None:
        if seed in seeds:
            for r in rows:
                if r["tick"] >= 100 * TPD:
                    r["deaths_old_age"] += 4
    return extra


class EvaluatorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.count = 0

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def rounds(self, extra=None, rules=RULES):
        self.count += 1
        short = load_round(write_round(self.root / f"r10_{self.count}", 64, 10, False, None, rules), 10)
        long = load_round(write_round(self.root / f"r120_{self.count}", 32, 120, True, extra, rules), 120)
        return short, long

    def failing(self, result):
        return {k: [c for c, v in t["clauses"].items() if not v["passed"]] for k, t in result.items() if not t["passed"]}

    def test_r_squared_is_the_share_explained_by_severity(self) -> None:
        calendar = [winter(0, 0, 0)] * 10 + [winter(1, 7, 1)] * 10 + [winter(2, 6, 2)] * 10
        self.assertAlmostEqual(r_squared(calendar), 1.0)
        self.assertIsNone(r_squared([winter(1, 7, s) for s in (0, 1, 2)]))
        # Counts independent of severity: R2 = 0.
        flat = [winter(n, 5 if n else 0, s) for s in (0, 1, 2) for n in (0, 1, 2)]
        self.assertAlmostEqual(r_squared(flat), 0.0)
        # Hand-computed: groups {0, 2} and {1, 3}; grand mean 1.5, total SS 5, within SS 2 + 2 = 4.
        self.assertAlmostEqual(r_squared([winter(0, 0, 0), winter(2, 5, 0), winter(1, 7, 1), winter(3, 5, 1)]), 1 - 4 / 5)

    def test_x3_clauses_on_constructed_winters(self) -> None:
        weights = (350, 400, 250)
        good = ([winter(0, 0, 0)] * 30 + [winter(1, 6, 0)] * 5
                + [winter(0, 0, 1)] * 10 + [winter(1, 7, 1)] * 15 + [winter(2, 5, 1)] * 10 + [winter(3, 4, 1)] * 5
                + [winter(1, 6, 2)] * 8 + [winter(2, 5, 2)] * 10 + [winter(3, 4, 2)] * 7)
        clauses = x3_clauses(good, weights)
        self.assertTrue(all(c["passed"] for c in clauses.values()), {k: c for k, c in clauses.items() if not c["passed"]})
        self.assertLessEqual(clauses["a_r2"]["observed"]["r2"], .75)
        calendar = [winter(0, 0, 0)] * 35 + [winter(1, 7, 1)] * 40 + [winter(2, 6, 2)] * 25
        failed = [k for k, c in x3_clauses(calendar, weights).items() if not c["passed"]]
        self.assertEqual(failed, ["a_r2", "b_spread_within_severity", "e_first_night_spread"])
        self.assertFalse(x3_clauses(good, None)["f_severity_shares"]["passed"], "no weights: shares undefined")
        self.assertFalse(x3_clauses([winter(1, 7, 1)] * 10, weights)["a_r2"]["passed"], "undefined R2 fails")

    def test_winters_table_adds_night_burn_and_fall(self) -> None:
        _, long = self.rounds()
        winters = winters_table(long)
        self.assertEqual(len(winters), 128)
        self.assertEqual({w["night_burn_mean"] for w in winters if w["seed"] == 1}, {4.0})
        self.assertEqual({w["night_burn_mean"] for w in winters if w["seed"] == 2}, {3.0})
        # The synthetic fall cycles 5, 6, 7 by day: seven days average 6 +- 1/7.
        self.assertTrue(all(abs(w["fall_mean"] - 6) <= 1 / 7 + 1e-9 for w in winters))
        self.assertEqual(rules_weights(long), (350, 400, 250))

    def test_per_seed_clauses_compare_failing_counts_with_the_reference(self) -> None:
        short, long = self.rounds(failing_replacement((3,)))
        _, reference = evaluate(short, long, RECEIPTS, p6.V11, (350, 400, 250))
        self.assertEqual(reference["Y3_replacement"]["failing"], ["3"])
        # Absolute mode: one failing seed fails the clause.
        result, _ = evaluate(short, long, RECEIPTS, p6.V11, (350, 400, 250))
        self.assertEqual(self.failing(result)["X2"], ["Y3_replacement"])
        # Comparison mode: two failing seeds against the reference's one pass; three fail.
        short2, long2 = self.rounds(failing_replacement((3, 9)))
        result, own = evaluate(short2, long2, RECEIPTS, p6.V11, (350, 400, 250), reference=reference)
        self.assertNotIn("X2", self.failing(result))
        observed = result["X2"]["clauses"]["Y3_replacement"]["observed"]
        self.assertEqual((observed["n_failing"], observed["reference_n_failing"]), (2, 1))
        self.assertEqual(observed["failing"], {"3": -4.0, "9": -4.0})
        self.assertEqual(own["Y3_replacement"]["values"]["0"], 0.0)
        short3, long3 = self.rounds(failing_replacement((3, 9, 11)))
        result, _ = evaluate(short3, long3, RECEIPTS, p6.V11, (350, 400, 250), reference=reference)
        self.assertEqual(self.failing(result)["X2"], ["Y3_replacement"])
        # Y2 is never in comparison mode.
        self.assertTrue(all(k in result["X2"]["clauses"] for k in ("Y2_deaths_starvation_10", "Y2_deaths_starvation_120")))
        self.assertFalse(per_seed_clause("Y3_x", {"failing": [], "values": {}, "requirement": "r"}, {})["passed"],
                         "a clause missing from the reference fails")

    def test_synthetic_village_fails_only_the_winters_and_receipts_it_lacks(self) -> None:
        # Every synthetic winter has one dark night, on the seventh.
        short, long = self.rounds()
        result, _ = evaluate(short, long, RECEIPTS, p6.V11, (350, 400, 250), x6=True, x7=True, baseline_ms=.0012)
        failing = self.failing(result)
        self.assertEqual(failing.pop("X3"), ["a_r2", "b_spread_within_severity", "c_no_dark_night", "d_two_or_more",
                                             "e_first_night_spread"])
        failing.pop("Y9", None)
        self.assertEqual(failing, {})

    def test_cli_reads_weights_from_the_run_and_records_overrides(self) -> None:
        write_round(self.root / "a", 64, 10, False)
        write_round(self.root / "b", 32, 120, True)
        (self.root / "v11.json").write_text(json.dumps({"measures": p6.V11}))
        args = ["--round10", str(self.root / "a"), "--round120", str(self.root / "b"), "--v11-targets", str(self.root / "v11.json")]
        self.assertEqual(main(args + ["--out", str(self.root / "out")]), 1)
        report = json.loads((self.root / "out" / "x_targets.json").read_text())
        self.assertEqual((report["severity_permille"], report["severity_source"]), ([350, 400, 250], "summary.json rules block"))
        self.assertIn("Y3_replacement", report["per_seed"])
        self.assertEqual(len(report["per_seed"]["Y3_one_settlement"]["values"]), 96)
        self.assertIn("## Per-seed clauses", (self.root / "out" / "x_targets.md").read_text())
        main(args + ["--out", str(self.root / "out2"), "--severity-permille", "520,280,200"])
        report = json.loads((self.root / "out2" / "x_targets.json").read_text())
        self.assertEqual((report["severity_permille"], report["severity_source"]), ([520, 280, 200], "--severity-permille override"))
        # Comparison with itself works; with other seeds it is invalid evidence.
        self.assertEqual(main(args + ["--out", str(self.root / "out3"), "--compare-to", str(self.root / "out" / "x_targets.json")]), 1)
        other = json.loads((self.root / "out" / "x_targets.json").read_text())
        other["seeds"]["round120"] = {"start": 32, "end_exclusive": 64}
        (self.root / "other.json").write_text(json.dumps(other))
        self.assertEqual(main(args + ["--out", str(self.root / "out4"), "--compare-to", str(self.root / "other.json")]), 2)
        self.assertEqual(main(args + ["--out", str(self.root / "out5"), "--severity-permille", "300,300,300"]), 2)

    def test_x1_accepts_the_held_out_set_of_64_long_seeds(self) -> None:
        short = load_round(write_round(self.root / "h10", 64, 10, False), 10)
        for seeds, ok in ((64, True), (32, True), (48, False)):
            long = load_round(write_round(self.root / f"h120_{seeds}", seeds, 120, True), 120)
            result, _ = evaluate(short, long, RECEIPTS, p6.V11, (350, 400, 250))
            self.assertEqual(result["X1"]["clauses"]["twins_120"]["passed"], ok, seeds)

    def test_schema_seven_rounds_are_invalid_evidence(self) -> None:
        with self.assertRaises(EvidenceError):
            load_round(p7.write_round(self.root / "bad", 2, 10, False), 10)


if __name__ == "__main__":
    unittest.main()
