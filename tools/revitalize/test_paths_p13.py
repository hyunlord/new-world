"""Synthetic checks of paths_p13 (PASS13_GOAL.md 5.4): identical replicates give PS = 0, identical worlds with noise
give PS ~ 1, a nonlinear static map gives PS = 0 (the property UBS lacked), the W/R correction is unbiased, and the
paired bootstrap separates a noisy scene from a noise-free one."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import paths_p13 as p13  # noqa: E402

T = 2881  # 120 days, every 60 minutes


class DecomposeTest(unittest.TestCase):
    def test_identical_replicates_give_ps_zero(self):
        theta = np.random.default_rng(1).normal(size=32)
        y = np.repeat((3.0 * theta + 10.0)[:, None], 4, axis=1)
        d = p13.decompose(y)
        self.assertEqual(d["W"], 0.0)
        self.assertGreater(d["B"], 0.0)
        self.assertEqual(d["PS"], 0.0)

    def test_identical_worlds_with_noise_give_ps_near_one(self):
        y = 5.0 + np.random.default_rng(2).normal(size=(64, 4))
        d = p13.decompose(y)
        print(f"\nidentical worlds + noise: W {d['W']:.3f} B {d['B']:.3f} PS {d['PS']:.3f}")
        self.assertGreater(d["PS"], 0.9)

    def test_nonlinear_static_map_gives_ps_zero(self):
        theta = np.random.default_rng(3).uniform(size=64)
        for g in (lambda t: np.where(t > 0.5, 1.0, 0.0), lambda t: 10.0 * t * t):
            y = np.repeat(g(theta)[:, None], 4, axis=1)
            self.assertEqual(p13.decompose(y)["PS"], 0.0)

    def test_between_correction_is_unbiased(self):
        rng = np.random.default_rng(4)
        y = rng.normal(0, 1.0, size=(2000, 1)) + rng.normal(0, 2.0, size=(2000, 4))
        d = p13.decompose(y)
        print(f"\nsigma_b 1, sigma_w 2: W {d['W']:.3f} B {d['B']:.3f} PS {d['PS']:.3f} (expected 0.8)")
        self.assertAlmostEqual(d["PS"], 0.8, delta=0.05)
        self.assertAlmostEqual(d["B"], 1.0, delta=0.15)

    def test_floor_at_zero_and_nan_when_both_zero(self):
        # Worlds whose means are closer than replicate noise implies: raw B < 0 -> floored, PS = 1.
        y = np.array([[0.0, 2.0], [2.0, 0.0], [1.0, 1.0 + 1e-9]])
        d = p13.decompose(y)
        self.assertEqual(d["B"], 0.0)
        self.assertEqual(d["PS"], 1.0)
        d = p13.decompose(np.full((5, 3), 7.0))
        self.assertEqual((d["W"], d["B"]), (0.0, 0.0))
        self.assertTrue(np.isnan(d["PS"]))

    def test_trailing_axes(self):
        rng = np.random.default_rng(5)
        y = rng.normal(size=(10, 3, 4, 2))
        d = p13.decompose(y)
        self.assertEqual(d["PS"].shape, (4, 2))
        self.assertAlmostEqual(float(d["PS"][2, 1]), p13.decompose(y[:, :, 2, 1])["PS"])


class BootstrapTest(unittest.TestCase):
    def test_paired_difference_excludes_zero(self):
        rng = np.random.default_rng(6)
        w, r, d, j = 32, 4, len(p13.DAYS), 2
        level = rng.normal(0, 1.0, size=(w, 1, d, j))
        quiet = np.repeat(level, r, axis=1)
        noisy = level + rng.normal(0, 1.0, size=(w, r, d, j))
        s = np.ones(j)
        pb = p13.paired_bootstrap(quiet, s, noisy, s, n_boot=200)
        lo, hi = pb["ci"]["PS_mean"]
        self.assertTrue(np.all(hi < 0), (lo, hi))
        self.assertTrue(np.all(pb["point"]["PS_mean"] < 0))
        same = p13.paired_bootstrap(noisy, s, noisy, s, n_boot=50)
        self.assertTrue(np.all(same["ci"]["PS_mean"][0] == 0) and np.all(same["ci"]["PS_mean"][1] == 0))

    def test_scene_interval_brackets_point(self):
        rng = np.random.default_rng(7)
        y = rng.normal(size=(24, 1, 4, 1)) + 0.7 * rng.normal(size=(24, 3, 4, 1))
        st = p13.scene_stats(y, np.ones(1))
        lo, hi = p13.bootstrap_scene(y, np.ones(1), n_boot=200)["PS_mean"]
        self.assertTrue(lo[0] <= st["PS_mean"][0] <= hi[0])


def write_episode(root: Path, name: str, scene: str, seed: int, rep, rng_seed: int, schema: int = 5) -> Path:
    """A tiny episode in the recorder's on-disk layout (meta.json with sha256, global.npy, settlements.npy)."""
    gnames = ["tick", "minute_of_day", "season", "winter_severity", "alive", "cold_members", "deadwood_total",
              "food_on_bushes_total", "adults_working", "deaths_starvation", "deaths_dehydration", "deaths_combat",
              "deaths_old_age", "cold_member_hours_total"]
    snames = ["members", "food_stock", "wood_stock", "wood_target", "fire_lit"]
    snames += ["fission_pressure", "fission_threshold", "depart_tick"]  # schema 6 layout
    rng = np.random.default_rng(rng_seed)
    tick = np.arange(T) * 60.0
    g = np.zeros((T, len(gnames)), dtype=np.float32)
    g[:, 0] = tick
    g[:, 1] = (tick + 480) % 1440
    g[:, gnames.index("alive")] = 20 + rng.integers(0, 5, T)
    g[:, gnames.index("deadwood_total")] = rng.normal(50, 3, T)
    s = np.full((T, 3, len(snames)), np.nan, dtype=np.float32)
    s[:, 0, 0] = 10
    s[:, 0, 1] = rng.normal(30, 2, T)
    s[:, 0, 2] = 5
    s[1000:, 1, 0] = 4
    s[1000:, 1, 1] = 2
    s[1000:, 1, 2] = 1
    s[2000:, 1, 0] = 0  # dissolved: members 0, still a slot
    path = root / name
    path.mkdir(parents=True)
    arrays = {}
    for fname, arr in (("global.npy", g), ("settlements.npy", s)):
        np.save(path / fname, arr)
        arrays[fname] = {"sha256": hashlib.sha256((path / fname).read_bytes()).hexdigest(),
                         "shape": list(arr.shape), "dtype": str(arr.dtype)}
    hidden = {"winter_severity", "wood_target", "fission_pressure", "fission_threshold"}
    fields = {"global.npy": [{"name": n, "hidden": n in hidden} for n in gnames],
              "settlements.npy": [{"name": n, "hidden": n in hidden} for n in snames]}
    meta = {"schema_version": schema, "scene": scene, "seed": seed, "episode_id": "none", "arrays": arrays,
            "fields": fields, "noise_fork": None if rep is None else {"tick": 1, "replicate": rep},
            "final_fingerprint": f"fp{rng_seed}"}
    (path / "meta.json").write_text(json.dumps(meta))
    (path / "interventions.json").write_text("[]")
    return path


