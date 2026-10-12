"""PASS8_GOAL.md 8.2: the recorder's hidden weather columns equal the probe keys.

Records game_v14 seeds 0-1 for 30 days with the recorder (`--every 60`) and the
probe (`--sample-every 60`) and compares, at every tick both sampled:
`night_burn_per_hour` and `winter_fall_today` (global.npy against the probe keys of
the same name), the settlement `wood_target` (summed over settlements, as the
probe sums it) and `winter_severity`. Both read the first three at the system
tick (the row's tick - 1). `winter_severity` follows the recorder's documented
convention (Pass 7: the completed tick, as the FFI's time snapshot) while the
probe reads the system tick; the two can differ only on a row taken at 07:00
(the burn end, where the night booking switches) of a day that starts a season.
Those rows are asserted to be the only exceptions.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
Skips without the release binaries (`cargo build --release -p sim-test`).
"""
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

import numpy as np

from wm_data import load_episode

ROOT = Path(__file__).resolve().parents[2]
WM_BIN = Path(os.environ.get("WM_BIN", ROOT / "rust/target/release/worldsim_wm"))
PROBE_BIN = Path(os.environ.get("PROBE_BIN", ROOT / "rust/target/release/worldsim_probe"))
SEEDS, DAYS, TPD, OFFSET = (0, 1), 30, 1440, 480


@unittest.skipUnless(WM_BIN.exists() and PROBE_BIN.exists(), f"binaries not built: {WM_BIN}, {PROBE_BIN}")
class HiddenColumnTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        root = Path(cls.tmp.name)
        env = {**os.environ, "WORLDSIM_GIT_SHA": "test"}
        seeds = f"{SEEDS[0]}..{SEEDS[-1] + 1}"
        subprocess.run([str(WM_BIN), "--scene", "game_v14", "--seeds", seeds, "--days", str(DAYS), "--every", "60",
                        "--workers", "2", "--out", str(root / "wm")], check=True, capture_output=True, env=env)
        subprocess.run([str(PROBE_BIN), "--scene", "game_v14", "--seeds", seeds, "--days", str(DAYS), "--sample-every", "60",
                        "--workers", "2", "--out", str(root / "probe")], check=True, capture_output=True, env=env)
        cls.root = root

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def probe_rows(self, seed: int) -> dict[int, dict]:
        rows = {}
        with (self.root / "probe" / f"game_v14_s{seed}.jsonl").open() as stream:
            for line in stream:
                row = json.loads(line)
                if row["sample_kind"] == "periodic":
                    rows[int(row["tick"])] = row
        return rows

    def test_hidden_columns_equal_the_probe_keys(self) -> None:
        for seed in SEEDS:
            ep = load_episode(self.root / "wm" / f"game_v14_s{seed}_none")
            self.assertEqual(ep.meta["schema_version"], 5)
            probe = self.probe_rows(seed)
            ticks = ep.column("global.npy", "tick").astype(np.int64)
            burn = ep.column("global.npy", "night_burn_per_hour")
            fall = ep.column("global.npy", "winter_fall_today")
            severity = ep.column("global.npy", "winter_severity")
            target = np.nansum(ep.column("settlements.npy", "wood_target"), axis=1)
            common = [i for i, t in enumerate(ticks) if int(t) in probe]
            self.assertGreater(len(common), DAYS * 24 - 2, "every hour is a common tick")
            exceptions = []
            nonzero_burns, winter_falls = set(), set()
            for i in common:
                row = probe[int(ticks[i])]
                self.assertEqual(burn[i], row["night_burn_per_hour"], f"seed {seed} tick {ticks[i]}")
                self.assertEqual(fall[i], row["winter_fall_today"], f"seed {seed} tick {ticks[i]}")
                self.assertAlmostEqual(float(target[i]), float(row["wood_target"]), places=3, msg=f"seed {seed} tick {ticks[i]}")
                if severity[i] != row["winter_severity"]:
                    exceptions.append(int(ticks[i]))
                if burn[i] > 0 and severity[i] >= 0:
                    nonzero_burns.add(float(burn[i]))
                if fall[i] >= 0:
                    winter_falls.add(float(fall[i]))
            for tick in exceptions:
                day, minute = (tick + OFFSET) // TPD, (tick + OFFSET) % TPD
                self.assertEqual((minute, day % 7), (420, 0), f"seed {seed}: severity differs at tick {tick}")
            # The first winter (days 21-27) is inside the 30 days: the draws show.
            self.assertTrue(winter_falls, f"seed {seed}: no winter fall recorded")
            self.assertTrue(nonzero_burns, f"seed {seed}: no winter night burn recorded")


if __name__ == "__main__":
    unittest.main()
