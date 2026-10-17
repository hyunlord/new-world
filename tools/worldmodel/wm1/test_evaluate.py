"""WM-1 harness tests on synthetic episodes (GOAL.md 6.2, 7): baselines reproduce bench_v0, rollouts never read
the future, T3b and T4 bookkeeping.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from test_bench_v0 import synthetic
from wm1 import data as d
from wm1 import evaluate as ev


def wrap(ep: b.Ep, interventions=()) -> d.EpisodeData:
    rows = len(ep.tick)
    return d.EpisodeData(ep.split, ep.role, ep.seed, ep.eid, ep, np.zeros((rows, len(d.ALL_STATE))),
                         np.zeros((rows, len(d.CALENDAR))), list(interventions))


class Oracle(ev.Predictor):
    """Returns the rows of a reference episode (the truth of a branch) — a perfect predictor."""
    name = "oracle"
    provides_t4 = True

    def __init__(self, by_actions):
        self.by_actions = by_actions

    def rollout(self, ep, starts, horizon, actions):
        ref = self.by_actions(actions)
        v = np.stack([ref.v[s + 1: s + 1 + horizon] for s in starts])
        ch = np.stack([ref.cold_hours[s + 1: s + 1 + horizon] for s in starts])
        de = np.stack([ref.deaths[s + 1: s + 1 + horizon] for s in starts])
        return ev.Rollout(v, None, ch, de)


class BaselineFidelityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.train = [synthetic(seed=s) for s in range(3)]
        self.model = b.Ridge().fit(*b.stack(self.train, False), 1.0)
        self.test = [synthetic(seed=7, role="test"), synthetic(seed=8, role="test")]
        self.std = b.split_std(self.test)

    def test_t1_matches_bench(self) -> None:
        ref = b.t1_scores(self.test, self.model, False, self.std)
        eps = [wrap(e) for e in self.test]
        for pred, key in ((ev.Persistence(), "persistence"), (ev.DailyNaive(), "daily_naive"),
                          (ev.RidgePredictor(self.model, False, "ridge"), "ridge")):
            np.testing.assert_allclose(ev.t1(pred, eps, self.std)["per_var"], ref[key], atol=1e-9)

    def test_t2_matches_bench(self) -> None:
        eps = [wrap(e) for e in self.test]
        for h in (24, 168):
            ref = b.t2_scores(self.test, self.model, False, self.std, h)
            for pred, key in ((ev.Persistence(), "persistence"), (ev.DailyNaive(), "daily_naive"),
                              (ev.RidgePredictor(self.model, False, "ridge"), "ridge")):
                np.testing.assert_allclose(ev.t2(pred, eps, self.std, h)["per_var"], ref[key], atol=1e-9)

    def test_step_rollout_equals_vectorized_daily_naive(self) -> None:
        ep = wrap(self.test[0])
        starts = ev.bench_starts(ep, 30)
        slow = ev.StepPredictor.rollout(ev.DailyNaive(), ep, starts, 30, ep.bench.actions).v
        np.testing.assert_array_equal(slow, ev.DailyNaive().rollout(ep, starts, 30, ep.bench.actions).v)

    def test_rollouts_never_read_the_future(self) -> None:
        ep = wrap(self.test[0])
        start = ev.bench_starts(ep, 24)[1]
        for pred in (ev.Persistence(), ev.DailyNaive(), ev.RidgePredictor(self.model, False, "ridge")):
            clean = pred.rollout(ep, np.array([start]), 24, ep.bench.actions).v
            bad = wrap(synthetic(seed=7, role="test"))
            bad.bench.v[start + 1:] = 1e6
            np.testing.assert_array_equal(pred.rollout(bad, np.array([start]), 24, bad.bench.actions).v, clean)

    def test_severity_modes_of_the_ridge(self) -> None:
        """hold = the bench's rollout; none = fit and rolled out without severity; true = each row's own severity."""
        eps = [wrap(e) for e in self.test]
        for e in eps:
            e.bench.severity = np.where(e.bench.season == 3, 2, -1)
        ep = eps[0]
        sev_model = b.Ridge().fit(*b.stack([e.bench for e in eps], False), 1.0)
        starts = ev.bench_starts(ep, 168)
        held = ev.RidgePredictor(sev_model, False, "ridge").rollout(ep, starts, 168, ep.bench.actions).v
        ref = np.stack([b.rollout(ep.bench, s, 168, sev_model, False) for s in starts])
        np.testing.assert_allclose(held[:, -1], ref, atol=1e-9)
        true = ev.RidgePredictor(sev_model, False, "ridge_true", "true").rollout(ep, starts, 168, ep.bench.actions).v
        self.assertGreater(np.abs(true - held).max(), 0.0)   # a 168 h rollout crosses a season boundary
        # With one severity on every row, "true" and "hold" must agree; with the severity of each row it equals a
        # rollout whose held value is replaced row by row.
        const = wrap(synthetic(seed=7, role="test"))
        const.bench.severity = np.ones_like(const.bench.severity)
        np.testing.assert_allclose(ev.RidgePredictor(sev_model, False, "t", "true").rollout(const, starts, 168, const.bench.actions).v,
                                   ev.RidgePredictor(sev_model, False, "h").rollout(const, starts, 168, const.bench.actions).v)
        s0 = starts[:1]
        buf = list(ep.bench.v[s0[0] - 23: s0[0] + 1])
        for k in range(24):
            t_ = s0[0] + k
            v_ = np.array(buf)
            x_ = b.features(np.vstack([np.zeros((s0[0] - 23, 8)), v_]), ep.bench.hour, ep.bench.season, ep.bench.severity, None, np.array([t_]))
            buf.append(buf[-1] + sev_model.predict(x_)[0])
        np.testing.assert_allclose(true[0, :24], np.array(buf[24:]), atol=1e-9)
        nosev, lam, _ = ev.fit_ridge_without_severity(eps, eps, False)
        self.assertEqual(nosev.model.w.shape[0], sev_model.w.shape[0] - 4)
        x = b.features(ep.bench.v, ep.bench.hour, ep.bench.season, ep.bench.severity, None, np.arange(30, 40))
        np.testing.assert_array_equal(ev.drop_severity(x), np.delete(x, range(77, 81), axis=1))
        other = wrap(synthetic(seed=7, role="test"))
        other.bench.severity = np.zeros_like(other.bench.severity)
        a = nosev.rollout(other, starts, 48, other.bench.actions).v
        c = nosev.rollout(eps[0], starts, 48, eps[0].bench.actions).v
        np.testing.assert_allclose(a, c)   # no severity input: changing the severity column changes nothing

    def test_any_hour_starts_cover_every_hour_of_the_start_days(self) -> None:
        ep = wrap(self.test[0])
        s = ev.any_hour_starts(ep, 24)
        self.assertEqual(s[0], (2 * b.TPD - b.OFFSET) // 60)
        self.assertEqual(len(s) % 24, 0)
        self.assertTrue(set(ev.bench_starts(ep, 24)) <= set(s))
        self.assertTrue(((ep.bench.hour[s] == np.tile(np.arange(24), len(s) // 24))).all())
        self.assertTrue((s + 24 < ep.rows).all())


class T3bTests(unittest.TestCase):
    def test_cases_count_dark_nights_from_the_third_winter_day(self) -> None:
        ep = synthetic(rows=60 * 24, seed=0)
        ep.v[:, b.VARS.index("fire_lit")] = 1.0
        # Dark at 02:00 after the evenings of days 24 and 27 (one-based).
        for day in (24, 27):
            ep.v[ev.row_of(day + 1, 120), b.VARS.index("fire_lit")] = 0.0
        cases = ev.t3b_cases(wrap(ep))
        self.assertEqual([c["day"] for c in cases], [24, 52])
        self.assertEqual(cases[0]["dark"], 2.0)
        self.assertEqual(cases[0]["r06"], ev.row_of(24, 360))
        self.assertEqual(cases[0]["nights"], [20, 44, 68, 92, 116])
        self.assertEqual(ep.season[cases[0]["r06"]], 3)
        self.assertEqual(len(cases[0]["x"]), 7)

    def test_model_count_uses_the_fire_at_the_five_02h_rows(self) -> None:
        ep = synthetic(rows=40 * 24, seed=0)
        ep.v[:, b.VARS.index("fire_lit")] = 1.0
        ep.v[ev.row_of(26, 120), b.VARS.index("fire_lit")] = 0.0
        e = wrap(ep)
        oracle = Oracle(lambda a: ep)
        self.assertEqual(ev.t3b(oracle, [e])["mae"], 0.0)
        self.assertEqual(ev.t3b(ev.Persistence(), [e])["mae"], 1.0)


class T4Tests(unittest.TestCase):
    def test_oracle_branches_reproduce_the_true_effects(self) -> None:
        rows = 36 * 24 + 1
        factual = synthetic(rows=rows, seed=2008, split="C_cf", role="test")
        factual.v[:, 2] = 1.0
        tick = 33720
        variant = synthetic(rows=rows, seed=2008, split="C_cf", role="test", eid="rain")
        variant.v[:, 2] = 1.0
        for day in (23, 24):
            variant.v[((day + 1) * b.TPD + 120 - b.OFFSET) // 60, 2] = 0.0
        variant.cold_hours = factual.cold_hours + np.where(np.arange(rows) * 60 > tick, 84.0, 0.0)
        variant.v[tick // 60 + 1:, 0] -= 7
        rec = {"tick": tick, "kind": "rain", "settlement": 1, "amount": 13, "applied": 13}
        eps = [wrap(factual), wrap(variant, [rec])]
        oracle = Oracle(lambda a: variant if a.sum() > 0 else factual)
        out = ev.t4(oracle, eps)
        self.assertEqual(out["effects"]["rain"]["2008"], {"dark_nights": 2.0, "cold_member_hours": 84.0,
                                                        "food_store": -7.0, "deaths": 0.0})
        sc = out["scores"]["rain"]
        self.assertEqual(sc["dark_nights"]["mae"], 0.0)
        self.assertEqual(sc["dark_nights"]["nmae"], 0.0)
        self.assertIsNone(sc["deaths"]["nmae"])
        # Persistence-like zero effect: nMAE = 1 on every target with a non-zero mean.
        zero = ev.t4_baseline_effects(b.c_cf_effects([factual, variant]), {})["zero_effect"]
        z = ev.t4_score(zero, b.c_cf_effects([factual, variant]))["rain"]
        self.assertEqual(z["cold_member_hours"]["nmae"], 1.0)

    def test_a_prefix_mismatch_is_refused(self) -> None:
        rows = 36 * 24 + 1
        factual = synthetic(rows=rows, seed=2008, split="C_cf", role="test")
        variant = synthetic(rows=rows, seed=2008, split="C_cf", role="test", eid="rain")
        variant.v[5, 0] += 1
        rec = {"tick": 33720, "kind": "rain", "settlement": 1, "amount": 13, "applied": 13}
        with self.assertRaises(ValueError):
            ev.t4_predicted_effects(Oracle(lambda a: factual), [wrap(factual), wrap(variant, [rec])])


if __name__ == "__main__":
    unittest.main()
