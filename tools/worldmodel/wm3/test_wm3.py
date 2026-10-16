"""WM-3 tests (docs/worldmodel/wm3/GOAL.md 4.3, 6, 7): the audit's checks and the harness's lag-1 response on a stub
model; the input list; `M1TFH` (bit-identical to `M1TF` with a WM-2 state dict, registered for `g.load_predictor`); the
cast-window sampler; the loss (equal to `losses_tf` without cast windows, the pair and hazard terms); the hazard rollout
and the expected split day; `effects3.predictions` against `wm2.effects.model_predictions` (val families, DGX only); the
validation baselines of V2.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest

import numpy as np

import wm3  # noqa: F401
import bench_v0 as b
from wm1 import common as c
from wm1 import data as d
from wm1 import m1_gru as m
from wm2 import v2
from wm2.test_wm2 import FakeFamily, fam_member, synth_v6

HAVE_TORCH = m.torch is not None
VARIANT = m.dataset_variant(c.dataset("v2_v19"), "sev-blind_action-aware")


def member_data(raw, seed: int, eid: str, rep: int, role: str = "train") -> d.EpisodeData:
    split = {(0, True): c.V19_FACT, (0, False): c.V19_ARM_R0, (1, True): c.V19_NONE_R1, (1, False): c.V19_ARM_R1}
    return d.from_episode(raw, split[(rep, eid == "none")], role, seed, eid, c.V17_EXTRA)


def synth_family(seed: int, arm: str = "gf240E", cast_row: int = 600):
    fac = {r: fam_member(seed, r, None) for r in (0, 1)}
    var = {arm: {r: fam_member(seed, r, arm, cast_row) for r in (0, 1)}}
    return FakeFamily(seed, fac, var)


class AuditA1Tests(unittest.TestCase):
    def test_structural_checks_pass_and_the_ratio_reads_the_jump_at_c_plus_1(self) -> None:
        from wm2 import effects as ef
        from wm3 import audit as au
        fam = synth_family(3)
        cases = ef.family_cases(fam, "train")
        members = {}
        for r, raw in fam.factual.items():
            members[(3, "none", r)] = member_data(raw, 3, "none", r)
        for arm, reps in fam.variants.items():
            for r, raw in reps.items():
                members[(3, arm, r)] = member_data(raw, 3, arm, r)
        train = [ep for ep in members.values()]
        norm = d.Normalizer.fit(train, VARIANT)
        for x in cases:
            row = au.a1_case(x, *au.pair(x, members), VARIANT, norm)
            self.assertTrue(all(row["checks"].values()), row["checks"])
            # fam_member adds +40 food from the cast row + 1 on, while the record says 240 were applied.
            self.assertAlmostEqual(row["ratio_c1"], 40.0 / 240.0, places=4)
            self.assertFalse(row["behaviour_ok"])
        summary = au.a1_summary([au.a1_case(x, *au.pair(x, members), VARIANT, norm) for x in cases])
        self.assertEqual(summary["structural_total"], 0)
        self.assertEqual(summary["behaviour_outside"], 2)
        # A shifted action (one row late) is a structural mismatch.
        x = cases[0]
        arm = members[(3, x.arm, x.rep)]
        arm.bench.actions = np.roll(arm.bench.actions, 1, axis=0)
        bad = au.a1_case(x, arm, members[(3, "none", x.rep)], VARIANT, norm)
        self.assertFalse(bad["checks"]["action_position"])
        self.assertEqual(au.decide(au.a1_summary([bad]), {"r": {"violations": 0}})["verdict"], "bug")


if HAVE_TORCH:
    nn = m.torch.nn

    class StubTF(nn.Module):
        """A stub transformer for the harness: `run` keeps M1TF's (mu, logvar, fire, kv) / valid contract; its mean
        change of the food store is the food action input x 120 / dstd[food] (gift positive, spoil negative), every
        other change 0; `blocks` makes `rollout_ext._is_tf` take the transformer path."""

        def __init__(self, lay: d.Layout, norm: d.Normalizer):
            super().__init__()
            self.blocks = nn.ModuleList()
            self.n_cont = lay.n_cont
            self.food = list(lay.cont).index(lay.names.index("s.food_stock"))
            self.dstd_food = float(norm.dstd[self.food])
            self.act0 = lay.n_state + len(d.CALENDAR)
            self.mu = nn.Linear(1, 1)

        def run(self, x, cache=None, valid=None):
            B, L, _ = x.shape
            torch = m.torch
            a = x[..., self.act0 + b.KINDS.index("gift_food")] - x[..., self.act0 + b.KINDS.index("spoil_food")]
            mu = torch.zeros(B, L, self.n_cont)
            mu[..., self.food] = a * 120.0 / self.dstd_food
            kv = [(torch.zeros(B, 1, L, 1), torch.zeros(B, 1, L, 1))]
            return mu, torch.zeros(B, L, self.n_cont), torch.full((B, L), -5.0), kv


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class HarnessLag1Tests(unittest.TestCase):
    def setUp(self) -> None:
        from wm1 import m1_tf
        self.ep = member_data(synth_v6(5, 900), 5, "none", 0, "val")
        norm = d.Normalizer.fit([member_data(synth_v6(9, 900), 9, "none", 0)], VARIANT)
        norm.lo, norm.hi = norm.lo - 1e4, norm.hi + 1e4       # no clamp within the stub's jumps
        self.lay = d.Layout(VARIANT)
        self.norm = norm
        self.pred = m1_tf.TFPredictor(StubTF(self.lay, norm), self.lay, norm, VARIANT, "cpu", 24, context=24)

    def test_the_harness_returns_the_stubs_jump_at_lag_1(self) -> None:
        from wm3 import audit as au
        food = list(d.V_STATE).index("s.food_stock")
        for arm, want in (("gf240E", 240.0), ("sp240L", -240.0), ("sp120L", -120.0), ("rw120E", 0.0)):
            (fv, av), = au.harness_lag1(self.pred, [(self.ep, arm, 400)])
            self.assertAlmostEqual(float(av[food] - fv[food]), want, places=3, msg=arm)
            others = [i for i in range(8) if i != food]
            np.testing.assert_allclose(av[others], fv[others], atol=1e-6)
        # A3's comparison on the stub: the stepped teacher-forced outputs equal the harness's lag-1 difference.
        prep = d.prepare(self.ep, VARIANT, self.norm, actions=au.arm_actions(self.ep, "gf240E", 400)[1])
        x, valid = au.window_input(prep, 400, 24)
        mu, fl = au.teacher_forced(self.pred, x[None].astype(np.float32), valid[None])
        nxt = au.stepped_lag1(self.pred, self.ep.table[400], mu[0], float(fl[0]))
        self.assertAlmostEqual(float(nxt[self.lay.names.index("s.food_stock")] - self.ep.table[400, self.lay.names.index("s.food_stock")]),
                               240.0, places=3)


WM2_CONFIG = c.REPO / "docs/worldmodel/wm2/runs/wm2_m1bact_v19_s0/config.json"
BASE = c.REPO / "docs/worldmodel/wm2/runs/effects_base/effects_base.json"


class InputListTests(unittest.TestCase):
    """GOAL.md 4.3: WM-2's action-aware list (74 = 67 state and calendar + 7 actions), no hidden field."""

    def test_input_list_is_wm2s_and_has_no_hidden_field(self) -> None:
        import json
        names = list(VARIANT.input_names())
        self.assertEqual(len(names), 74)
        self.assertEqual(names, json.loads(WM2_CONFIG.read_text())["input_names"])
        hidden = set(d.HIDDEN_V2) | set(v2.HIDDEN_FIELDS) | set(d.FORBIDDEN) | set(d.SEVERITY_ONLY) | set(d.NOT_INPUTS_V5)
        for n in names:
            self.assertNotIn(n.split(".", 1)[-1], hidden, n)
            self.assertNotIn(n, hidden, n)
        self.assertIn("g.fissions_total", names)          # the hazard target is an observable input column already


