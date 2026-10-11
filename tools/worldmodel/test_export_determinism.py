"""Section 9.4: same arguments give byte-identical files, whatever the worker count.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
Needs the release recorder (`cargo build --release -p sim-test --bin worldsim_wm`);
skips with a message when it is not built.
"""
import filecmp
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BIN = Path(os.environ.get("WM_BIN", ROOT / "rust/target/release/worldsim_wm"))


def tree(path: Path) -> dict[str, bytes]:
    """Every file under `path` except timing.json (wall time is not reproducible)."""
    return {str(p.relative_to(path)): p.read_bytes() for p in sorted(path.rglob("*"))
            if p.is_file() and p.name != "timing.json"}


@unittest.skipUnless(BIN.exists(), f"recorder not built: {BIN}")
class ExportDeterminismTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.schedules = self.root / "schedules"
        self.schedules.mkdir()
        for seed in range(4):
            rows = [{"tick": 600, "kind": "rain", "settlement": "home", "amount": 13},
                    {"tick": 2040, "kind": "windfall", "settlement": "home", "amount": 10}]
            (self.schedules / f"s{seed}_a.json").write_text(json.dumps(rows))

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def record(self, name: str, workers: int, *extra: str) -> Path:
        out = self.root / name
        env = {**os.environ, "WORLDSIM_GIT_SHA": "test"}
        subprocess.run([str(BIN), "--scene", "game_v13", "--seeds", "0..4", "--days", "10", "--every", "60",
                        "--workers", str(workers), "--out", str(out), *extra],
                       check=True, capture_output=True, env=env)
        return out

    def test_runs_and_worker_counts_are_byte_identical(self) -> None:
        for label, extra in (("plain", ()), ("agents", ("--agents",)),
                             ("schedule", ("--agents", "--schedule-dir", str(self.schedules), "--include-factual"))):
            with self.subTest(label):
                first = tree(self.record(f"{label}_a", 4, *extra))
                second = tree(self.record(f"{label}_b", 4, *extra))
                single = tree(self.record(f"{label}_c", 1, *extra))
                self.assertTrue(first)
                self.assertEqual(first.keys(), second.keys())
                self.assertEqual(first, second)
                self.assertEqual(first, single)

    def test_schedule_episodes_share_the_factual_prefix(self) -> None:
        import numpy as np
        out = self.record("prefix", 2, "--schedule-dir", str(self.schedules), "--include-factual")
        for seed in range(4):
            factual = np.load(out / f"game_v13_s{seed}_none" / "row_digest.npy")
            acted = np.load(out / f"game_v13_s{seed}_a" / "row_digest.npy")
            # Row t is taken before the interventions of tick t: rows 0..10 (ticks <= 600) match.
            self.assertTrue((factual[:11] == acted[:11]).all())
            self.assertFalse((factual[11:] == acted[11:]).all())
            log = json.loads((out / f"game_v13_s{seed}_a" / "interventions.json").read_text())
            self.assertEqual([r["tick"] for r in log], [600, 2040])

    def test_resume_keeps_verified_episodes(self) -> None:
        out = self.record("resume", 2)
        before = tree(out)
        meta = out / "game_v13_s0_none" / "meta.json"
        stamp = meta.stat().st_mtime_ns
        self.record("resume", 2, "--resume")
        self.assertEqual(meta.stat().st_mtime_ns, stamp)
        self.assertEqual(tree(out), before)
        # A damaged array is re-recorded.
        (out / "game_v13_s1_none" / "global.npy").write_bytes(b"broken")
        self.record("resume", 2, "--resume")
        self.assertEqual(tree(out), before)


if __name__ == "__main__":
    unittest.main()
