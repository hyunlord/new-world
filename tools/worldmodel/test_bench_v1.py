"""WM-Bench v1 on synthetic arrays (PASS8_GOAL.md 11.4).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest
from pathlib import Path

import numpy as np

import bench_v1 as b
from wm_data import Episode

DAYS = 35
T = DAYS * 24 + 1


def fake_episode(hidden=("winter_severity",), settlement_hidden=("wood_target",)) -> Episode:
    gnames = ["tick", "minute_of_day", "season", "winter_severity", "cold_members", "alive", "deadwood_total",
              "food_on_bushes_total", "adults_working", "deaths_starvation", "deaths_dehydration", "deaths_combat",
              "deaths_old_age", "cold_member_hours_total"]
    snames = ["members", "food_stock", "wood_stock", "wood_target", "fire_lit"]
    tick = np.arange(T) * 60
    minute = (tick + 480) % 1440
    day = (tick + 480) // 1440
    season = (day // 7) % 4
    g = np.zeros((T, len(gnames)), dtype=np.float32)
    g[:, 0], g[:, 1], g[:, 2] = tick, minute, season
    g[:, 3] = np.where(season == 3, 2, -1)
    g[:, 5] = 40
    g[:, 6] = 20 - (season == 3) * 10
    s = np.zeros((T, 1, len(snames)), dtype=np.float32)
    s[:, 0, 0], s[:, 0, 1], s[:, 0, 2], s[:, 0, 4] = 40, 200, 100 - tick / 1000, 1
    meta = {"schema_version": 2,
            "fields": {"global.npy": [{"name": n, "hidden": n in hidden} for n in gnames],
                       "settlements.npy": [{"name": n, "hidden": n in settlement_hidden} for n in snames]}}
    meta["observable_fields"] = {a: [f["name"] for f in fs if not f["hidden"]] for a, fs in meta["fields"].items()}
    return Episode(Path("fake"), meta, {"global.npy": g, "settlements.npy": s}, [])


def ep_from(arrays: dict, seed=0, role="test", split="A_obs", eid="none") -> b.Ep:
    return b.Ep(split, role, seed, eid, **arrays)


class BenchV1Tests(unittest.TestCase):
    def test_inputs_are_observable_fields_only(self) -> None:
        arrays = b.episode_arrays(fake_episode())
        self.assertEqual(arrays["v"].shape, (T, len(b.VARS)))
        # The hidden severity is carried for labels and oracles, never as V.
        self.assertTrue(np.all(arrays["hidden_severity"][arrays["season"] == 3] == 2))
        with self.assertRaises(b.HiddenInput):
            b.episode_arrays(fake_episode(hidden=("winter_severity", "alive")))
        with self.assertRaises(b.HiddenInput):
            b.episode_arrays(fake_episode(settlement_hidden=("wood_target", "wood_stock")))
        ep = fake_episode()
        with self.assertRaises(b.HiddenInput):
            b.observable_column(ep, "global.npy", "winter_severity")
        # Non-oracle features have no severity columns; the oracle adds four.
        a = ep_from(arrays)
        t = np.array([30, 31])
        plain = b.features(a.v, a.hour, a.season, t)
        oracle = b.features(a.v, a.hour, a.season, t, severity=a.hidden_severity)
        self.assertEqual(oracle.shape[1] - plain.shape[1], 4)
        self.assertEqual(plain.shape[1], len(b.LAGS) * len(b.VARS) + 24 + 5)

    def test_t2_starts_every_hour_of_the_start_days(self) -> None:
        rows = 120 * 24 + 1
        every = b.rollout_starts(rows, 24, True)
        six = b.rollout_starts(rows, 24, False)
        self.assertEqual(every[:24], [b.row_of(2, 60 * h) for h in range(24)])
        self.assertEqual(len(every), 24 * len(six))
        self.assertEqual(six[0], b.row_of(2, 360))
        self.assertTrue(all(r + 168 < rows for r in b.rollout_starts(rows, 168, True)))
        # Start days are 3, 10, 17, ... (one-based): the second block is day 10.
        self.assertEqual(every[24], b.row_of(9, 0))

    def test_rollout_batch_matches_a_constant_model(self) -> None:
        a = ep_from(b.episode_arrays(fake_episode()))

        class Const:
            def predict(self, x):
                return np.tile(np.arange(len(b.VARS), dtype=float), (len(x), 1))
        starts = np.array(b.rollout_starts(T, 24, True)[:5])
        out = b.rollout_batch(a, starts, 24, Const(), b.Variant("ridge"))
        np.testing.assert_allclose(out, a.v[starts] + 24 * np.arange(len(b.VARS)))

    def test_rollout_oracle_reads_each_rows_severity(self) -> None:
        # Review round 1 (M1): a start in autumn whose rollout enters winter sees the winter's severity.
        a = ep_from(b.episode_arrays(fake_episode()))
        seen = []

        class Spy:
            def predict(self, x):
                seen.append(x[:, -4:].copy())
                return np.zeros((len(x), len(b.VARS)))
        start = np.array([b.row_of(20, 0)])  # autumn's last day
        b.rollout_batch(a, start, 48, Spy(), b.Variant("ridge_severity_oracle", oracle=True))
        levels = [int(np.argmax(x[0])) - 1 for x in seen]
        self.assertEqual(levels[0], -1)
        self.assertEqual(levels[-1], 2, "the rollout row in winter carries the winter's severity")

    def test_t3b_counts_dark_nights_among_the_five_remaining(self) -> None:
        arrays = b.episode_arrays(fake_episode())
        fire = b.VARS.index("fire_lit")
        # Dark at 02:00 after evenings 21 and 22 (before the outlook) and 23, 25 (counted).
        for evening in (21, 22, 23, 25):
            arrays["v"][b.row_of(evening + 1, 120), fire] = 0
        outlooks = b.winter_outlooks(ep_from(arrays))
        self.assertEqual(len(outlooks), 1, "35 days hold one winter")
        o = outlooks[0]
        self.assertEqual((o["year"], o["severity"], o["dark"]), (0, 2, 2))
        self.assertEqual(len(o["x"]), len(b.T3B_FEATURES))
        wood = arrays["wood_stock"]
        self.assertAlmostEqual(o["x"][2], wood[b.row_of(22, 1080)] - wood[b.row_of(23, 360)])
        scores = b.t3b_scores(np.array([1.6, 2.0, 0.2]), np.array([2.0, 2.0, 1.0]))
        self.assertAlmostEqual(scores["mae"], (0.4 + 0 + 0.8) / 3)
        self.assertAlmostEqual(scores["exact_accuracy"], 2 / 3)

    def test_t4_is_normalized_per_target_and_never_mixes_units(self) -> None:
        test = {"rain": {1: {"dark_nights": 1.0, "cold_member_hours": 100.0, "food_store": -4.0, "deaths": 0.0},
                         2: {"dark_nights": 0.0, "cold_member_hours": 300.0, "food_store": 0.0, "deaths": 0.0}}}
        val = {"rain": {9: {"dark_nights": 1.0, "cold_member_hours": 200.0, "food_store": -2.0, "deaths": 0.0}}}
        scores = b.t4_scores(test, val)["rain"]
        zero = scores["zero_effect"]
        self.assertAlmostEqual(zero["cold_member_hours"]["mae"], 200.0)
        self.assertAlmostEqual(zero["cold_member_hours"]["nmae"], 1.0)
        self.assertIsNone(zero["deaths"]["nmae"], "mean |effect| 0: undefined")
        self.assertFalse(zero["deaths"]["ranked"])
        mean = scores["val_mean_effect"]
        self.assertAlmostEqual(mean["cold_member_hours"]["mae"], 100.0)
        self.assertAlmostEqual(mean["cold_member_hours"]["nmae"], 0.5)
        self.assertAlmostEqual(mean["food_store"]["nmae"], 1.0)
        self.assertNotIn("mean", scores["zero_effect"])
        dist = b.effect_distribution(test)
        self.assertEqual(dist["rain"]["changed_ranked_share"], 1.0)

    def test_t5_metrics_on_constructed_agents(self) -> None:
        n_rows, n_agents = 30, 2
        ag = {k: np.zeros((n_rows, n_agents)) for k in ("x", "y", "state_tag", "target_kind", "hunger", "thirst", "fatigue")}
        ag["alive"] = np.ones((n_rows, n_agents), dtype=bool)
        ag["x"][:, 0] = np.arange(n_rows)            # agent 0 walks one tile an hour
        ag["state_tag"][:, 1] = np.arange(n_rows) % 2  # agent 1 alternates its state
        ag["hunger"][:, 0] = np.arange(n_rows) * 2.0
        ag["alive"][25, 1] = False                    # agent 1 dead at row 25: pairs with it are dropped
        e = ep_from(b.episode_arrays(fake_episode()))
        e.agents = ag
        s = b.t5_scores([e])
        p = s["persistence"]
        # One pair set for both baselines: alive at t+1, t and t-23 (t = 23..28); agent 1 is dead at row 25, so its
        # pairs with t = 24 and t = 25 are dropped: 12 - 2 = 10 pairs.
        pairs = 2 * (n_rows - 1 - 23) - 2
        self.assertEqual(p["n"], pairs)
        self.assertAlmostEqual(p["position_chebyshev_mean"], (n_rows - 1 - 23) / pairs)
        self.assertAlmostEqual(p["state_tag_accuracy"], (n_rows - 1 - 23) / pairs)
        self.assertEqual(p["target_kind_accuracy"], 1.0)
        self.assertIsNone(p["thirst_nmae"], "constant thirst: std 0")
        d = s["day_earlier"]
        self.assertEqual(d["n"], pairs, "the same pairs as persistence")
        self.assertAlmostEqual(d["position_chebyshev_mean"], 24 * 6 / pairs, msg="agent 0 moved 24 tiles in a day")
        self.assertEqual(d["state_tag_accuracy"], 1.0, "an even lag sees the same alternation phase")
        # The same denominator: hunger nMAE scales with the error (2 per hour vs 48 per day) on the same pairs.
        self.assertAlmostEqual(d["hunger_nmae"], 24 * p["hunger_nmae"])


if __name__ == "__main__":
    unittest.main()
