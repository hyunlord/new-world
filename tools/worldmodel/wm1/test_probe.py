"""World identification probe (GOAL_1_2.md 9.3): the probe row, the ridge probe on synthetic arrays, and
`M1TF.hidden` against the model's own heads (torch; skipped without torch).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest

import numpy as np

import wm1  # noqa: F401
from wm1 import probe_world as pw

try:
    import torch
    from wm1 import m1_tf
except ImportError:
    torch = None


class ProbeTests(unittest.TestCase):
    def test_probe_row_is_0600_of_day_8(self) -> None:
        self.assertEqual(pw.probe_row(), 166)

    def test_ridge_probe_recovers_a_linear_parameter_and_a_category(self) -> None:
        rng = np.random.default_rng(0)
        x = {r: rng.normal(size=(n, 6)) for r, n in (("train", 80), ("val", 16), ("test", 16))}
        y = {r: {p: 5 * x[r][:, 0] + rng.normal(size=len(x[r])) * 0.05 for p in pw.PARAMS}
             | {"climate": x[r][:, 1:4].argmax(axis=1).astype(float)} for r in x}
        out = pw.probe(x, y)
        self.assertGreater(out["starting_agents"]["r2_test"], 0.99)
        self.assertGreaterEqual(out["climate"]["accuracy_test"], 0.75)
        self.assertIn(out["climate"]["lambda"], pw.LAMBDAS)
        self.assertIsNone(pw.r2(np.ones(4), np.zeros(4)))

    @unittest.skipIf(torch is None, "torch not installed")
    def test_hidden_is_the_state_the_heads_read(self) -> None:
        torch.manual_seed(0)
        model = m1_tf.M1TF(12, 5, dm=32, layers=2, heads=4)
        model.eval()
        x = torch.randn(3, 20, 12)
        valid = torch.ones(3, 20, dtype=torch.bool)
        valid[:, :2] = False                                    # rows before row 0, as in the probe window
        with torch.no_grad():
            mu, _, fl, _ = model.run(x, valid=valid)
            h = model.hidden(x, valid)
            self.assertEqual(tuple(h.shape), (3, 32))
            torch.testing.assert_close(model.mu(h), mu[:, -1])
            torch.testing.assert_close(model.fire(h).squeeze(-1), fl[:, -1])


if __name__ == "__main__":
    unittest.main()
