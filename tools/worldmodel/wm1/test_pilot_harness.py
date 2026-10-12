"""The WM-1 harness on recorder schema 4 (GOAL_1_2.md 7.1-7.3, 4.2): the observable feature table, the input field
list and hidden-field absence per dataset, the bias-removed T2 score, `evaluate(tasks=...)` end to end on synthetic
schema-4 episodes, the refusal of severity inputs, the start sets against `bench_v1`, the calendar selection on
bias-removed tasks, and what `final` runs for a run's config.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
import bench_v1 as bv1
from wm_data import Episode
from wm1 import climatology as cl
from wm1 import common as c
from wm1 import data as d
from wm1 import evaluate as ev

# The field lists of a recorded game_v16 episode (recorder schema 4, `58119582`).
GLOBAL = ("tick", "day_index", "minute_of_day", "season", "day_of_season", "year", "winter_severity", "alive",
          "children", "elders", "adults", "births", "deaths_starvation", "deaths_dehydration", "deaths_combat",
          "deaths_old_age", "cold_members", "deadwood_total", "food_on_bushes_total", "adults_working", "idle_adults",
          "meals_from_store_total", "ground_meals_total", "forage_trips_total", "wood_trips_total",
          "wood_gathered_total", "urgent_wood_trips_total", "wood_retargets_total", "cold_member_hours_total",
          "interventions_applied_total", "settlements_under_rain", "cold_snap_nights_left", "cold_snap_tonight",
          "night_burn_per_hour", "winter_fall_today", "deaths_exposure", "injured_members")
SETTLE = ("members", "food_stock", "wood_stock", "wood_target", "fire_lit", "wood_in_hand", "formation_x",
          "formation_y", "rain_ticks_left")
HIDDEN_G = ("winter_severity", "night_burn_per_hour", "winter_fall_today")
HIDDEN_S = ("wood_target",)
SENTINEL = 12345.0
# The observable state columns of the pilot, written out (GOAL_1_2.md 4.2: the input list asserted exactly).
STATE_OBS = (
    "g.alive", "g.children", "g.elders", "g.adults", "g.cold_members", "g.deadwood_total", "g.food_on_bushes_total",
    "g.adults_working", "g.idle_adults", "g.settlements_under_rain", "g.cold_snap_nights_left", "g.cold_snap_tonight",
    "d.births", "d.deaths_starvation", "d.deaths_dehydration", "d.deaths_combat", "d.deaths_old_age",
    "d.meals_from_store_total", "d.ground_meals_total", "d.forage_trips_total", "d.wood_trips_total",
    "d.wood_gathered_total", "d.urgent_wood_trips_total", "d.wood_retargets_total", "d.cold_member_hours_total",
    "d.interventions_applied_total", "s.members", "s.food_stock", "s.wood_stock", "s.wood_in_hand", "s.fire_lit",
    "s.rain_ticks_left", "s.present", "d.deaths_exposure", "g.injured_members")
TALLIES = ("births", "deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age",
           "meals_from_store_total", "ground_meals_total", "forage_trips_total", "wood_trips_total",
           "wood_gathered_total", "urgent_wood_trips_total", "wood_retargets_total", "cold_member_hours_total",
           "deaths_exposure")


def synth_episode(seed: int, rows: int = 1200, settlements: int = 1) -> Episode:
    """A schema-4 episode with a per-world level (the `starting_agents` analogue) and noisy dynamics; hidden columns
    carry SENTINEL so a leak shows."""
    rng = np.random.default_rng(seed)
    tick = 60 * np.arange(rows)
    day = (tick + 480) // 1440
    g = np.zeros((rows, len(GLOBAL)), np.float32)
    put = lambda n, v: g.__setitem__((slice(None), GLOBAL.index(n)), v)
    put("tick", tick)
    put("day_index", day)
    put("minute_of_day", (tick + 480) % 1440)
    put("season", (day % 28) // 7)
    put("day_of_season", day % 7)
    put("year", day // 28)
    put("winter_severity", np.where((day % 28) // 7 == 3, seed % 3, -1))
    put("night_burn_per_hour", SENTINEL)
    put("winter_fall_today", SENTINEL)
    level = 18 + 3 * (seed % 4)
    put("alive", level + rng.integers(0, 2, rows))
    for n in ("children", "elders", "adults", "cold_members", "adults_working", "idle_adults", "injured_members",
              "settlements_under_rain", "cold_snap_nights_left", "cold_snap_tonight"):
        put(n, rng.integers(0, 4, rows))
    put("deadwood_total", 50 + 5 * (seed % 3) + np.cumsum(rng.normal(0, 0.3, rows)))
    put("food_on_bushes_total", 30 + np.sin(np.arange(rows) / 24.0) * 5 + seed % 5)
    for n in TALLIES:
        put(n, np.cumsum(rng.random(rows) < 0.05))
    s = np.zeros((rows, settlements, len(SETTLE)), np.float32)
    for k in range(settlements):
        s[:, k, SETTLE.index("members")] = level
        s[:, k, SETTLE.index("food_stock")] = 40 + seed % 7 + np.cumsum(rng.normal(0, 0.5, rows))
        s[:, k, SETTLE.index("wood_stock")] = 20 + np.cumsum(rng.normal(0, 0.3, rows))
        s[:, k, SETTLE.index("wood_target")] = SENTINEL
        s[:, k, SETTLE.index("fire_lit")] = (np.arange(rows) % 24) >= 10
    fields = lambda names, hidden: [{"name": n, "hidden": n in hidden} for n in names]
    meta = {"schema_version": 4, "scene": "game_v16", "seed": seed,
            "fields": {"global.npy": fields(GLOBAL, HIDDEN_G), "settlements.npy": fields(SETTLE, HIDDEN_S)},
            "observable_fields": {"global.npy": [n for n in GLOBAL if n not in HIDDEN_G],
                                  "settlements.npy": [n for n in SETTLE if n not in HIDDEN_S]},
            "world_variety": {"hidden": True, "values": {"starting_agents": level}}}
    return Episode(Path(f"/synthetic/s{seed}"), meta, {"global.npy": g, "settlements.npy": s}, [])


def synth(seed: int, role: str = "test", rows: int = 1200) -> d.EpisodeData:
    return d.from_episode(synth_episode(seed, rows), "game_v16", role, seed, "none")


class OffsetPredictor(ev.Predictor):
    """The truth plus a per-episode constant: only a static per-world bias, nothing wrong about the movement."""
    name = "offset"

    def rollout(self, ep, starts, horizon, actions):
        rows = np.asarray(starts)[:, None] + np.arange(1, horizon + 1)[None]
        return ev.Rollout(ep.bench.v[rows] + (1.0 + ep.seed % 3))


class ObservableTableTests(unittest.TestCase):
    def test_table_is_the_observable_list_without_hidden_values(self) -> None:
        ep = synth(1)
        self.assertEqual(ep.names, d.ALL_STATE_OBS)
        self.assertEqual(ep.schema, 4)
        self.assertEqual(d.ALL_STATE_OBS, tuple(n for n in d.ALL_STATE if n != "s.wood_target")
                         + ("d.deaths_exposure", "g.injured_members"))
        self.assertFalse(np.any(ep.table == SENTINEL))
        raw = synth_episode(1)
        exp = raw.arrays["global.npy"][:, GLOBAL.index("deaths_exposure")].astype(np.float64)
        np.testing.assert_array_equal(ep.table[:, ep.names.index("d.deaths_exposure")], np.concatenate([[0.0], np.diff(exp)]))
        # bench: severity is the hidden label; deaths include exposure (as Layout.deaths).
        np.testing.assert_array_equal(ep.bench.severity, raw.arrays["global.npy"][:, GLOBAL.index("winter_severity")])
        four = sum(raw.arrays["global.npy"][:, GLOBAL.index(k)] for k in TALLIES[1:5])
        np.testing.assert_array_equal(ep.bench.deaths, four + raw.arrays["global.npy"][:, GLOBAL.index("deaths_exposure")])
        lay = d.Layout(d.variant_for("sev-blind_action-blind", d.ALL_STATE_OBS))
        self.assertEqual([lay.names[i] for i in lay.deaths], list(d.DEATHS) + ["d.deaths_exposure"])

    def test_a_hidden_column_is_refused_as_an_input(self) -> None:
        with self.assertRaises(bv1.HiddenInput):
            bv1.observable_column(synth_episode(0), "settlements.npy", "wood_target")

    def test_v0_layout_deaths_unchanged(self) -> None:
        self.assertEqual([d.Layout(d.MAIN).names[i] for i in d.Layout(d.MAIN).deaths], list(d.DEATHS))


class InputListTests(unittest.TestCase):
    """GOAL_1_2.md 4.2: the input field list per dataset, asserted exactly, and each hidden field absent."""

    def test_input_fields_per_dataset(self) -> None:
        from wm1 import m1_gru
        for name in ("p10_v16", "p10_v16n", "p10_v15"):
            v = m1_gru.dataset_variant(c.dataset(name), "sev-blind_action-blind")
            self.assertEqual(v.input_names(), STATE_OBS + d.CALENDAR)
            self.assertEqual(len(v.input_names()), 65)
            for n in v.input_names():
                for bad in d.HIDDEN_V2 + ("formation",):
                    self.assertNotIn(bad, n, (name, n))
                self.assertFalse(n.startswith("sev_") or n.startswith("act."), n)
            for bad in ("g.tick", "g.day_index", "g.year", "g.minute_of_day", "g.season", "g.day_of_season"):
                self.assertNotIn(bad, v.input_names())
        v0 = m1_gru.dataset_variant(c.dataset("v0"), "sev-blind_action-blind")
        self.assertEqual(v0, d.VARIANTS["sev-blind_action-blind"])

    def test_severity_inputs_are_refused_on_schema_4(self) -> None:
        with self.assertRaises(ValueError):
            d.variant_for("sev-on_action-blind", d.ALL_STATE_OBS)
        with self.assertRaises(ValueError):
            d.Variant(True, False, d.ALL_STATE_OBS)
        ep = synth(2, "train")
        with self.assertRaises(ValueError):
            ep.severity_onehot()
        with self.assertRaises(ValueError):
            d.prepare(ep, d.VARIANTS["sev-on_action-blind"], d.Normalizer.fit([synth(3, "train")], d.VARIANTS["sev-blind_action-blind"]))
        x = np.zeros((1, len(bv1.LAGS) * 8 + 24 + 5))
        model = bv1.Ridge().fit(x, np.zeros((1, 8)), 1.0)
        for sev in ("hold", "true"):
            with self.assertRaises(ValueError):
                ev.RidgePredictor(model, False, "ridge", sev).rollout(ep, np.array([40]), 2, ep.bench.actions)
        ev.RidgePredictor(model, False, "ridge", "none").rollout(ep, np.array([40]), 2, ep.bench.actions)


class HarnessTests(unittest.TestCase):
    def test_start_sets_equal_bench_v1(self) -> None:
        ep = synth(4, rows=1200)
        for h in (24, 168):
            self.assertEqual(ev.any_hour_starts(ep, h).tolist(), bv1.rollout_starts(ep.rows, h, True))
            self.assertEqual(ev.bench_starts(ep, h).tolist(), bv1.rollout_starts(ep.rows, h, False))

    def test_bias_removed_score_removes_a_per_episode_constant_only(self) -> None:
        eps = [synth(s) for s in (5, 6, 7)]
        std = c.split_stds({"test": eps})["test"]
        out = ev.t2(OffsetPredictor(), eps, std, 24, any_hour=True, bias_removed=True)
        self.assertGreater(out["mean"], 0.0)
        self.assertLess(out["bias_removed"]["mean"], 1e-12)
        # A predictor whose error moves within the episode keeps a non-zero bias-removed score.
        pers = ev.t2(ev.Persistence(), eps, std, 24, any_hour=True, bias_removed=True)
        self.assertGreater(pers["bias_removed"]["mean"], 0.0)
        # The definition: per episode and variable, error minus its mean over the episode's scored starts.
        preds, trues = [np.array([[1.0, 2.0], [3.0, 2.0]])], [np.array([[0.0, 0.0], [0.0, 4.0]])]
        np.testing.assert_allclose(ev.demeaned_errors(preds, trues), [[-1.0, 2.0], [1.0, -2.0]])
        self.assertEqual(ev.bias_removed_score(preds, trues, np.array([1.0, 2.0]))["per_var"], [1.0, 1.0])

    def test_persistence_vectorized_equals_its_step(self) -> None:
        ep = synth(8)
        starts = np.array([40, 100, 500])
        fast = ev.Persistence().rollout(ep, starts, 30, ep.bench.actions).v
        slow = ev.StepPredictor.rollout(ev.Persistence(), ep, starts, 30, ep.bench.actions).v
        np.testing.assert_array_equal(fast, slow)

    def test_evaluate_end_to_end_on_schema_4(self) -> None:
        train = [synth(s, "train") for s in range(6)]
        val = [synth(s, "val") for s in (10, 11)]
        test = {"test": [synth(s) for s in (20, 21, 22)]}
        stds = c.split_stds(test)
        fit = cl.CalendarFit(train, split="game_v16")
        ridge, lam, _ = ev.fit_ridge_without_severity(train, val, action_aware=False)
        for pred in (ev.Persistence(), cl.CalendarPredictor(fit, "apfit_abs"), cl.CalendarPredictor(fit, "an5"), ridge):
            out = ev.evaluate(pred, test, stds, cf_test=None, t3_split=None, tasks=("T1", "T2", "T2_any"),
                              bias_removed=True)
            self.assertEqual(set(out), {"T1", "T2", "T2_any"})
            for key in ("T2", "T2_any"):
                for h in ("24", "168"):
                    node = out[key][h]["test"]
                    self.assertEqual(len(node["per_var"]), 8)
                    self.assertTrue(np.isfinite(node["mean"]) and np.isfinite(node["bias_removed"]["mean"]))
            self.assertTrue(np.isfinite(out["T1"]["test"]["mean"]))
        with self.assertRaises(ValueError):
            ev.evaluate(ev.Persistence(), test, stds, cf_test=None, t3_split=None, tasks=("T1", "T3"))
        with self.assertRaises(ValueError):
            ev.evaluate(ev.Persistence(), test, stds, cf_test=None, tasks=("T4",))

    def test_calendar_selection_on_bias_removed_tasks(self) -> None:
        tasks = cl.tasks_for(c.dataset("p10_v16"))
        self.assertNotIn("T3", tasks)
        self.assertNotIn("T3b", tasks)
        self.assertEqual(tasks["T2anybr_168"], (("T2_any", "168"), ("val",), "bias_removed"))
        val = {}
        for i, m in enumerate(cl.MEMBERS):
            node = {"mean": 1.0 + i, "bias_removed": {"mean": 10.0 - i}}
            val[m] = {"T1": {"val": {"mean": 1.0 + i}},
                      "T2": {h: {"val": node} for h in ("24", "168")}, "T2_any": {h: {"val": node} for h in ("24", "168")}}
        sel = cl.select(val, tasks)
        self.assertEqual(sel["T2any_168"]["cal_best"], "cal_abs")
        self.assertEqual(sel["T2anybr_168"]["cal_best"], "an5_off")
        self.assertEqual(sel["T2anybr_168"]["cal_best_anom"], "an5_off")
        self.assertEqual(sel["T2anybr_168"]["cal_best_anom_fair"], "apfit_year")
        self.assertNotIn("cal_best_anom", sel["T2any_168"])
        with self.assertRaises(ValueError):
            cl.CalendarFit([synth(0, "train")])          # split defaults to A_obs: a pilot episode is refused


class FinalPlanTests(unittest.TestCase):
    def test_final_reads_the_dataset_from_config(self) -> None:
        from wm1 import m1_gru
        ds, groups, kw = m1_gru.test_plan({"variant": "sev-blind_action-aware"})        # a WM-1 run: no dataset key
        self.assertEqual((ds.name, groups, kw), ("v0", ["A_test", "B_test", "OOD_v12", "OOD_long", "C_test"],
                                                 {"t3_split": "A_test"}))
        ds, groups, kw = m1_gru.test_plan({"variant": "sev-blind_action-blind", "dataset": "p10_v16"})
        self.assertEqual((ds.name, groups), ("p10_v16", ["test"]))
        self.assertEqual(kw, {"t3_split": None, "tasks": ("T1", "T2", "T2_any"), "bias_removed": True})


if __name__ == "__main__":
    unittest.main()
