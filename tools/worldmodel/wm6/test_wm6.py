"""WM-6 unit tests (docs/worldmodel/wm6/GOAL.md 7.6). Synthetic tests run everywhere (torch tests skip without torch);
the data tests (hidden rows, observable features under a hidden-field perturbation, the WM-5 reproduction) need the
recorded v2 data (`WM_DATA_ROOT`, the DGX) and skip elsewhere.

    python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm6  # noqa: F401
from wm1 import common as c
from wm1 import data as d
from wm2 import effects as ef
from wm2 import v2
from wm5 import effects5 as e5
from wm6 import data6 as d6
from wm6 import effects6 as e6

try:
    import torch  # noqa: F401
    HAVE_TORCH = True
except ImportError:                                   # pragma: no cover
    HAVE_TORCH = False

DATA = Path(os.environ.get("WM_DATA_ROOT", "/home/hyunlord/work/worldsim-wm-data"))
HAVE_V2 = (DATA / "v2" / "v19" / "fam" / "game_v19_s40000").is_dir()
BASE = c.REPO / "docs/worldmodel/wm2/runs/effects_base/effects_base.json"


# ------------------------------------------------------------------ synthetic prep ----------------------------------
def synthetic_prep(tmp: Path, fx, fs=None, n_obs: int = 8, n_seq: int = 74, seed: int = 0, v2x: bool = False):
    """A prep directory with WM-2's v2 roles (train 40000–40399 both replicates, val 40400–40479 both replicates; with
    `v2x` also 50000–51599 replicate 0): every case's scaled truth of every target = fx(obs) (+ fs(seq) when given),
    the stds all 1."""
    rng = np.random.default_rng(seed)
    stds = {f"{a}|{o}|{lag}": 1.0 for a in v2.ARMS for o in v2.OUTCOMES for lag in v2.LAGS}

    def part(seeds, reps, role):
        cases, keys, obs, seq, pos = [], [], [], [], {}
        for s in seeds:
            for r in reps:
                for arm in e6.ARMS:
                    row = 490 if arm.endswith("E") else 300 + (s % 50)
                    k = f"{s}:{r}:{row}"
                    if k not in pos:
                        pos[k] = len(keys)
                        keys.append(k)
                        obs.append(rng.normal(size=n_obs))
                        seq.append(rng.normal(size=(e6.SEQ_LEN, n_seq)).astype(np.float32))
                    i = pos[k]
                    val = fx(obs[i]) + (fs(seq[i]) if fs else 0.0) + 0.3 * ("gf" in arm)
                    truth = {o: {lag: float(val) for lag in v2.LAGS} for o in v2.OUTCOMES}
                    cases.append(ef.Case(s, arm, r, role, row * 60, row, truth, 0.0, (False, False), None))
        return cases, keys, obs, seq

    out = {}
    specs = {"v2": [(range(40000, 40400), (0, 1), "train"), (range(40400, 40480), (0, 1), "val")]}
    if v2x:
        specs["v2x"] = [(range(50000, 51600), (0,), "train")]
    for name, groups in specs.items():
        cs, ks, ob, sq = [], [], [], []
        for seeds, reps, role in groups:
            a, b_, o_, q = part(seeds, reps, role)
            cs += a; ks += b_; ob += o_; sq += q
        p = e6.Part(cs, ks, np.array(ob), np.zeros((len(ks), 3)), np.array(sq), np.zeros((len(ks), e6.SEQ_LEN, 2), np.float32),
                    {"kind": name})
        e6.save_part(p, tmp, name)
        out[name] = p
    (tmp / "base.json").write_text(json.dumps({"stds": stds}))
    return out


class TestNested(unittest.TestCase):
    def test_nested_sets(self):
        sets = {n: e6.train_seeds(n) for n in e6.NS}
        self.assertEqual(sets[400], list(range(40000, 40400)))
        self.assertEqual({n: len(s) for n, s in sets.items()}, {400: 400, 800: 800, 1600: 1600, 2000: 2000})
        for a, b in ((400, 800), (800, 1600), (1600, 2000)):
            self.assertTrue(set(sets[a]) < set(sets[b]))
        self.assertEqual(sorted(set(sets[2000]) - set(sets[400])), list(range(50000, 51600)))
        self.assertEqual(sorted(set(sets[800]) - set(sets[400])), list(range(50000, 50400)))
        self.assertEqual(sorted(set(sets[1600]) - set(sets[400])), list(range(50000, 51200)))
        val, test = set(range(*v2.ROLES["game_v19"]["val"])), set(range(*v2.ROLES["game_v19"]["test"]))
        self.assertFalse(set(sets[2000]) & (val | test))
        self.assertEqual(d6.SEEDS, (50000, 51600))
        with self.assertRaises(ValueError):
            e6.train_seeds(1000)


class TestReplicateZero(unittest.TestCase):
    def test_training_cases_are_replicate_0_and_val_unchanged(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            parts = synthetic_prep(t, lambda x: x[0], v2x=True)
            base = json.loads((t / "base.json").read_text())
            for n in e6.NS:
                for feat in ("obs", "priv"):
                    tr, va, _ = e6.load_sets(t, base, n, feat, False)
                    self.assertEqual({x.rep for x in tr.cases}, {0})
                    self.assertEqual({x.seed for x in tr.cases}, set(e6.train_seeds(n)))
                    self.assertEqual({x.arm for x in tr.cases}, set(e6.ARMS))
                    self.assertEqual({x.seed for x in va.cases}, set(range(40400, 40480)))
                    self.assertEqual({x.rep for x in va.cases}, {0, 1})
            # the N = 400 set never reads the v2x part; the v2 part still holds replicate 1 (the WM-5 reproduction)
            self.assertIn(1, {x.rep for x in parts["v2"].cases if x.role == "train"})


class TestLeakage(unittest.TestCase):
    def setUp(self):
        self.var = d.variant_for(e5.VARIANT_NAME, d.ALL_STATE_OBS + c.V17_EXTRA)

    def test_observable_inputs_are_wm2_lists(self):
        names = self.var.input_names()
        self.assertEqual(len(names), 74)                         # WM-2's 74 action-aware inputs
        self.assertEqual(len(self.var.state_names), 37)
        hidden = set(d.HIDDEN_V2) | set(d.FORBIDDEN) | {"s.wood_target"}
        for n in names:
            self.assertNotIn(n.split(".", 1)[-1], hidden, n)
            self.assertNotIn(n, hidden)
        obs = e5.obs_feature_names(self.var.state_names, tuple(names[37:67]))
        self.assertEqual(len(obs), 145)
        for n in obs:
            self.assertFalse(n.startswith("hidden."), n)
            self.assertNotIn(n.split("@")[0].split(".", 1)[-1], hidden, n)
        self.assertTrue(all(n.startswith("hidden.") for n in e5.priv_feature_names(("a", "b"))))

    def test_observable_models_read_only_observable_arrays(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            parts = synthetic_prep(t, lambda x: x[0])
            p = parts["v2"]
            p.priv[:] = 99.0
            p.seqh[:] = 99.0
            stds = ef.stds_from(json.loads((t / "base.json").read_text())["stds"])
            cs = p.cases[:20]
            g = e6.gather([p], cs, "obs", stds, True)
            self.assertEqual(g.x.shape[1], p.obs.shape[1])
            self.assertEqual(g.seq.shape[-1], 74)
            self.assertFalse((g.x == 99.0).any() or (g.seq == 99.0).any())
            gp = e6.gather([p], cs, "priv", stds, True)
            self.assertEqual(gp.x.shape[1], p.obs.shape[1] + 3)
            self.assertEqual(gp.seq.shape[-1], 76)

    @unittest.skipUnless(HAVE_V2, "needs the recorded v2 data (DGX)")
    def test_hidden_rows_and_observable_features_on_a_recorded_member(self):
        from wm_data import load_episode
        path = e6.member_path(DATA, "v2", 40000, 0)
        ep = load_episode(path, arrays=e5.ARRAYS_PRIV)
        h, keys = e6.hidden_rows(ep)
        for r in (0, 238, 490, 1500, 2880):
            np.testing.assert_array_equal(h[r], e5.priv_features(ep, r)[0])
        epd = d.from_episode(ep, "v19_fam", "train", 40000, "none", c.V17_EXTRA)
        f0 = e5.obs_features(epd, 490)
        s0 = e6.seq_inputs(epd)
        # perturb every hidden field the recorder writes; the observable features must not move
        ep.arrays["latent.npy"] = ep.arrays["latent.npy"] + 7.0
        g = ep.arrays["global.npy"].copy()
        for n in e5.HIDDEN_GLOBAL:
            g[..., ep.field_index("global.npy", n)] += 3.0
        ep.arrays["global.npy"] = g
        s = ep.arrays["settlements.npy"].copy()
        for n in e5.HIDDEN_SETTLEMENT:
            s[..., ep.field_index("settlements.npy", n)] += 5.0
        ep.arrays["settlements.npy"] = s
        epd2 = d.from_episode(ep, "v19_fam", "train", 40000, "none", c.V17_EXTRA)
        np.testing.assert_array_equal(f0, e5.obs_features(epd2, 490))
        np.testing.assert_array_equal(s0, e6.seq_inputs(epd2))
        self.assertFalse(np.array_equal(h[490], e6.hidden_rows(ep)[0][490]))


@unittest.skipUnless(HAVE_TORCH, "needs torch")
class TestPlanted(unittest.TestCase):
    """The MLP and the GRU recover a planted nonlinear effect on synthetic families (GOAL.md 7.6)."""

    def run_model(self, model, fx, fs, max_seconds):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            synthetic_prep(t, fx, fs, n_seq=6 if fs else 74)
            out = t / "run"
            rc = e6.main(["train", "--model", model, "--feat", "obs", "--n", "400", "--seed", "0", "--prep", str(t),
                          "--base", str(t / "base.json"), "--max-seconds", str(max_seconds), "--device", "auto",
                          "--out", str(out)])
            self.assertEqual(rc, 0)
            val = json.loads((out / "val_metrics.json").read_text())
            cfg = json.loads((out / "config.json").read_text())
            self.assertEqual(cfg["train_reps"], [0])
            # the zero predictor's val error and a linear fit's, on the same synthetic val set
            base = json.loads((t / "base.json").read_text())
            tr, va, stds = e6.load_sets(t, base, 400, "obs", fs is not None)
            zero = e6.cell_scores(va.cases, e6.to_predictions(va.cases, np.zeros((len(va.cases), 8)), stds), stds)["both"]
            X = np.c_[tr.x, tr.arm, np.ones(len(tr.x))]
            w = np.linalg.lstsq(X, tr.y, rcond=None)[0]
            lin = e6.cell_scores(va.cases, e6.to_predictions(va.cases, np.c_[va.x, va.arm, np.ones(len(va.x))] @ w, stds),
                                 stds)["both"]
            return val, zero, lin

    def test_mlp_recovers_a_planted_nonlinear_effect(self):
        fx = lambda x: 2.0 * np.sin(2.0 * x[0]) * (x[1] > 0) + x[2] ** 2 - 1.0
        val, zero, lin = self.run_model("mlp", fx, None, 120)
        self.assertIn(val["stop"], ("patience", "max_epochs", "max_seconds"))
        self.assertLess(val["val"]["both"], 0.5 * lin)
        self.assertLess(val["val"]["both"], 0.4 * zero)

    def test_gru_recovers_a_planted_sequence_effect(self):
        fs = lambda s: 2.0 * np.abs(s[-1, 0]) - 1.6          # nonlinear in the window's last (pre-act) row only
        val, zero, lin = self.run_model("seq", lambda x: 0.0, fs, 60)
        self.assertLess(val["val"]["both"], 0.3 * lin)
        self.assertLess(val["val"]["both"], 0.3 * zero)


class TestTimelineMerge(unittest.TestCase):
    def test_merge_concatenates_runs_in_seed_order(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            for a, seeds in ((10, [11, 10]), (12, [12])):
                (t / f"c{a}").mkdir()
                (t / f"c{a}" / "summary.json").write_text(json.dumps(
                    {"scene": "game_v19", "determinism_checked": True, "runs": [{"seed": s} for s in seeds]}))
            m = d6.merge_timeline(t, 10, 13, 2)
            self.assertEqual([r["seed"] for r in m["runs"]], [10, 11, 12])
            self.assertTrue(m["determinism_checked"])
            (t / "c12" / "summary.json").write_text(json.dumps({"scene": "game_v19", "runs": [{"seed": 13}]}))
            with self.assertRaises(ValueError):
                d6.merge_timeline(t, 10, 13, 2)


class TestKS(unittest.TestCase):
    def test_ks_statistic_and_p(self):
        r = np.random.default_rng(0)
        same = d6.ks_2samp(r.normal(size=400), r.normal(size=1600))
        shifted = d6.ks_2samp(r.normal(size=400), r.normal(0.5, 1.0, size=1600))
        self.assertGreater(same["p"], 0.05)
        self.assertLess(shifted["p"], 1e-6)
        self.assertAlmostEqual(d6.ks_2samp([0, 1, 2], [0, 1, 2])["D"], 0.0)
        self.assertAlmostEqual(d6.ks_2samp([0, 0], [1, 1])["D"], 1.0)
        # D by brute force over every threshold
        a, b = r.integers(0, 4, 50), r.integers(0, 5, 70)
        brute = max(abs(np.mean(a <= t) - np.mean(b <= t)) for t in np.concatenate([a, b]))
        self.assertAlmostEqual(d6.ks_2samp(a, b)["D"], brute)


@unittest.skipUnless(HAVE_V2 and BASE.exists(), "needs the recorded v2 data (DGX)")
class TestWM5Reproduction(unittest.TestCase):
    """WM-5's ridge at N = 400 with both replicates, through WM-6's feature path, equals WM-5's committed numbers."""

    def test_ridge_reproduces_wm5(self):
        base = json.loads(BASE.read_text())
        stds = ef.stds_from(base["stds"])
        cases = {r: [ef.case_from(x) for x in base["cases"][r]] for r in ("train", "val", "test")}
        parts = [e6.build_part(DATA, "v2", cases[r], r) for r in ("train", "val", "test")]
        for feat in ("obs", "priv"):
            name = f"ridge_{feat}"
            wm5 = json.loads((c.REPO / f"docs/worldmodel/wm5/runs/{name}/{name}.json").read_text())
            X = {r: e6.feature_dict(parts, cases[r], feat) for r in ("train", "val", "test")}
            fits = e5.fit_cells(cases["train"], cases["val"], X["train"], X["val"], stds, e6.ALL_CELLS)
            self.assertEqual({"|".join(k): v["lambda"] for k, v in fits.items()}, wm5["lambdas"])
            for role, key in (("val", "v2_val_predictions"), ("test", "v2_test_predictions")):
                got = e5.ridge_predict(fits, cases[role], X[role])
                cmp = e6.compare_preds(got, wm5[key])
                self.assertTrue(cmp["keys_equal"])
                self.assertLess(cmp["max_abs_diff"], 1e-6, f"{name} {role}")


if __name__ == "__main__":
    unittest.main()
