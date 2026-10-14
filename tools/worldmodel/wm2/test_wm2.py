"""WM-2 harness tests (GOAL.md 4.3, 7): input lists and hidden-field absence, the v2 registry, `burn_in` (window =
WM-1.3 bit for bit, full = the teacher-forced state from row 0), the validation-start subsample, the batched rollout
extension against the predictors' own rollouts, the split event and restricted split day, the noise-floor estimator,
the effect evaluation and its baselines, the split-timing baselines and the W verdict rule.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import unittest

import numpy as np

import wm2  # noqa: F401
from wm_data import Episode
from wm1 import common as c
from wm1 import data as d
from wm1 import m1_gru as m
from wm1.test_pilot_harness import GLOBAL, SENTINEL, SETTLE, STATE_OBS
from wm1.test_w13 import synth_v5
from wm2 import v2

HAVE_TORCH = m.torch is not None
V19_STATE = STATE_OBS + ("g.settlements_alive", "g.fissions_total")
W13_V17 = c.REPO / "docs/worldmodel/wm1/runs/m1tf_w13v17_n400_s0/config.json"


def synth_v6(seed: int, rows: int = 1200, settlements: int = 2, fissions_at: tuple | None = None) -> Episode:
    """The schema-5 synthetic episode with schema 6's three settlement columns appended (`fission_pressure`,
    `fission_threshold` hidden, `depart_tick` observable but never an input; all SENTINEL so a leak shows);
    `fissions_at` (a tuple, possibly empty) sets `fissions_total` to +1 steps at those rows instead of the random ones."""
    ep = synth_v5(seed, rows, settlements)
    g = ep.arrays["global.npy"].copy()
    if fissions_at is not None:
        f = np.zeros(rows, np.float32)
        for r in fissions_at:
            f[r:] += 1
        g[:, len(GLOBAL) + 1] = f
    s = np.concatenate([ep.arrays["settlements.npy"], np.full((rows, settlements, 3), SENTINEL, np.float32)], axis=2)
    meta = json.loads(json.dumps(ep.meta))
    meta["schema_version"] = 6
    meta["fields"]["settlements.npy"] += [{"name": "fission_pressure", "hidden": True},
                                          {"name": "fission_threshold", "hidden": True},
                                          {"name": "depart_tick", "hidden": False}]
    meta["observable_fields"]["settlements.npy"] += ["depart_tick"]
    return Episode(ep.path, meta, {"global.npy": g, "settlements.npy": s}, [])


class InputListTests(unittest.TestCase):
    """GOAL.md 4.3: game_v19 inputs = WM-1.3's game_v17 list exactly (67), + the 7 action columns (74)."""

    def test_v2_input_lists(self) -> None:
        blind = m.dataset_variant(c.dataset("v2_v19"), "sev-blind_action-blind")
        aware = m.dataset_variant(c.dataset("v2_v19"), "sev-blind_action-aware")
        self.assertEqual(blind.input_names(), V19_STATE + d.CALENDAR)
        self.assertEqual(len(blind.input_names()), 67)
        self.assertEqual(aware.input_names(), V19_STATE + d.CALENDAR + d.ACTION_NAMES)
        self.assertEqual(len(aware.input_names()), 74)
        self.assertEqual(aware, m.dataset_variant(c.dataset("v2_v19_fact"), "sev-blind_action-aware"))
        if W13_V17.exists():
            self.assertEqual(list(blind.input_names()), json.loads(W13_V17.read_text())["input_names"])
        v16 = m.dataset_variant(c.dataset("v2_v16"), "sev-blind_action-blind")
        self.assertEqual(v16.input_names(), STATE_OBS + d.CALENDAR)
        for v in (blind, aware, v16):
            for n in v.input_names():
                for bad in v2.HIDDEN_FIELDS + d.HIDDEN_V2 + d.NOT_INPUTS_V5 + ("depart_tick", "slot", "formation"):
                    self.assertNotIn(bad, n)

    def test_schema_6_table_has_no_hidden_or_settlement_slot_field(self) -> None:
        raw = synth_v6(4)
        ep = d.from_episode(raw, "v19_fact", "train", 4, "none", c.V17_EXTRA)
        self.assertEqual(ep.names, V19_STATE)
        self.assertFalse(np.any(ep.table == SENTINEL))
        self.assertFalse(np.any(ep.bench.v == SENTINEL))
        np.testing.assert_array_equal(ep.table[:, :37], d.from_episode(synth_v5(4), "x", "train", 4, "none",
                                                                       c.V17_EXTRA).table)
        self.assertEqual(len(SETTLE) + 5, raw.arrays["settlements.npy"].shape[2])


