"""Recorder equivalence (GOAL_1_3.md 6.1): columns matched by name, exact / bit-identical, extra columns listed.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
from wm_data import sha256_file
from wm1 import recorder_equiv as re_


def write_episode(path: Path, g_names, g, s_names, s, schema: int) -> None:
    path.mkdir(parents=True)
    arrays = {"global.npy": g, "settlements.npy": s, "row_digest.npy": np.arange(len(g), dtype=np.uint64)}
    meta = {"schema_version": schema, "arrays": {}, "fields": {"global.npy": [{"name": n} for n in g_names],
                                                               "settlements.npy": [{"name": n} for n in s_names]}}
    for name, a in arrays.items():
        np.save(path / name, a)
        meta["arrays"][name] = {"sha256": sha256_file(path / name), "shape": list(a.shape), "dtype": str(a.dtype)}
    (path / "meta.json").write_text(json.dumps(meta))
    (path / "interventions.json").write_text("[]")


class EquivalenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        rng = np.random.default_rng(0)
        self.g4 = rng.random((10, 3)).astype(np.float32)
        self.s4 = rng.random((10, 1, 2)).astype(np.float32)
        write_episode(self.root / "old/e_s1_none", ("a", "b", "c"), self.g4, ("m", "n"), self.s4, 4)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def new(self, g, s, g_names=("a", "b", "c", "x"), s_names=("m", "n", "y")) -> dict:
        write_episode(self.root / "new/e_s1_none", g_names, g, s_names, s, 5)
        return re_.compare_dirs(self.root / "new", self.root / "old")

    def test_appended_columns_are_listed_and_the_rest_is_identical(self) -> None:
        g = np.concatenate([self.g4, np.ones((10, 1), np.float32)], 1)
        s = np.concatenate([self.s4, np.zeros((10, 1, 1), np.float32)], 2)
        res = self.new(g, s)
        self.assertTrue(res["identical"])
        self.assertEqual(res["only_new"], ["global.npy:x", "settlements.npy:y"])
        self.assertEqual(res["episodes"][0]["compared"], 3 + 2 + 1)

    def test_columns_are_matched_by_name_not_position(self) -> None:
        g = np.concatenate([np.ones((10, 1), np.float32), self.g4], 1)
        s = np.concatenate([self.s4, np.zeros((10, 1, 1), np.float32)], 2)
        self.assertTrue(self.new(g, s, g_names=("x", "a", "b", "c"))["identical"])

    def test_one_bit_of_difference_is_found(self) -> None:
        g = np.concatenate([self.g4, np.ones((10, 1), np.float32)], 1)
        g[3, 1] = np.nextafter(g[3, 1], np.float32(2))
        s = np.concatenate([self.s4, np.zeros((10, 1, 1), np.float32)], 2)
        res = self.new(g, s)
        self.assertFalse(res["identical"])
        self.assertEqual(res["episodes"][0]["differ"], ["global.npy:b"])

    def test_dtype_change_and_signed_zero_differ(self) -> None:
        self.assertFalse(re_.identical(np.zeros(3, np.float32), np.zeros(3, np.float64)))
        self.assertFalse(re_.identical(np.array([0.0]), np.array([-0.0])))
        self.assertTrue(re_.identical(np.array([np.nan]), np.array([np.nan])))


if __name__ == "__main__":
    unittest.main()
