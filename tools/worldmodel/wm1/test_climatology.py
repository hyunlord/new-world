"""WM-1.1 calendar-family tests on synthetic episodes (GOAL_1_1.md 6.3). numpy only.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import calcore as cc
from wm1 import climatology as cl
from wm1 import data as d
from wm1 import evaluate as ev


def synth_ep(v: np.ndarray, seed: int, split: str = "A_obs", role: str = "train", severity=None,
             actions=None) -> d.EpisodeData:
    """A synthetic episode with the v0 calendar (tick = 60 r) around a given V array [T, 8]."""
    t = len(v)
    r = np.arange(t)
    pos = cc.year_pos(r)
    hour = (r + 8) % 24
    season = pos // 168
    sev = np.zeros(t, dtype=np.int64) if severity is None else severity
    acts = np.zeros((t, len(b.KINDS))) if actions is None else actions
    bench = b.Ep(split, role, seed, "none", v=v.astype(np.float64), tick=60 * r, hour=hour, season=season,
                 severity=sev, actions=acts, wood_stock=v[:, 1].copy(), members=np.full(t, 10.0),
                 cold_hours=np.zeros(t), deaths=np.zeros(t))
    cal = d.calendar_features(hour, season, (pos % 168) // 24)
    return d.EpisodeData(split, role, seed, "none", bench, np.zeros((t, len(d.ALL_STATE))), cal, [])


def calendar_signal(t: int) -> np.ndarray:
    r = np.arange(t)
    base = np.stack([np.sin(2 * np.pi * cc.year_pos(r) / cc.YEAR + j) * (j + 1) + 10 * j for j in range(8)], axis=1)
    return base + 0.3 * np.sin(2 * np.pi * r / 24)[:, None]


def synth_split(n: int, t: int, seed: int, role: str = "train", noise: float = 0.5) -> list[d.EpisodeData]:
    rng = np.random.default_rng(seed)
    cal = calendar_signal(t)
    return [synth_ep(cal + noise * rng.standard_normal((t, 8)), 1000 * seed + i, role=role) for i in range(n)]


class CalendarRuleTests(unittest.TestCase):
    def test_wrap_rule_and_analog_shift(self) -> None:
        for r in (2881, 3552, 3553, 5760):
            w = int(cc.wrap_row(r, 2881))
            self.assertEqual(cc.year_pos(w), cc.year_pos(r))
            self.assertLess(w, 2881)
            self.assertGreaterEqual(w, 2881 - 672)
        self.assertEqual(int(cc.wrap_row(2880, 2881)), 2880)
        s, h = 2860, 168
        delta = int(cc.analog_shift(s, h, 2881))
        self.assertGreater(delta, 0)
        self.assertGreaterEqual(s - 23 - delta, 0)
        self.assertLess(s + h - delta, 2881)
        self.assertEqual(int(cc.analog_shift(100, 168, 2881)), 0)

    def test_year_position_and_tick_convention(self) -> None:
        self.assertEqual(int(cc.year_pos(0)), 8)
        self.assertEqual(cc.row_of(2, 360), 22)
        self.assertEqual(ev.row_of(2, 360), 22)
        self.assertEqual(int(cc.year_pos(22)), 30)
        self.assertEqual(cc.row_of(30, 360), 694)
        self.assertEqual(int(cc.year_pos(694)), 30)
        self.assertEqual(int(cc.display_day(22)), 2)
        self.assertEqual(int(cc.display_day(694)), 30)

    def test_alpha_recovers_an_ar1_coefficient(self) -> None:
        rng = np.random.default_rng(3)
        e, t, phi = 60, 1500, 0.8
        cal = calendar_signal(t)
        a = np.zeros((e, t, 8))
        a[:, 0] = rng.standard_normal((e, 8))
        for r in range(1, t):
            a[:, r] = phi * a[:, r - 1] + rng.standard_normal((e, 8))
        v = cal[None] + a
        m_abs, _ = cc.calendar_means(v)
        alpha = cc.fit_alpha(v, m_abs, 24)
        np.testing.assert_allclose(alpha[1], phi, atol=0.02)
        np.testing.assert_allclose(alpha[2], phi ** 2, atol=0.03)


class PredictorTests(unittest.TestCase):
    T = 800

    @classmethod
    def setUpClass(cls) -> None:
        cls.train = synth_split(12, cls.T, 1)
        cls.fit = cl.CalendarFit(cls.train)
        cls.test = synth_split(2, cls.T, 2, role="test")

    def test_members_never_read_future_rows_actions_or_severity(self) -> None:
        ep = self.test[0]
        starts = np.array([30, 200, 500])
        h = 48
        for member, pred in cl.predictors(self.fit).items():
            base = pred.rollout(ep, starts, h, ep.bench.actions).v
            for s, i in zip(starts, range(len(starts))):
                v2 = ep.bench.v.copy()
                v2[s + 1:] += 100.0 * np.random.default_rng(int(s)).standard_normal(v2[s + 1:].shape)
                other = synth_ep(v2, ep.seed, role="test")
                got = pred.rollout(other, np.array([s]), h, other.bench.actions).v
                np.testing.assert_array_equal(got[0], base[i], err_msg=f"{member} reads rows > {s}")
                # Grouped call (every start at once): the perturbed start's row still matches.
                got_all = pred.rollout(other, starts, h, other.bench.actions).v
                np.testing.assert_allclose(got_all[i], base[i], rtol=0, atol=1e-9,
                                           err_msg=f"{member} reads rows > {s} in a grouped call")
            # Actions, severity, wood_target and lifespans: the predictor reads none of them.
            acts = np.ones_like(ep.bench.actions) * 50.0
            sev = np.full(self.T, 2, dtype=np.int64)
            changed = synth_ep(ep.bench.v, ep.seed, role="test", severity=sev, actions=acts)
            changed.table[:] = 7.0   # every state column, `s.wood_target` and the tallies included
            np.testing.assert_array_equal(pred.rollout(changed, starts, h, acts).v, base, err_msg=member)

    def test_definitions(self) -> None:
        f, ep = self.fit, self.test[0]
        s, h = np.array([100]), 30
        tgt = np.arange(101, 131)
        np.testing.assert_allclose(cl.CalendarPredictor(f, "cal_abs").rollout(ep, s, h, None).v[0], f.m_abs[tgt])
        np.testing.assert_allclose(cl.CalendarPredictor(f, "cal_year").rollout(ep, s, h, None).v[0],
                                   f.m_year[cc.year_pos(tgt)])
        ap1 = cl.CalendarPredictor(f, "ap1_abs").rollout(ep, s, h, None).v[0]
        np.testing.assert_allclose(ap1, f.m_abs[tgt] + (ep.bench.v[100] - f.m_abs[100]))
        apfit = cl.CalendarPredictor(f, "apfit_year").rollout(ep, s, h, None).v[0]
        m = f.m_year[cc.year_pos(tgt)]
        np.testing.assert_allclose(apfit, m + f.alpha["year"][1:31] * (ep.bench.v[100] - f.m_year[cc.year_pos(100)]))
        # m_year is the mean over rows >= 24 of the same year position.
        q = int(cc.year_pos(300))
        rows = [r for r in range(24, self.T) if cc.year_pos(r) == q]
        np.testing.assert_allclose(f.m_year[q], f.lib[:, rows].mean(axis=(0, 1)))

    def test_analog_finds_the_copied_episode(self) -> None:
        rng = np.random.default_rng(9)
        lib = [synth_ep(calendar_signal(self.T) + 3.0 * rng.standard_normal((self.T, 8)), i) for i in range(10)]
        fit = cl.CalendarFit(lib)
        test = synth_ep(lib[7].bench.v + 0.001 * rng.standard_normal((self.T, 8)), 99, role="test")
        starts = np.array([40, 300, 700])
        near, delta = cc.analog_neighbours(test.bench.v, starts, 24, fit.lib, fit.inv_std, 5)
        self.assertTrue((near[:, 0] == 7).all())
        np.testing.assert_array_equal(delta, 0)
        an1 = cl.CalendarPredictor(fit, "an1").rollout(test, starts, 24, None).v
        np.testing.assert_allclose(an1[0], lib[7].bench.v[41:65])
        # An A_train episode is never its own analog.
        near_self, _ = cc.analog_neighbours(lib[7].bench.v, starts, 24, fit.lib, fit.inv_std, 5, exclude=7)
        self.assertFalse((near_self == 7).any())
        an5 = cl.CalendarPredictor(fit, "an5").rollout(test, starts, 24, None).v
        off = cl.CalendarPredictor(fit, "an5_off").rollout(test, starts, 24, None).v
        nn = near[:, :5]
        base = fit.lib[nn, starts[:, None]].mean(axis=1)
        np.testing.assert_allclose(off, an5 + (test.bench.v[starts] - base)[:, None, :])

    def test_analog_shift_past_the_library(self) -> None:
        fit = self.fit
        long_v = calendar_signal(self.T + 700) + 0.5 * np.random.default_rng(4).standard_normal((self.T + 700, 8))
        ep = synth_ep(long_v, 5, split="OOD_long", role="test")
        s, h = self.T + 100, 40
        near, delta = cc.analog_neighbours(ep.bench.v, np.array([s]), h, fit.lib, fit.inv_std, 5)
        self.assertEqual(int(delta[0]), 672)
        pred = cl.CalendarPredictor(fit, "an1").rollout(ep, np.array([s]), h, None).v
        np.testing.assert_allclose(pred[0], fit.lib[near[0, 0], s + 1 - 672: s + h + 1 - 672])
        # CalAbs wraps by whole years and keeps the year position.
        cal = cl.CalendarPredictor(fit, "cal_abs").rollout(ep, np.array([s]), h, None).v[0]
        rows = cc.wrap_row(np.arange(s + 1, s + h + 1), self.T)
        self.assertTrue((cc.year_pos(rows) == cc.year_pos(np.arange(s + 1, s + h + 1))).all())
        np.testing.assert_allclose(cal, fit.m_abs[rows])

    def test_selection_uses_val_scores_and_fair_subset(self) -> None:
        def res(x):
            node = {"A_val": {"mean": x, "mae": x}, "B_val": {"mean": x, "mae": x}}
            return {"T1": node, "T2": {"24": node, "168": node}, "T2_any": {"24": node, "168": node},
                    "T3": {"brier": x}, "T3b": node}
        scores = dict(zip(cl.MEMBERS, [0.5, 0.4, 0.3, 0.35, 0.2, 0.25, 0.1, 0.15, 0.12]))
        sel = cl.select({m: res(x) for m, x in scores.items()})
        for task in cl.TASKS:
            self.assertEqual(sel[task]["cal_best"], "an1")
            self.assertEqual(sel[task]["cal_best_fair"], "apfit_year")

    def test_fit_refuses_non_train_episodes(self) -> None:
        with self.assertRaises(ValueError):
            cl.CalendarFit(self.test)


if __name__ == "__main__":
    unittest.main()