class RegistryTests(unittest.TestCase):
    def test_v2_groups(self) -> None:
        ds = c.dataset("v2_v19")
        self.assertEqual(ds.manifest, c.V2_MANIFEST)
        self.assertEqual(ds.groups["train"], (c.V19_ALL, "train"))
        self.assertEqual(ds.groups["val"], ((c.V19_FACT, c.V19_ARM_R0), "val"))
        self.assertEqual(ds.groups["test"], (c.V19_ALL, "test"))
        self.assertEqual((ds.train_groups(True), list(ds.val), ds.extra_state), (["train"], ["val"], c.V17_EXTRA))
        fact = c.dataset("v2_v19_fact")
        self.assertEqual({k: v for k, v in fact.groups.items()},
                         {"train": ("v19_fact", "train"), "val": ("v19_fact", "val"), "test": ("v19_fact", "test")})
        self.assertEqual(c.dataset("v2_v16").groups["test"], ("v16", "test"))
        self.assertEqual(c.dataset("v2_v16").extra_state, ())
        self.assertEqual(c.split_names("a"), ("a",))
        self.assertEqual(c.split_names(("a", "b")), ("a", "b"))

    @unittest.skipUnless(c.V2_MANIFEST.exists(), "v2_manifest.json not written yet")
    def test_manifest_roles_splits_and_casts(self) -> None:
        raw = json.loads(c.V2_MANIFEST.read_text())
        seen = set()
        for e in raw["episodes"]:
            self.assertEqual(e["role"], v2.role_of(e["scene"], e["seed"]))
            key = (e["scene"], e["seed"], e["schedule_id"], e["replicate"])
            self.assertNotIn(key, seen)
            seen.add(key)
            self.assertEqual(e["rows"], 2881)
            for cast in e["casts"]:
                self.assertEqual(cast["tick"] % 60, 0)
            if e["scene"] == "game_v19":
                self.assertEqual(e["fork_tick"], v2.FORK_TICK)
                want = ("v19_fact" if e["replicate"] == 0 else "v19_none_r1") if e["arm"] == "none" else \
                    ("v19_arm_r0" if e["replicate"] == 0 else "v19_arm_r1")
                self.assertEqual(e["split"], want)
                self.assertEqual(len(e["casts"]), 0 if e["arm"] == "none" else 1)
            else:
                self.assertEqual((e["split"], e["arm"], e["fork_tick"]), ("v16", "none", None))
        fact = [e for e in raw["episodes"] if e["split"] == "v19_fact"]
        self.assertEqual(sorted(e["seed"] for e in fact), list(range(40000, 40560)))
        self.assertEqual(sorted(e["seed"] for e in raw["episodes"] if e["split"] == "v16"), list(range(42000, 42560)))


