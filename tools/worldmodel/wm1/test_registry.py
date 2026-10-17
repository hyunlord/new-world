"""Dataset registry and the p10 pilot manifest (GOAL_1_2.md 6.3, 4.2).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import unittest

from wm1 import common as c


class RegistryTests(unittest.TestCase):
    def test_v0_entry_equals_the_hard_coded_constants(self) -> None:
        v0 = c.dataset("v0")
        self.assertEqual(v0.manifest, c.MANIFEST)
        self.assertEqual(v0.groups, c.GROUPS)
        self.assertEqual(v0.val, c.VAL_SPLITS)
        self.assertEqual(v0.test, c.TEST_SPLITS)
        self.assertEqual(v0.train_groups(False), ["A_train"])
        self.assertEqual(v0.train_groups(True), ["A_train", "B_train"])
        self.assertEqual((v0.cf, v0.t3_test, v0.tasks, v0.observable), ("C_test", "A_test", None, False))
        self.assertEqual(c.manifest_sha(), c.manifest_sha("v0"))
        self.assertEqual(len(c.manifest()["episodes"]), 400)

    def test_pilot_roles_and_scenes(self) -> None:
        raw = json.loads(c.PILOT_MANIFEST.read_text())
        self.assertEqual(raw["episode_count"], 336)
        for name, scene in (("p10_v16", "game_v16"), ("p10_v16n", "game_v16_neutral"), ("p10_v15", "game_v15")):
            ds = c.dataset(name)
            self.assertIsNone(ds.cf)
            self.assertEqual(ds.tasks, ("T1", "T2", "T2_any"))
            eps = c.manifest(name)["episodes"]
            self.assertEqual(len(eps), 112)
            self.assertEqual({e["scene"] for e in eps} | {e["split"] for e in eps}, {scene})
            for role, lo, hi in (("train", 20000, 20079), ("val", 20080, 20095), ("test", 20096, 20111)):
                seeds = sorted(e["seed"] for e in eps if e["role"] == role)
                self.assertEqual(seeds, list(range(lo, hi + 1)), (name, role))
            for e in eps:
                self.assertEqual(e["schedule_id"], "none")
                self.assertEqual(e["path"], f"p10/pilot/{scene}/{scene}_s{e['seed']}_none")
                self.assertEqual(e["rows"], 2881)
                self.assertTrue(e["files"] and all(len(f["sha256"]) == 64 for f in e["files"].values()))

    def test_unknown_dataset_is_refused(self) -> None:
        with self.assertRaises(ValueError):
            c.dataset("v2")


if __name__ == "__main__":
    unittest.main()
