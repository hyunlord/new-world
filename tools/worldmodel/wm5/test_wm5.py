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


if __name__ == "__main__":
    unittest.main()
