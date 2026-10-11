"""Loader round-trip on the committed sample (docs/worldmodel/sample).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

import numpy as np

from wm_data import load_episode, verify_episode

ROOT = Path(__file__).resolve().parents[2]
SAMPLE = ROOT / "docs/worldmodel/sample"
EPISODES = sorted(p for p in SAMPLE.iterdir() if (p / "meta.json").exists()) if SAMPLE.exists() else []


class LoaderTests(unittest.TestCase):
    def test_sample_exists(self) -> None:
        self.assertEqual(len(EPISODES), 2)

    def test_numpy_loads_every_file_and_meta_matches(self) -> None:
        for path in EPISODES:
            ep = load_episode(path)
            self.assertEqual(set(ep.arrays), set(ep.meta["arrays"]))
            files = {p.name for p in path.glob("*.npy")}
            self.assertEqual(files, set(ep.meta["arrays"]))
            t = ep.meta["rows"]
            self.assertEqual(t, ep.meta["days"] * ep.meta["ticks_per_day"] // ep.meta["every"] + 1)
            for name, data in ep.arrays.items():
                self.assertEqual(list(data.shape), ep.meta["arrays"][name]["shape"])
                if name in ep.meta["fields"] and name not in ("resource_keys.npy", "building_keys.npy"):
                    self.assertEqual(data.shape[-1], len(ep.meta["fields"][name]), name)
                if data.ndim >= 1 and name not in ("terrain.npy", "elevation.npy", "settlement_ids.npy", "agent_ids.npy",
                                                   "resource_keys.npy", "building_keys.npy"):
                    self.assertEqual(data.shape[0], t, name)

    def test_every_meta_field_is_documented(self) -> None:
        readme = (ROOT / "docs/worldmodel/README.md").read_text()
        for path in EPISODES:
            meta = json.loads((path / "meta.json").read_text())
            for array, fields in meta["fields"].items():
                self.assertIn(array, readme)
                for field in fields:
                    self.assertTrue(field["unit"] and field["meaning"], field)
                    self.assertIn(f"`{field['name']}`", readme, f"{array}:{field['name']} not documented")
            for key in meta:
                self.assertIn(f"`{key}`", readme, f"meta key {key} not documented")

    def test_columns_by_name_and_simulation_invariants(self) -> None:
        ep = load_episode(EPISODES[0])
        tick = ep.column("global.npy", "tick")
        self.assertTrue((np.diff(tick) == ep.meta["every"]).all())
        alive = ep.column("global.npy", "alive")
        parts = ep.column("global.npy", "children") + ep.column("global.npy", "elders") + ep.column("global.npy", "adults")
        self.assertTrue((alive == parts).all())
        self.assertTrue((alive == ep.arrays["agent_alive.npy"].sum(axis=1)).all())
        agents = ep.arrays["agents.npy"]
        self.assertTrue(np.isnan(agents[ep.arrays["agent_alive.npy"] == 0]).all())
        self.assertFalse(np.isnan(agents[ep.arrays["agent_alive.npy"] == 1][:, :15]).any())
        hidden = [f["name"] for f in ep.meta["fields"]["agents.npy"] if f["hidden"]]
        self.assertEqual(hidden, ["lifespan_days"])
        with self.assertRaises(KeyError):
            ep.column("global.npy", "nonexistent")

    def test_a_damaged_file_fails_verification(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / EPISODES[0].name
            shutil.copytree(EPISODES[0], copy)
            self.assertTrue(verify_episode(copy))
            data = bytearray((copy / "latent.npy").read_bytes())
            data[-1] ^= 1
            (copy / "latent.npy").write_bytes(bytes(data))
            self.assertFalse(verify_episode(copy))
            with self.assertRaises(ValueError):
                load_episode(copy)
            load_episode(copy, verify=False)


if __name__ == "__main__":
    unittest.main()
