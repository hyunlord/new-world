"""dyn-eta2 of diversity_p11: 0 for constant per-seed offsets, large for diverging histories."""
import unittest

import numpy as np

import diversity_p11 as d


class DynEta2Tests(unittest.TestCase):
    def test_offsets_on_a_shared_trajectory_give_zero(self):
        t = np.arange(2881, dtype=np.float64)
        shared = np.sin(t / 50.0)
        v = np.stack([np.stack([shared + 10 * e] * 8, axis=1) for e in range(5)])
        self.assertTrue(np.allclose(d.dyn_eta2(v), 0.0))
        # Plain eta2 sees the offsets.
        self.assertGreater(float(d.cc.eta2(v, 0)[0]), 0.5)

    def test_a_late_divergence_is_seen(self):
        t = np.arange(2881, dtype=np.float64)
        shared = np.sin(t / 50.0)
        v = np.stack([np.stack([shared] * 8, axis=1) for _ in range(5)])
        v[2, 1500:, 0] += 3.0
        dyn = d.dyn_eta2(v)
        self.assertGreater(float(dyn[0]), 0.5)
        self.assertTrue(np.allclose(dyn[1:], 0.0))


if __name__ == "__main__":
    unittest.main()
