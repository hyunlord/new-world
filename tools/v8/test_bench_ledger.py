"""Unit tests of tools/v8/bench_ledger.py (the benchmark ledger, M1.5 6.1; the M5 rule, goal 5.3)."""
import contextlib
import io
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench_ledger  # noqa: E402


def bench(mean=1000.0, p99=2000.0, per_agent=3000.0, commit="aaaaaaaa", alloc=None):
    return {
        "commit": commit,
        "bytes_per_agent": per_agent,
        "alloc_per_agent": alloc,
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

    def gate_log(self, head, status=0, name="gate.txt", finished=True):
        path = self.dir / name
        lines = [f"head {head}", "dirty 0", "started 2026-10-23 10:00:00 KST", " 1 fmt 3 s exit 0"]
        if finished:
            lines.append(f"finished 2026-10-23 11:30:00 KST status {status}")
        path.write_text("\n".join(lines) + "\n")
        return str(path)

    def append(self, bench_file, *args):
        """append with a passing gate log of the run's commit unless the test gives its own."""
        if "--gate-log" not in args and "--re-measure" not in args:
            commit = json.loads(Path(bench_file).read_text())["commit"]
            args = (*args, "--gate-log", self.gate_log(commit + "0123456789abcdef0123456789abcdef"))
        return self.run_main("append", bench_file, *args)

    def check(self, *args):
        """check as candidate pass "cand" unless the test names the pass."""
        if "--pass" not in args:
            args = (*args, "--pass", "cand")
        return self.run_main("check", *args)

    def check_output(self, *args):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = self.check(*args)
        return code, out.getvalue()

    def test_append_then_parse_round_trips(self):
        self.assertEqual(self.append(self.write(bench()), str(self.ledger), "--pass", "base"), 0)
        rows = bench_ledger.parse_ledger(self.ledger.read_text())
        self.assertEqual([r["scene"] for r in rows], ["n200", "n1000"])
        self.assertEqual(rows[0]["tick mean (µs)"], "1000.0")
        self.assertEqual(rows[0]["RSS/agent (B)"], "3000")
        self.assertEqual(rows[0]["load (1/5/15 min)"], "5.0 4.0 3.0")

    def test_check_passes_within_ten_percent_and_fails_beyond(self):
        self.append(self.write(bench()), str(self.ledger), "--pass", "base")
        self.assertEqual(self.check(self.write(bench(mean=1099.0)), str(self.ledger)), 0)
        self.assertEqual(self.check(self.write(bench(mean=1101.0)), str(self.ledger)), 1)
        self.assertEqual(self.check(self.write(bench(p99=2300.0)), str(self.ledger)), 1)
        self.assertEqual(self.check(self.write(bench(per_agent=3400.0)), str(self.ledger)), 1)

    def test_a_ceiling_miss_fails_even_when_justified(self):
        self.append(self.write(bench()), str(self.ledger), "--pass", "base")
        missed = bench()
        missed["ceilings"] = {"n1000_mean_le_5ms": "MISS"}
        self.assertEqual(self.check(self.write(missed), str(self.ledger),
                                       "--justified", "the index costs 12 % at N = 200 on purpose",
                                       "--changelog", str(self.changelog)), 1)
        none = bench()
        none["ceilings"] = {}
        self.assertEqual(self.check(self.write(none), str(self.ledger)), 1)
        self.assertEqual(self.check(self.write(bench()), str(self.ledger)), 0)

    def test_check_uses_only_the_latest_pass(self):
        self.append(self.write(bench(mean=500.0)), str(self.ledger), "--pass", "old")
        self.append(self.write(bench(mean=1000.0)), str(self.ledger), "--pass", "new")
        # 1,050 is +110 % over "old" but +5 % over "new".
        self.assertEqual(self.check(self.write(bench(mean=1050.0)), str(self.ledger)), 0)

    def test_justification_must_be_quoted_from_the_changelog(self):
        self.append(self.write(bench()), str(self.ledger), "--pass", "base")
        slow = self.write(bench(mean=1120.0))
        self.assertEqual(self.check(slow, str(self.ledger), "--justified",
                                       "not in the changelog", "--changelog", str(self.changelog)), 1)
        self.assertEqual(self.check(slow, str(self.ledger), "--justified",
                                       "the index costs 12 % at N = 200 on purpose",
                                       "--changelog", str(self.changelog)), 0)

    def test_a_contended_run_needs_a_note_and_is_never_a_base(self):
        self.append(self.write(bench(mean=1000.0)), str(self.ledger), "--pass", "quiet")
        busy = bench(mean=2000.0)
        busy["scenes"][0]["per_repeat"] = [{"load_before": "19.50 22.00 21.00"}]
        self.assertEqual(self.append(self.write(busy), str(self.ledger), "--pass", "busy"), 2)
        self.assertEqual(self.append(self.write(busy), str(self.ledger), "--pass", "busy",
                                       "--note", "host busy"), 0)
        rows = bench_ledger.parse_ledger(self.ledger.read_text())
        self.assertEqual([r["note"] for r in rows if r["pass"] == "busy"],
                         ["contended, not a base: host busy"] * 2)
        # 1,050 passes against the quiet 1,000 rows; the contended 2,000 rows are skipped.
        self.assertEqual(self.check(self.write(bench(mean=1050.0)), str(self.ledger)), 0)
        self.assertEqual(self.check(self.write(bench(mean=1150.0)), str(self.ledger)), 1)

    def test_a_ledger_of_contended_rows_only_has_no_base(self):
        busy = bench()
        busy["scenes"][0]["per_repeat"] = [{"load_before": "8.00 7.00 6.00"}]
        self.append(self.write(busy), str(self.ledger), "--pass", "busy", "--note", "x")
        self.assertEqual(self.check(self.write(bench()), str(self.ledger)), 1)

    def test_an_empty_ledger_fails(self):
        self.assertEqual(self.check(self.write(bench()), str(self.ledger)), 1)

    # --- M5: the ledger rule, second fix (goal 5.3) ---

    def test_check_requires_the_candidate_pass(self):
        self.append(self.write(bench()), str(self.ledger), "--pass", "base")
        self.assertEqual(self.run_main("check", self.write(bench()), str(self.ledger)), 2)

    def test_a_candidate_whose_rows_are_already_in_the_ledger_is_compared_with_the_previous_pass(self):
        # The M4 bug: M4 appended its rows before its gate, and the gate compared M4 with M4.
        self.append(self.write(bench(mean=1000.0)), str(self.ledger), "--pass", "M3")
        self.append(self.write(bench(mean=1300.0, commit="bbbbbbbb")), str(self.ledger), "--pass", "M4")
        m4 = self.write(bench(mean=1300.0, commit="bbbbbbbb"), "m4.json")
        code, out = self.check_output(m4, str(self.ledger), "--pass", "M4")
        self.assertEqual(code, 1)
        self.assertIn("compared with pass M3", out)
        self.assertIn("x1.300", out)
        # The next pass compares with M4.
        code, out = self.check_output(m4, str(self.ledger), "--pass", "M5")
        self.assertEqual(code, 0)
        self.assertIn("compared with pass M4", out)

    def test_rows_descending_from_the_base_are_excluded(self):
        if shutil.which("git") is None:
            self.skipTest("git is not available")
        repo = self.dir / "repo"
        repo.mkdir()

        def run_git(*args):
            return subprocess.run(["git", "-C", str(repo), *args], check=True, capture_output=True,
                                  text=True).stdout.strip()

        run_git("init", "-q")
        shas = []
        for i in range(3):
            run_git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", f"c{i}")
            shas.append(run_git("rev-parse", "HEAD"))
        old_dir = bench_ledger.GIT_DIR
        bench_ledger.GIT_DIR = repo
        self.addCleanup(setattr, bench_ledger, "GIT_DIR", old_dir)
        c0, c1, c2 = (sha[:8] for sha in shas)
        self.append(self.write(bench(mean=500.0, commit=c0)), str(self.ledger), "--pass", "M3")
        self.append(self.write(bench(mean=1000.0, commit=c1)), str(self.ledger), "--pass", "base M4")
        self.append(self.write(bench(mean=2000.0, commit=c2)), str(self.ledger), "--pass", "early M5")
        candidate = self.write(bench(mean=1500.0, commit=c2), "cand.json")
        # Without --base the c2 rows (another label) are the base: x0.75 passes.
        code, out = self.check_output(candidate, str(self.ledger), "--pass", "M5")
        self.assertEqual(code, 0)
        self.assertIn("compared with pass early M5", out)
        # --base c1: c2 descends from c1 and is excluded; c1 itself is not a strict descendant.
        code, out = self.check_output(candidate, str(self.ledger), "--pass", "M5", "--base", c1)
        self.assertEqual(code, 1)
        self.assertIn("compared with pass base M4", out)
        # --base c0: both later commits are excluded.
        code, out = self.check_output(candidate, str(self.ledger), "--pass", "M5", "--base", c0)
        self.assertIn("compared with pass M3", out)
        # A commit git cannot resolve is kept, with a note.
        self.append(self.write(bench(mean=1490.0, commit="zzzzzzzz")), str(self.ledger), "--pass", "odd")
        code, out = self.check_output(candidate, str(self.ledger), "--pass", "M5", "--base", c1)
        self.assertEqual(code, 0)
        self.assertIn("zzzzzzzz cannot be resolved", out)
        self.assertIn("compared with pass odd", out)

    def test_append_without_a_gate_log_is_refused(self):
        self.assertEqual(self.run_main("append", self.write(bench()), str(self.ledger), "--pass", "M5"), 2)
        self.assertEqual(bench_ledger.parse_ledger(self.ledger.read_text()), [])

    def test_append_with_a_failing_or_mismatched_gate_log_is_refused(self):
        run = self.write(bench(commit="aaaaaaaa"))
        full = "aaaaaaaa" + "1" * 32
        for log in (self.gate_log(full, status=1, name="failed.txt"),
                    self.gate_log("bbbbbbbb" + "1" * 32, name="other.txt"),
                    self.gate_log(full, finished=False, name="unfinished.txt"),
                    str(self.dir / "missing.txt")):
            self.assertEqual(self.run_main("append", run, str(self.ledger), "--pass", "M5", "--gate-log", log),
                             2, log)
        not_a_summary = self.dir / "bench.log"
        not_a_summary.write_text("bench: PASS\nfinished x status 0\n")
        self.assertEqual(self.run_main("append", run, str(self.ledger), "--pass", "M5", "--gate-log",
                                       str(not_a_summary)), 2)
        unknown = self.write(bench(commit="unknown"), "unknown.json")
        self.assertEqual(self.run_main("append", unknown, str(self.ledger), "--pass", "M5", "--gate-log",
                                       self.gate_log(full, name="u.txt")), 2)
        self.assertEqual(bench_ledger.parse_ledger(self.ledger.read_text()), [])

    def test_append_with_a_matching_passing_gate_log_succeeds(self):
        run = self.write(bench(commit="aaaaaaaa", alloc=1234.4))
        log = self.gate_log("aaaaaaaa" + "1" * 32)
        self.assertEqual(self.run_main("append", run, str(self.ledger), "--pass", "M5", "--gate-log", log), 0)
        rows = bench_ledger.parse_ledger(self.ledger.read_text())
        self.assertEqual([r["pass"] for r in rows], ["M5", "M5"])
        self.assertEqual(rows[0]["alloc/agent (B)"], "1234")
        # The prefix match works either way (a short head in the log).
        self.assertEqual(self.run_main("append", self.write(bench(commit="aaaaaaaa1111")), str(self.ledger),
                                       "--pass", "M5b", "--gate-log", self.gate_log("aaaaaaaa", name="s.txt")), 0)

    def test_re_measure_is_the_escape_hatch_and_its_note_says_so(self):
        run = self.write(bench())
        self.assertEqual(self.run_main("append", run, str(self.ledger), "--pass", "M4",
                                       "--re-measure", "quiet again"), 2)
        self.assertEqual(self.run_main("append", run, str(self.ledger), "--pass", "M4",
                                       "--re-measure", "re-measure: x", "--note", "y"), 2)
        self.assertEqual(self.run_main("append", run, str(self.ledger), "--pass", "M4",
                                       "--re-measure", "re-measure: M4 941410fc, quiet"), 0)
        rows = bench_ledger.parse_ledger(self.ledger.read_text())
        self.assertEqual([r["note"] for r in rows], ["re-measure: M4 941410fc, quiet"] * 2)

    def test_alloc_per_agent_decides_when_both_have_it_and_rss_is_information(self):
        self.append(self.write(bench(per_agent=3000.0, alloc=1000.0)), str(self.ledger), "--pass", "M5")
        code, out = self.check_output(self.write(bench(alloc=1101.0)), str(self.ledger))
        self.assertEqual(code, 1)
        self.assertIn("memory metric: alloc/agent decides", out)
        self.assertIn("bench: FAIL (> +10 %: alloc/agent)", out)
        self.assertEqual(self.check(self.write(bench(alloc=1099.0)), str(self.ledger)), 0)
        # RSS +13 % with the allocator number flat: printed, never fails.
        code, out = self.check_output(self.write(bench(per_agent=3400.0, alloc=1000.0)), str(self.ledger))
        self.assertEqual(code, 0)
        self.assertIn("RSS/agent: 3000 -> 3400 B (x1.133) information only", out)

    def test_base_rows_without_alloc_keep_the_rss_rule(self):
        self.append(self.write(bench(per_agent=3000.0)), str(self.ledger), "--pass", "M4")
        code, out = self.check_output(self.write(bench(per_agent=3400.0, alloc=1000.0)), str(self.ledger))
        self.assertEqual(code, 1)
        self.assertIn("memory metric: RSS/agent decides (the base rows have no alloc/agent)", out)
        self.assertIn("bench: FAIL (> +10 %: RSS/agent)", out)

    def test_old_format_rows_parse_with_an_empty_alloc(self):
        old = bench_ledger.OLD_COLUMNS
        self.assertEqual(len(old), len(bench_ledger.COLUMNS) - 1)
        self.assertEqual(bench_ledger.COLUMNS[bench_ledger.COLUMNS.index("RSS/agent (B)") + 1], "alloc/agent (B)")
        cells = ["2026-10-23", "bf16de16", "M3", "200", "n200", "313.2", "689", "1192", "2030", "82.5",
                 "4.32 5.01 6.11", "quiet"]
        text = ("| " + " | ".join(old) + " |\n|" + "---|" * len(old) + "\n| " + " | ".join(cells) + " |\n")
        rows = bench_ledger.parse_ledger(text)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["alloc/agent (B)"], "")
        self.assertEqual(rows[0]["RSS/agent (B)"], "2030")
        self.assertEqual(rows[0]["RSS/tile (B)"], "82.5")
        self.assertEqual(rows[0]["note"], "quiet")
        # An old-format base still compares on RSS.
        self.ledger.write_text(text)
        code, out = self.check_output(self.write(bench(mean=313.0, p99=689.0, per_agent=2100.0, alloc=900.0)),
                                      str(self.ledger))
        self.assertIn("compared with pass M3", out)
        self.assertIn("memory metric: RSS/agent decides", out)

    def test_the_repository_ledger_parses_every_row(self):
        path = Path(__file__).resolve().parents[2] / "docs" / "perf" / "PERF_LEDGER.md"
        text = path.read_text()
        table = [line for line in text.splitlines() if line.startswith("| ") and not line.startswith("| date")]
        rows = bench_ledger.parse_ledger(text)
        self.assertEqual(len(rows), len(table))
        self.assertGreaterEqual(len(rows), 50)
        self.assertTrue(all(set(r) == set(bench_ledger.COLUMNS) for r in rows))


if __name__ == "__main__":
    unittest.main()
