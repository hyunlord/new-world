"""Run: python3 -m unittest discover -s tools/revitalize -p test_fork_metrics.py."""
import json
import math
import tempfile
import unittest
from pathlib import Path

import numpy as np

from fork_metrics import (constant_rows, dark_night_differs, deaths_effect, divergence, effect_stats, identity,
                          newborn_key_match, replicate0_identity, series, summarize_divergence, targets_of, vrf_of)


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
        self.assertEqual((stats["seeds_factual_noisy"], stats["seeds_both_noisy"]), (6, 6))
        self.assertAlmostEqual(stats["mean_effect"], 2.0)
        self.assertEqual(stats["sign_stability"], 1.0)

    def test_no_noise_is_excluded(self) -> None:
        y_f = np.ones((4, 3))
        stats = effect_stats(y_f + 1, y_f, resamples=20)
        self.assertIsNone(stats["vrf"])
        self.assertTrue(stats["no_agent_noise"])
        self.assertFalse(stats["passes_g0b"])

    def test_constant_factual_gives_vrf_one(self) -> None:
        # Noise only in the variant arm: pairing has nothing to cancel.
        rng = np.random.default_rng(3)
        y_f = np.repeat(np.arange(5.0)[:, None], 8, axis=1)
        y_v = y_f + rng.integers(0, 3, size=(5, 8))
        stats = effect_stats(y_v, y_f, resamples=50)
        self.assertEqual(stats["vrf"], 1.0)
        self.assertEqual(stats["seeds_factual_noisy"], 0)
        self.assertIsNone(stats["vrf_seeds_both_noisy"])
        self.assertFalse(stats["passes_g0b"])

    def test_constant_fractional_columns_are_exactly_quiet(self) -> None:
        # Review round 1, L4: a constant non-integer column is noise-free, not a rounding ratio.
        y_f = np.full((6, 6), 0.1) + np.arange(6)[:, None] * 0.37
        stats = effect_stats(y_f + 2.0, y_f, resamples=20)
        self.assertEqual(stats["v_paired"], 0.0)
        self.assertIsNone(stats["vrf"])

    def test_constancy_is_per_row_and_scale_aware(self) -> None:
        # Review round 2, R2-3: a +-1 difference at level 1e5 is noise; a constant
        # fractional row is not; R = 7 rows of 0.1 are constant.
        rows = np.array([[1e5, 1e5 + 1, 1e5, 1e5, 1e5, 1e5, 1e5], [0.1] * 7, [0.3 * 3] * 7])
        self.assertEqual(constant_rows(rows).tolist(), [False, True, True])
        y_f = np.full((32, 8), 1e5)
        y_v = y_f.copy()
        y_v[0, 3] += 1
        stats = effect_stats(y_v, y_f, resamples=20)
        self.assertFalse(stats["no_agent_noise"])
        self.assertEqual((stats["seeds_factual_noisy"], stats["seeds_variant_noisy"]), (0, 1))

    def test_quiet_bootstrap_resamples_are_excluded(self) -> None:
        # Review round 1, L5: one noisy seed among quiet ones; resamples without it have no VRF.
        y_f = np.zeros((4, 4))
        y_f[0] = [0, 1, 0, 1]
        y_v = y_f.copy()
        stats = effect_stats(y_v, y_f, resamples=200)
        self.assertTrue(math.isinf(stats["vrf"]))
        self.assertGreater(stats["bootstrap_quiet_resamples"], 0)
        self.assertTrue(math.isinf(stats["vrf_lo"]))

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
            self.assertFalse(result["interventions_equal"] and result["schedules_equal_without_amount_zero"])
            # A real variant against itself unforked: same nonzero schedule, identical.
            same_gift = episode(root, "g2", data, real, {**placebo_meta, "noise_fork": None,
                                                         "schedule": [{**placebo_meta["schedule"][0], "amount": 3}]})
            self.assertTrue(identity(gift, same_gift)["identical"])
            other = episode(root, "o", data, [], {**meta, "seed": 4})
            self.assertEqual(identity(other, factual)["meta_differing"], ["seed"])


class ReplicateZeroTests(unittest.TestCase):
    def test_missing_unforked_run_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            meta = {"scene": "game_v15", "episode_id": "none", "noise_fork": {"tick": 5, "replicate": 0}, "schedule": None}
            data = {"global.npy": np.arange(4.0)}
            (root / "forks").mkdir()
            (root / "unforked").mkdir()
            episode(root / "forks", "game_v15_s1_none_r0", data, [], meta)
            episode(root / "forks", "game_v15_s2_none_r0", data, [], meta)
            episode(root / "unforked", "game_v15_s1_none", data, [], {**meta, "noise_fork": None})
            rows = replicate0_identity(root / "forks", root / "unforked", "game_v15", [1, 2], ["none"])
            self.assertEqual([r["identical"] for r in rows], [True, False])
            self.assertTrue(rows[1]["missing_unforked_run"])


class DivergenceTests(unittest.TestCase):
    def test_moved_share_rows_are_found_by_tick(self) -> None:
        # Review round 2, R2-6: row 0 is the intervention tick; "after 1 h" is row 1.
        ticks = [29400 + 60 * i for i in range(80)]
        per_seed = [{"ticks": ticks, "moved_share": [float(i) for i in range(80)], "first_moved_tick": 29460,
                     "first_target_diff_tick": {k: None for k in ("dark_nights", "cold_member_hours", "food_store", "deaths", "births")}}]
        out = summarize_divergence(per_seed, 29400)
        self.assertEqual([out[f"median_moved_share_after_{h}h"] for h in (1, 6, 24, 72)], [1.0, 6.0, 24.0, 72.0])
        self.assertEqual(out["first_moved"]["median_ticks_after"], 60.0)

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


