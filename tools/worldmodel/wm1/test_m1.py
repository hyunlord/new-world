"""M1 GRU tests on the sample episodes (skip without torch; the DGX gate runs them with torch).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest

import numpy as np

import wm1  # noqa: F401
from wm1 import data as d
from wm1 import evaluate as ev
from wm1 import m1_gru as m
from wm1.test_data import sample_episode

HAVE_TORCH = m.torch is not None
if not HAVE_TORCH:
    print("wm1.test_m1: torch is not installed; M1 tests skip (they run in the DGX training environment)")


@unittest.skipUnless(HAVE_TORCH, "torch is not installed (the DGX gate runs these)")
class M1Tests(unittest.TestCase):
    def setUp(self) -> None:
        m.set_seed(0, 2)
        self.ep = sample_episode(0)
        self.variant = d.MAIN
        self.norm = d.Normalizer.fit([self.ep], self.variant)
        self.lay = d.Layout(self.variant)
        self.model = m.M1GRU(self.lay.n_input, self.lay.n_cont, 16, 2)
        self.stepper = m.Stepper(self.lay, self.norm, "cpu")

    def test_one_training_step_runs_and_lowers_nothing_to_nan(self) -> None:
        prep = d.prepare(self.ep, self.variant, self.norm)
        rng = np.random.default_rng(0)
        bt = m.batch_tensors(d.sample_windows([prep], 48, 4, rng), "cpu")
        opt = m.torch.optim.AdamW(self.model.parameters(), lr=1e-3)
        tf, parts, roll = m.losses(self.model, self.stepper, bt, 6, {"roll_starts": 3, "roll_min_context": 40, "tf_l1": 1.0}, rng)
        (tf + roll).backward()
        opt.step()
        self.assertTrue(np.isfinite(float(tf)) and np.isfinite(float(roll)))
        self.assertEqual(set(parts), {"nll", "bce", "roll"})
        tf5, _, roll5 = m.losses(self.model, self.stepper, bt, 6, {"roll_starts": 3, "tf_l1": 1.0, "v_weight": 5.0}, rng)
        self.assertTrue(np.isfinite(float(tf5.detach())) and np.isfinite(float(roll5.detach())))
        w = self.stepper.feature_weights(5.0).numpy()
        self.assertAlmostEqual(float(w.mean()), 1.0, places=5)
        v_cont = [list(self.lay.cont).index(i) for i in self.lay.v if i != self.lay.fire]
        self.assertAlmostEqual(float(w[v_cont[0]] / w.min()), 5.0, places=4)
        self.assertEqual(len(v_cont), 7)
        np.testing.assert_array_equal(self.stepper.feature_weights(1.0).numpy(), np.ones(self.lay.n_cont))
        # Rollout starts honour the minimum context when it fits, and always leave room for k steps.
        pos = m.roll_positions(48, 6, {"roll_starts": 50, "roll_min_context": 40}, rng)
        self.assertTrue((pos >= 40).all() and (pos <= 42).all())
        pos = m.roll_positions(48, 30, {"roll_starts": 50, "roll_min_context": 40}, rng)
        self.assertTrue((pos == 18).all())

    def test_layerwise_states_continue_the_sequence(self) -> None:
        """The per-position states of `run` resume a sequence exactly as one pass would."""
        x = m.torch.randn(2, 10, self.lay.n_input)
        mu_full, _, _, outs = self.model.run(x)
        h = outs[:, :, 5]
        mu_rest, _, _, _ = self.model.run(x[:, 6:], h)
        np.testing.assert_allclose(mu_rest.detach().numpy(), mu_full[:, 6:].detach().numpy(), atol=1e-5)

    def test_predictor_rollout_shapes_and_no_future_reads(self) -> None:
        pred = m.GRUPredictor(self.model, self.lay, self.norm, self.variant, "cpu", warmup=24)
        starts = np.array([3, 30, 100])
        ro = pred.rollout(self.ep, starts, 12, self.ep.bench.actions)
        self.assertEqual(ro.v.shape, (3, 12, 8))
        self.assertEqual(ro.fire_prob.shape, (3, 12))
        self.assertEqual(ro.cold_hours.shape, (3, 12))
        self.assertTrue(set(np.unique(ro.v[:, :, 2])) <= {0.0, 1.0})
        bad = sample_episode(0)
        bad.table[101:] = 1e3
        bad.bench.v[101:] = 1e3
        ro_bad = pred.rollout(bad, np.array([100]), 12, bad.bench.actions)
        np.testing.assert_allclose(ro_bad.v, ro.v[2:], rtol=1e-5, atol=1e-4)
        # A start's result does not depend on the other starts in the batch (masked warm-up).
        alone = pred.rollout(self.ep, np.array([3]), 12, self.ep.bench.actions)
        np.testing.assert_allclose(alone.v, ro.v[:1], rtol=1e-5, atol=1e-4)
        # The harness accepts it.
        out = ev.t2(pred, [self.ep], np.ones(8), 24)
        self.assertIn("mean", out)

    def test_actions_change_the_rollout_only_from_their_row(self) -> None:
        pred = m.GRUPredictor(self.model, self.lay, self.norm, self.variant, "cpu", warmup=24)
        acts = np.zeros_like(self.ep.bench.actions)
        acts[110, 2] = 80.0
        a = pred.rollout(self.ep, np.array([100]), 20, np.zeros_like(acts)).v
        bb = pred.rollout(self.ep, np.array([100]), 20, acts).v
        np.testing.assert_allclose(a[0, :10], bb[0, :10], atol=1e-6)   # rows 101..110 unaffected
        self.assertGreater(np.abs(a[0, 10:] - bb[0, 10:]).max(), 0.0)  # row 111 on sees the gift



@unittest.skipUnless(HAVE_TORCH, "torch is not installed (the DGX gate runs these)")
class M1bTests(unittest.TestCase):
    def setUp(self) -> None:
        from wm1 import m1_tf
        self.tf = m1_tf
        m.set_seed(0, 2)
        self.ep = sample_episode(0)
        self.norm = d.Normalizer.fit([self.ep], d.MAIN)
        self.lay = d.Layout(d.MAIN)
        self.model = m1_tf.M1TF(self.lay.n_input, self.lay.n_cont, 32, 2, 4, context=24)

    def test_cached_decoding_equals_a_full_pass(self) -> None:
        x = m.torch.randn(3, 14, self.lay.n_input)
        full_mu, _, _, _ = self.model.run(x)
        mu, _, _, kv = self.model.run(x[:, :10])
        ones = m.torch.ones(3, 1, dtype=m.torch.bool)
        cache = self.tf.append(None, kv, m.torch.ones(3, 10, dtype=m.torch.bool))
        for i in range(10, 14):
            mi, _, _, new = self.model.run(x[:, i:i + 1], cache, ones)
            cache = self.tf.append(cache, new, ones)
            np.testing.assert_allclose(mi[:, 0].detach().numpy(), full_mu[:, i].detach().numpy(), atol=1e-5)
        # Invalid (padding) keys are ignored: left padding does not change the last position.
        pad = m.torch.cat([m.torch.randn(3, 4, self.lay.n_input), x[:, :10]], 1)
        valid = m.torch.cat([m.torch.zeros(3, 4, dtype=m.torch.bool), m.torch.ones(3, 10, dtype=m.torch.bool)], 1)
        mp, _, _, _ = self.model.run(pad, valid=valid)
        np.testing.assert_allclose(mp[:, -1].detach().numpy(), mu[:, -1].detach().numpy(), atol=1e-5)

    def test_training_step_and_predictor(self) -> None:
        prep = d.prepare(self.ep, d.MAIN, self.norm)
        rng = np.random.default_rng(0)
        bt = m.batch_tensors(d.sample_windows([prep], 30, 3, rng), "cpu")
        stepper = m.Stepper(self.lay, self.norm, "cpu")
        tf, parts, roll = self.tf.losses_tf(self.model, stepper, bt, 5, {"roll_starts": 2, "tf_l1": 1.0}, rng)
        (tf + roll).backward()
        self.assertTrue(np.isfinite(float(tf.detach())) and np.isfinite(float(roll.detach())))
        pred = self.tf.TFPredictor(self.model, self.lay, self.norm, d.MAIN, "cpu", 24, context=24)
        ro = pred.rollout(self.ep, np.array([5, 60]), 30, self.ep.bench.actions)
        self.assertEqual(ro.v.shape, (2, 30, 8))
        bad = sample_episode(0)
        bad.table[61:] = 1e3
        np.testing.assert_allclose(pred.rollout(bad, np.array([60]), 30, bad.bench.actions).v, ro.v[1:], rtol=1e-5, atol=1e-4)
        alone = pred.rollout(self.ep, np.array([5]), 30, self.ep.bench.actions)
        np.testing.assert_allclose(alone.v, ro.v[:1], rtol=1e-5, atol=1e-4)


if __name__ == "__main__":
    unittest.main()