def tiny_cfg(**kw) -> dict:
    from wm1 import m1_tf
    return dict(m1_tf.DEFAULTS, d_model=16, layers=2, heads=2, context=24, warmup=24, **kw)


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class ModelTests(unittest.TestCase):
    def setUp(self) -> None:
        from wm1 import m1_tf
        m.set_seed(0, 2)
        self.lay = d.Layout(VARIANT)
        self.ref = m1_tf.M1TF(self.lay.n_input, self.lay.n_cont, 16, 2, 2, context=24)

    def test_wm2_state_dict_gives_bit_identical_heads(self) -> None:
        from wm1 import m1_tf
        from wm3 import model as wm
        h = wm.M1TFH(self.lay.n_input, self.lay.n_cont, 16, 2, 2, context=24, seed=3)
        wm.load_init(h, self.ref.state_dict())
        res = h.load_state_dict(self.ref.state_dict(), strict=False)
        self.assertEqual(sorted(res.missing_keys), ["haz.bias", "haz.weight"])
        self.assertEqual(res.unexpected_keys, [])
        torch = m.torch
        g = torch.Generator().manual_seed(1)
        x = torch.randn(3, 30, self.lay.n_input, generator=g)
        valid = torch.ones(3, 30, dtype=torch.bool)
        valid[0, :5] = False
        a, b_ = self.ref.run(x, valid=valid), h.run(x, valid=valid)
        for i in range(3):
            self.assertTrue(torch.equal(a[i], b_[i]), i)
        mu, lv, fl, hz, kv = h.run_h(x, valid=valid)
        self.assertTrue(torch.equal(mu, a[0]) and torch.equal(fl, a[2]))
        self.assertEqual(tuple(hz.shape), (3, 30))
        # The cached (rollout) path: one appended token gives the same heads.
        cache = m1_tf.append(None, a[3], valid, keep=23)
        x1 = torch.randn(3, 1, self.lay.n_input, generator=g)
        ones = torch.ones(3, 1, dtype=torch.bool)
        r1, r2 = self.ref.run(x1, cache, ones), h.run(x1, cache, ones)
        for i in range(3):
            self.assertTrue(torch.equal(r1[i], r2[i]))
        # The hazard layer is seeded: two builds with the same seed are equal, a different seed is not.
        h2 = wm.M1TFH(self.lay.n_input, self.lay.n_cont, 16, 2, 2, context=24, seed=3)
        h3 = wm.M1TFH(self.lay.n_input, self.lay.n_cont, 16, 2, 2, context=24, seed=4)
        self.assertTrue(torch.equal(h.haz.weight, h2.haz.weight))
        self.assertFalse(torch.equal(h.haz.weight, h3.haz.weight))
        with self.assertRaises(ValueError):
            wm.load_init(h, {k: v for k, v in self.ref.state_dict().items() if k != "mu.bias"})

    def test_run_directory_round_trips_through_load_predictor(self) -> None:
        import json
        import tempfile
        from pathlib import Path
        from wm3 import model as wm
        norm = d.Normalizer.fit([member_data(synth_v6(9, 400), 9, "none", 0)], VARIANT)
        cfg = tiny_cfg(clamp_nonneg=1, haz_seed=0, hazard=10.0)
        net = wm.build(self.lay, cfg)
        with tempfile.TemporaryDirectory() as tmp:
            run = Path(tmp)
            m.torch.save(net.state_dict(), run / "best.pt")
            (run / "config.json").write_text(json.dumps(
                {"model": "m1_tfh", "dataset": "v2_v19", "variant": VARIANT.name, "input_names": list(VARIANT.input_names()),
                 "normalizer": norm.to_json(), "hyper": cfg, "round_counts": True, "best_step": 0, "stop": "patience",
                 "selected": "init", "seed": 0}))
            pred, config = m.load_predictor(run, "cpu")
        self.assertIsInstance(pred.model, wm.M1TFH)
        self.assertTrue(pred.round_counts)
        self.assertEqual(pred.context, 24)
        for k, v in net.state_dict().items():
            self.assertTrue(m.torch.equal(v, pred.model.state_dict()[k]), k)


