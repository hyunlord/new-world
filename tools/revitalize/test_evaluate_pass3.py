# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass3.py."""
import tempfile
import json
import subprocess
import sys
import unittest
from pathlib import Path
from dataclasses import replace
from evaluate_pass3 import evaluate, Receipts
from pass3_evidence import KEYS, TEXT_KEYS, raw_seed, EvidenceError, RoundEvidence, SeedEvidence, load_round, quantiles

def fixture(duration: int) -> RoundEvidence:
    runs = []
    for seed in range(64 if duration == 10 else 32):
        days = {day: dict(alive=40., children=8., deaths_starvation=0., deaths_dehydration=0., settlements=1., shelters_completed=2., stockpile_food=60., food_need_per_day=80., home_distance_mean=3.) for day in range(1, duration + 1)}
        clocks = {day: {minute: dict(alive=40., sleeping_in_shelter=32., idle_adults=10., idle_adults_near_hearth=6., state_idle=10.) for minute in (120, 720, 1170)} for day in range(2, duration + 1)}
        runs.append(SeedEvidence(seed, days, 24., clocks, 2, .5, .0008, dict(children_observed=20, maturations_observed=12, child_deaths=0, children_still_alive=8, late_children=0), {day: .005 for day in days}))
    return RoundEvidence(Path("fixture"), duration, tuple(runs), True, {day: quantiles([.005] * len(runs)) for day in range(1, duration + 1)}, "a" * 40)

class EvaluatorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.receipts = Receipts(True, True, 0, 0, True, {"legacy": "test", "v8": "compare", "probe_10": "run10", "probe_60": "run60"}, "a" * 40)

    def test_paired_population_change_cannot_use_marginal_medians(self) -> None:
        short, long = fixture(10), fixture(60)
        # Same day45/day60 marginal median 40, but each paired change is 12.
        for run in long.runs:
            run.days[45]["alive"] = 34. if run.seed % 2 else 46.
            run.days[60]["alive"] = 46. if run.seed % 2 else 34.
        result = evaluate(short, long, self.receipts)
        self.assertFalse(result["U6"]["clauses"]["stability"]["passed"])
        self.assertEqual(result["U6"]["clauses"]["stability"]["observed"]["median"], 12.)

    def test_child_share_uses_per_seed_ratios(self) -> None:
        short, long = fixture(10), fixture(60)
        for run in short.runs:
            run.days[10]["alive"] = 20. if run.seed % 2 else 60.
            run.days[10]["children"] = 10. if run.seed % 2 else 0.
        result = evaluate(short, long, self.receipts)
        self.assertEqual(result["U9"]["clauses"]["child_share"]["observed"]["median"], .25)

    def test_missing_clock_never_passes_rhythm(self) -> None:
        short, long = fixture(10), fixture(60)
        long = replace(long, runs=tuple(replace(run, clocks={}) for run in long.runs))
        result = evaluate(short, long, self.receipts)
        self.assertFalse(result["U8"]["passed"])

    def test_missing_external_success_and_dropped_children_fail(self) -> None:
        result = evaluate(fixture(10), fixture(60), replace(self.receipts, v8_verified=False, probe_60_exit_code=1, children_implemented=False))
        self.assertFalse(result["U1"]["passed"])
        self.assertFalse(result["U9"]["passed"])

    def test_receipt_revision_mismatch_never_passes(self) -> None:
        result = evaluate(fixture(10), fixture(60), replace(self.receipts, expected_revision="c" * 40))
        self.assertFalse(result["U1"]["passed"])

    def test_mixed_producer_and_coincident_fingerprint_rejected(self) -> None:
        base = {key: 0 for key in KEYS - TEXT_KEYS}
        base.update(schema_version=3, scene="game_v9", git_sha="a" * 40, fingerprint="b" * 16, sample_kind="periodic")
        for key, change, message in (("git_sha", "c" * 40, "mixed producer"), ("fingerprint", "c" * 16, "fingerprint")):
            with tempfile.TemporaryDirectory() as temp:
                path = Path(temp) / "raw.jsonl"
                second = dict(base, sample_kind="clock")
                second[key] = change
                path.write_text(json.dumps(base) + "\n" + json.dumps(second) + "\n")
                with self.assertRaisesRegex(EvidenceError, message):
                    raw_seed(path, 0, (10, 120))

    def test_median_is_not_mean(self) -> None:
        self.assertEqual(quantiles([0., 1., 100.])["median"], 1.)

    def test_empty_seed_summary_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "summary.json").write_text('{"schema_version":3,"scene":"game_v9","seeds":{"start":0,"end_exclusive":0},"runs":[]}')
            with self.assertRaises(EvidenceError):
                load_round(root, 10)


