"""WM-1.3 harness (GOAL_1_3.md 4.1-4.2, 6.3, 7.1-7.4): the two input lists and hidden-field absence, schema-5 tables,
the nested training sets, `ramp_steps`, WM-1.2's online stopping rule, per-world scores, the world-level bootstrap,
the cross-dataset refusal, and (data-dependent, DGX gate) one WM-1.2 calendar test number recomputed and a schema-5
`game_v16` episode's feature table against its p10 twin.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import os
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
from wm_data import Episode, load_episode
from wm1 import bootstrap as bs
from wm1 import common as c
from wm1 import data as d
from wm1 import evaluate as ev
from wm1 import m1_gru as m
from wm1 import m1_tf as mt
from wm1.test_pilot_harness import GLOBAL, SENTINEL, SETTLE, STATE_OBS, OffsetPredictor, synth_episode

V17_STATE = STATE_OBS + ("g.settlements_alive", "g.fissions_total")
DATA_ROOT = Path(os.environ.get("WM_DATA_ROOT", "/home/hyunlord/work/worldsim-wm-data"))
EQUIV_DIR = Path(os.environ.get("WM13_EQUIV_DIR", "/home/hyunlord/work/wm1-runs/_scratch_w13/equiv_v16"))


def synth_v5(seed: int, rows: int = 1200, settlements: int = 2) -> Episode:
    """The schema-4 synthetic episode with schema 5's two globals and two settlement columns appended (the new
    settlement columns carry SENTINEL: they must never reach the table)."""
    ep = synth_episode(seed, rows, settlements)
    rng = np.random.default_rng(100 + seed)
    alive = 1 + (np.arange(rows) > rows // 2) * (seed % 2)
    fissions = np.cumsum(rng.random(rows) < 0.002)
    g = np.concatenate([ep.arrays["global.npy"], alive[:, None].astype(np.float32),
                        fissions[:, None].astype(np.float32)], axis=1)
    s = np.concatenate([ep.arrays["settlements.npy"], np.full((rows, settlements, 2), SENTINEL, np.float32)], axis=2)
    meta = json.loads(json.dumps(ep.meta))
    meta["schema_version"] = 5
    meta["fields"]["global.npy"] += [{"name": "settlements_alive", "hidden": False}, {"name": "fissions_total", "hidden": False}]
    meta["fields"]["settlements.npy"] += [{"name": "parent_slot", "hidden": False}, {"name": "founded_tick", "hidden": False}]
    meta["observable_fields"]["global.npy"] += ["settlements_alive", "fissions_total"]
    meta["observable_fields"]["settlements.npy"] += ["parent_slot", "founded_tick"]
    return Episode(ep.path, meta, {"global.npy": g, "settlements.npy": s}, [])


class InputListTests(unittest.TestCase):
    def test_game_v16_arm_is_wm12s_65_names_and_game_v17_adds_two_state_columns(self) -> None:
        for name in ("w13_v16_n80", "w13_v16_n200", "w13_v16_n400"):
            v = m.dataset_variant(c.dataset(name), "sev-blind_action-blind")
            self.assertEqual(v.input_names(), STATE_OBS + d.CALENDAR)
            self.assertEqual(len(v.input_names()), 65)
        v17 = m.dataset_variant(c.dataset("w13_v17_n400"), "sev-blind_action-blind")
        self.assertEqual(v17.input_names(), V17_STATE + d.CALENDAR)
        self.assertEqual(len(v17.state_names), 37)
        self.assertEqual(len(v17.input_names()), 67)
        # Before the calendar: state features, normalized, predicted and rolled out.
        self.assertIn("g.fissions_total", v17.continuous_names)
        self.assertEqual(d.Layout(v17).n_state, 37)
        for v in (m.dataset_variant(c.dataset("w13_v16_n400"), "sev-blind_action-blind"), v17):
            for n in v.input_names():
                for bad in d.HIDDEN_V2 + d.NOT_INPUTS_V5 + ("formation", "centre", "latent"):
                    self.assertNotIn(bad, n)
                self.assertFalse(n.startswith(("sev_", "act.")), n)
            self.assertFalse({"d.fissions_total", "d.settlements_alive"} & set(v.input_names()))

    def test_schema_5_table_per_dataset(self) -> None:
        raw4, raw5 = synth_episode(3, settlements=2), synth_v5(3)
        e4 = d.from_episode(raw4, "game_v16", "train", 3, "none")
        e5 = d.from_episode(raw5, "game_v16", "train", 3, "none")
        self.assertEqual(e5.names, d.ALL_STATE_OBS)          # schema 5 of game_v16: exactly the 65-input table
        np.testing.assert_array_equal(e5.table, e4.table)
        e17 = d.from_episode(raw5, "game_v17", "train", 3, "none", c.V17_EXTRA)
        self.assertEqual(e17.names, V17_STATE)
        np.testing.assert_array_equal(e17.table[:, :35], e4.table)
        g = raw5.arrays["global.npy"]
        np.testing.assert_array_equal(e17.table[:, 35], g[:, len(GLOBAL)])                 # raw level
        np.testing.assert_array_equal(e17.table[:, 36], g[:, len(GLOBAL) + 1])             # cumulative, not differenced
        self.assertFalse(np.any(e17.table == SENTINEL))
        with self.assertRaises(KeyError):
            d.from_episode(synth_episode(3), "game_v17", "train", 3, "none", c.V17_EXTRA)   # schema 4: no such column
        self.assertEqual(len(SETTLE) + 2, raw5.arrays["settlements.npy"].shape[2])


class RegistryTests(unittest.TestCase):
    def test_nested_training_sets(self) -> None:
        for name, lo, hi in (("w13_v16_n80", 30000, 30080), ("w13_v16_n200", 30000, 30200),
                             ("w13_v16_n400", 30000, 30400), ("w13_v17_n400", 32000, 32400)):
            ds = c.dataset(name)
            self.assertEqual(ds.train_seeds, (lo, hi))
            self.assertEqual(c.group_seeds(ds, "train"), set(range(lo, hi)))
            self.assertIsNone(c.group_seeds(ds, "val"))
            self.assertIsNone(c.group_seeds(ds, "test"))
            self.assertEqual((ds.cf, ds.tasks, ds.observable, ds.manifest), (None, ("T1", "T2", "T2_any"), True,
                                                                            c.W13_MANIFEST))
        self.assertEqual(c.dataset("w13_v17_n400").extra_state, c.V17_EXTRA)
        self.assertEqual(c.dataset("w13_v16_n400").extra_state, ())
        self.assertIsNone(c.dataset("p10_v16").train_seeds)

    @unittest.skipUnless(c.W13_MANIFEST.exists(), "w13_manifest.json not written yet")
    def test_manifest_roles_and_nested_seeds(self) -> None:
        raw = json.loads(c.W13_MANIFEST.read_text())
        self.assertEqual(raw["episode_count"], 1120)
        roles = {}
        for name in ("w13_v16_n80", "w13_v16_n200", "w13_v16_n400", "w13_v17_n400"):
            ds = c.dataset(name)
            eps = c.manifest(name)["episodes"]
            keep = c.group_seeds(ds, "train")
            roles[name] = {r: sorted(e["seed"] for e in eps if e["role"] == r and (r != "train" or e["seed"] in keep))
                           for r in ("train", "val", "test")}
            for e in eps:
                self.assertEqual((e["schedule_id"], e["rows"], e["split"]), ("none", 2881, e["scene"]))
                self.assertEqual(e["path"], f"w13/{e['scene']}/{e['scene']}_s{e['seed']}_none")
        self.assertEqual(roles["w13_v16_n80"]["train"], list(range(30000, 30080)))
        self.assertEqual(roles["w13_v16_n200"]["train"], list(range(30000, 30200)))
        self.assertEqual(roles["w13_v16_n400"]["train"], list(range(30000, 30400)))
        for name in ("w13_v16_n80", "w13_v16_n200", "w13_v16_n400"):
            self.assertEqual(roles[name]["val"], list(range(30400, 30480)))
            self.assertEqual(roles[name]["test"], list(range(30480, 30560)))
        self.assertEqual(roles["w13_v17_n400"], {"train": list(range(32000, 32400)), "val": list(range(32400, 32480)),
                                                 "test": list(range(32480, 32560))})


class CurriculumTests(unittest.TestCase):
    def old_k(self, step, cfg):            # WM-1.2's formula (m1_gru.py at a43bdab2, L349)
        half = cfg["max_steps"] // 2
        return 1 + int((cfg["k_max"] - 1) * min(1.0, step / max(1, half)))

    def test_ramp_steps_0_reproduces_the_old_ramp_and_patience_start(self) -> None:
        self.assertEqual(m.DEFAULTS["ramp_steps"], 0)
        self.assertEqual(mt.DEFAULTS["ramp_steps"], 0)
        self.assertIsInstance(m.DEFAULTS["ramp_steps"], int)
        for base in (m.DEFAULTS, mt.DEFAULTS, {**mt.DEFAULTS, **m.R3, "max_steps": 3000}):
            cfg = dict(base)
            self.assertEqual(m.ramp_half(cfg), cfg["max_steps"] // 2)
            self.assertEqual([m.k_at(s, cfg) for s in range(1, cfg["max_steps"] + 1)],
                             [self.old_k(s, cfg) for s in range(1, cfg["max_steps"] + 1)])

    def test_long_run_ramp_equals_r3_for_the_first_3000_steps(self) -> None:
        r3 = {**mt.DEFAULTS, **m.R3, "max_steps": 3000}
        long = dict(r3, max_steps=9000, ramp_steps=1500)
        self.assertEqual(m.ramp_half(long), 1500)
        self.assertEqual([m.k_at(s, long) for s in range(1, 3001)], [self.old_k(s, r3) for s in range(1, 3001)])
        self.assertEqual({m.k_at(s, long) for s in range(1500, 9001)}, {168})

    def test_parser_accepts_ramp_steps(self) -> None:
        import argparse
        ap = argparse.ArgumentParser()
        for k, v in mt.DEFAULTS.items():
            ap.add_argument(f"--{k.replace('_', '-')}", dest=k, type=type(v), default=None)
        self.assertEqual(ap.parse_args(["--ramp-steps", "1500"]).ramp_steps, 1500)


class Wm12RuleTests(unittest.TestCase):
    def run_rule(self, scores, eval_every=150):
        rule = m.Wm12Rule({"eval_every": eval_every})
        saved = [step for step, sc in zip(range(eval_every, eval_every * (len(scores) + 1), eval_every), scores)
                 if rule.update(step, sc)]
        return rule, saved

    def test_patience_counts_from_step_1500(self) -> None:
        # WM-1.2's check `bad >= patience and step >= half`: bad evaluations before step 1,500 count, but the rule
        # acts only from step 1,500.
        rule, saved = self.run_rule([1.0] + [2.0] * 12 + [0.5])
        self.assertEqual(saved, [150])
        self.assertEqual((rule.stop, rule.stop_step, rule.best_step), ("patience", 1500, 150))
        self.assertFalse(rule.update(2100, 0.1))           # nothing after the rule stopped
        rule, saved = self.run_rule([1.0 - 0.01 * i for i in range(14)] + [2.0] * 20)
        self.assertEqual(saved[-1], 2100)
        self.assertEqual((rule.stop, rule.stop_step, rule.best_step), ("patience", 2850, 2100))

    def test_cap_at_3000_and_inactive_off_grid(self) -> None:
        rule, saved = self.run_rule([1.0 - 0.01 * i for i in range(40)])
        self.assertEqual((rule.stop, rule.stop_step, rule.best_step), ("max_steps", 3000, 3000))
        self.assertEqual(saved[-1], 3000)
        self.assertEqual(rule.summary(6000, "patience")["stop_step"], 3000)
        off, saved = self.run_rule([1.0, 0.5], eval_every=300)
        self.assertFalse(off.active)
        self.assertEqual(saved, [])
        self.assertIsNone(off.summary(600, "patience"))

    def test_run_ending_first_is_recorded(self) -> None:
        rule, _ = self.run_rule([1.0, 0.9])
        self.assertEqual(rule.summary(300, "time")["stop"], "run_ended_time")


class PerWorldTests(unittest.TestCase):
    def test_mean_of_world_means_is_the_pooled_mean(self) -> None:
        eps = [d.from_episode(synth_episode(s, 1200), "game_v16", "test", s, "none") for s in range(5)]
        std = c.split_stds({"test": eps})["test"]
        for pred in (OffsetPredictor(), ev.Persistence()):
            for any_hour in (False, True):
                for h in (24, 168):
                    out = ev.t2(pred, eps, std, h, any_hour=any_hour, bias_removed=True, per_world=True)
                    pw = out["per_world"]
                    self.assertEqual(sorted(pw, key=int), [str(s) for s in range(5)])
                    self.assertEqual(len({r["n"] for r in pw.values()}), 1)
                    self.assertEqual(sum(r["n"] for r in pw.values()), out["n"])
                    self.assertAlmostEqual(np.mean([r["mean"] for r in pw.values()]), out["mean"], delta=1e-12)
                    self.assertAlmostEqual(np.mean([r["bias_removed"]["mean"] for r in pw.values()]),
                                           out["bias_removed"]["mean"], delta=1e-12)
                    np.testing.assert_allclose(np.mean([r["per_var"] for r in pw.values()], axis=0), out["per_var"],
                                               rtol=0, atol=1e-12)
                    ws = bs.world_scores(out)
                    self.assertAlmostEqual(np.mean(list(ws.values())), out["mean"], delta=1e-12)

    def test_off_by_default(self) -> None:
        eps = [d.from_episode(synth_episode(1, 1200), "game_v16", "test", 1, "none")]
        self.assertNotIn("per_world", ev.t2(ev.Persistence(), eps, c.split_stds({"t": eps})["t"], 24))


class BootstrapTests(unittest.TestCase):
    def setUp(self) -> None:
        rng = np.random.default_rng(1)
        self.a = {str(30480 + i): float(v) for i, v in enumerate(rng.random(80) + 0.2)}

    def test_constant_shift_is_a_degenerate_interval_at_the_shift(self) -> None:
        b = {k: v - 0.03 for k, v in self.a.items()}
        ci = bs.paired(self.a, b)
        self.assertAlmostEqual(ci["lo"], 0.03, delta=1e-12)
        self.assertAlmostEqual(ci["hi"], 0.03, delta=1e-12)
        self.assertEqual(bs.paired(self.a, dict(self.a)) | {}, bs.paired(self.a, dict(self.a)))
        same = bs.paired(self.a, dict(self.a))
        self.assertEqual((same["lo"], same["hi"], same["point"]), (0.0, 0.0, 0.0))

    def test_interval_by_hand(self) -> None:
        keys = sorted(self.a, key=int)
        x = np.array([self.a[k] for k in keys])
        idx = np.random.default_rng(0).integers(0, 80, size=(2000, 80))
        manual = np.sort([x[row].mean() for row in idx])
        ci = bs.interval(self.a)
        self.assertAlmostEqual(ci["lo"], float(np.percentile(manual, 2.5)), delta=1e-15)
        self.assertAlmostEqual(ci["hi"], float(np.percentile(manual, 97.5)), delta=1e-15)
        self.assertTrue(ci["lo"] < ci["point"] < ci["hi"])

    def test_verdicts(self) -> None:
        base = self.a
        better = {k: 0.7 * v for k, v in base.items()}
        self.assertEqual(bs.verdict([better, better], base)["verdict"], "KEEP")
        slightly = {k: v - 0.05 for k, v in base.items()}
        self.assertEqual(bs.verdict([slightly], base)["verdict"], "WEAK")
        self.assertEqual(bs.verdict([dict(base)], base)["verdict"], "TIE")
        worse = {k: v + 0.05 for k, v in base.items()}
        self.assertEqual(bs.verdict([worse], base)["verdict"], "NONE")
        # One seed above the baseline blocks KEEP even with a seed mean <= 0.8 x and CI below 0.
        mixed = bs.verdict([{k: 0.3 * v for k, v in base.items()}, {k: 1.02 * v for k, v in base.items()}], base)
        self.assertEqual(mixed["verdict"], "WEAK")
        with self.assertRaises(ValueError):
            bs.paired(self.a, {k: v for k, v in list(self.a.items())[:-1]})

    def test_world_scores_need_equal_start_counts(self) -> None:
        node = {"per_world": {"1": {"per_var": [1.0] * 8, "n": 3}, "2": {"per_var": [2.0] * 8, "n": 4}}}
        with self.assertRaises(ValueError):
            bs.world_scores(node)


class ProbeObservableTests(unittest.TestCase):
    def test_first_fission_day_inside_the_train_range(self) -> None:
        from wm1 import probe_world as pw
        rng = np.random.default_rng(0)
        x = {r: rng.normal(size=(n, 4)) for r, n in (("train", 60), ("val", 20), ("test", 20))}
        lab = {}
        for r in x:
            day = 29 + np.round(2 * np.abs(x[r][:, 0]))
            day[::4] = np.nan                                   # worlds that never split
            if r == "test":
                day[1] = 85.0
            lab[r] = {"fission_by_day120": np.isfinite(day).astype(float),
                      "settlements_alive_row2832": 1.0 + np.isfinite(day), "first_fission_day": day}
        out = pw.probe_observable(x, lab)
        ff = out["first_fission_day"]
        self.assertEqual(ff["test_outside_train_range"], [85.0])
        self.assertEqual(ff["worlds"]["test"], 15)
        self.assertIsNotNone(ff["r2_test_inside_train_range"])
        self.assertEqual(pw.first_split_day(np.array([0, 0, 1, 1.0])), float((2 * 60 + 480) // 1440 + 1))
        self.assertTrue(np.isnan(pw.first_split_day(np.zeros(5))))
        self.assertIn("accuracy_test", out["fission_by_day120"])


class CrossDatasetTests(unittest.TestCase):
    def test_input_list_must_match(self) -> None:
        v16 = list(m.dataset_variant(c.dataset("p10_v16"), "sev-blind_action-blind").input_names())
        cfg = {"variant": "sev-blind_action-blind", "input_names": v16, "dataset": "p10_v16"}
        m.check_eval_dataset(cfg, "w13_v16_n400")
        m.check_eval_dataset(cfg, "w13_v16_n80")
        with self.assertRaises(SystemExit):
            m.check_eval_dataset(cfg, "w13_v17_n400")
        v17 = list(m.dataset_variant(c.dataset("w13_v17_n400"), "sev-blind_action-blind").input_names())
        with self.assertRaises(SystemExit):
            m.check_eval_dataset({"variant": "sev-blind_action-blind", "input_names": v17}, "p10_v16")
        ds, groups, kw = m.test_plan(cfg, "w13_v16_n400")
        self.assertEqual((ds.name, groups, kw["per_world"]), ("w13_v16_n400", ["test"], True))

    def test_checkpoint_rounding(self) -> None:
        import tempfile
        with tempfile.TemporaryDirectory() as tmp:
            run = Path(tmp)
            (run / "val_metrics.json").write_text(json.dumps({"wm12rule": {"round_counts": False}}))
            self.assertTrue(m.checkpoint_rounding(run, {"round_counts": True}, "best.pt"))
            self.assertFalse(m.checkpoint_rounding(run, {"round_counts": True}, m.WM12_CKPT))
            with self.assertRaises(SystemExit):
                m.checkpoint_rounding(run, {}, "last.pt")


@unittest.skipUnless((DATA_ROOT / "p10/pilot/game_v16").exists(), "p10 data not on this machine (DGX gate)")
class WM12NumberTests(unittest.TestCase):
    """GOAL_1_3.md 4.1: WM-1.2's `test.an5.T2_any["168"].test.mean` on p10_v16 (calendar_p10_v16_1a7dfdb6)."""

    def test_an5_every_hour_168_on_p10_v16(self) -> None:
        from wm1 import climatology as cl
        g = c.load_groups(["train", "test"], DATA_ROOT, "p10_v16")
        fit = cl.CalendarFit(g["train"], cl.T_TR, split="game_v16")
        std = c.split_stds({"test": g["test"]})["test"]
        got = ev.t2(cl.CalendarPredictor(fit, "an5"), g["test"], std, 168, any_hour=True)["mean"]
        self.assertAlmostEqual(got / 0.2064778167503853, 1.0, delta=1e-9)