class SplitEventTests(unittest.TestCase):
    def test_split_event_and_restricted_day(self) -> None:
        f = np.zeros(2881)
        f[1000:] = 1
        f[2000:] = 2
        self.assertEqual(v2.split_event_row(f, 500), 1000)
        self.assertEqual(v2.split_event_row(f, 1000), 2000)       # the reference row's level is the base
        self.assertIsNone(v2.split_event_row(f, 2000))
        self.assertEqual(v2.restricted_split_day(f, 500), (int(v2.day_of_row(1000)), False))
        self.assertEqual(v2.restricted_split_day(f, 2000), (v2.LAST_DAY, True))
        p = np.zeros(2881)
        p[1500:] = 0.49
        self.assertEqual(v2.restricted_split_day(p, 500), (v2.LAST_DAY, True))   # below the 0.5 threshold
        p[1600:] = 0.5
        self.assertEqual(v2.split_event_row(p, 500), 1600)
        self.assertEqual(v2.LAST_DAY, 121)                       # row 2,880 = 08:00 of display day 121
        self.assertEqual(int(v2.day_of_row(v2.row_of_tick(v2.SPLIT_START_TICK))), 25)
        self.assertEqual([int(v2.day_of_row(v2.row_of_tick(s))) for s in v2.FLOOR_STARTS], [15, 40, 70])
        with self.assertRaises(ValueError):
            v2.row_of_tick(14281)


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class BurnInTests(unittest.TestCase):
    def setUp(self) -> None:
        m.set_seed(0, 2)
        self.variant = m.dataset_variant(c.dataset("v2_v19"), "sev-blind_action-aware")
        self.eps = [d.from_episode(synth_v6(s, 400), "v19_fact", "train", s, "none", c.V17_EXTRA) for s in (1, 2)]
        self.norm = d.Normalizer.fit(self.eps, self.variant)
        self.lay = d.Layout(self.variant)
        self.preps = [d.prepare(ep, self.variant, self.norm) for ep in self.eps]
        self.model = m.M1GRU(self.lay.n_input, self.lay.n_cont, 16, 2)

    def test_window_batch_is_wm13s_batch_bit_for_bit(self) -> None:
        cfg = dict(m.DEFAULTS, window=48, batch=5)
        self.assertEqual(cfg["burn_in"], "window")
        a = m.make_batch(self.preps, cfg, np.random.default_rng(7), "cpu", self.model)
        b = m.batch_tensors(d.sample_windows(self.preps, 48, 5, np.random.default_rng(7)), "cpu")
        self.assertEqual(set(a), set(b))
        for k in a:
            self.assertTrue(m.torch.equal(a[k], b[k]), k)
        win, which, starts = d.sample_windows(self.preps, 48, 5, np.random.default_rng(7), return_starts=True)
        for k in win:
            np.testing.assert_array_equal(win[k], b[k].numpy())
        self.assertEqual(len(which), 5)
        # A seeded training step is unchanged by the new code path (loss and gradients equal).
        rng1, rng2 = np.random.default_rng(3), np.random.default_rng(3)
        cfg2 = {"roll_starts": 1, "roll_min_context": 20, "tf_l1": 1.0, "v_weight": 5.0}
        st = m.Stepper(self.lay, self.norm, "cpu", True)
        tf1, _, r1 = m.losses(self.model, st, a, 6, cfg2, rng1)
        bb = dict(b)
        tf2, _, r2 = m.losses(self.model, st, bb, 6, cfg2, rng2)
        self.assertEqual(float(tf1.detach()), float(tf2.detach()))
        self.assertEqual(float(r1.detach()), float(r2.detach()))

    def test_full_burn_in_states_equal_the_sequential_teacher_forced_states(self) -> None:
        cfg = dict(m.DEFAULTS, window=48, batch=6, burn_in="full")
        bt = m.make_batch(self.preps, cfg, np.random.default_rng(11), "cpu", self.model)
        win, which, starts = d.sample_windows(self.preps, 48, 6, np.random.default_rng(11), return_starts=True)
        self.assertIn("h0", bt)
        for i, (w, s) in enumerate(zip(which, starts)):
            p = self.preps[w]
            if s == 0:
                self.assertTrue(m.torch.equal(bt["h0"][:, i], m.torch.zeros_like(bt["h0"][:, i])))
                continue
            x = m.torch.tensor(d.model_input(p.x_state[:s], p.calendar[:s], p.actions[:s], p.sev[:s]),
                               dtype=m.torch.float32)[None]
            _, _, _, outs = self.model.run(x)
            np.testing.assert_allclose(bt["h0"][:, i].numpy(), outs[:, 0, -1].detach().numpy(), atol=1e-5)
        # The window then continues the teacher-forced sequence: equal to one pass over rows 0 .. start + L - 1.
        i = int(np.argmax(starts))
        p, s = self.preps[which[i]], int(starts[i])
        L = bt["dtarget"].shape[1]
        x = m.torch.tensor(d.model_input(p.x_state[: s + L], p.calendar[: s + L], p.actions[: s + L], p.sev[: s + L]),
                           dtype=m.torch.float32)[None]
        mu_full, _, _, _ = self.model.run(x)
        inp = m.torch.cat([bt["x_state"], bt["calendar"], bt["actions"], bt["sev"]], -1)[:, :L]
        mu_w, _, _, _ = self.model.run(inp[i: i + 1], bt["h0"][:, i: i + 1])
        np.testing.assert_allclose(mu_w[0].detach().numpy(), mu_full[0, s:].detach().numpy(), atol=1e-4)
        tf, _, roll = m.losses(self.model, m.Stepper(self.lay, self.norm, "cpu", True), bt, 5,
                               {"roll_starts": 1, "roll_min_context": 20, "tf_l1": 1.0}, np.random.default_rng(0))
        (tf + roll).backward()
        self.assertTrue(np.isfinite(float(tf.detach())))

    def test_full_predictor_is_the_warm_up_from_row_0(self) -> None:
        ep = self.eps[0]
        full = m.GRUPredictor(self.model, self.lay, self.norm, self.variant, "cpu", burn_in="full")
        starts = np.array([30, 200, 350])
        a = full.rollout(ep, starts, 20, ep.bench.actions)
        for i, s in enumerate(starts):
            win = m.GRUPredictor(self.model, self.lay, self.norm, self.variant, "cpu", warmup=int(s) + 1)
            b = win.rollout(ep, np.array([s]), 20, ep.bench.actions)
            np.testing.assert_allclose(a.v[i], b.v[0], rtol=1e-5, atol=1e-4)
        # No future reads: rows after the start do not change the rollout.
        bad = d.from_episode(synth_v6(1, 400), "v19_fact", "train", 1, "none", c.V17_EXTRA)
        bad.table[201:] = 1e3
        np.testing.assert_allclose(full.rollout(bad, np.array([200]), 20, bad.bench.actions).v, a.v[1:2],
                                   rtol=1e-5, atol=1e-4)
        with self.assertRaises(ValueError):
            m.GRUPredictor(self.model, self.lay, self.norm, self.variant, "cpu", burn_in="nope")