def synth_train(seeds=(1, 2, 3), cast_row: int = 400, rows: int = 900):
    """Synthetic train members (factual and one gift arm, both replicates) whose arm equals its partner through the
    cast row and gets +240 food after it."""
    from wm2.test_wm2 import SETTLE
    eps = []
    for s in seeds:
        for rep in (0, 1):
            fac = synth_v6(10 * s + rep, rows)
            eps.append(member_data(fac, s, "none", rep))
            arm = synth_v6(10 * s + rep, rows)
            arm.arrays["settlements.npy"][cast_row + 1:, 0, SETTLE.index("food_stock")] += 240.0
            arm.interventions = [{"tick": cast_row * 60, "kind": "gift_food", "amount": 240, "applied": 240,
                                  "settlement": 1}]
            eps.append(member_data(arm, s, "gf240E", rep))
    return eps


class SamplerTests(unittest.TestCase):
    def setUp(self) -> None:
        from wm3 import sampling as sp
        self.sp = sp
        self.eps = synth_train()
        self.norm = d.Normalizer.fit(self.eps, VARIANT)
        self.preps = [d.prepare(ep, VARIANT, self.norm) for ep in self.eps]
        self.cases = sp.train_cases(self.eps)

    def test_cases_and_cast_windows(self) -> None:
        sp = self.sp
        self.assertEqual(len(self.cases), 6)
        self.assertEqual({(x.arm, x.cast_row) for x in self.cases}, {("gf240E", 400)})
        smp = sp.Sampler(self.preps, self.cases, 5, 336, 3, 4, True)
        win, drawn = smp.draw()
        self.assertEqual(win["x_state"].shape[0], 3 + 4 + 4)
        self.assertEqual(smp.batch, 11)
        act = win["actions"]
        k = b.KINDS.index("gift_food")
        for i in range(4):
            arm, part = 3 + i, 7 + i
            self.assertEqual(float(act[arm, sp.CAST_POS, k]), 2.0)            # 240 / ACTION_SCALE 120
            self.assertEqual(int((act[arm] != 0).sum()), 1)
            self.assertFalse(act[part].any())
            for key in ("x_state", "calendar", "state"):
                np.testing.assert_array_equal(win[key][arm, : sp.CAST_POS + 1], win[key][part, : sp.CAST_POS + 1])
            np.testing.assert_array_equal(win["dtarget"][arm, : sp.CAST_POS], win["dtarget"][part, : sp.CAST_POS])
            self.assertGreater(abs(float(win["state"][arm, sp.CAST_POS + 1, 0] - win["state"][part, sp.CAST_POS + 1, 0])) + 1, 0)
        food = list(VARIANT.state_names).index("s.food_stock")
        self.assertAlmostEqual(float(win["state"][3, sp.CAST_POS + 1, food] - win["state"][7, sp.CAST_POS + 1, food]), 240.0,
                               places=3)
        # The plain rows are `sample_windows` on default_rng([seed, 0]).
        ref = d.sample_windows(self.preps, 336, 3, np.random.default_rng([5, 0]))
        for key in ref:
            np.testing.assert_array_equal(win[key][:3], ref[key])
        # An arm that differs from its partner before the cast is refused.
        bad = synth_train((7,))
        bad[1].table[10, 0] += 1.0
        with self.assertRaises(ValueError):
            sp.train_cases(bad)


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class LossTests(unittest.TestCase):
    def setUp(self) -> None:
        from wm3 import model as wm
        from wm3 import sampling as sp
        m.set_seed(0, 2)
        self.sp = sp
        self.eps = synth_train()
        self.norm = d.Normalizer.fit(self.eps, VARIANT)
        self.lay = d.Layout(VARIANT)
        self.preps = [d.prepare(ep, VARIANT, self.norm) for ep in self.eps]
        self.cases = sp.train_cases(self.eps)
        self.model = wm.M1TFH(self.lay.n_input, self.lay.n_cont, 16, 1, 2, context=24, seed=0)
        self.st = m.Stepper(self.lay, self.norm, "cpu", True)

    def test_no_cast_windows_equals_losses_tf(self) -> None:
        from wm1 import m1_tf
        from wm3 import losses as ls
        for rs in (1, 2):
            cfg = {"roll_starts": rs, "roll_min_context": 40, "tf_l1": 1.0, "v_weight": 5.0, "pair": 0.0, "hazard": 0.0}
            bt = m.batch_tensors(d.sample_windows(self.preps, 64, 5, np.random.default_rng(3)), "cpu")
            tf1, p1, r1 = m1_tf.losses_tf(self.model, self.st, bt, 6, cfg, np.random.default_rng(9))
            tf2, p2, r2, pair, haz = ls.wm3_losses(self.model, self.st, bt, 6, cfg, np.random.default_rng(9), 5)
            self.assertTrue(m.torch.equal(tf1, tf2))
            self.assertTrue(m.torch.equal(r1, r2))
            self.assertEqual(float(pair), 0.0)
            self.assertEqual(float(haz), 0.0)
            # The gradients are equal too.
            g1 = m.torch.autograd.grad(tf1 + r1, list(self.model.parameters())[:3], retain_graph=True)
            g2 = m.torch.autograd.grad(tf2 + r2, list(self.model.parameters())[:3], retain_graph=True)
            for a, b_ in zip(g1, g2):
                self.assertTrue(m.torch.equal(a, b_))

    def test_pair_and_hazard_terms(self) -> None:
        from wm3 import losses as ls
        sp = self.sp
        cfg = {"roll_starts": 1, "roll_min_context": 167, "tf_l1": 1.0, "v_weight": 5.0, "pair": 1.0, "hazard": 10.0}
        smp = sp.Sampler(self.preps, self.cases, 2, 336, 2, 3, True)
        win, _ = smp.draw()
        bt = m.batch_tensors(win, "cpu")
        tf, parts, roll, pair, haz = ls.wm3_losses(self.model, self.st, bt, 8, cfg, np.random.default_rng(1), 2, 3, True)
        self.assertGreater(float(pair), 0.0)
        self.assertTrue(np.isfinite(float(haz)) and float(haz) > 0)
        (tf + roll + pair + 10 * haz).backward()
        self.assertIsNotNone(self.model.haz.weight.grad)
        # By hand: the pair loss is the mean over steps and continuous features of the weighted, scaled error of the
        # arm - partner difference, from two separate rollouts of the cast window and its partner at p.
        want = self.hand_pair(bt, 2, 3, 8, cfg)
        self.assertAlmostEqual(float(pair), want, places=5)
        # A cast window paired with itself has zero pair loss.
        same = {k: m.torch.cat([v[:2], v[2:5], v[2:5]]) for k, v in bt.items()}
        _, _, _, pair0, _ = ls.wm3_losses(self.model, self.st, same, 8, cfg, np.random.default_rng(1), 2, 3, True)
        self.assertEqual(float(pair0), 0.0)
        # Partners that do not share the start state are refused.
        bad = {k: v.clone() for k, v in bt.items()}
        bad["state"][5, sp.CAST_POS, 0] += 1.0
        with self.assertRaises(ValueError):
            ls.wm3_losses(self.model, self.st, bad, 8, cfg, np.random.default_rng(1), 2, 3, True)

    def hand_pair(self, bt, n_plain, n_cast, k, cfg) -> float:
        from wm1 import m1_tf
        torch = m.torch
        p = self.sp.CAST_POS
        L = bt["dtarget"].shape[1]
        w = self.st.feature_weights(cfg["v_weight"])
        cont = self.st.cont
        total = 0.0
        with torch.no_grad():
            for i in range(n_cast):
                curs, trues = [], []
                for row in (n_plain + i, n_plain + n_cast + i):
                    one = {kk: v[row: row + 1] for kk, v in bt.items()}
                    inp = torch.cat([one["x_state"], one["calendar"], one["actions"], one["sev"]], -1)
                    mu, _, fl, kv = self.model.run(inp[:, :L])
                    cache = {"kv": kv, "valid": (torch.arange(L) <= p)[None]}
                    mu_j, fl_j, cur = mu[:, p], fl[:, p], one["state"][:, p]
                    cs, ts = [], []
                    for s in range(k):
                        cur = self.st.next_state(cur, mu_j, fl_j, hard=False)
                        cs.append(cur[0, cont])
                        ts.append(one["state"][0, p + s + 1, cont])
                        nxt = torch.cat([self.st.normalize(cur), one["calendar"][:, p + s + 1], one["actions"][:, p + s + 1],
                                         one["sev"][:, p]], -1)
                        mm, _, ff, new_kv = self.model.run(nxt.unsqueeze(1), cache, torch.ones(1, 1, dtype=torch.bool))
                        cache = m1_tf.append(cache, new_kv, torch.ones(1, 1, dtype=torch.bool))
                        mu_j, fl_j = mm[:, 0], ff[:, 0]
                    curs.append(torch.stack(cs))
                    trues.append(torch.stack(ts))
                dd = ((curs[0] - curs[1]) - (trues[0] - trues[1])) / self.st.std[cont]
                total += float((dd.abs() * w).mean(dim=1).sum())
        return total / (n_cast * k) * 1.0

    def test_hazard_target(self) -> None:
        from wm3 import losses as ls
        torch = m.torch
        fi = self.lay.names.index("g.fissions_total")
        state = torch.zeros(2, 6, self.lay.n_state)
        state[0, 3:, fi] = 1.0
        state[1, :, fi] = torch.tensor([0.0, 0.2, 0.4, 1.0, 1.0, 2.0])
        y = ls.hazard_target(state, fi)
        self.assertEqual(y.tolist(), [[0, 0, 1, 0, 0], [0, 0, 1, 0, 1]])


