"""WM-3 tests (docs/worldmodel/wm3/GOAL.md 6, 7): the audit's checks and the harness's lag-1 response on a stub model.

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


if __name__ == "__main__":
    unittest.main()