class ValSubsampleTests(unittest.TestCase):
    def test_fixed_subsample_of_validation_starts(self) -> None:
        from wm1 import evaluate as ev
        eps = [d.from_episode(synth_v6(s, 900), "v19_fact", "val", s, "none", c.V17_EXTRA) for s in (1, 2, 3)]
        val = {"val": eps}
        self.assertIsNone(m.val_starts(val, 1.0))
        a, b = m.val_starts(val, 0.5), m.val_starts(val, 0.5)
        for ep in eps:
            key = ("val", ep.seed, "none")
            n = len(ev.bench_starts(ep, 24))
            self.assertEqual(len(a[key]), int(np.ceil(0.5 * n)))
            np.testing.assert_array_equal(a[key], b[key])
            self.assertTrue(set(a[key]) <= set(ev.bench_starts(ep, 24)))
        # Every start kept: the subsample path equals the pooled score.
        full = {k: ev.bench_starts(e, 24) for k, e in (((("val", e.seed, "none")), e) for e in eps)}
        stds = c.split_stds(val)
        p = ev.Persistence()
        self.assertAlmostEqual(m.val_score(p, val, stds, full)[0], m.val_score(p, val, stds)[0], places=12)
        with self.assertRaises(ValueError):
            m.val_starts({"val": eps + eps[:1]}, 0.5)


@unittest.skipUnless(HAVE_TORCH, "torch is not installed")
class RolloutExtTests(unittest.TestCase):
    def setUp(self) -> None:
        from wm1 import m1_tf
        from wm2 import rollout_ext as rx
        self.rx, self.tf = rx, m1_tf
        m.set_seed(0, 2)
        self.variant = m.dataset_variant(c.dataset("v2_v19"), "sev-blind_action-aware")
        self.eps = [d.from_episode(synth_v6(s, 300), "v19_fact", "test", s, "none", c.V17_EXTRA) for s in (1, 2)]
        self.norm = d.Normalizer.fit([d.from_episode(synth_v6(9, 300), "v19_fact", "train", 9, "none", c.V17_EXTRA)],
                                     self.variant)
        self.lay = d.Layout(self.variant)

    def preds(self):
        gru = m.M1GRU(self.lay.n_input, self.lay.n_cont, 16, 2)
        tfm = self.tf.M1TF(self.lay.n_input, self.lay.n_cont, 32, 2, 4, context=24)
        return {"gru_window": m.GRUPredictor(gru, self.lay, self.norm, self.variant, "cpu", warmup=24, nonneg=True),
                "gru_full": m.GRUPredictor(gru, self.lay, self.norm, self.variant, "cpu", burn_in="full"),
                "tf": self.tf.TFPredictor(tfm, self.lay, self.norm, self.variant, "cpu", 24, context=24, nonneg=True)}

    def test_jobs_reproduce_the_predictor_rollouts_and_add_the_two_columns(self) -> None:
        acts = np.zeros_like(self.eps[0].bench.actions)
        acts[120, 1] = 240.0
        for name, pred in self.preds().items():
            for rc in (False, True):
                pred.round_counts = rc
                jobs = [self.rx.Job(self.eps[0], 120, acts, 40), self.rx.Job(self.eps[1], 50, self.eps[1].bench.actions, 90),
                        self.rx.Job(self.eps[0], 120, np.zeros_like(acts), 40), self.rx.Job(self.eps[1], 280, acts, 19)]
                got = self.rx.rollout_jobs(pred, jobs, chunk=3)
                for j, ro in zip(jobs, got):
                    ref = pred.rollout(j.ep, np.array([j.start]), j.horizon, j.actions)
                    for f in ("v", "fire_prob", "cold_hours", "deaths"):
                        np.testing.assert_allclose(getattr(ro, f), getattr(ref, f), rtol=1e-5, atol=1e-4,
                                                   err_msg=f"{name} {f} rc={rc}")
                    self.assertEqual(ro.settlements_alive.shape, (1, j.horizon))
                    self.assertEqual(ro.fissions_total.shape, (1, j.horizon))
                # The action at the start row changes the arm branch (unrounded outputs).
                if not rc:
                    self.assertGreater(np.abs(got[0].v - got[2].v).max(), 0)
                # The extras are the stepped state: one step of the rollout continues the start row's level.
                ft0 = self.eps[1].table[50, self.lay.names.index("g.fissions_total")]
                self.assertLess(abs(got[1].fissions_total[0, 0] - ft0), 50.0)