def write_round(root: Path, duration: int) -> None:
    """Small complete probe-shaped fixture for boundary and CLI tests."""
    root.mkdir()
    count = 64 if duration == 10 else 32
    base = {key: 0 for key in KEYS - TEXT_KEYS}
    base.update(scene="game_v9", git_sha="a" * 40, fingerprint="b" * 16, schema_version=3,
                alive=40, adults=32, children=8, shelters_completed=2, settlements=1,
                stockpile_food=60, food_need_per_day=80., home_distance_mean=3.,
                sleeping_in_shelter=32, idle_adults=10, idle_adults_near_hearth=6)
    rows = {}
    for tick in range(0, duration * 1440 + 1, 120):
        rows.setdefault(tick, []).append("periodic")
    for day in range(1, duration + 1):
        rows.setdefault(day * 1440, []).append("day")
        for minute in (120, 720, 1170):
            tick = (day - 1) * 1440 + minute - 480
            if tick >= 0:
                rows.setdefault(tick, []).append("clock")
    day_values = {}
    for seed in range(count):
        with (root / f"game_v9_s{seed}.jsonl").open("w") as stream:
            for tick, kinds in sorted(rows.items()):
                row = dict(base, seed=seed, tick=tick, day=tick / 1440, minute_of_day=(tick + 480) % 1440)
                for kind in kinds:
                    stream.write(json.dumps(dict(row, sample_kind=kind)) + "\n")
                    if kind == "day":
                        day_values[str(tick // 1440)] = {key: quantiles([float(value)]) for key, value in row.items() if key not in TEXT_KEYS | {"schema_version", "seed", "tick", "day"}}
                        day_values[str(tick // 1440)]["idle_breach_actionable_share"] = quantiles([0.])
    runs = [dict(seed=seed, max_owned_housing_overflow=2, social_completed_per_agent_day=.5, ms_per_agent_tick=.0008, maturation=dict(children_observed=20, maturations_observed=12, child_deaths=0, children_still_alive=8, late_children=0)) for seed in range(count)]
    (root / "summary.json").write_text(json.dumps(dict(schema_version=3, scene="game_v9", seeds=dict(start=0, end_exclusive=count), runs=runs, days=day_values, determinism_checked=True)))

class BoundaryTests(unittest.TestCase):
    def test_real_cli_writes_reports_and_rejects_missing_periodic_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_round(root / "ten", 10)
            write_round(root / "sixty", 60)
            receipts = root / "receipts.json"
            (root / "command.log").write_text("Synthetic fixture success evidence\n")
            receipts.write_text(json.dumps(dict(legacy_verified=True, v8_verified=True, probe_10_exit_code=0, probe_60_exit_code=0, children_implemented=True, expected_revision="a" * 40, evidence={key: "command.log" for key in ("legacy", "v8", "probe_10", "probe_60")})))
            command = [sys.executable, str(Path(__file__).with_name("evaluate_pass3.py")), "--round10", str(root / "ten"), "--round60", str(root / "sixty"), "--receipts", str(receipts), "--out", str(root / "out")]
            result = subprocess.run(command, text=True, capture_output=True, check=False, timeout=60)
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((root / "out/u_targets.json").read_text())
            self.assertTrue(report["passed"])
            self.assertIn("U11", (root / "out/u_targets.md").read_text())
            raw = root / "ten/game_v9_s0.jsonl"
            lines = raw.read_text().splitlines()
            raw.write_text("\n".join(line for line in lines if json.loads(line)["tick"] != 120) + "\n")
            result = subprocess.run(command, text=True, capture_output=True, check=False, timeout=60)
            self.assertEqual(result.returncode, 2)
            self.assertIn("incomplete periodic", result.stderr)
            self.assertEqual(json.loads((root / "out/u_targets.json").read_text())["status"], "invalid")

if __name__ == "__main__":
    unittest.main()
