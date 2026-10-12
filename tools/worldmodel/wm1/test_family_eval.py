"""Family evaluator (GOAL_1_2.md 10.2) on synthetic families written to disk and loaded by `wm_data.load_family`:
2 root seeds x 2 variants x 3 replicates x 3 days, two settlements whose home (lowest id) is the second slot.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
import bench_v1 as bv1
from wm_data import load_family, sha256_file
from wm1 import evaluate as ev
from wm1 import families as f
from wm1.test_pilot_harness import GLOBAL, SETTLE, synth_episode

ROWS, FORK = 73, 24                     # 3 days at every 60; fork at tick 1440
WIN = f.Window(nights=(42, 66), cold=(30, 70), food=70, deaths=72)
IDS = np.array([7, 3], dtype=np.int64)   # home = id 3 = slot 1
REPS = 3


def member(seed: int, eid: str, k: int) -> tuple[dict, list]:
    """Arrays and interventions of one member: the shared prefix (rows <= FORK) of root seed `seed`, then
    replicate noise; `gift` adds 5 + k food to each settlement from row 30 and, for k >= 1, one exposure death at
    row 60; `dark` puts the home slot's fire out at the night rows (the other settlement stays lit)."""
    base = synth_episode(seed, ROWS, settlements=2)
    g, s = base.arrays["global.npy"].copy(), base.arrays["settlements.npy"].copy()
    s[:, :, SETTLE.index("fire_lit")] = 1.0
    rng = np.random.default_rng(1000 * seed + k)
    after = slice(FORK + 1, None)
    s[after, :, SETTLE.index("food_stock")] += rng.normal(0, 1.0, (ROWS - FORK - 1, 1))
    g[after, GLOBAL.index("cold_member_hours_total")] += np.cumsum(rng.integers(0, 3, ROWS - FORK - 1))
    acts = []
    if eid == "gift":
        s[30:, :, SETTLE.index("food_stock")] += 5 + k
        if k >= 1:
            g[60:, GLOBAL.index("deaths_exposure")] += 1
        acts = [{"tick": 30 * 60, "kind": "gift_food", "settlement": 3, "amount": 50, "applied": 50}]
    elif eid == "dark":
        for r in WIN.nights:
            s[r, 1, SETTLE.index("fire_lit")] = 0.0
        acts = [{"tick": 40 * 60, "kind": "rain", "settlement": 3, "amount": 5, "applied": 5}]
    meta = dict(base.meta, episode_id=eid, every=60, rows=ROWS, noise_fork={"tick": FORK * 60, "replicate": k})
    return meta | {"_arrays": {"global.npy": g, "settlements.npy": s, "settlement_ids.npy": IDS}}, acts


def write_family(root: Path, seed: int) -> Path:
    fam = root / f"game_v16_s{seed}"
    for eid in ("none", "gift", "dark"):
        for k in range(REPS):
            meta, acts = member(seed, eid, k)
            arrays = meta.pop("_arrays")
            out = fam / f"game_v16_s{seed}_{eid}_r{k}"
            out.mkdir(parents=True)
            meta["arrays"] = {}
            for name, a in arrays.items():
                np.save(out / name, a)
                meta["arrays"][name] = {"sha256": sha256_file(out / name), "shape": list(a.shape), "dtype": str(a.dtype)}
            (out / "meta.json").write_text(json.dumps(meta))
            (out / "interventions.json").write_text(json.dumps(acts))
    return fam


class FoodActionPredictor(ev.Predictor):
    """Fire always lit; food = the start row's food + 0.2 x the gift_food requested after the start (cumulative)."""
    name = "food_action"

    def rollout(self, ep, starts, horizon, actions):
        out = np.repeat(ep.bench.v[np.asarray(starts)][:, None, :], horizon, axis=1)
        gift = b.KINDS.index("gift_food")
        for i, s in enumerate(starts):
            out[i, :, f.V_FOOD] += 0.2 * np.cumsum(actions[s + 1: s + 1 + horizon, gift])
        out[:, :, ev.V_FIRE] = 1.0
        return ev.Rollout(out)


class FamilyEvalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        root = Path(cls.tmp.name)
        cls.fams = [load_family(write_family(root, s), arrays=f.ARRAYS) for s in (5, 6)]

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_c_cf_window_equals_bench_v1(self) -> None:
        w = f.Window.c_cf()
        self.assertEqual(w.nights, tuple(bv1.row_of(d + 1, 120) for d in range(21, 28)))
        self.assertEqual(w.cold, (bv1.row_of(21, 1080), bv1.row_of(28, 420)))
        self.assertEqual((w.food, w.deaths), (bv1.row_of(28, 360), bv1.row_of(28, 360)))

    def test_family_shape_and_home_slot(self) -> None:
        fam = self.fams[0]
        self.assertEqual((fam.replicates, sorted(fam.variants), f.fork_row(fam)), (REPS, ["dark", "gift"], FORK))
        self.assertEqual(f.home_slot(fam.factual[0].arrays["settlement_ids.npy"]), 1)
        f.check_prefix(fam, FORK)

    def test_true_effects_read_the_home_slot_and_pair_replicates(self) -> None:
        t = f.true_effects(self.fams[0], WIN)
        # Home slot dark at both night rows; the other settlement stays lit (the max over settlements would read 0).
        self.assertEqual([t["dark"]["per_replicate"][str(k)]["dark_nights"] for k in range(REPS)], [2.0, 2.0, 2.0])
        self.assertEqual(t["dark"]["spread"]["dark_nights"], 0.0)
        # Food: (5 + k) in each of two settlements, paired with the same replicate's factual noise.
        food = [t["gift"]["per_replicate"][str(k)]["food_store"] for k in range(REPS)]
        np.testing.assert_allclose(food, [10.0, 12.0, 14.0], atol=1e-4)
        self.assertAlmostEqual(t["gift"]["mean"]["food_store"], 12.0, places=4)
        self.assertAlmostEqual(t["gift"]["spread"]["food_store"], 2.0, places=4)
        # Deaths include exposure: one death in replicates 1 and 2.
        self.assertEqual([t["gift"]["per_replicate"][str(k)]["deaths"] for k in range(REPS)], [0.0, 1.0, 1.0])

    def test_scores_mae_sign_and_coverage(self) -> None:
        out = f.evaluate_families({"food_action": FoodActionPredictor(), "persistence": ev.Persistence(),
                                   "zero_effect": None}, self.fams, WIN)
        fa = out["predictors"]["food_action"]
        for seed in ("5", "6"):
            self.assertAlmostEqual(fa["effects"][seed]["gift"]["food_store"], 10.0)
            self.assertEqual(fa["effects"][seed]["dark"]["food_store"], 0.0)
        g = fa["scores"]["gift"]["food_store"]
        self.assertAlmostEqual(g["mean_effect_mae"], 2.0, places=4)    # |10 - 12|
        self.assertEqual(g["sign_accuracy"], 1.0)
        self.assertEqual(g["coverage"], 1.0)                             # 12 within 10 +- 2
        self.assertEqual(g["seeds"], 2)
        dark = fa["scores"]["dark"]["dark_nights"]
        self.assertEqual((dark["mean_effect_mae"], dark["sign_accuracy"], dark["coverage"]), (2.0, 0.0, 0.0))
        z = out["predictors"]["zero_effect"]["scores"]
        self.assertAlmostEqual(z["gift"]["food_store"]["mean_effect_mae"], 12.0, places=4)
        self.assertEqual(z["gift"]["deaths"]["sign_accuracy"], 0.0)
        # Persistence ignores actions: its effects are exactly zero, as the zero-effect baseline's.
        self.assertEqual(out["predictors"]["persistence"]["scores"], z)

    def test_prefix_mismatch_is_refused(self) -> None:
        fam = self.fams[1]
        bad = fam.variants["gift"][2]
        saved = bad.arrays["global.npy"].copy()
        bad.arrays["global.npy"][FORK, GLOBAL.index("alive")] += 1
        try:
            with self.assertRaises(ValueError):
                f.predicted_effects(ev.Persistence(), fam, WIN)
        finally:
            bad.arrays["global.npy"][:] = saved


if __name__ == "__main__":
    unittest.main()