class FloorTests(unittest.TestCase):
    def test_identical_replicates_give_zero(self) -> None:
        from wm2 import floor as fl
        y = np.tile(np.arange(8.0), (4, 1))
        corr, unc = fl.floor_errors(y, np.ones(8))
        self.assertTrue((corr == 0).all() and (unc == 0).all())

    def test_gaussian_noise_gives_sigma_sqrt_2_over_pi(self) -> None:
        from wm2 import floor as fl
        rng = np.random.default_rng(0)
        sigma, std = 3.0, np.array([2.0] * 7 + [0.0])
        values = {(w, s, 24): 50.0 + w + sigma * rng.standard_normal((4, 8)) for w in range(3000) for s in (1,)}
        out = fl.floor_scores(values, std)["24"]
        want = sigma * np.sqrt(2 / np.pi) / 2.0
        self.assertLess(abs(out["floor"] / want - 1), 0.05)
        self.assertGreater(out["floor_uncorrected"], out["floor"])          # x sqrt(4/3) before the correction
        self.assertAlmostEqual(out["floor_uncorrected"] / out["floor"], np.sqrt(4 / 3), places=6)
        self.assertTrue(np.isnan(out["per_var"][7]))                        # zero-std variable skipped
        self.assertEqual(len(out["per_world"]), 3000)

    def test_matched_score_of_the_truth_is_zero(self) -> None:
        from wm1 import evaluate as ev
        from wm2 import floor as fl
        ep = d.from_episode(synth_v6(5, 2881), "v19_floor", "test", 5, "none", c.V17_EXTRA)

        class Truth(ev.Predictor):
            def rollout(self, e, starts, horizon, actions):
                rows = np.asarray(starts)[:, None] + np.arange(1, horizon + 1)[None]
                return ev.Rollout(e.bench.v[rows])
        s = v2.FLOOR_STARTS[0]
        row = v2.row_of_tick(s)
        values = {(5, s, h): np.stack([ep.bench.v[row + h]] * 4) for h in (24, 168)}
        out = fl.matched_scores(Truth(), {(5, s): ep}, values, np.ones(8))
        self.assertEqual(out["24"]["score"], 0.0)
        self.assertEqual(out["168"]["score"], 0.0)
        values[(5, s, 24)][0, 0] += 1          # replicate 0 must be the rolled episode
        with self.assertRaises(ValueError):
            fl.matched_scores(Truth(), {(5, s): ep}, values, np.ones(8))


class FakeFamily:
    def __init__(self, seed, factual, variants):
        self.seed, self.factual, self.variants = seed, factual, variants


