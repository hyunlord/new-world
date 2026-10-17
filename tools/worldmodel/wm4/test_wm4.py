"""WM-4 tests (docs/worldmodel/wm4/GOAL.md 7.5): the policy (probabilities per tercile, a single cast, determinism,
overlap), schedule rows, the prefix check of observed episodes, the baselines on synthetic worlds with a known effect
(no bias under a randomized policy; `naive` biased and `ipw` unbiased under a confounded one — a test that fails if the
weights are wrong), daily tokenization (cast token, phase alignment, daily tallies), CRN / CT forward passes and the
gradient-reversal sign, the prediction contract, the input list (no hidden field), the obs dataset registration, the
floored scales, and the hazard-head recalibration arithmetic.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

import wm4  # noqa: F401
from wm1 import common as c
from wm1 import data as d
from wm1 import m1_gru as m
from wm2 import effects as ef
from wm2 import v2
from wm2.test_wm2 import fam_member, write_episode
from wm4 import effects4 as e4
from wm4 import obs_split as osp
from wm4 import policy as pol
from wm4 import tokens as tk

HAVE_TORCH = m.torch is not None
Q = {"q_f1": 1.0, "q_f2": 2.0, "q_w2": 5.0}


def state(f: float, w: float = 1.0, n: int = 2881, alive: float = 10.0) -> dict:
    return pol.state_series(np.full(n, f * alive), np.full(n, w * alive), np.full(n, alive), np.ones(n))


class PolicyTests(unittest.TestCase):
    def test_probabilities_per_tercile_and_overlap(self) -> None:
        self.assertEqual(pol.probs(0.5, 1.0, Q), (0.030, 0.002, 0.004))
        self.assertEqual(pol.probs(1.5, 1.0, Q), (0.008, 0.008, 0.004))
        self.assertEqual(pol.probs(2.0, 6.0, Q), (0.002, 0.030, 0.012))      # f = q_f2 is the upper tercile
        self.assertEqual(pol.probs(0.5, 6.0, Q, decision=False), (0.0, 0.0, 0.0))
        for f in (0.1, 1.2, 3.0):
            for w in (1.0, 9.0):
                self.assertTrue(all(p > 0 for p in pol.probs(f, w, Q)))       # overlap in every state

    def test_decision_rows_and_ticks(self) -> None:
        self.assertEqual(pol.tick_of_day(11), 14280)                           # v2's fork tick
        self.assertEqual(pol.display_day(pol.tick_of_day(8)), 8)
        self.assertEqual(len(pol.DECISION_ROWS), 53)
        self.assertTrue(all((pol.tick_of_day(dd)) % 60 == 0 for dd in pol.DAYS))
        self.assertTrue(all(r % 24 == tk.PHASE_06 for r in pol.DECISION_ROWS))   # every decision at 06:00

    def test_single_cast_determinism_and_the_draw(self) -> None:
        s = state(0.5)
        runs = [pol.run(46001 + i, s, Q) for i in range(60)]
        again = [pol.run(46001 + i, s, Q) for i in range(60)]
        self.assertEqual(json.dumps(runs), json.dumps(again))
        for w in runs:
            casts = [r for r in w["rows"] if r["cast"]]
            self.assertLessEqual(len(casts), 1)
            if w["cast"]:
                self.assertEqual(w["rows"][-1]["cast"], w["cast"]["arm"])      # no decision after the cast
                r = w["rows"][-1]
                self.assertEqual(r["u"], float(np.random.default_rng([w["seed"], r["day"]]).random()))
                self.assertEqual(pol.choose(r["u"], (r["p_g"], r["p_s"], r["p_r"])), w["cast"]["arm"])
            else:
                self.assertEqual(len(w["rows"]), 53)
        # hungry worlds get gifts more often than spoils
        arms = [w["cast"]["arm"] for w in (pol.run(s0, s, Q) for s0 in range(2000)) if w["cast"]]
        self.assertGreater(arms.count("gf240"), 5 * arms.count("sp240"))

    def test_dead_world_is_not_a_decision(self) -> None:
        s = state(0.5, alive=0.0)
        w = pol.run(3, s, Q)
        self.assertIsNone(w["cast"])
        self.assertTrue(all(not r["decision"] and r["p_g"] == 0 for r in w["rows"]))

    def test_terciles(self) -> None:
        q = pol.terciles([state(f) for f in np.linspace(0, 3, 31)])
        allf = np.repeat(np.linspace(0, 3, 31), 53)
        self.assertAlmostEqual(q["q_f1"], float(np.percentile(allf, 33)))
        self.assertAlmostEqual(q["q_f2"], float(np.percentile(allf, 67)))

    def test_schedule_rows(self) -> None:
        rows = pol.schedule_rows("gf240", pol.tick_of_day(20))
        self.assertEqual(rows, [{"tick": 27240, "kind": "gift_food", "settlement": "home", "amount": 240}])
        self.assertEqual(set(rows[0]), {"tick", "kind", "settlement", "amount"})
        self.assertEqual(pol.schedule_rows("rw120", 41640)[0]["kind"], "rot_wood")
        with self.assertRaises(AssertionError):
            pol.schedule_rows("sp240", 41641)
        self.assertEqual(pol.ref_tick({"cast": None}), 41640)
        self.assertEqual(pol.ref_tick({"cast": {"tick": 27240}}), 27240)


def with_ids(ep):
    ep.arrays["settlement_ids.npy"] = np.array([1, 2], np.uint64)
    return ep


class PrefixTests(unittest.TestCase):
    def test_observed_equals_factual_through_the_cast_row(self) -> None:
        seed, cast_row = 46003, 166 + 24 * 5
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fac = with_ids(fam_member(seed, 0, None, cast_row))
            obs = with_ids(fam_member(seed, 0, "gf240E", cast_row))
            obs.interventions[0]["amount"] = 240
            write_episode(root, f"factual/game_v19_s{seed}_none", fac, seed, "none")
            write_episode(root, f"observed/game_v19_s{seed}_none", fac, seed, "none")
            write_episode(root, f"observed/game_v19_s{seed}_obs", obs, seed, "obs")
            policy = {"worlds": [{"seed": seed, "cast": {"arm": "gf240", "row": cast_row, "tick": cast_row * 60}}]}
            out = osp.observed_check(root / "factual", root / "observed", policy)
            self.assertTrue(out["all_ok"], out["rows"])
            self.assertTrue(out["rows"][0]["after_differs"])
            # a difference before the cast row is caught
            bad = with_ids(fam_member(seed, 0, "gf240E", cast_row - 30))
            bad.interventions[0].update({"amount": 240, "tick": cast_row * 60})
            write_episode(root, f"observed2/game_v19_s{seed}_none", fac, seed, "none")
            write_episode(root, f"observed2/game_v19_s{seed}_obs", bad, seed, "obs")
            out = osp.observed_check(root / "factual", root / "observed2", policy)
            self.assertFalse(out["all_ok"])
            self.assertFalse(out["rows"][0]["prefix_ok"])


# ------------------------------------------------------------------ baselines on synthetic worlds ------------------
TAU = 100.0


def synth_worlds(n: int, confounded: bool, seed: int = 0) -> list:
    """Worlds with a latent level L ~ U(0, 1): the no-cast food path grows by (L − 0.5) × 2 per row (state-dependent
    trend), f = 3 L; a gift adds TAU to food from the row after the cast on (a constant effect). The policy is the WM-4
    policy on f (confounded: hungry = low L gets gifts) or the same probabilities shuffled across worlds (randomized)."""
    rng = np.random.default_rng(seed)
    out = []
    T = v2.LAST_ROW + 1
    for i in range(n):
        L = rng.random()
        food = 100.0 + (L - 0.5) * 2.0 * np.arange(T)
        alive = np.full(T, 10.0)
        f_policy = 3.0 * (L if confounded else rng.random())
        s = pol.state_series(np.full(T, f_policy * 10.0), np.full(T, 10.0), alive, np.ones(T))
        w = pol.run(10_000 + i, s, Q)
        if w["cast"] and w["cast"]["arm"] == "gf240":
            food = food + TAU * (np.arange(T) > w["cast"]["row"])
        series = {"food_stock": food, "alive": alive, "deaths": np.zeros(T), "settlements_alive": np.ones(T),
                  "fissions_total": np.zeros(T)}
        out.append(e4.world_of(w, series, np.zeros((T, 8))))
    return out


class BaselineTests(unittest.TestCase):
    def test_randomized_policy_no_bias(self) -> None:
        est = e4.pooled_estimates(synth_worlds(3000, confounded=False))
        for name in ("naive", "ipw"):
            self.assertAlmostEqual(est[name]["gf240"]["7d"]["food_stock"], TAU, delta=12.0)
        self.assertAlmostEqual(est["ipw"]["sp240"]["7d"]["food_stock"], 0.0, delta=15.0)

    def test_confounded_policy_naive_biased_ipw_unbiased(self) -> None:
        est = e4.pooled_estimates(synth_worlds(3000, confounded=True))
        naive, ipw = est["naive"]["gf240"]["7d"]["food_stock"], est["ipw"]["gf240"]["7d"]["food_stock"]
        self.assertLess(naive, TAU - 60.0)                 # gifts go to falling worlds: naive is biased down
        self.assertAlmostEqual(ipw, TAU, delta=15.0)
        self.assertLess(abs(ipw - TAU), 0.25 * abs(naive - TAU))
        c7 = est["counts"]["gf240"]["7d"]
        self.assertLess(c7["ess_treated"], c7["treated"])
        self.assertLess(c7["ess_controls"], c7["controls"])

    def test_weights_by_hand(self) -> None:
        worlds = synth_worlds(40, confounded=True, seed=3)
        treated, controls = e4.stack(worlds, 168)
        u = treated["gf240"][0]
        w = next(x for x in worlds if x.seed == u.seed)
        self.assertAlmostEqual(u.weight, 1.0 / w.probs[u.row][0])
        k = controls[pol.DECISION_ROWS[0]][0]
        wk = next(x for x in worlds if x.seed == k.seed)
        pi = np.prod([1 - sum(wk.probs[j]) for j in pol.DECISION_ROWS if k.row <= j < k.row + 168])
        self.assertAlmostEqual(k.weight, 1.0 / pi)
        self.assertTrue(wk.cast_row is None or wk.cast_row >= k.row + 168)

    def test_match5_and_zero_and_selection(self) -> None:
        worlds = synth_worlds(300, confounded=True)
        bl = e4.Baselines(worlds, np.ones(8))
        case = ef.Case(1, "gf240E", 0, "val", pol.DECISION_ROWS[10] * 60, pol.DECISION_ROWS[10],
                       {o: {"7d": TAU if o == "food_stock" else 0.0, "30d": TAU if o == "food_stock" else 0.0}
                        for o in v2.OUTCOMES}, 0.0, (True, True), np.zeros((24, 8)))
        p = bl.predict(case)
        self.assertEqual(set(p), set(e4.BASELINES))
        self.assertEqual(p["zero"]["effects"]["food_stock"]["7d"], 0.0)
        self.assertEqual(len(p["match5"]["neighbours"]["7d"]["treated"]), 5)
        stds = {(a, o, lag): 1.0 for a in e4.V2_ARMS for o in v2.OUTCOMES for lag in v2.LAGS}
        cases = [ef.Case(1, a, 0, "val", case.cast_tick, case.cast_row, case.truth, 0.0, (True, True), case.window)
                 for a in e4.V2_ARMS]
        preds = {x.key: bl.predict(x) for x in cases}
        sel = e4.select(cases, {n: {k: v[n] for k, v in preds.items()} for n in e4.BASELINES}, stds)
        self.assertIn(sel["7d"]["chosen"], e4.BASELINES)
        # the truth is a pure food effect of the gift: zero misses it in one of 12 cells; the selection compares them
        self.assertEqual(set(sel["30d"]["val"]), set(e4.BASELINES))


class ScaleTests(unittest.TestCase):
    def test_floor_is_the_pooled_std(self) -> None:
        cases = []
        for i, arm in enumerate(["gf240E", "sp240L", "rw120E", "sp120L"] * 5):
            val = {"gf240E": 0.0, "sp240L": 10.0 * (i % 3), "rw120E": 1.0 * (i % 2), "sp120L": 5.0 * (i % 2)}[arm]
            cases.append({"seed": i, "arm": arm, "rep": 0, "role": "train", "cast_tick": 0, "cast_row": 300,
                          "truth": {o: {"7d": val, "30d": val} for o in v2.OUTCOMES}, "shift": 0.0,
                          "censored": [False, False]})
        tr = [ef.case_from(j) for j in cases]
        base = {"cases": {"train": cases}, "stds": ef.stds_json(ef.scale_stds(tr))}
        fl, un = e4.scales(base)
        pooled = float(np.std([j["truth"]["alive"]["7d"] for j in cases]))
        self.assertAlmostEqual(fl[("gf240E", "alive", "7d")], pooled)          # gf240E's own std is 0 -> the floor
        self.assertAlmostEqual(fl[("sp240L", "alive", "7d")], max(pooled, un[("sp240L", "alive", "7d")]))
        self.assertEqual(e4.TO_V2, {"gf240": "gf240E", "sp240": "sp240L", "rw120": "rw120E"})


# ------------------------------------------------------------------ tokens ------------------------------------------
VARIANT = m.dataset_variant(c.dataset("v2_v19"), "sev-blind_action-aware")


def fake_raw(cast=None, T=2881, seed=0) -> tk.Raw:
    rng = np.random.default_rng(seed)
    names = VARIANT.state_names
    st = np.abs(rng.normal(5, 2, (T, 37)))
    tally = np.array([n.startswith("d.") for n in names])
    st[:, tally] = rng.poisson(0.3, (T, int(tally.sum())))
    for n in tk.UNIT:
        st[:, names.index(n)] = rng.integers(0, 2, T)
    cal = d.calendar_features((8 + np.arange(T)) % 24, np.zeros(T, int), np.zeros(T))
    act = np.zeros((T, 7))
    if cast is not None:
        act[cast[0], cast[1]] = cast[2]
    return tk.Raw(seed, "x", st, cal, act, tally)


class TokenTests(unittest.TestCase):
    def test_cast_token_and_phase(self) -> None:
        for row in (pol.DECISION_ROWS[7], 490):                           # 06:00 (obs, `L`) and 18:00 (`E`)
            raw = fake_raw((row, 0, 240.0))
            norm = tk.Norm.fit([raw])
            seq = tk.tokenize(raw, tk.phase_of_row(row), norm)
            self.assertEqual(len(seq.rows), tk.N_TOKENS)
            self.assertTrue(np.all(seq.rows % 24 == tk.phase_of_row(row)))
            k = tk.token_of_row(row)
            self.assertEqual(seq.rows[k], row)
            cast = np.nonzero(seq.a[:, 0] == 0)[0]
            self.assertEqual(cast.tolist(), [k])                              # the cast falls in exactly one token
            self.assertEqual(seq.a[k, 1], 1.0)
            self.assertAlmostEqual(seq.a[k, tk.N_CLASSES], 240.0 / 120.0)
            self.assertTrue(np.array_equal(seq.a[k], tk.arm_treatment("gift_food", 240)))
        self.assertEqual(tk.PHASE_06, pol.DECISION_ROWS[0] % 24)
        self.assertEqual(tk.PHASE_18, 490 % 24)

    def test_daily_tallies_and_levels(self) -> None:
        raw = fake_raw()
        rows = tk.token_rows(22)
        s = tk.daily_state(raw, rows)
        i = list(VARIANT.state_names).index("d.deaths_starvation")
        j = list(VARIANT.state_names).index("g.alive")
        self.assertAlmostEqual(s[3, i], raw.state[rows[3] - 23: rows[3] + 1, i].sum())
        self.assertAlmostEqual(s[0, i], raw.state[: rows[0] + 1, i].sum())   # the first token sums what it has
        self.assertEqual(s[3, j], raw.state[rows[3], j])
        with self.assertRaises(ValueError):
            tk.encode(np.array([[240.0, 120.0, 0, 0, 0, 0, 0]]))             # two kinds in one token

    def test_inputs_are_wm2s_list_without_hidden_fields(self) -> None:
        names = VARIANT.input_names()
        self.assertEqual(len(names), 74)
        self.assertEqual(VARIANT.state_names, tuple(names[:37]))
        hidden = set(v2.HIDDEN_FIELDS) | set(d.HIDDEN_V2)
        self.assertFalse([n for n in names if n.split(".", 1)[-1] in hidden])
        self.assertEqual(len(tk.DEATHS), len(v2.DEATH_COLUMNS))
        self.assertTrue(all(f"d.{x}" in VARIANT.state_names for x in v2.DEATH_COLUMNS))


class RegistryTests(unittest.TestCase):
    def test_obs_dataset(self) -> None:
        from wm4 import m1_obs
        m1_obs.register()
        ds = c.dataset("obs_v19")
        self.assertEqual(ds.groups, {"train": ("obs_train", "train"), "val": ("obs_val", "val"),
                                     "test": ("obs_test", "test")})
        self.assertEqual((ds.train, ds.train_action, ds.val, ds.test), (("train",), ("train",), ("val",), ("test",)))
        self.assertEqual(ds.extra_state, c.V17_EXTRA)
        self.assertEqual(m.dataset_variant(ds, "sev-blind_action-aware").input_names(), VARIANT.input_names())
        self.assertIn("--max-steps", "--max-steps 8000")
        self.assertEqual(m1_obs.FLAGS[m1_obs.FLAGS.index("--window") + 1], "336")


# ------------------------------------------------------------------ CRN / CT ----------------------------------------
@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class CausalTests(unittest.TestCase):
    def test_grad_reverse_sign(self) -> None:
        from wm4 import causal as ca
        torch = m.torch
        x = torch.ones(3, requires_grad=True)
        (ca.grad_reverse(x, 0.7) * torch.tensor([1.0, 2.0, 3.0])).sum().backward()
        self.assertTrue(torch.allclose(x.grad, -0.7 * torch.tensor([1.0, 2.0, 3.0])))
        self.assertAlmostEqual(ca.alpha_at(0.0, 1.0), 0.0)
        self.assertAlmostEqual(ca.alpha_at(1.0, 1.0), 2 / (1 + np.exp(-10)) - 1)

    def test_head_reversal_reaches_the_representation(self) -> None:
        from wm4 import causal as ca
        torch = m.torch
        torch.manual_seed(0)
        h = ca.head(8, 6, 5, 1.0, "grad_reverse")
        br = torch.randn(2, 4, 6, requires_grad=True)
        h.alpha = 1.0
        h.build_treatment(br).sum().backward()
        g_rev = br.grad.clone()
        br.grad = None
        h.balancing = "none"
        h.build_treatment(br).sum().backward()
        self.assertTrue(torch.allclose(g_rev, -br.grad))
        self.assertEqual(h.build_treatment(br).shape[-1], tk.N_CLASSES)
        self.assertEqual(h.build_outcome(br, torch.zeros(2, 4, tk.N_TREAT)).shape[-1], ca.N_OUT)

    def _batch(self, B=2, K=12):
        torch = m.torch
        a = torch.zeros(B, K, tk.N_TREAT); a[..., 0] = 1
        return {"y": torch.randn(B, K, 37), "v": torch.randn(B, K, 30), "a": a, "prev_a": torch.zeros(B, K, tk.N_TREAT)}

    def test_forward_shapes_and_causality(self) -> None:
        from wm4 import causal as ca
        torch = m.torch
        torch.manual_seed(0)
        bt = self._batch()
        enc = ca.CRNEncoder(ca.DEFAULTS["crn"]).eval()
        t, o, br = enc(bt)
        self.assertEqual((t.shape[-1], o.shape[-1], br.shape[-1]), (4, 37, 24))
        net = ca.CT(ca.DEFAULTS["ct"]).eval()
        t, o, _ = net(bt, split=torch.tensor([5, 12]))
        self.assertEqual((t.shape, o.shape), ((2, 12, 4), (2, 12, 37)))
        # causal: changing token 8 leaves the outputs at tokens <= 7 unchanged (both models)
        bt2 = {k: v.clone() for k, v in bt.items()}
        bt2["y"][:, 8:] += 5.0
        _, o1, _ = net(bt, split=torch.tensor([12, 12]))
        _, o2, _ = net(bt2, split=torch.tensor([12, 12]))
        self.assertTrue(torch.allclose(o1[:, :8], o2[:, :8], atol=1e-5))
        self.assertFalse(torch.allclose(o1[:, 8:], o2[:, 8:]))
        _, e1, _ = enc(bt)
        _, e2, _ = enc(bt2)
        self.assertTrue(torch.allclose(e1[:, :8], e2[:, :8], atol=1e-6))
        # CT: masked covariates after the split do not matter
        bt3 = {k: v.clone() for k, v in bt.items()}
        bt3["v"][:, 6:] = 99.0
        _, o3, _ = net(bt, split=torch.tensor([6, 6]))
        _, o4, _ = net(bt3, split=torch.tensor([6, 6]))
        self.assertTrue(torch.allclose(o3, o4, atol=1e-5))

    def test_prediction_contract(self) -> None:
        """effect = arm rollout − none rollout from the same history at the cast token, read at tokens + 7 / + 30;
        deaths summed over the daily deaths; the plans carry the arm at the first token only."""
        from wm4 import causal as ca
        names = VARIANT.state_names
        seen = []

        def fake_rollout(model, jobs, device, chunk=512):
            seen.extend(jobs)
            out = []
            for j in jobs:
                base = np.zeros((ca.HORIZON, 37))
                if j.plan[0][0] == 0:            # the arm's rollout
                    base[:, names.index("s.food_stock")] = np.arange(1, ca.HORIZON + 1)
                    base[:, names.index("d.deaths_starvation")] = 1.0
                out.append(base)
            return out

        cast_row = 490
        raw_ep = fam_member(46500, 0, None, cast_row)
        ep = d.from_episode(raw_ep, "v19_fam", "test", 46500, "none", c.V17_EXTRA)

        class F:
            seed, factual = 46500, {0: ep}

        case = ef.Case(46500, "gf240E", 0, "test", cast_row * 60, cast_row,
                       {o: {"7d": 0.0, "30d": 0.0} for o in v2.OUTCOMES}, 0.0, (True, True), None)
        norm = tk.Norm.fit([tk.raw_of(ep, VARIANT)])
        orig = ca.rollout
        ca.rollout = fake_rollout
        try:
            p = ca.predict_cases({"names": names, "norm": norm}, [F], [case], "cpu")[case.key]
        finally:
            ca.rollout = orig
        self.assertEqual(p["effects"]["food_stock"]["7d"], 7.0)
        self.assertEqual(p["effects"]["food_stock"]["30d"], 30.0)
        self.assertEqual(p["effects"]["deaths"]["7d"], 7.0)
        self.assertEqual(p["effects"]["deaths"]["30d"], 30.0)
        self.assertEqual(p["effects"]["alive"]["7d"], 0.0)
        arm, none = seen
        self.assertEqual(arm.k, tk.token_of_row(cast_row))
        self.assertEqual(arm.seq.rows[arm.k], cast_row)
        self.assertTrue(np.array_equal(arm.plan[0], tk.arm_treatment("gift_food", 240)))
        self.assertTrue(all(np.array_equal(r, tk.NONE) for r in arm.plan[1:]))
        self.assertTrue(all(np.array_equal(r, tk.NONE) for r in none.plan))
        self.assertIs(arm.seq, none.seq)


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class RecalTests(unittest.TestCase):
    def test_head_rewrite_equals_platt(self) -> None:
        from wm4 import recal_hazard as rh
        torch = m.torch

        class Mod(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.haz = torch.nn.Linear(4, 1)

        torch.manual_seed(0)
        mod = Mod()
        x = torch.randn(10, 4)
        before = mod.haz(x).detach().numpy()[:, 0]
        rh.write_head(mod, 1.7, -0.4)
        after = mod.haz(x).detach().numpy()[:, 0]
        self.assertTrue(np.allclose(after, 1.7 * before - 0.4, atol=1e-5))

    def test_p30_and_fit(self) -> None:
        from wm4 import recal_hazard as rh
        L = np.full((3, 720), -6.0)
        p = rh.p30(L, 1.0, 0.0)
        h = 1 / (1 + np.exp(6.0))
        self.assertTrue(np.allclose(p, 1 - (1 - h) ** 720))
        rng = np.random.default_rng(0)
        L = rng.normal(-7, 1, (1500, 720)) + rng.normal(0, 1.5, (1500, 1))
        y = (rng.random(1500) < rh.p30(L, 1.3, 0.8)).astype(float)
        f = rh.fit_platt(L, y)
        self.assertLess(f["nll_after"], f["nll_before"])
        self.assertAlmostEqual(f["a"], 1.3, delta=0.35)


if __name__ == "__main__":
    unittest.main()
