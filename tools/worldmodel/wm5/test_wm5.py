"""WM-5 unit tests (docs/worldmodel/wm5/GOAL.md 7.5) on tiny synthetic inputs (no recorder, no data; torch tests skip
without torch).

    python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm5  # noqa: F401
from wm4 import policy as pol4
from wm5 import policy2 as p2

T = 2881
Q = {"q_f1": 8.079227234753551, "q_f2": 12.1, "q_w2": 3.111944444444449}


def series(f=10.0, w=2.0, alive=True) -> dict:
    return {"f": np.full(T, float(f)), "w": np.full(T, float(w)), "decision": np.full(T, bool(alive))}


class SyntheticWorld:
    """A world whose f after each cast row moves by the cast's effect (gift +6, spoil −6, rot: w −2), rows <= the cast
    row unchanged: the recorder's prefix property, so the rounds can be simulated and compared with the replay."""

    def __init__(self, seed: int, f0: float, w0: float):
        self.seed = seed
        rng = np.random.default_rng(seed)
        self.f = f0 + np.cumsum(rng.normal(0, 0.05, T))
        self.w = w0 + np.cumsum(rng.normal(0, 0.02, T))

    def episode(self, casts: list[dict]) -> dict:
        f, w = self.f.copy(), self.w.copy()
        for c in casts:
            r = int(c["row"])
            if c["arm"] == "gf240":
                f[r + 1:] += 6.0
            elif c["arm"] == "sp240":
                f[r + 1:] -= 6.0
            else:
                w[r + 1:] -= 2.0
        return {"f": f, "w": w, "decision": np.ones(T, bool)}


def rounds(world: SyntheticWorld) -> tuple[list[dict], list[list[dict]]]:
    """GOAL.md 6.2.3: each round reads the episode recorded with every cast so far."""
    casts, rows = [], []
    while True:
        res = p2.step(world.seed, world.episode(casts), Q, casts)
        rows.append(res["rows"])
        if res["cast"] is None:
            return casts, rows
        casts.append(res["cast"])