def write_episode(path: Path, exposure_at_0600_day29: float, starvation: float = 0.0) -> Path:
    """A minimal schema-4 episode (rows every 60 ticks, 35 days) with the columns the C_cf targets read."""
    import hashlib
    t = 35 * 24 + 1
    gnames = ["tick", "minute_of_day", "season", "winter_severity", "cold_members", "alive", "deadwood_total",
              "food_on_bushes_total", "adults_working", "cold_member_hours_total", "births", "deaths_starvation",
              "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure"]
    g = np.zeros((t, len(gnames)), dtype=np.float32)
    g[:, 0] = 60 * np.arange(t)
    g[:, 1] = (60 * np.arange(t) + 480) % 1440
    g[:, 5] = 40
    row = (28 * 1440 + 360 - 480) // 60
    g[row:, gnames.index("deaths_exposure")] = exposure_at_0600_day29
    g[row:, gnames.index("deaths_starvation")] = starvation
    s = np.zeros((t, 1, 4), dtype=np.float32)
    s[:, 0, 2] = 1.0  # fire lit
    s[:, 0, 3] = 40
    agents = np.zeros((t, 1, 2), dtype=np.float32)
    path.mkdir(parents=True)
    arrays = {"global.npy": g, "settlements.npy": s, "agents.npy": agents, "agent_alive.npy": np.ones((t, 1), dtype=np.uint8),
              "agent_keys.npy": np.array([7], dtype=np.uint64)}
    meta = {"schema_version": 4, "seed": 1, "episode_id": "none", "arrays": {}, "fields": {
        "global.npy": [{"name": n, "hidden": n == "winter_severity"} for n in gnames],
        "settlements.npy": [{"name": n, "hidden": False} for n in ("food_stock", "wood_stock", "fire_lit", "members")],
        "agents.npy": [{"name": n, "hidden": False} for n in ("x", "y")]}}
    meta["observable_fields"] = {k: [f["name"] for f in v if not f["hidden"]] for k, v in meta["fields"].items()}
    for name, data in arrays.items():
        np.save(path / name, data)
        meta["arrays"][name] = {"shape": list(data.shape), "dtype": str(data.dtype),
                                "sha256": hashlib.sha256((path / name).read_bytes()).hexdigest()}
    (path / "meta.json").write_text(json.dumps(meta))
    (path / "interventions.json").write_text("[]")
    return path


class ExposureDeathsTests(unittest.TestCase):
    def test_deaths_with_exposure_reads_the_day29_0600_row(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            ep = write_episode(Path(tmp) / "e", exposure_at_0600_day29=3.0, starvation=1.0)
            self.assertEqual(targets_of(ep)["deaths"], 1.0)
            self.assertEqual(targets_of(ep, with_exposure=True)["deaths"], 4.0)
            plain, counted = series(ep), series(ep, with_exposure=True)
            row = (28 * 1440 + 360 - 480) // 60
            self.assertEqual((float(plain["deaths"][row]), float(counted["deaths"][row])), (1.0, 4.0))
            self.assertEqual(float(counted["deaths"][row - 1]), 0.0)

    def test_deaths_effect_and_v15_comparison(self) -> None:
        y_f = np.zeros((4, 2))
        y_v = np.array([[1.0, 0.0], [0.0, 0.0], [-1.0, 0.0], [2.0, 2.0]])
        d = deaths_effect(y_v, y_f)
        self.assertEqual((d["share_positive"], d["share_negative"]), (0.5, 0.25))
        self.assertAlmostEqual(d["mean"], (0.5 + 0 - 0.5 + 2) / 4)
        c = dark_night_differs(y_v, y_f, np.zeros((4, 8)), np.zeros((4, 8)))
        self.assertEqual((c["seeds_differing"], c["share_differs"]), (3, 0.75))



class HomeSlotFireTests(unittest.TestCase):
    """Pass 11: with --home-slot-fire, dark nights come from the home settlement's slot."""

    def test_a_daughter_fire_does_not_hide_a_dark_home(self) -> None:
        import hashlib
        import fork_metrics
        from bench_v1 import row_of as r_of
        with tempfile.TemporaryDirectory() as tmp:
            ep = write_episode(Path(tmp) / "e", exposure_at_0600_day29=0.0)
            s = np.load(ep / "settlements.npy")
            two = np.concatenate([s, s], axis=1)
            two[:, 0, 2] = 1.0  # the daughter (id 5, slot 0) always lit
            for day in range(21, 28):  # the home (id 1, slot 1) dark at 02:00 of days 23..29
                two[r_of(day + 1, 120), 1, 2] = 0.0
            np.save(ep / "settlements.npy", two)
            np.save(ep / "settlement_ids.npy", np.array([5, 1], dtype=np.int64))
            meta = json.loads((ep / "meta.json").read_text())
            for name in ("settlements.npy", "settlement_ids.npy"):
                data = np.load(ep / name)
                meta["arrays"][name] = {"shape": list(data.shape), "dtype": str(data.dtype),
                                        "sha256": hashlib.sha256((ep / name).read_bytes()).hexdigest()}
            (ep / "meta.json").write_text(json.dumps(meta))
            self.assertEqual(targets_of(ep)["dark_nights"], 0.0)
            try:
                fork_metrics.HOME_SLOT_FIRE = True
                self.assertEqual(targets_of(ep)["dark_nights"], 7.0)
                self.assertEqual(float(series(ep)["dark_nights"][r_of(23, 120)]), 0.0)
            finally:
                fork_metrics.HOME_SLOT_FIRE = False


if __name__ == "__main__":
    unittest.main()