class LoaderTest(unittest.TestCase):
    def test_member_name_parsing(self):
        self.assertEqual(p13.parse_member("game_v18_s901003_none_r2"), (901003, 2))
        self.assertEqual(p13.parse_member("game_v18_nofission_s7_none_r0"), (7, 0))
        self.assertIsNone(p13.parse_member("game_v18_s901003_gift_wood_r1"))
        self.assertIsNone(p13.parse_member("game_v18_s901003_none_r1.tmp"))
        self.assertIsNone(p13.parse_member("game_v18_s901003_none"))

    def test_load_scene_and_unforked_check(self):
        # The newest schema wm_data accepts; the settlements table always carries schema 6's three extra columns,
        # so the loader is shown not to depend on the number of settlement columns.
        schema = max(p13.wm_data.SCHEMAS)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "fam"
            for seed in (11, 12, 13):
                for k in range(2):
                    write_episode(root, f"sc_s{seed}_none_r{k}", "sc", seed, k, 100 * seed + k, schema)
            write_episode(root, "sc_s14_none_r0", "sc", 14, 0, 1400, schema)  # incomplete world: dropped
            write_episode(root, "sc_s11_gift_wood_r0", "sc", 11, 0, 9, schema)  # a variant: ignored
            (root / "sc_s15_none_r0.tmp").mkdir()  # being written: ignored
            sc = p13.load_scene("sc", root)
            self.assertEqual(sc.seeds, [11, 12, 13])
            self.assertEqual(sc.r, 2)
            self.assertEqual(sc.dropped, {"14": [0]})
            self.assertEqual(sc.y.shape, (3, 2, len(p13.DAYS), len(p13.VARS)))
            ja = p13.VARS.index("settlements_alive")
            # day 30 rows < 1000: one settlement; day 60 rows in 1000..2000: two; day 118: the second has 0 members
            self.assertEqual(sc.y[0, 0, :, ja].tolist(), [1.0, 2.0, 1.0, 1.0])
            jw = p13.VARS.index("wood_stock")
            self.assertEqual(sc.y[0, 0, :, jw].tolist(), [5.0, 6.0, 6.0, 6.0])
            self.assertTrue(np.all(sc.s >= 0))
            block = p13.scene_block(sc, n_boot=20)
            self.assertEqual(block["vars"]["wood_stock"]["PS"]["30"], None)  # W = B = 0 -> NaN -> null
            json.dumps(block)
            unforked = Path(tmp) / "unforked"
            unforked.mkdir()
            shutil.copytree(root / "sc_s11_none_r0", unforked / "sc_s11_none")
            write_episode(unforked, "sc_s12_none", "sc", 12, None, 777, schema)
            chk = p13.check_unforked(root, unforked, [11, 12])
            self.assertTrue(chk["seeds"]["11"]["identical"])
            self.assertFalse(chk["seeds"]["12"]["identical"])
            self.assertIn("global.npy", chk["seeds"]["12"]["differ"])
            self.assertFalse(chk["all_identical"])
            out = Path(tmp) / "out"
            rc = p13.main(["--scene", f"a={root}", "--scene", f"b={root}", "--diff", "a:b", "--boot", "10",
                           "--check-unforked", str(root), str(unforked), "--seeds", "11", "--out", str(out)])
            self.assertEqual(rc, 0)
            data = json.loads((out / "paths.json").read_text())
            self.assertEqual(data["diffs"]["a - b"]["vars"]["alive"]["PS_mean"], 0.0)
            self.assertIn("## a — W 3, R 2", (out / "paths.md").read_text())


if __name__ == "__main__":
    unittest.main()
