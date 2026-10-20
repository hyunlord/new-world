"""M2 / T5 tests on the sample agent episodes (GOAL.md 7.6, 10).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
from wm_data import load_episode
from wm1 import agents as A
from wm1 import data as d
from wm1 import m2_agents as m

SAMPLE = Path(__file__).resolve().parents[3] / "docs/worldmodel/sample"


def sample(seed: int = 0) -> A.AgentEpisode:
    ep = load_episode(SAMPLE / f"game_v13_s{seed}_none",
                      arrays=["global.npy", "settlements.npy", "agents.npy", "agent_alive.npy", "terrain.npy"])
    return A.from_loaded(ep, "A_obs", "train", seed, "none")


class T5Tests(unittest.TestCase):
    def test_hidden_lifespan_is_never_loaded(self) -> None:
        ep = sample()
        self.assertNotIn("lifespan_days", A.AGENT_FIELDS)
        self.assertEqual(ep.agents.shape[-1], len(A.AGENT_FIELDS))

    def test_scored_rows_and_baselines(self) -> None:
        ep = sample()
        t, n = A.scored_index(ep)
        self.assertTrue((ep.alive[t, n] & ep.alive[t + 1, n] & ep.alive[t - 23, n]).all())
        self.assertTrue((t >= 23).all() and (t <= ep.rows - 2).all())
        p = A.t5(A.persistence, [ep])
        self.assertTrue(0 <= p["state_tag_accuracy"] <= 1)
        truth = A.t5(lambda e, tt, nn: A.truth(e, tt, nn), [ep])
        self.assertEqual(truth["state_tag_accuracy"], 1.0)
        self.assertEqual(truth["position_chebyshev"], 0.0)
        self.assertEqual(truth["needs_nmae_mean"], 0.0)


class M2FeatureTests(unittest.TestCase):
    def test_features_shapes_and_alignment(self) -> None:
        ep = sample()
        norm = d.Normalizer.fit([ep.settlement], d.MAIN)
        X = m.Featurizer(norm).episode(ep)
        self.assertEqual(X.shape[:2], ep.alive.shape)
        self.assertFalse(np.isnan(X).any())
        self.assertTrue((X[~ep.alive][:, :49] == 0).all())   # own and neighbour features are zero when not alive
        st, tk, dx, dy, dn = m.targets(ep)
        t, n = 50, int(np.argmax(ep.alive[50] & ep.alive[51]))
        self.assertEqual(st[t, n], int(ep.col("state_tag")[t + 1, n]))
        self.assertEqual(dx[t, n] - m.DMAX, int(np.clip(ep.col("x")[t + 1, n] - ep.col("x")[t, n], -3, 3)))
        np.testing.assert_allclose(dn[t, n, 0], ep.col("hunger")[t + 1, n] - ep.col("hunger")[t, n], atol=1e-4)

    def test_nearest_neighbours_on_a_line(self) -> None:
        ep = sample()
        T, N = 2, 10
        ep.alive = np.ones((T, N), dtype=bool)
        ep.agents = np.zeros((T, N, len(A.AGENT_FIELDS)))
        ep.agents[..., A.AGENT_FIELDS.index("x")] = np.arange(N)
        own = m.own_features(ep)
        near = m.near_features(ep, own)
        # Agent 0's 8 nearest are agents 1..8: mean distance 4.5 tiles, max 8.
        self.assertAlmostEqual(near[0, 0, 0] * 10, 4.5)
        half = near.shape[-1] // 2
        self.assertAlmostEqual(near[0, 0, half] * 10, 8.0)


if __name__ == "__main__":
    unittest.main()