class ExpectedDayTests(unittest.TestCase):
    def test_constant_hazard_is_the_geometric_expectation(self) -> None:
        from wm3 import effects3 as e3
        ref = 2000
        K = v2.LAST_ROW - ref
        h = 0.01
        days = v2.day_of_row(ref + np.arange(1, K + 1)).astype(float)
        pk = (1 - h) ** np.arange(K) * h
        want = float((pk * days).sum() + (1 - h) ** K * v2.LAST_DAY)
        got = e3.expected_day(np.full(K, h), ref)
        self.assertAlmostEqual(got["E"], want, places=9)
        self.assertAlmostEqual(got["S_K"], (1 - h) ** K, places=12)
        self.assertAlmostEqual(got["p30"], 1 - (1 - h) ** 720, places=12)

    def test_certain_split_at_step_5(self) -> None:
        from wm3 import effects3 as e3
        ref = 2500
        h = np.zeros(v2.LAST_ROW - ref)
        h[4] = 1.0
        got = e3.expected_day(h, ref)
        self.assertEqual(got["E"], float(v2.day_of_row(ref + 5)))
        self.assertEqual(got["S_K"], 0.0)
        none = e3.expected_day(np.zeros(v2.LAST_ROW - ref), ref)
        self.assertEqual(none["E"], float(v2.LAST_DAY))
        with self.assertRaises(ValueError):
            e3.expected_day(np.zeros(10), ref)


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class HazardRolloutTests(unittest.TestCase):
    def test_hazard_rollout_reproduces_rollout_jobs(self) -> None:
        from wm1 import m1_tf
        from wm2 import rollout_ext as rx
        from wm3 import effects3 as e3
        from wm3 import model as wm
        m.set_seed(0, 2)
        lay = d.Layout(VARIANT)
        norm = d.Normalizer.fit([member_data(synth_v6(9, 300), 9, "none", 0)], VARIANT)
        eps = [member_data(synth_v6(s, 300), s, "none", 0, "val") for s in (1, 2)]
        net = wm.M1TFH(lay.n_input, lay.n_cont, 32, 2, 4, context=24, seed=1)
        pred = m1_tf.TFPredictor(net, lay, norm, VARIANT, "cpu", 24, context=24, nonneg=True)
        acts = np.zeros_like(eps[0].bench.actions)
        acts[120, 1] = 240.0
        jobs = [rx.Job(eps[0], 120, acts, 40), rx.Job(eps[1], 50, eps[1].bench.actions, 90),
                rx.Job(eps[0], 120, np.zeros_like(acts), 40), rx.Job(eps[1], 280, acts, 19)]
        for rc in (False, True):
            pred.round_counts = rc
            ref = rx.rollout_jobs(pred, jobs, chunk=3)
            got, hs = e3.rollout_jobs_h(pred, jobs, chunk=3)
            for j, a, b_, h in zip(jobs, ref, got, hs):
                for f in ("v", "fire_prob", "cold_hours", "deaths", "settlements_alive", "fissions_total"):
                    np.testing.assert_array_equal(getattr(a, f), getattr(b_, f), err_msg=f)
                self.assertEqual(h.shape, (j.horizon,))
                self.assertTrue(((h > 0) & (h < 1)).all())
        # The first hazard is the head's output at the start row's position of the warm-up window.
        from wm3 import audit as au
        prep = d.prepare(eps[1], VARIANT, norm)
        x, valid = au.window_input(prep, 50, 24)
        with m.torch.no_grad():
            out = net.run_h(m.torch.tensor(x[None], dtype=m.torch.float32), valid=m.torch.tensor(valid[None]))
        self.assertAlmostEqual(float(m.torch.sigmoid(out[3][0, -1])), float(hs[1][0]), places=6)