def fam_member(seed: int, rep: int, arm: str | None, cast_row: int = 600, base_fission: int | None = 2000,
               arm_fission: int | None = 1500) -> Episode:
    """A schema-6 synthetic member of length 2,881: the world's rows (seed), replicate noise after row 238, and for an
    arm +40 food in the home store and a split at `arm_fission` from the cast row on (the factual splits at
    `base_fission`; None = never)."""
    ep = synth_v6(seed, 2881, settlements=2, fissions_at=() if base_fission is None else (base_fission,))
    g, s = ep.arrays["global.npy"].copy(), ep.arrays["settlements.npy"].copy()
    rng = np.random.default_rng(100 * seed + rep)
    s[239:, 0, SETTLE.index("food_stock")] += rng.normal(0, 1, 2881 - 239).astype(np.float32)
    acts = []
    if arm is not None:
        s[cast_row + 1:, 0, SETTLE.index("food_stock")] += 40.0
        f = np.zeros(2881, np.float32)
        if arm_fission is not None:
            f[arm_fission:] = 1
        g[:, len(GLOBAL) + 1] = np.where(np.arange(2881) <= cast_row, g[:, len(GLOBAL) + 1], f)
        kind, amount = v2.ARM_SPEC[arm]
        acts = [{"tick": cast_row * 60, "kind": kind, "amount": amount, "applied": amount, "settlement": 1}]
    return Episode(ep.path, ep.meta, {"global.npy": g, "settlements.npy": s}, acts)


class EffectTests(unittest.TestCase):
    def family(self, seed, cast_row=600, base_fission=2000, arm_fission=1500):
        fac = {r: fam_member(seed, r, None, base_fission=base_fission) for r in (0, 1)}
        var = {"gf240E": {r: fam_member(seed, r, "gf240E", cast_row, base_fission, arm_fission) for r in (0, 1)}}
        return FakeFamily(seed, fac, var)

    def test_true_effects_alignment_and_censoring(self) -> None:
        from wm2 import effects as ef
        cases = ef.family_cases(self.family(7), "test")
        self.assertEqual(len(cases), 2)
        x = cases[0]
        self.assertEqual((x.cast_row, x.cast_tick), (600, 36000))
        self.assertAlmostEqual(x.truth["food_stock"]["7d"], 40.0, places=3)
        self.assertAlmostEqual(x.truth["food_stock"]["30d"], 40.0, places=3)
        self.assertEqual(x.truth["alive"]["30d"], 0.0)
        self.assertEqual(x.shift, float(v2.day_of_row(1500) - v2.day_of_row(2000)))
        self.assertEqual(x.window.shape, (24, 8))
        # Censored on both sides: no split after the cast in either branch -> shift 0.
        both = ef.family_cases(self.family(8, base_fission=None, arm_fission=None), "test")[0]
        self.assertEqual((both.shift, both.censored), (0.0, (True, True)))
        one = ef.family_cases(self.family(8, base_fission=None, arm_fission=1500), "test")[0]
        self.assertEqual((one.shift, one.censored), (float(v2.day_of_row(1500) - v2.LAST_DAY), (False, True)))
        late = ef.family_cases(self.family(9, cast_row=2300), "test")[0]
        self.assertIn("7d", late.truth["food_stock"])
        self.assertNotIn("30d", late.truth["food_stock"])          # c + 30 days past the last row

    def cases(self):
        from wm2 import effects as ef
        out = []
        for seed in range(20):
            out += ef.family_cases(self.family(seed, cast_row=500 + 10 * seed), "train" if seed < 14 else "test")
        return [x for x in out if x.role == "train"], [x for x in out if x.role == "test"]

    def test_truth_predictor_scores_zero_and_zero_predictor_scores_mean_abs_over_std(self) -> None:
        from wm2 import effects as ef
        train, test = self.cases()
        for x in train[::3]:       # make the train effects vary so every std > 0
            x.truth["alive"] = {k: 1.0 for k in x.truth["alive"]}
            x.truth["deaths"] = {k: 2.0 for k in x.truth["deaths"]}
            x.truth["settlements_alive"] = {k: 1.0 for k in x.truth["settlements_alive"]}
            x.truth["food_stock"] = {k: v + 3 for k, v in x.truth["food_stock"].items()}
        stds = ef.scale_stds(train)
        truth = {x.key: {"effects": x.truth, "shift": x.shift} for x in test}
        errs = ef.cell_errors(test, truth, stds)
        worlds = sorted({x.seed for x in test})
        gf = [cell for cell in ef.w3_cells(stds) if cell[0] == "gf240E"]
        self.assertEqual(len(gf), 4)
        self.assertEqual(ef.cell_stat(errs, gf, worlds), 0.0)
        self.assertEqual(ef.cell_stat(errs, [("gf240E", "shift")], worlds), 0.0)
        zero = {x.key: ef.predict_zero(x) for x in test}
        ez = ef.cell_errors(test, zero, stds)
        cell = ("gf240E", "food_stock", "30d")
        want = np.mean([abs(x.truth["food_stock"]["30d"]) for x in test]) / stds[cell]
        self.assertAlmostEqual(ef.cell_summary(ez[cell])["mae"], want, places=9)
        self.assertAlmostEqual(ef.cell_summary(ez[("gf240E", "shift")])["mae"], np.mean([abs(x.shift) for x in test]))
        # arm_mean: the train mean at each lag.
        means = ef.arm_mean(train)
        self.assertAlmostEqual(means["gf240E"]["effects"]["food_stock"]["30d"],
                               np.mean([x.truth["food_stock"]["30d"] for x in train]))

    def test_an5_arm_takes_one_case_per_world(self) -> None:
        from wm2 import effects as ef
        train, test = self.cases()
        q = test[0]
        near = ef.nearest_worlds(q, train, np.ones(8))
        self.assertEqual(len(near), 5)
        self.assertEqual(len({x.seed for x in near}), 5)
        cand = [ef.Case(1, "gf240E", r, "train", 0, 0, {}, 0.0, (False, False), q.window + r) for r in (0, 1)]
        cand += [ef.Case(2, "gf240E", 0, "train", 0, 0, {}, 0.0, (False, False), q.window + 5)]
        self.assertEqual([(x.seed, x.rep) for x in ef.nearest_worlds(q, cand, np.ones(8), k=5)], [(1, 0), (2, 0)])

    def test_rule_and_paired_bootstrap(self) -> None:
        from wm2 import effects as ef
        rows_base = [(w, r, 1.0, 1.0, 0.0) for w in range(30) for r in (0, 1)]
        rows_good = [(w, r, 0.5, 1.0, 0.5) for w in range(30) for r in (0, 1)]
        cells = [("a", "shift")]
        same = ef.compare([{cells[0]: rows_base}], {cells[0]: rows_base}, cells, list(range(30)))
        self.assertEqual((same["verdict"], same["ci_diff"]["lo"], same["ci_diff"]["hi"]), ("TIE", 0.0, 0.0))
        one = ef.compare([{cells[0]: rows_good}], {cells[0]: rows_base}, cells, list(range(30)))
        self.assertEqual(one["verdict"], "WEAK")                   # one seed: WEAK at most
        two = ef.compare([{cells[0]: rows_good}] * 2, {cells[0]: rows_base}, cells, list(range(30)))
        self.assertEqual(two["verdict"], "KEEP")
        worse = ef.compare([{cells[0]: rows_base}] * 2, {cells[0]: rows_good}, cells, list(range(30)))
        self.assertEqual(worse["verdict"], "NONE")
        self.assertEqual(ef.rule([0.9, 0.95], 1.0, {"lo": -0.2, "hi": -0.01}), "WEAK")


