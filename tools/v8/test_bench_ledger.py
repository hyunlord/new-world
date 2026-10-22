"""Unit tests of tools/v8/bench_ledger.py (the benchmark ledger, M1.5 6.1)."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench_ledger  # noqa: E402


def bench(mean=1000.0, p99=2000.0, per_agent=3000.0, commit="aaaaaaaa"):
    return {
        "commit": commit,
        "bytes_per_agent": per_agent,
        "bytes_per_tile": 40.0,
        "ceilings": {"n1000_mean_le_5ms": "PASS"},
        "scenes": [
            {"label": "n200", "agents": 200, "tick_mean_us": mean, "tick_p99_us": p99, "tick_max_us": p99 * 2,
             "per_repeat": [{"load_before": "5.0 4.0 3.0"}]},
            {"label": "n1000", "agents": 1000, "tick_mean_us": mean * 5, "tick_p99_us": p99 * 5,
             "tick_max_us": p99 * 10, "per_repeat": []},
        ],
    }


class LedgerTest(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())
        self.ledger = self.dir / "PERF_LEDGER.md"
        self.ledger.write_text("# ledger\n\n| " + " | ".join(bench_ledger.COLUMNS) + " |\n|"
                               + "---|" * len(bench_ledger.COLUMNS) + "\n")
        self.changelog = self.dir / "CHANGELOG.md"
        self.changelog.write_text("## M1.5\n- the index costs 12 % at N = 200 on purpose\n")

    def write(self, data, name="bench.json"):
        path = self.dir / name
        path.write_text(json.dumps(data))
        return str(path)

    def run_main(self, *args):
        return bench_ledger.main(list(args))

    def test_append_then_parse_round_trips(self):
        self.assertEqual(self.run_main("append", self.write(bench()), str(self.ledger), "--pass", "base"), 0)
        rows = bench_ledger.parse_ledger(self.ledger.read_text())
        self.assertEqual([r["scene"] for r in rows], ["n200", "n1000"])
        self.assertEqual(rows[0]["tick mean (µs)"], "1000.0")
        self.assertEqual(rows[0]["RSS/agent (B)"], "3000")
        self.assertEqual(rows[0]["load (1/5/15 min)"], "5.0 4.0 3.0")

    def test_check_passes_within_ten_percent_and_fails_beyond(self):
        self.run_main("append", self.write(bench()), str(self.ledger), "--pass", "base")
        self.assertEqual(self.run_main("check", self.write(bench(mean=1099.0)), str(self.ledger)), 0)
        self.assertEqual(self.run_main("check", self.write(bench(mean=1101.0)), str(self.ledger)), 1)
        self.assertEqual(self.run_main("check", self.write(bench(p99=2300.0)), str(self.ledger)), 1)
        self.assertEqual(self.run_main("check", self.write(bench(per_agent=3400.0)), str(self.ledger)), 1)

    def test_a_ceiling_miss_fails_even_when_justified(self):
        self.run_main("append", self.write(bench()), str(self.ledger), "--pass", "base")
        missed = bench()
        missed["ceilings"] = {"n1000_mean_le_5ms": "MISS"}
        self.assertEqual(self.run_main("check", self.write(missed), str(self.ledger),
                                       "--justified", "the index costs 12 % at N = 200 on purpose",
                                       "--changelog", str(self.changelog)), 1)
        none = bench()
        none["ceilings"] = {}
        self.assertEqual(self.run_main("check", self.write(none), str(self.ledger)), 1)
        self.assertEqual(self.run_main("check", self.write(bench()), str(self.ledger)), 0)

    def test_check_uses_only_the_latest_pass(self):
        self.run_main("append", self.write(bench(mean=500.0)), str(self.ledger), "--pass", "old")
        self.run_main("append", self.write(bench(mean=1000.0)), str(self.ledger), "--pass", "new")
        # 1,050 is +110 % over "old" but +5 % over "new".
        self.assertEqual(self.run_main("check", self.write(bench(mean=1050.0)), str(self.ledger)), 0)

    def test_justification_must_be_quoted_from_the_changelog(self):
        self.run_main("append", self.write(bench()), str(self.ledger), "--pass", "base")
        slow = self.write(bench(mean=1120.0))
        self.assertEqual(self.run_main("check", slow, str(self.ledger), "--justified",
                                       "not in the changelog", "--changelog", str(self.changelog)), 1)
        self.assertEqual(self.run_main("check", slow, str(self.ledger), "--justified",
                                       "the index costs 12 % at N = 200 on purpose",
                                       "--changelog", str(self.changelog)), 0)

    def test_an_empty_ledger_fails(self):
        self.assertEqual(self.run_main("check", self.write(bench()), str(self.ledger)), 1)


if __name__ == "__main__":
    unittest.main()
