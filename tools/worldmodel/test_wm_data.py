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

    def test_schema_one_sample_has_observable_columns(self) -> None:
        # The committed v0 sample is schema 1: `observable` falls back to the
        # per-field hidden flags (only the agents' lifespan is hidden there).
        ep = load_episode(EPISODES[0])
        self.assertEqual(ep.meta["schema_version"], 1)
        names = ep.observable_names("global.npy")
        self.assertEqual(names, [f["name"] for f in ep.meta["fields"]["global.npy"]])
        self.assertEqual(ep.observable("global.npy").shape, ep.arrays["global.npy"].shape)
        self.assertNotIn("lifespan_days", ep.observable_names("agents.npy"))
        self.assertEqual(ep.observable("agents.npy").shape[-1], len(ep.meta["fields"]["agents.npy"]) - 1)

    def test_schema_two_meta_drops_hidden_columns(self) -> None:
        ep = load_episode(EPISODES[0])
        fields = [dict(f) for f in ep.meta["fields"]["global.npy"]]
        for f in fields:
            f["hidden"] = f["name"] == "winter_severity"
        names = [f["name"] for f in fields if not f["hidden"]]
        ep.meta = {**ep.meta, "schema_version": 2, "fields": {**ep.meta["fields"], "global.npy": fields},
                   "observable_fields": {"global.npy": names}}
        self.assertNotIn("winter_severity", ep.observable_names("global.npy"))
        got = ep.observable("global.npy")
        self.assertEqual(got.shape[-1], len(fields) - 1)
        np.testing.assert_array_equal(got[:, names.index("alive")], ep.column("global.npy", "alive"))

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