class SplitDayTests(unittest.TestCase):
    def test_baselines_on_synthetic_worlds(self) -> None:
        from wm2 import split_task as st
        mk = lambda s, f: d.from_episode(synth_v6(s, 2881, fissions_at=f), "v19_fact", "train", s, "none", c.V17_EXTRA)
        eps = [mk(s, (900 + 20 * s,)) for s in range(12)] + [mk(99, ()), mk(98, (100,))]
        el = st.eligible(eps)
        self.assertNotIn(98, [e.seed for e in el])                       # already split at the start row
        self.assertEqual(st.target(eps[12]), (v2.LAST_DAY, True))
        self.assertEqual(st.target(eps[0])[0], int(v2.day_of_row(900)))
        an5 = st.an5_day(el[:3], el, np.ones(8))
        self.assertEqual(len(an5[el[0].seed][1]), 5)
        ridge, lam, scores = st.ridge_day(el, el)
        self.assertIn(lam, b_lambdas())
        out = st.summarize({1: 10.0, 2: 30.0}, {1: 12.0, 2: 30.0})
        self.assertEqual((out["mae"], out["within_7d"]), (1.0, 1.0))
        r2 = st.loo_r2(np.arange(20.0)[:, None], 2 * np.arange(20.0) + 1)
        self.assertGreater(r2["r2_loo"], 0.95)


def b_lambdas():
    import bench_v0
    return bench_v0.LAMBDAS