class V2BaselineTests(unittest.TestCase):
    """GOAL.md 8 V2: the 7-day validation averages of WM-2's baselines (both replicates) are an5_arm 0.489,
    arm_mean 0.499, zero 0.647, so an5_arm is V2's baseline."""

    def test_seven_day_validation_choice(self) -> None:
        import json
        from wm2 import effects as ef
        base = json.loads(BASE.read_text())
        stds = ef.stds_from(base["stds"])
        val = [ef.case_from(j) for j in base["cases"]["val"]]
        cells = [(arm, o, "7d") for arm in v2.W3_ARMS for o in v2.OUTCOMES if stds[(arm, o, "7d")] is not None]
        self.assertEqual(len(cells), 12)
        sel = ef.select_baseline(val, base["val_predictions"], stds, cells)
        self.assertEqual(sel["chosen"], "an5_arm")
        self.assertEqual({k: round(v, 3) for k, v in sel["val"].items()}, {"an5_arm": 0.489, "arm_mean": 0.499, "zero": 0.647})


def val_family_dirs():
    from pathlib import Path
    from wm2 import effects as ef
    root = Path(c.DEFAULT_ROOT)
    return ef.family_dirs(root, "val")[:2] if (root / "v2" / "v19" / "fam").is_dir() else []


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
@unittest.skipUnless(val_family_dirs(), "dataset v2 families not on this machine")
class PredictionsEqualTests(unittest.TestCase):
    def test_predictions_reproduce_model_predictions_on_two_val_families(self) -> None:
        import json
        from pathlib import Path
        from wm1 import m1_tf
        from wm2 import effects as ef
        from wm3 import effects3 as e3
        dirs = val_family_dirs()
        base = json.loads(BASE.read_text())
        seeds = {int(p.name.rsplit("_s", 1)[1]) for p in dirs}
        cases = [ef.case_from(j) for j in base["cases"]["val"] if j["seed"] in seeds]
        norm = d.Normalizer.from_json(json.loads(WM2_CONFIG.read_text())["normalizer"])
        lay = d.Layout(VARIANT)
        m.set_seed(0, 4)
        pred = m1_tf.TFPredictor(m1_tf.M1TF(lay.n_input, lay.n_cont, 16, 1, 2, context=24), lay, norm, VARIANT, "cpu",
                                 24, context=24, nonneg=True)
        std = np.ones(8)
        orig = ef.family_dirs
        ef.family_dirs = lambda root, role: dirs
        try:
            want, wdrift = ef.model_predictions(pred, Path(c.DEFAULT_ROOT), cases, std)
        finally:
            ef.family_dirs = orig
        fams = e3.load_families(dirs, "val")
        got, gdrift = e3.predictions(pred, fams, cases, None, std)
        self.assertEqual(sorted(want), sorted(got))
        self.assertGreater(len(want), 0)
        for key in want:
            self.assertEqual(want[key], got[key], key)
        self.assertEqual(wdrift, gdrift)
        # The pooled 720-row path (val_eff) agrees with the per-family one within float noise.
        a, _ = e3.predictions(pred, fams, cases, 720)
        bb, _ = e3.predictions(pred, fams, cases, 720, pooled=512)
        for key in a:
            for o in v2.OUTCOMES:
                for lag, v in a[key]["effects"][o].items():
                    self.assertAlmostEqual(v, bb[key]["effects"][o][lag], places=3)
                    self.assertAlmostEqual(v, got[key]["effects"][o][lag], places=3)


if __name__ == "__main__":
    unittest.main()
