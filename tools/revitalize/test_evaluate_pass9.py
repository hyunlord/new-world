"""Run: python3 -m unittest discover -s tools/revitalize -p test_evaluate_pass9.py."""
import json
import tempfile
import unittest
from pathlib import Path

import test_evaluate_pass6 as p6
import test_evaluate_pass8 as p8t
from evaluate_pass4 import EvidenceError
from evaluate_pass9 import baseline_ms, keyed_draws, load_round, main

RULES9 = {**p8t.RULES, "keyed_draws": 1, "noise_replicate": 0}


def write_round(root: Path, seeds: int, days: int, clocks: bool, rules=RULES9) -> Path:
    p8t.write_round(root, seeds, days, clocks, None, rules)
    for file in root.glob("*.jsonl"):
        rows = [json.loads(line) for line in file.read_text().splitlines()]
        for r in rows:
            r["schema_version"] = 9
        file.write_text("".join(json.dumps(r) + "\n" for r in rows))
    summary = json.loads((root / "summary.json").read_text())
    summary["schema_version"] = 9
    (root / "summary.json").write_text(json.dumps(summary))
    return root


class EvaluatorNineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_schema_nine_only_and_keyed_draws_read(self) -> None:
        r = load_round(write_round(self.root / "a", 2, 10, False), 10)
        self.assertEqual(keyed_draws(r), 1)
        with self.assertRaises(EvidenceError):
            load_round(p8t.write_round(self.root / "b", 2, 10, False), 10)

    def test_timing_baseline_is_game_v14(self) -> None:
        (self.root / "v14.json").write_text(json.dumps({"scene": "game_v14", "timing": {"ms_per_agent_tick": {"median": .002}}}))
        (self.root / "v13.json").write_text(json.dumps({"scene": "game_v13", "timing": {"ms_per_agent_tick": {"median": .002}}}))
        self.assertEqual(baseline_ms(self.root / "v14.json"), .002)
        with self.assertRaises(EvidenceError):
            baseline_ms(self.root / "v13.json")

    def test_cli_matches_pass8_evaluation_and_compares(self) -> None:
        write_round(self.root / "a", 64, 10, False)
        write_round(self.root / "b", 32, 120, True)
        (self.root / "v11.json").write_text(json.dumps({"measures": p6.V11}))
        args = ["--round10", str(self.root / "a"), "--round120", str(self.root / "b"), "--v11-targets", str(self.root / "v11.json")]
        self.assertEqual(main(args + ["--out", str(self.root / "out")]), 1)
        report = json.loads((self.root / "out" / "x_targets.json").read_text())
        self.assertEqual((report["probe_schema"], report["keyed_draws"]), (9, 1))
        self.assertEqual(len(report["per_seed"]["Y3_one_settlement"]["values"]), 96)
        self.assertIn("game_v14", report["targets"]["X8"]["clauses"]["day10"]["requirement"])
        self.assertEqual(main(args + ["--out", str(self.root / "out2"), "--compare-to", str(self.root / "out" / "x_targets.json")]), 1)
        other = json.loads((self.root / "out" / "x_targets.json").read_text())
        other["seeds"]["round10"] = {"start": 64, "end_exclusive": 128}
        (self.root / "other.json").write_text(json.dumps(other))
        self.assertEqual(main(args + ["--out", str(self.root / "out3"), "--compare-to", str(self.root / "other.json")]), 2)


if __name__ == "__main__":
    unittest.main()