class TestPolicy2(unittest.TestCase):
    def test_probabilities_are_wm4_doubled(self):
        self.assertEqual(p2.P_GIFT, (0.06, 0.016, 0.004))
        self.assertEqual(p2.P_SPOIL, (0.004, 0.016, 0.06))
        self.assertAlmostEqual(p2.P_ROT_HIGH, 0.024)
        self.assertAlmostEqual(p2.P_ROT_LOW, 0.008)
        self.assertEqual(p2.probs(5.0, 4.0, Q), (0.06, 0.004, 0.024))
        self.assertEqual(p2.probs(10.0, 1.0, Q), (0.016, 0.016, 0.008))
        self.assertEqual(p2.probs(20.0, 1.0, Q), (0.004, 0.06, 0.008))
        self.assertEqual(p2.probs(20.0, 1.0, Q, decision=False), (0.0, 0.0, 0.0))
        # confounding ratios equal WM-4's
        for t in range(3):
            self.assertAlmostEqual(p2.P_GIFT[t] / p2.P_SPOIL[t], pol4.P_GIFT[t] / pol4.P_SPOIL[t])

    def test_frozen_terciles_are_wm4s(self):
        q = p2.frozen_terciles()
        for k in ("q_f1", "q_f2", "q_w2"):
            self.assertEqual(q[k], Q[k])

    def test_stream_distinct_from_wm4(self):
        self.assertNotEqual(p2.draw(47000, 8), pol4.draw(47000, 8))
        self.assertEqual(p2.draw(47000, 8), float(np.random.default_rng([47000, 8, 2]).random()))

    def test_refractory_and_cap(self):
        s = series(f=5.0, w=4.0)
        casts = []
        for day in (10, 18, 26, 34, 42):
            casts.append(p2.cast_record("gf240", day))
        rec = p2.row_record(1, 30, s, Q, casts[:3])          # last cast day 26 -> refractory through day 33
        self.assertEqual(rec["status"], "refractory")
        self.assertEqual((rec["p_g"], rec["p_s"], rec["p_r"]), (0.0, 0.0, 0.0))
        self.assertEqual(p2.row_record(1, 33, s, Q, casts[:3])["status"], "refractory")
        self.assertEqual(p2.row_record(1, 34, s, Q, casts[:3])["status"], "decision")
        self.assertEqual(p2.row_record(1, 50, s, Q, casts)["status"], "cap")
        self.assertEqual(p2.row_record(1, 9, series(alive=False), Q, [])["status"], "no_decision")

    def test_no_second_cast_within_seven_days_and_at_most_five(self):
        n_casts = []
        for seed in range(300):
            rep = p2.replay(seed, series(f=5.0, w=4.0), Q)       # the highest hazard: 0.088 per decision row
            days = [c["day"] for c in rep["casts"]]
            self.assertLessEqual(len(days), p2.MAX_CASTS)
            self.assertTrue(all(b - a >= 8 for a, b in zip(days, days[1:])), days)
            self.assertEqual([r["day"] for r in rep["rows"]], list(p2.DAYS))
            n_casts.append(len(days))
        self.assertGreater(max(n_casts), 1)

    def test_determinism(self):
        a = p2.replay(47123, series(f=9.0, w=4.0), Q)
        b = p2.replay(47123, series(f=9.0, w=4.0), Q)
        self.assertEqual(a, b)

    def test_overlap_every_arm_positive_on_decision_rows(self):
        for f in (1.0, 10.0, 30.0):
            for w in (0.0, 10.0):
                self.assertTrue(all(p > 0 for p in p2.probs(f, w, Q)))

    def test_expected_casts_per_world(self):
        # mid tercile, low wood: hazard 0.04 per decision row
        n = [len(p2.replay(s, series(f=10.0, w=1.0), Q)["casts"]) for s in range(400)]
        self.assertTrue(1.0 < np.mean(n) < 3.0, np.mean(n))

    def test_rounds_equal_replay_on_final_episode(self):
        """The casts of the rounds (each reading the episode with the casts so far) equal the replay on the final
        episode, row probabilities included: the propensity record can be rebuilt from the observed episode."""
        for seed in range(60):
            world = SyntheticWorld(seed, f0=[6.0, 10.0, 14.0][seed % 3], w0=[2.0, 4.0][seed % 2])
            casts, rnd_rows = rounds(world)
            rep = p2.replay(seed, world.episode(casts), Q)
            self.assertEqual([(c["arm"], c["row"]) for c in rep["casts"]], [(c["arm"], c["row"]) for c in casts])
            flat = [r for rows in rnd_rows for r in rows]
            self.assertEqual(len(flat), len(rep["rows"]))
            for a, b in zip(flat, rep["rows"]):
                self.assertEqual((a["row"], a["status"], a["p_g"], a["p_s"], a["p_r"], a["cast"]),
                                 (b["row"], b["status"], b["p_g"], b["p_s"], b["p_r"], b["cast"]))

    def test_round_schedules_keep_every_earlier_cast(self):
        casts = [p2.cast_record("sp240", 12), p2.cast_record("gf240", 25)]
        rows = p2.schedule_rows(casts)
        self.assertEqual([r["tick"] for r in rows], [pol4.tick_of_day(12), pol4.tick_of_day(25)])
        self.assertEqual([r["kind"] for r in rows], ["spoil_food", "gift_food"])
        self.assertTrue(all(r["tick"] % 60 == 0 and r["settlement"] == "home" for r in rows))
        self.assertEqual([r["amount"] for r in rows], [240, 240])

    def test_ref_rows_and_family_schedules(self):
        never = p2.ref_rows([])
        self.assertEqual(never["c1"], pol4.row_of_day(30))
        self.assertEqual(never["c2"], pol4.row_of_day(44))
        one = [p2.cast_record("rw120", 15)]
        r1 = p2.ref_rows(one)
        self.assertEqual((r1["c1"], r1["c2"]), (pol4.row_of_day(15), pol4.row_of_day(29)))
        two = one + [p2.cast_record("gf240", 40)]
        r2 = p2.ref_rows(two)
        self.assertEqual(r2["c2"], pol4.row_of_day(40))
        fam = p2.family_schedules(two + [p2.cast_record("sp240", 50)], r2["c2"])
        self.assertEqual(fam["pre"], p2.schedule_rows(one))            # casts before c2 only; nothing after c2
        for arm in p2.ARMS:
            self.assertEqual(fam[arm][:-1], fam["pre"])
            self.assertEqual(fam[arm][-1]["tick"], r2["c2"] * 60)
            self.assertEqual(fam[arm][-1]["kind"], p2.ARM_SPEC[arm][0])
        self.assertEqual(p2.family_schedules(two, r2["c1"])["pre"], [])


