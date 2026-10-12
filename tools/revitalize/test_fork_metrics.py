"""Run: python3 -m unittest discover -s tools/revitalize -p test_fork_metrics.py."""
import json
import math
import tempfile
import unittest
from pathlib import Path

import numpy as np

from fork_metrics import effect_stats, identity, newborn_key_match, divergence, vrf_of


class StatisticsTests(unittest.TestCase):
    def test_shared_noise_gives_infinite_vrf(self) -> None:
        rng = np.random.default_rng(0)
        noise = rng.integers(0, 9, size=(6, 8)).astype(float)
        y_f = noise + np.arange(6)[:, None]
        y_v = y_f + 2.0
        stats = effect_stats(y_v, y_f, resamples=50)
        self.assertEqual(stats["v_paired"], 0.0)
        self.assertTrue(math.isinf(stats["vrf"]) and math.isinf(stats["vrf_lo"]))
        self.assertTrue(stats["passes_g0b"])
        self.assertAlmostEqual(stats["mean_effect"], 2.0)
        self.assertEqual(stats["sign_stability"], 1.0)

    def test_no_noise_is_excluded(self) -> None:
        y_f = np.ones((4, 3))
        stats = effect_stats(y_f + 1, y_f, resamples=20)
        self.assertIsNone(stats["vrf"])
        self.assertTrue(stats["no_agent_noise"])
        self.assertFalse(stats["passes_g0b"])

    def test_hand_computed_vrf(self) -> None:
        # One seed, R = 3. Factual 0, 1, 2; variant 1, 3, 2.
        y_f = np.array([[0.0, 1.0, 2.0]])
        y_v = np.array([[1.0, 3.0, 2.0]])
        # paired 1, 2, 0 -> var 1; unpaired (k' = k+1): 1-1, 3-2, 2-0 = 0, 1, 2 -> var 1.
        self.assertEqual(vrf_of(y_v, y_f), (1.0, 1.0, 1.0))
        y_v = np.array([[0.5, 1.5, 2.5]])
        # paired constant 0.5 -> 0; unpaired -0.5, -0.5, 2.5 -> var 3.
        v_p, v_u, vrf = vrf_of(y_v, y_f)
        self.assertEqual(v_p, 0.0)
        self.assertAlmostEqual(v_u, 3.0)
        self.assertTrue(math.isinf(vrf))

    def test_variances_stay_within_seeds(self) -> None:
        # Seeds differ by large offsets in both arms; within each seed paired noise is
        # tiny and unpaired noise is that of the factual: the offsets never enter.
        rng = np.random.default_rng(1)
        y_f = rng.normal(size=(10, 8)) + 1000 * np.arange(10)[:, None]
        y_v = y_f + 0.01 * rng.normal(size=(10, 8))
        stats = effect_stats(y_v, y_f, resamples=200)
        self.assertGreater(stats["vrf"], 1000)
        self.assertGreater(stats["vrf_lo"], 1)

    def test_bootstrap_is_reproducible_and_brackets(self) -> None:
        rng = np.random.default_rng(2)
        y_f = rng.normal(size=(12, 8))
        y_v = y_f + 0.5 * rng.normal(size=(12, 8)) + 1
        a, b = effect_stats(y_v, y_f), effect_stats(y_v, y_f)
        self.assertEqual(a, b)
        self.assertLessEqual(a["vrf_lo"], a["vrf"])
        self.assertLessEqual(a["vrf"], a["vrf_hi"])

    def test_sign_stability_counts_zero_only_with_zero(self) -> None:
        y_f = np.zeros((3, 2))
        y_v = np.array([[1.0, 2.0], [0.0, 0.0], [1.0, -1.0]])
        stats = effect_stats(y_v, y_f, resamples=10)
        self.assertAlmostEqual(stats["sign_stability"], 2 / 3)


def episode(root: Path, name: str, npy: dict, interventions: list, meta: dict) -> Path:
    path = root / name
    path.mkdir()
    for file, array in npy.items():
        np.save(path / file, array)
    (path / "interventions.json").write_text(json.dumps(interventions))
    (path / "meta.json").write_text(json.dumps(meta))
    return path


class IdentityTests(unittest.TestCase):
    def test_placebo_identity_rules(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            meta = {"scene": "game_v15", "episode_id": "none", "noise_fork": {"tick": 5, "replicate": 1}, "schedule": None}
            data = {"global.npy": np.arange(6.0).reshape(3, 2)}
            factual = episode(root, "f", data, [], meta)
            zero = [{"tick": 9, "kind": "rain", "settlement": 1, "amount": 0, "applied": 0}]
            placebo_meta = {**meta, "episode_id": "p0", "schedule": [{"tick": 9, "kind": "rain", "settlement": "home", "amount": 0}]}
            placebo = episode(root, "p", data, zero, placebo_meta)
            self.assertTrue(identity(placebo, factual)["identical"])
            moved = episode(root, "m", {"global.npy": np.arange(6.0).reshape(3, 2) + 1}, zero, placebo_meta)
            self.assertEqual(identity(moved, factual)["npy_differing"], ["global.npy"])
            real = [{**zero[0], "amount": 3, "applied": 3}]
            gift = episode(root, "g", data, real, {**placebo_meta, "schedule": [{**placebo_meta["schedule"][0], "amount": 3}]})
            result = identity(gift, factual)
            self.assertFalse(result["identical"])
            self.assertFalse(result["interventions_equal"] and result["schedules_only_amount_zero"])
            other = episode(root, "o", data, [], {**meta, "seed": 4})
            self.assertEqual(identity(other, factual)["meta_differing"], ["seed"])


class DivergenceTests(unittest.TestCase):
    def arm(self, xy, alive, keys, food):
        t = len(food)
        zeros = np.zeros(t)
        return {"tick": np.arange(t) * 60, "dark_nights": zeros, "cold_member_hours": zeros, "food_store": np.array(food, float),
                "deaths": zeros, "births": zeros, "alive": np.array(alive, bool), "xy": np.array(xy, float), "keys": np.array(keys, np.uint64)}

    def test_agents_are_matched_by_key_not_by_column(self) -> None:
        # Same agents in a different column order; agent 7 moves at row 2.
        f = self.arm([[[0, 0], [1, 1]], [[0, 0], [1, 1]], [[0, 0], [1, 1]]], [[1, 1]] * 3, [7, 9], [5, 5, 5])
        v = self.arm([[[1, 1], [0, 0]], [[1, 1], [0, 0]], [[1, 1], [0, 1]]], [[1, 1]] * 3, [9, 7], [5, 5, 4])
        d = divergence(f, v, 0)
        self.assertEqual(d["moved_share"], [0.0, 0.0, 0.5])
        self.assertEqual(d["first_moved_tick"], 120)
        self.assertEqual(d["first_target_diff_tick"]["food_store"], 120)
        self.assertIsNone(d["first_target_diff_tick"]["deaths"])

    def test_newborn_keys(self) -> None:
        # Column 0 a founder; columns 1-2 newborns from row 1 (keys 5 and 6 vs 5 and 8).
        f = self.arm(np.zeros((3, 3, 2)), [[1, 0, 0], [1, 1, 1], [1, 1, 1]], [1, 5, 6], [0, 0, 0])
        v = self.arm(np.zeros((3, 3, 2)), [[1, 0, 0], [1, 1, 1], [1, 1, 1]], [1, 5, 8], [0, 0, 0])
        self.assertEqual(newborn_key_match(f, v, 0), {"newborns_either": 3, "newborns_both_equal_key": 1, "share": 1 / 3})


if __name__ == "__main__":
    unittest.main()
