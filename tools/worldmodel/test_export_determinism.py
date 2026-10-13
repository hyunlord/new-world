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

    def record(self, name: str, workers: int, *extra: str, scene: str = "game_v13", days: int = 10) -> Path:
        out = self.root / name
        env = {**os.environ, "WORLDSIM_GIT_SHA": "test"}
        subprocess.run([str(BIN), "--scene", scene, "--seeds", "0..4", "--days", str(days), "--every", "60",
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

    def test_v14_runs_and_worker_counts_are_byte_identical(self) -> None:
        # Pass 8 section 8.2: schema-2 game_v14 episodes, 30 days (the first
        # winter's draws included), two runs at 4 workers and one at 1.
        for label, extra in (("v14_agents", ("--agents",)),
                             ("v14_schedule", ("--schedule-dir", str(self.schedules), "--include-factual"))):
            with self.subTest(label):
                first = tree(self.record(f"{label}_a", 4, *extra, scene="game_v14", days=30))
                second = tree(self.record(f"{label}_b", 4, *extra, scene="game_v14", days=30))
                single = tree(self.record(f"{label}_c", 1, *extra, scene="game_v14", days=30))
                self.assertTrue(first)
                self.assertEqual(first, second)
                self.assertEqual(first, single)
                meta = json.loads(first["game_v14_s0_none/meta.json"])
                self.assertEqual(meta["schema_version"], 6)

    def test_v15_fork_families_are_byte_identical(self) -> None:
        # Pass 9 Y7: schema-3 game_v15 branch families (agents, lineage keys,
        # noise forks at tick 1,440, R = 3), 4 days: two runs at 4 workers and one at 1.
        extra = ("--agents", "--schedule-dir", str(self.schedules), "--include-factual", "--fork-tick", "1440", "--replicates", "3")
        first = tree(self.record("v15_fork_a", 4, *extra, scene="game_v15", days=4))
        second = tree(self.record("v15_fork_b", 4, *extra, scene="game_v15", days=4))
        single = tree(self.record("v15_fork_c", 1, *extra, scene="game_v15", days=4))
        self.assertEqual(len({k.split("/")[0] for k in first}), 4 * 2 * 3)
        self.assertEqual(first, second)
        self.assertEqual(first, single)
        meta = json.loads(first["game_v15_s2_a_r2/meta.json"])
        self.assertEqual((meta["schema_version"], meta["keyed_draws"], meta["noise_fork"]), (6, True, {"tick": 1440, "replicate": 2}))
        self.assertIn("game_v15_s2_a_r2/agent_keys.npy", first)

    def test_v16_fork_families_are_byte_identical(self) -> None:
        # Pass 10 H9: schema-4 game_v16 branch families (agents, hp, lineage keys, the hidden
        # world variety in meta, noise forks at tick 1,440, R = 3), 4 days: two runs at 4 workers
        # and one at 1.
        extra = ("--agents", "--schedule-dir", str(self.schedules), "--include-factual", "--fork-tick", "1440", "--replicates", "3")
        first = tree(self.record("v16_fork_a", 4, *extra, scene="game_v16", days=4))
        second = tree(self.record("v16_fork_b", 4, *extra, scene="game_v16", days=4))
        single = tree(self.record("v16_fork_c", 1, *extra, scene="game_v16", days=4))
        self.assertEqual(len({k.split("/")[0] for k in first}), 4 * 2 * 3)
        self.assertEqual(first, second)
        self.assertEqual(first, single)
        meta = json.loads(first["game_v16_s2_a_r2/meta.json"])
        self.assertEqual((meta["schema_version"], meta["noise_fork"]), (6, {"tick": 1440, "replicate": 2}))
        self.assertIs(meta["world_variety"]["hidden"], True)

    def test_v17_fork_families_are_byte_identical(self) -> None:
        # Pass 11 J10: schema-5 game_v17 branch families (agents, hidden hardiness, founding
        # flags, parent slots, regions in the hidden meta, noise forks at tick 1,440, R = 3),
        # 4 days: two runs at 4 workers and one at 1.
        extra = ("--agents", "--schedule-dir", str(self.schedules), "--include-factual", "--fork-tick", "1440", "--replicates", "3")
        first = tree(self.record("v17_fork_a", 4, *extra, scene="game_v17", days=4))
        second = tree(self.record("v17_fork_b", 4, *extra, scene="game_v17", days=4))
        single = tree(self.record("v17_fork_c", 1, *extra, scene="game_v17", days=4))
        self.assertEqual(len({k.split("/")[0] for k in first}), 4 * 2 * 3)
        self.assertEqual(first, second)
        self.assertEqual(first, single)
        meta = json.loads(first["game_v17_s2_a_r2/meta.json"])
        self.assertEqual((meta["schema_version"], meta["noise_fork"]), (6, {"tick": 1440, "replicate": 2}))
        self.assertIn("regions", meta["world_variety"])

    def test_v18_fork_families_are_byte_identical(self) -> None:
        # Pass 12 J10: schema-5 game_v18 and game_v18_nofission branch families (agents, noise
        # forks at tick 1,440, R = 3), 4 days: two runs at 4 workers and one at 1.
        extra = ("--agents", "--schedule-dir", str(self.schedules), "--include-factual", "--fork-tick", "1440", "--replicates", "3")
        for scene in ("game_v18_nofission", "game_v18"):
            first = tree(self.record(f"{scene}_fork_a", 4, *extra, scene=scene, days=4))
            second = tree(self.record(f"{scene}_fork_b", 4, *extra, scene=scene, days=4))
            single = tree(self.record(f"{scene}_fork_c", 1, *extra, scene=scene, days=4))
            self.assertEqual(len({k.split("/")[0] for k in first}), 4 * 2 * 3)
            self.assertEqual(first, second)
            self.assertEqual(first, single)
            meta = json.loads(first[f"{scene}_s2_a_r2/meta.json"])
            self.assertEqual((meta["schema_version"], meta["noise_fork"]), (6, {"tick": 1440, "replicate": 2}))
            self.assertIn("regions", meta["world_variety"])

    def test_v19_fork_families_are_byte_identical(self) -> None:
        # Pass 13 J10: schema-6 game_v19 and game_v19_nofission branch families (agents, noise
        # forks at tick 1,440, R = 3), 4 days, two runs at 4 workers and one at 1; and plain
        # 40-day game_v19 runs (pressure accumulating after day 10) at 4 and 1 workers.
        extra = ("--agents", "--schedule-dir", str(self.schedules), "--include-factual", "--fork-tick", "1440", "--replicates", "3")
        for scene in ("game_v19_nofission", "game_v19"):
            first = tree(self.record(f"{scene}_fork_a", 4, *extra, scene=scene, days=4))
            second = tree(self.record(f"{scene}_fork_b", 4, *extra, scene=scene, days=4))
            single = tree(self.record(f"{scene}_fork_c", 1, *extra, scene=scene, days=4))
            self.assertEqual(len({k.split("/")[0] for k in first}), 4 * 2 * 3)
            self.assertEqual(first, second)
            self.assertEqual(first, single)
            meta = json.loads(first[f"{scene}_s2_a_r2/meta.json"])
            self.assertEqual((meta["schema_version"], meta["noise_fork"]), (6, {"tick": 1440, "replicate": 2}))
        many = tree(self.record("v19_long_a", 4, scene="game_v19", days=40))
        one = tree(self.record("v19_long_b", 1, scene="game_v19", days=40))
        self.assertEqual(many, one)

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