class TestCastsCheck(unittest.TestCase):
    def test_check_casts(self):
        from wm5 import obs2
        casts = [p2.cast_record("sp240", 12), p2.cast_record("gf240", 25)]
        recs = [{"tick": r["tick"], "kind": r["kind"], "amount": r["amount"], "applied": 1} for r in p2.schedule_rows(casts)]
        self.assertTrue(obs2.check_casts(recs, casts))
        self.assertFalse(obs2.check_casts(recs[:1], casts))
        bad = [dict(recs[0]), dict(recs[1], kind="spoil_food")]
        self.assertFalse(obs2.check_casts(bad, casts))


# ------------------------------------------------------------------ CRN / CT as the authors train them (7.4) --------
from wm1 import m1_gru as _g  # noqa: E402
HAVE_TORCH = _g.torch is not None


def tiny_data(n: int = 6, tokens: int = 40, seed: int = 0):
    """A stand-in for `wm4.causal.Data`: n episodes of tokens × 24 + 1 hourly rows, 37 positive state features (the last
    five tallies), the calendar, and a gift on some episodes."""
    from wm4 import causal as c4
    from wm4 import tokens as tk
    rng = np.random.default_rng(seed)
    T = 24 * tokens + 1
    names = tuple(f"x{i}" for i in range(32)) + tk.DEATHS
    tally = np.array([nm.startswith("d.") for nm in names])
    raws = []
    for i in range(n):
        st = np.abs(rng.normal(5, 1, (T, 37))).cumsum(0) / 50
        st[:, tally] = rng.poisson(0.05, (T, int(tally.sum())))
        act = np.zeros((T, 7))
        if i % 2:
            act[24 * 10 + 22, 0] = 240.0
        raws.append(tk.Raw(1000 + i, f"e{i}", st, rng.normal(0, 1, (T, 30)), act, tally))

    class D:
        pass
    dd = D()
    dd.raws, dd.names, dd.seeds = raws, names, [r.seed for r in raws]
    dd.seqs = lambda phases, norm: [tk.tokenize(r, p, norm) for r, p in zip(dd.raws, phases)]
    return dd


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class TestCausal5(unittest.TestCase):
    def setUp(self):
        from wm4 import causal as c4
        from wm4 import tokens as tk
        from wm5 import causal5 as c5
        self.c4, self.tk, self.c5 = c4, tk, c5
        _g.torch.manual_seed(0)
        self.tr, self.va = tiny_data(6, seed=0), tiny_data(3, seed=1)
        self.norm = tk.Norm.fit(self.tr.raws)
        self.val = c5.Val(c4.val_seqs(self.va, [tk.PHASE_06], self.norm), 3, [tk.PHASE_06])

    def test_alpha_schedule_reaches_max(self):
        c5 = self.c5
        for planned in (12, 11850):
            self.assertGreaterEqual(c5.alpha_at(planned - 1, planned, 0.01), 0.99 * 0.01)
        self.assertEqual(c5.alpha_at(0, 100, 1.0), 0.0)
        self.assertLess(c5.alpha_at(10, 100, 1.0), 0.5)

    def test_eval_points_at_most_fifteen(self):
        for planned in (12, 11850, 53800):
            pts = self.c5.eval_points(planned)
            self.assertLessEqual(len(pts) + 1, 15)
            self.assertTrue(all(0 < p < planned for p in pts))

    def test_plan_reduced_only_when_cost_exceeds_budget(self):
        c5 = self.c5
        cost = {"stages": {"ct": {"s_per_step": 0.83, "eval_seconds": 12.0}}}
        self.assertEqual(c5.scaled_plan(cost, "ct", 14400)["planned"]["ct"], 11850)
        cost["stages"]["ct"]["s_per_step"] = 2.0
        p = c5.scaled_plan(cost, "ct", 14400)
        self.assertTrue(p["reduced"])
        self.assertEqual(p["planned"]["ct"], int((14400 - 15 * 12.0) // 2.0))

    def _crn(self):
        c5 = self.c5
        hp = dict(self.c4.DEFAULTS["crn"]) | c5.CONFIG["crn"] | {"enc_batch": 2, "dec_batch": 16, "enc_hidden": 16}
        logs = []
        res = c5.train_crn(hp, self.tr, self.val, "fixed_06", self.norm, 0, "cpu", {"crn_enc": 12, "crn_dec": 9},
                           1e6, logs.append, __import__("time").time())
        return res, logs

    def test_crn_planned_run_reaches_alpha_max_and_keeps_last_and_best(self):
        (model, last, best, info), logs = self._crn()
        for st in ("encoder", "decoder"):
            self.assertEqual(info[st]["steps"], info[st]["planned"])
            self.assertEqual(info[st]["stop"], "planned")
            self.assertGreaterEqual(info[st]["alpha_last"], 0.99 * info[st]["alpha_max"])
            self.assertLessEqual(info[st]["evals"], 15)
        dec = [r for r in logs if r["stage"] == "decoder"]
        self.assertEqual(dec[-1]["steps"], 9)
        self.assertEqual(info["decoder"]["best_val_roll"], min(r["val_roll"] for r in dec))
        self.assertEqual(info["decoder"]["val_roll_last"], dec[-1]["val_roll"])
        for k in last["enc"]:                     # both checkpoints share the last encoder
            self.assertTrue(_g.torch.equal(last["enc"][k], best["enc"][k]))
        same = all(_g.torch.equal(last["dec"][k], best["dec"][k]) for k in last["dec"])
        self.assertEqual(same, info["decoder"]["best_step"] == 9)

    def test_ct_planned_run_reaches_alpha_max_and_keeps_last_and_best(self):
        c5 = self.c5
        hp = dict(self.c4.DEFAULTS["ct"]) | c5.CONFIG["ct"] | {"batch": 2, "d_model": 16, "heads": 2, "layers": 1,
                                                                "br": 8, "fc": 8}
        logs = []
        model, last, best, info = c5.train_ct(hp, self.tr, self.val, "fixed_06", self.norm, 0, "cpu", {"ct": 10}, 1e6,
                                              logs.append, __import__("time").time())
        st = info["ct"]
        self.assertEqual((st["steps"], st["stop"]), (10, "planned"))
        self.assertGreaterEqual(st["alpha_last"], 0.99 * st["alpha_max"])
        self.assertEqual(st["best_val_roll"], min(r["val_roll"] for r in logs))
        same = all(_g.torch.equal(last[k], best[k]) for k in last)
        self.assertEqual(same, st["best_step"] == 10)
        # the primary checkpoint is the EMA state of the last evaluation and the model carries it
        for k, v in model["net"].state_dict().items():
            self.assertTrue(_g.torch.equal(v, last[k]))

    def test_time_cap_is_reported(self):
        c5 = self.c5
        hp = dict(self.c4.DEFAULTS["ct"]) | c5.CONFIG["ct"] | {"batch": 2, "d_model": 16, "heads": 2, "layers": 1}
        _, _, _, info = c5.train_ct(hp, self.tr, self.val, "fixed_06", self.norm, 0, "cpu", {"ct": 10 ** 6}, 0.0,
                                    lambda r: None, __import__("time").time())
        self.assertEqual(info["ct"]["stop"], "time")
        self.assertLess(info["ct"]["steps"], 10 ** 6)


class TestRounding(unittest.TestCase):
    def test_readout_rounds_food_alive_deaths_not_settlements(self):
        from wm4 import tokens as tk
        from wm5 import causal5 as c5
        names = ("s.food_stock", "g.alive", "g.settlements_alive") + tuple(f"x{i}" for i in range(29)) + tk.DEATHS
        H = 30
        ar, fa = np.zeros((H, 37)), np.zeros((H, 37))
        ar[:, 0], fa[:, 0] = 100.4, 99.6               # food: rounded 100 − 100 = 0; unrounded 0.8
        ar[:, 1], fa[:, 1] = 10.3, 10.6                # alive: rounded 10 − 11 = −1; unrounded −0.3
        ar[:, 2], fa[:, 2] = 1.4, 1.2                  # settlements: never rounded, 0.2
        ar[:, 32] = 0.02                               # predicted daily starvation deaths
        truth = {o: {"7d": 0.0, "30d": 0.0} for o in ("food_stock", "alive", "deaths", "settlements_alive")}
        r = c5.readout(ar, fa, 3.2, names, truth, True)
        u = c5.readout(ar, fa, 3.2, names, truth, False)
        self.assertEqual(r["food_stock"]["7d"], 0.0)
        self.assertAlmostEqual(u["food_stock"]["7d"], 0.8)
        self.assertEqual(r["alive"]["30d"], -1.0)
        self.assertAlmostEqual(u["alive"]["30d"], -0.3)
        self.assertAlmostEqual(r["settlements_alive"]["7d"], 0.2)
        self.assertAlmostEqual(u["deaths"]["7d"], 7 * 0.02)
        self.assertEqual(r["deaths"]["7d"], 0.0)          # round(3.2 + 0.14) − round(3.2) = 0
        self.assertEqual(r["deaths"]["30d"], 1.0)         # round(3.2 + 0.6) − round(3.2) = 4 − 3


# ------------------------------------------------------------------ floors, baselines, regressions (7.1–7.3) --------
def synth_confounded(n: int, seed: int = 0, h: int = 4, tau: float = 1.0, delta: float = 4.0, gamma: float = 6.0):
    """A multi-cast confounded policy with a known regime effect, for GOAL.md 7.2's estimators. One risk-set row c0 per
    world (rows c0 … c0 + h − 1 are decision rows, the last row is c0 + h). At c0 a prognostic state z (+γ in the
    outcome) raises the gift probability (0.4 vs 0.05): `naive` is confounded. A gift adds τ and sends half the worlds
    into a regime with +δ and a later cast hazard of 0.5 per row (the other half: none), so the treated who survive
    (c0, c0 + h) without another cast are mostly the low regime: only the post-cast survival factor restores them.
    True effects: gift τ + δ / 2, spoil −1. Every third world has an earlier cast in its history (allowed)."""
    from wm5 import effects5 as e5
    rng = np.random.default_rng(seed)
    c0 = 5
    rows, last = tuple(range(c0, c0 + h)), c0 + h
    ws = []
    for i in range(n):
        z = rng.random() < 0.5
        probs = {c0: (0.4 if z else 0.05, 0.05, 0.05)}
        casts = {2: "rw120"} if i % 3 == 0 else {}
        u, p = rng.random(), probs[c0]
        a = "gf240" if u < p[0] else "sp240" if u < p[0] + p[1] else "rw120" if u < sum(p) else None
        high = False
        if a:
            casts[c0] = a
        if a == "gf240":
            high = rng.random() < 0.5
        y_end = gamma * z + rng.normal(0, 1) + (tau + delta * high if a == "gf240" else 0.0) + (-1.0 if a == "sp240" else 0.0)
        for j in rows[1:]:
            probs[j] = (0.25, 0.25, 0.0) if high else (0.0, 0.0, 0.0) if a == "gf240" else (0.02, 0.02, 0.02)
            q, u = probs[j], rng.random()
            b = "gf240" if u < q[0] else "sp240" if u < q[0] + q[1] else "rw120" if u < sum(q) else None
            if b:
                casts[j] = b
                y_end += 50.0                              # a later cast: such units must leave both sets
        y, zero = np.zeros(last + 1), np.zeros(last + 1)
        y[last] = y_end
        ws.append(e5.World5(i, {"food_stock": y, "alive": zero, "deaths": zero, "settlements_alive": zero},
                            np.zeros((last + 1, 8)), casts, probs))
    return ws, rows, last


class TestBaselines5(unittest.TestCase):
    def estimates(self, survival=True):
        from wm5 import effects5 as e5
        ws, rows, last = synth_confounded(20000, 0)
        t, c = e5.stack(ws, 4, decision_rows=rows, last_row=last, survival=survival)
        allc = [u for us in c.values() for u in us]
        est = lambda a, wt: e5.wmean(t[a], "food_stock", wt) - e5.wmean(allc, "food_stock", wt)
        return est, t, allc

    def test_ipw_unbiased_naive_biased(self):
        est, t, allc = self.estimates()
        self.assertLess(abs(est("gf240", True) - 3.0), 0.4)
        self.assertLess(abs(est("sp240", True) + 1.0), 0.4)
        self.assertGreater(abs(est("gf240", False) - 3.0), 1.0)
        self.assertTrue(all(u.dy["food_stock"] < 40 for u in t["gf240"] + allc))   # later casts excluded

    def test_dropping_the_survival_factor_biases_ipw(self):
        est, _, _ = self.estimates(survival=False)
        self.assertGreater(abs(est("gf240", True) - 3.0), 1.0)

    def test_weights_by_hand(self):
        from wm5 import effects5 as e5
        T = 20
        z = np.zeros(T + 1)
        w = e5.World5(1, {"food_stock": np.arange(T + 1.0), "alive": z, "deaths": z, "settlements_alive": z},
                      np.zeros((T + 1, 8)), {4: "gf240", 12: "sp240"},
                      {2: (0.1, 0.1, 0.1), 4: (0.2, 0.05, 0.05), 6: (0.0, 0.0, 0.0), 8: (0.1, 0.1, 0.0),
                       10: (0.05, 0.05, 0.0), 12: (0.1, 0.2, 0.1)})
        t, c = e5.stack([w], 6, decision_rows=(2, 4, 6, 8, 10, 12), last_row=T)
        # treated at 4 (h = 6): rows 6, 8 in (4, 10): p_tot 0 and 0.2 -> 1 / (0.2 * 1.0 * 0.8)
        self.assertEqual([u.row for u in t["gf240"]], [4])
        self.assertAlmostEqual(t["gf240"][0].weight, 1 / (0.2 * 0.8))
        self.assertEqual(t["gf240"][0].dy["food_stock"], 6.0)
        # controls: row 2 has the cast at 4 in [2, 8): out; row 6 is refractory (p 0): not at risk; row 8: rows 8, 10
        # in [8, 14) have no cast but 12 does -> out; row 10: the cast at 12 -> out
        self.assertEqual(sorted(c), [])
        t, c = e5.stack([w], 2, decision_rows=(2, 4, 6, 8, 10, 12), last_row=T)
        self.assertEqual(sorted(c), [2, 8, 10])
        self.assertAlmostEqual(c[8][0].weight, 1 / 0.8)
        # the treated at 12 has an earlier cast (4) in its history: allowed
        self.assertEqual([u.row for u in t["sp240"]], [12])
        self.assertAlmostEqual(t["sp240"][0].weight, 1 / 0.2)


def fake_case(seed, arm, rep, truth7, truth30=0.0):
    from wm2 import effects as ef
    tr = {o: {"7d": truth7, "30d": truth30} for o in ("food_stock", "alive", "deaths", "settlements_alive")}
    return ef.Case(seed, arm, rep, "test", 600, 10, tr, 0.0, (False, False), None)


class TestFloor(unittest.TestCase):
    def test_other_replicate_truth(self):
        from wm5 import effects5 as e5
        cases = [fake_case(1, "gf240E", e5.rep_code("c1", 0), 5.0), fake_case(1, "gf240E", e5.rep_code("c1", 1), 7.0),
                 fake_case(1, "gf240E", e5.rep_code("c2", 0), -2.0), fake_case(1, "gf240E", e5.rep_code("c2", 1), 1.0),
                 fake_case(2, "sp240L", 0, 3.0)]
        fl = e5.floor_predictions(cases)
        self.assertEqual(fl["1:gf240E:0"]["effects"]["food_stock"]["7d"], 7.0)
        self.assertEqual(fl["1:gf240E:1"]["effects"]["food_stock"]["7d"], 5.0)
        self.assertEqual(fl["1:gf240E:2"]["effects"]["alive"]["7d"], 1.0)
        self.assertNotIn("2:sp240L:0", fl)                 # no other replicate: no floor prediction
        self.assertEqual((e5.ref_of(3), e5.other_rep(2)), (("c2", 1), 3))


class TestRidge(unittest.TestCase):
    def test_recovers_a_planted_linear_effect(self):
        from wm5 import effects5 as e5
        rng = np.random.default_rng(0)
        X = rng.normal(3, 2, (400, 6))
        w = np.array([1.5, -2.0, 0.0, 0.5, 0.0, 0.0])
        y = X @ w + 4.0 + rng.normal(0, 0.01, 400)
        m, s = e5.norm_stats(X)
        r = e5.Ridge(0.01).fit(X, y, m, s)
        Xt = rng.normal(3, 2, (50, 6))
        self.assertLess(np.max(np.abs(r.predict(Xt) - (Xt @ w + 4.0))), 0.05)
        self.assertLess(np.max(np.abs(r.w / s - w)), 0.01)

    def test_lambda_chosen_on_val(self):
        from wm5 import effects5 as e5
        rng = np.random.default_rng(1)
        tr = [fake_case(i, "gf240E", 0, 0.0) for i in range(60)]
        va = [fake_case(100 + i, "gf240E", 0, 0.0) for i in range(30)]
        X = {x.key: rng.normal(0, 1, 5) for x in tr + va}
        for x in tr + va:                                   # pure noise targets: heavy shrinkage wins
            x.truth["food_stock"]["7d"] = float(rng.normal())
        cell = ("gf240E", "food_stock", "7d")
        fits = e5.fit_cells(tr, va, X, X, {cell: 2.0}, [cell])
        self.assertGreaterEqual(fits[cell]["lambda"], 100.0)
        for x in tr + va:                                   # a strong linear signal: little shrinkage wins
            x.truth["food_stock"]["7d"] = float(X[x.key] @ np.array([3.0, -2.0, 1.0, 0.0, 0.5]))
        fits = e5.fit_cells(tr, va, X, X, {cell: 2.0}, [cell])
        self.assertLessEqual(fits[cell]["lambda"], 1.0)
        pred = e5.ridge_predict(fits, va[:3], X)
        for x in va[:3]:
            self.assertAlmostEqual(pred[x.key]["effects"]["food_stock"]["7d"], x.truth["food_stock"]["7d"], delta=0.2)

    def test_observable_features_have_no_hidden_field(self):
        from wm1 import data as d
        from wm2 import v2
        from wm4 import tokens  # noqa: F401
        from wm5 import effects5 as e5
        from wm4.test_wm4 import VARIANT
        st = VARIANT.state_names
        cal = tuple(VARIANT.input_names()[len(st): len(st) + 30])
        names = e5.obs_feature_names(st, cal)
        self.assertEqual(len(names), 37 * 3 + 30 + 4)
        hidden = set(v2.HIDDEN_FIELDS) | set(d.HIDDEN_V2) | set(e5.HIDDEN_GLOBAL) | set(e5.HIDDEN_SETTLEMENT)
        base = lambda n: n.split("@")[0].split(".", 1)[-1]
        self.assertFalse([n for n in names if base(n) in hidden or n.startswith("hidden")])
        priv = e5.priv_feature_names(("berry_bushes", "climate"))
        self.assertTrue(all(n.startswith("hidden.") for n in priv))
        self.assertEqual(len(priv), 3 + 7 + 6 + 2)


# ------------------------------------------------------------------ paper numbers (11.2) ----------------------------
class TestPaperNumbers(unittest.TestCase):
    def test_formats(self):
        from wm5 import figures as fg
        self.assertEqual(fg.fmt_value(0.4871, "3"), "0.487")
        self.assertEqual(fg.fmt_value(-0.0123, "3s"), "−0.012")
        self.assertEqual(fg.fmt_value(0.0123, "3s"), "+0.012")
        self.assertEqual(fg.fmt_value(1640, "0"), "1,640")
        self.assertEqual(fg.fmt_value(0.0312, "pct1"), "3.1 %")
        self.assertEqual(fg.fmt_ci({"point": 0.1, "lo": -0.02, "hi": 0.2}, 2), "+0.10 [−0.02, +0.20]")

    def test_render_and_check_on_a_synthetic_source(self):
        import json
        import tempfile
        from wm5 import figures as fg
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            src = {"a": {"b": 0.25, "ci": {"point": 0.1, "lo": 0.05, "hi": 0.15}}}
            (td / "s.json").write_text(json.dumps(src))
            fg._cache[str(td / "s.json")] = src
            old = dict(fg.SOURCES)
            fg.SOURCES["j5"] = str(td / "s.json")
            try:
                (td / "src.md").write_text("x {{j5:a/b#2}} and {{j5:a/ci#ci2}}.")
                j = fg.render(td / "src.md", td / "paper.md", td / "n.json")
                self.assertEqual((td / "paper.md").read_text(), "x 0.25 and +0.10 [+0.05, +0.15].")
                self.assertEqual(j["count"], 2)
                self.assertEqual(fg.check(td / "n.json", td / "paper.md"), [])
                (td / "paper.md").write_text("x 0.26 and +0.10 [+0.05, +0.15].")
                self.assertTrue(fg.check(td / "n.json", td / "paper.md"))
            finally:
                fg.SOURCES.clear(); fg.SOURCES.update(old)
                fg._cache.pop(str(td / "s.json"), None)

    def test_every_quoted_number_of_the_paper_is_in_its_source(self):
        from wm5 import figures as fg
        if not (fg.PAPER / "paper_numbers.json").exists():
            self.skipTest("paper not rendered yet")
        self.assertEqual(fg.check(), [])


if __name__ == "__main__":
    unittest.main()
