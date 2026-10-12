"""Loader round-trip on the committed sample (docs/worldmodel/sample).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import numpy as np

from wm_data import load_episode, sha256_file, verify_episode

ROOT = Path(__file__).resolve().parents[2]
SAMPLE = ROOT / "docs/worldmodel/sample"
BIN = Path(os.environ.get("WM_BIN", ROOT / "rust/target/release/worldsim_wm"))
V1_CF = Path(os.environ.get("WM_DATA_ROOT", "/Users/rexxa/work/worldsim-wm-data")) / "v1/C_cf/game_v14_s12000_none"
# Pass 10: a real schema-3 episode recorded by Pass 9 (read-only).
P9_FORK = Path(os.environ.get("WM_DATA_ROOT", "/Users/rexxa/work/worldsim-wm-data")) / "p9/unforked/game_v15_s12000_none"
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


@unittest.skipUnless(BIN.exists(), f"recorder not built: {BIN}")
class SchemaThreeTests(unittest.TestCase):
    """Pass 9: the loader reads schema 1 (the committed sample, above), schema 2
    and schema 3 episodes; Pass 10: and schema 4 (the recorder's own)."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        out = Path(self.tmp.name) / "wm"
        subprocess.run([str(BIN), "--scene", "game_v15", "--seeds", "3..4", "--days", "1", "--every", "60",
                        "--agents", "--workers", "1", "--out", str(out)],
                       check=True, capture_output=True, env={**os.environ, "WORLDSIM_GIT_SHA": "test"})
        self.path = out / "game_v15_s3_none"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_schema_three_carries_lineage_keys_as_identity(self) -> None:
        ep = load_episode(self.path)
        self.assertEqual(ep.meta["schema_version"], 4)
        self.assertIs(ep.meta["keyed_draws"], True)
        self.assertIsNone(ep.meta["noise_fork"])
        keys = ep.lineage_keys()
        self.assertEqual(keys.dtype, np.uint64)
        self.assertEqual(keys.shape, ep.arrays["agent_ids.npy"].shape)
        self.assertTrue((keys != 0).all())
        self.assertEqual(len(set(keys.tolist())), len(keys))
        field = ep.meta["fields"]["agent_keys.npy"][0]
        self.assertTrue(field["identity"] and field["hidden"])
        self.assertEqual(ep.meta["observable_fields"]["agent_keys.npy"], [])

    def _rewrite_as_schema_three(self) -> None:
        # A schema-3 episode is a schema-4 one without the two appended global
        # columns, the appended agent `hp` field and the `world_variety` meta key.
        meta = json.loads((self.path / "meta.json").read_text())
        for name, drop in (("global.npy", 2), ("agents.npy", 1)):
            data = np.load(self.path / name)
            np.save(self.path / name, np.ascontiguousarray(data[..., :-drop]))
            meta["arrays"][name]["shape"] = list(data.shape[:-1]) + [data.shape[-1] - drop]
            meta["arrays"][name]["sha256"] = sha256_file(self.path / name)
            meta["fields"][name] = meta["fields"][name][:-drop]
            meta["observable_fields"][name] = [f["name"] for f in meta["fields"][name] if not f["hidden"]]
        del meta["world_variety"]
        meta["schema_version"] = 3
        (self.path / "meta.json").write_text(json.dumps(meta))

    def test_schema_four_appends_exposure_columns_and_hp(self) -> None:
        ep = load_episode(self.path)
        g = [f["name"] for f in ep.meta["fields"]["global.npy"]]
        self.assertEqual(g[-2:], ["deaths_exposure", "injured_members"])
        self.assertEqual(ep.meta["fields"]["agents.npy"][-1]["name"], "hp")
        self.assertIn("hp", ep.observable_names("agents.npy"))
        self.assertIsNone(ep.meta["world_variety"])
        self.assertIsNone(ep.world_variety())
        self.assertEqual((ep.column("global.npy", "deaths_exposure") == 0).all(), True)
        hp = ep.column("agents.npy", "hp")
        self.assertTrue((hp[~np.isnan(hp)] == 100.0).all())  # day one is spring: nobody is cold
        # A hidden meta block is dropped by observable_meta.
        ep.meta["world_variety"] = {"hidden": True, "values": {"starting_agents": 20}}
        self.assertEqual(ep.world_variety(), {"starting_agents": 20})
        self.assertNotIn("world_variety", ep.observable_meta())
        self.assertIn("scene", ep.observable_meta())

    def test_schema_three_layout_still_loads(self) -> None:
        self._rewrite_as_schema_three()
        ep = load_episode(self.path)
        self.assertEqual(ep.meta["schema_version"], 3)
        self.assertEqual(ep.arrays["global.npy"].shape[1], 35)
        self.assertEqual(ep.arrays["agents.npy"].shape[2], 17)
        self.assertIsNone(ep.world_variety())
        self.assertIsNotNone(ep.lineage_keys())

    @unittest.skipUnless(P9_FORK.exists(), f"Pass 9 fork data not present: {P9_FORK}")
    def test_recorded_schema_three_episode_loads(self) -> None:
        ep = load_episode(P9_FORK)
        self.assertEqual(ep.meta["schema_version"], 3)
        self.assertEqual(ep.arrays["global.npy"].shape[1], 35)

    def test_schema_two_layout_still_loads(self) -> None:
        # A schema-2 episode is a schema-3 one without the lineage keys and the
        # two randomness meta keys (Pass 8's recorder wrote exactly that).
        self._rewrite_as_schema_three()
        meta = json.loads((self.path / "meta.json").read_text())
        (self.path / "agent_keys.npy").unlink()
        for key in ("keyed_draws", "noise_fork"):
            del meta[key]
        del meta["arrays"]["agent_keys.npy"], meta["fields"]["agent_keys.npy"], meta["observable_fields"]["agent_keys.npy"]
        meta["schema_version"] = 2
        (self.path / "meta.json").write_text(json.dumps(meta))
        ep = load_episode(self.path)
        self.assertEqual(ep.meta["schema_version"], 2)
        self.assertIsNone(ep.lineage_keys())
        self.assertIn("agents.npy", ep.arrays)

    @unittest.skipUnless(V1_CF.exists(), f"dataset v1 not present: {V1_CF}")
    def test_recorded_schema_two_episode_loads(self) -> None:
        ep = load_episode(V1_CF)
        self.assertEqual(ep.meta["schema_version"], 2)
        self.assertEqual(sha256_file(V1_CF / "global.npy"), ep.meta["arrays"]["global.npy"]["sha256"])


if __name__ == "__main__":
    unittest.main()
