"""WM-Bench v0 metrics and baselines on synthetic arrays (section 12.5).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest

import numpy as np

import bench_v0 as b
import make_dataset as md

TPD, OFFSET = 1440, 480


def synthetic(rows: int = 24 * 40, seed: int = 0, split: str = "A_obs", role: str = "train", eid: str = "none",
              severity: int = 1) -> b.Ep:
    """A noiseless episode whose next-hour change is linear in its lags and the hour."""
    rng = np.random.default_rng(seed)
    tick = np.arange(rows) * 60
    minute = (tick + OFFSET) % TPD
    hour = minute // 60
    day = (tick + OFFSET) // TPD
    season = (day // 7) % 4
    v = np.zeros((rows, 8))
    v[0] = rng.uniform(10, 20, 8)
    for t in range(rows - 1):
        v[t + 1] = 0.9 * v[t] + 0.1 * 15 + np.where(hour[t] < 12, 1.0, -1.0)
    return b.Ep(split, role, seed, eid, v=v, tick=tick, hour=hour, season=season,
                severity=np.where(season == 3, severity, -1), actions=np.zeros((rows, 7)), wood_stock=v[:, 1],
                members=np.full(rows, 40.0), cold_hours=np.cumsum(np.zeros(rows)), deaths=np.zeros(rows))


class MetricTests(unittest.TestCase):
    def test_nmae_divides_by_the_test_std(self) -> None:
        true = np.array([[0.0, 1.0], [2.0, 1.0]])
        pred = np.array([[1.0, 1.0], [1.0, 2.0]])
        out = b.nmae(pred, true, np.array([2.0, 0.0]))
        self.assertAlmostEqual(out[0], 0.5)
        self.assertTrue(np.isnan(out[1]))

    def test_auc_is_the_rank_statistic_with_ties(self) -> None:
        self.assertEqual(b.auc(np.array([.1, .4, .35, .8]), np.array([0, 0, 1, 1])), 0.75)
        self.assertEqual(b.auc(np.array([.5, .5, .5, .5]), np.array([0, 1, 0, 1])), 0.5)
        self.assertEqual(b.auc(np.array([.9, .1]), np.array([1, 0])), 1.0)
        self.assertTrue(np.isnan(b.auc(np.array([.2, .3]), np.array([1, 1]))))
        self.assertAlmostEqual(b.brier(np.array([1.0, 0.0, .5]), np.array([1.0, 1.0, 0.0])), (0 + 1 + .25) / 3)

    def test_ridge_recovers_a_linear_map_and_lambda_shrinks(self) -> None:
        rng = np.random.default_rng(1)
        x = rng.normal(size=(500, 4))
        y = x @ np.array([[1.0], [-2.0], [0.5], [0.0]]) + 3.0
        model = b.Ridge().fit(x, y, 1e-6)
        np.testing.assert_allclose(model.predict(x), y, atol=1e-6)
        heavy = b.Ridge().fit(x, y, 1e6)
        self.assertLess(np.abs(heavy.predict(x) - 3.0).max(), 0.1)

    def test_t1_ridge_beats_persistence_on_a_learnable_series(self) -> None:
        train = [synthetic(seed=s) for s in range(3)]
        val = [synthetic(seed=10)]
        lam, model, scores = b.choose_lambda(train, val, False)
        self.assertEqual(lam, float(min(scores, key=scores.get)))
        test = [synthetic(seed=20)]
        out = b.t1_scores(test, model, False, b.split_std(test))
        self.assertLess(np.nanmean(out["ridge"]), 0.05 * np.nanmean(out["persistence"]))

    def test_rollout_feeds_predictions_back(self) -> None:
        train = [synthetic(seed=s) for s in range(3)]
        model = b.Ridge().fit(*b.stack(train, False), 1e-6)
        ep = synthetic(seed=5)
        start = b.rollout_starts(ep, 24)[0]
        self.assertEqual(start, (2 * TPD + 360 - OFFSET) // 60)
        rolled = b.rollout(ep, start, 24, model, False)
        np.testing.assert_allclose(rolled, ep.v[start + 24], rtol=1e-2)
        # The rollout never reads the true future: corrupt it and get the same answer.
        corrupted = synthetic(seed=5)
        corrupted.v[start + 1:] = 1e6
        np.testing.assert_array_equal(b.rollout(corrupted, start, 24, model, False), rolled)
        scores = b.t2_scores([ep], model, False, b.split_std([ep]), 168)
        self.assertLess(np.nanmean(scores["ridge"]), np.nanmean(scores["persistence"]))

    def test_action_window_sums_the_last_24_rows(self) -> None:
        actions = np.zeros((30, 7))
        actions[2, 4] = 13
        actions[5, 0] = 60
        window = b.actions_window(actions)
        self.assertEqual(window[2, 4], 13)
        self.assertEqual(window[25, 4], 13)
        self.assertEqual(window[26, 4], 0)
        self.assertEqual(window[5, 0], 60)
        self.assertEqual(window[1].sum(), 0)

    def test_c_cf_effects_compare_each_variant_with_its_factual(self) -> None:
        rows = 36 * 24 + 1
        factual = synthetic(rows=rows, seed=2008, split="C_cf", role="test")
        factual.v[:, 2] = 1.0
        variant = synthetic(rows=rows, seed=2008, split="C_cf", role="test", eid="rain")
        variant.v[:, 2] = 1.0
        # Fire out at 02:00 after the evenings of one-based days 24 and 25.
        for d in (23, 24):
            variant.v[((d + 1) * TPD + 120 - OFFSET) // 60, 2] = 0.0
        variant.cold_hours = factual.cold_hours + np.where(np.arange(rows) * 60 >= 24 * TPD, 84.0, 0.0)
        variant.v[:, 0] = factual.v[:, 0] - 7
        effects = b.c_cf_effects([factual, variant])
        self.assertEqual(effects["rain"][2008], {"dark_nights": 2.0, "cold_member_hours": 84.0, "food_store": -7.0, "deaths": 0.0})
        scores = b.t4_scores(effects, effects)
        self.assertEqual(scores["rain"]["val_mean_effect"]["dark_nights"]["mae"], 0.0)
        self.assertEqual(scores["rain"]["zero_effect"]["dark_nights"]["mae"], 2.0)
        self.assertEqual(scores["rain"]["zero_effect"]["deaths"]["sign_accuracy"], 1.0)
        dist = b.effect_distribution(effects)
        self.assertEqual(dist["rain"]["changed_share"], 1.0)

    def test_winter_evenings_and_the_logistic_baseline(self) -> None:
        ep = synthetic(rows=30 * 24, seed=0)
        ep.severity = np.where(ep.season == 3, 2, -1)
        evenings = b.winter_evenings(ep, (2, 3, 4), 13)
        self.assertEqual(len(evenings), 7)
        self.assertEqual([e["night"] for e in evenings], list(range(1, 8)))
        self.assertTrue(all(e["tonight"] == 52 for e in evenings))
        rng = np.random.default_rng(3)
        x = rng.normal(size=(400, 2))
        y = (x[:, 0] + 0.2 * rng.normal(size=400) > 0).astype(float)
        model = b.Logistic().fit(x, y)
        self.assertGreater(b.auc(model.predict(x), y), 0.95)


class ScheduleTests(unittest.TestCase):
    def test_b_act_schedules_are_seeded_sorted_and_on_the_grids(self) -> None:
        a, again = md.b_act_schedule(1000), md.b_act_schedule(1000)
        self.assertEqual(a, again)
        self.assertNotEqual(a, md.b_act_schedule(1001))
        self.assertEqual(len(a), 8)
        self.assertEqual([r["tick"] for r in a], sorted(r["tick"] for r in a))
        for row in a:
            self.assertTrue(md.B_FIRST <= row["tick"] <= md.B_LAST and (row["tick"] - md.B_FIRST) % 60 == 0)
            self.assertIn(row["amount"], md.B_GRIDS[row["kind"]])
        self.assertEqual((md.B_FIRST, md.B_LAST), (960, 169380))

    def test_ticks_of_the_named_moments(self) -> None:
        self.assertEqual(md.Z6_TICK, 31920)
        self.assertEqual(md.C_VARIANTS["gift_wood"]["tick"], 29400)
        self.assertEqual(md.C_VARIANTS["rain"]["tick"], 33720)
        self.assertEqual(md.C_VARIANTS["cold_snap"]["tick"], 35160)
        self.assertEqual(md.tick_of(29, 360), 40200)

    def test_splits_do_not_share_seeds_and_agent_runs_cover_their_seeds(self) -> None:
        seen = {}
        for split, spec in md.SPLITS.items():
            for role, seeds in spec["roles"].items():
                for seed in seeds:
                    self.assertNotIn(seed, seen, f"{seed} in {seen.get(seed)} and {split}/{role}")
                    seen[seed] = (split, role)
            covered = [s for r, _ in md.runs(split, spec) for s in r]
            self.assertEqual(covered, list(spec["seeds"]))
            self.assertEqual(sorted(s for r, flag in md.runs(split, spec) if flag for s in r), sorted(spec["agents"]))


if __name__ == "__main__":
    unittest.main()