@unittest.skipUnless((EQUIV_DIR / "game_v16_s20000_none").exists() and (DATA_ROOT / "p10/pilot/game_v16").exists(),
                     "the WM-1.3 scratch recording and p10 data are on the DGX only")
class Schema5TwinTests(unittest.TestCase):
    """GOAL_1_3.md 7.1: a schema-5 game_v16 episode gives the same 65-column table as its schema-4 p10 twin."""

    def test_identical_feature_tables(self) -> None:
        arrays = ["global.npy", "settlements.npy"]
        new = load_episode(EQUIV_DIR / "game_v16_s20000_none", arrays=arrays)
        old = load_episode(DATA_ROOT / "p10/pilot/game_v16/game_v16_s20000_none", arrays=arrays)
        self.assertEqual((new.meta["schema_version"], old.meta["schema_version"]), (5, 4))
        a = d.from_episode(new, "game_v16", "train", 20000, "none")
        b = d.from_episode(old, "game_v16", "train", 20000, "none")
        self.assertEqual(a.names, b.names)
        self.assertEqual(len(a.names) + len(d.CALENDAR), 65)
        np.testing.assert_array_equal(a.table, b.table)
        np.testing.assert_array_equal(a.calendar, b.calendar)
        np.testing.assert_array_equal(a.bench.v, b.bench.v)


if __name__ == "__main__":
    unittest.main()
