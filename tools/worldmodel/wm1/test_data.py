"""WM-1 data pipeline tests on the repository's sample episodes and synthetic edits (GOAL.md 6.1, 4.2).

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm_data import load_episode
from wm1 import data as d

SAMPLE = Path(__file__).resolve().parents[3] / "docs/worldmodel/sample"


def sample_episode(seed: int = 0, role: str = "train", interventions=None) -> d.EpisodeData:
    ep = load_episode(SAMPLE / f"game_v13_s{seed}_none", arrays=["global.npy", "settlements.npy"])
    if interventions is not None:
        ep.interventions = interventions
    return d.from_episode(ep, "A_obs", role, seed, "none")


class InputFieldTests(unittest.TestCase):
    """Each variant's input field list, asserted exactly (GOAL.md 4.2)."""

    def test_input_fields_of_each_variant(self) -> None:
        state_blind = (
            "g.alive", "g.children", "g.elders", "g.adults", "g.cold_members", "g.deadwood_total",
            "g.food_on_bushes_total", "g.adults_working", "g.idle_adults", "g.settlements_under_rain",
            "g.cold_snap_nights_left", "g.cold_snap_tonight", "d.births", "d.deaths_starvation",
            "d.deaths_dehydration", "d.deaths_combat", "d.deaths_old_age", "d.meals_from_store_total",
            "d.ground_meals_total", "d.forage_trips_total", "d.wood_trips_total", "d.wood_gathered_total",
            "d.urgent_wood_trips_total", "d.wood_retargets_total", "d.cold_member_hours_total",
            "d.interventions_applied_total", "s.members", "s.food_stock", "s.wood_stock", "s.wood_in_hand",
            "s.fire_lit", "s.rain_ticks_left", "s.present")
        cal = tuple(f"hour_{h}" for h in range(24)) + ("season_-1", "season_0", "season_1", "season_2", "season_3",
                                                       "day_of_season")
        acts = ("act.gift_food", "act.spoil_food", "act.gift_wood", "act.rot_wood", "act.rain", "act.cold_snap",
                "act.windfall")
        sev = ("sev_mild", "sev_normal", "sev_harsh", "sev_none")
        self.assertEqual(d.Variant(False, True).input_names(), state_blind + cal + acts)
        self.assertEqual(d.Variant(False, False).input_names(), state_blind + cal)
        self.assertEqual(d.Variant(True, True).input_names(), state_blind + ("s.wood_target",) + cal + acts + sev)
        self.assertEqual(d.Variant(True, False).input_names(), state_blind + ("s.wood_target",) + cal + sev)

    def test_no_variant_reads_hidden_or_severity_revealing_fields(self) -> None:
        for v in d.VARIANTS.values():
            names = v.input_names()
            for bad in ("lifespan", "latent", "winter_severity", "wood_winter", "regen_acc", "formation"):
                self.assertFalse(any(bad in n for n in names), (v.name, bad))
            for bad in ("g.tick", "g.day_index", "g.year", "g.minute_of_day", "g.season", "g.day_of_season"):
                self.assertNotIn(bad, names)
            self.assertEqual("s.wood_target" in names, v.sev_on)
            self.assertEqual(any(n.startswith("sev_") for n in names), v.sev_on)
            self.assertEqual(any(n.startswith("act.") for n in names), v.action_aware)
        self.assertEqual(d.MAIN, d.Variant(sev_on=False, action_aware=True))

    def test_severity_blind_state_has_no_wood_target_column(self) -> None:
        ep = sample_episode()
        blind = ep.state(d.Variant(False, True))
        on = ep.state(d.Variant(True, True))
        self.assertEqual(on.shape[1], blind.shape[1] + 1)
        np.testing.assert_array_equal(on[:, -1], np.nan_to_num(ep.table[:, d.ALL_STATE.index("s.wood_target")]))
        # The blind state equals the severity-on state without that column.
        np.testing.assert_array_equal(on[:, :-1], blind)


class FeatureTests(unittest.TestCase):
    def test_shapes_and_bench_consistency(self) -> None:
        ep = sample_episode()
        self.assertEqual(ep.table.shape, (ep.rows, len(d.ALL_STATE)))
        self.assertEqual(ep.calendar.shape, (ep.rows, len(d.CALENDAR)))
        self.assertFalse(np.isnan(ep.table).any())
        lay = d.Layout(d.MAIN)
        # V reconstructed from the state equals the bench's V on every row.
        np.testing.assert_allclose(ep.state(d.MAIN)[:, lay.v], ep.bench.v)
        # Presence flag: row 0 has no settlement yet.
        present = ep.table[:, d.ALL_STATE.index("s.present")]
        self.assertEqual(present[0], 0.0)
        self.assertTrue((present[1:] == 1.0).all())
        # Tallies enter as hourly differences that sum back to the cumulative column.
        g = load_episode(SAMPLE / "game_v13_s0_none", arrays=["global.npy"])
        cum = g.column("global.npy", "wood_trips_total")
        np.testing.assert_allclose(np.cumsum(ep.table[:, d.ALL_STATE.index("d.wood_trips_total")]), cum - cum[0])
        # Calendar one-hots are one-hot.
        np.testing.assert_array_equal(ep.calendar[:, :24].sum(1), 1)
        np.testing.assert_array_equal(ep.calendar[:, 24:29].sum(1), 1)

    def test_action_alignment(self) -> None:
        """An intervention of tick t is a feature of row t and first shows in row t+1 (its change target)."""
        tick = 600  # row 10
        ep = sample_episode(interventions=[{"tick": tick, "kind": "gift_wood", "settlement": 1, "amount": 50,
                                            "applied": 50}])
        r = tick // 60
        self.assertEqual(ep.bench.actions[r, b.KINDS.index("gift_wood")], 50)
        self.assertEqual(ep.bench.actions.sum(), 50)
        np.testing.assert_array_equal(d.actions_of(ep.interventions, ep.bench.tick), ep.bench.actions)
        # Make the wood stock jump in row t+1 as the simulation would; the jump is row t's change target.
        wood = d.ALL_STATE.index("s.wood_stock")
        ep.table[r + 1:, wood] += 50
        norm = d.Normalizer.fit([ep], d.MAIN)
        prep = d.prepare(ep, d.MAIN, norm)
        cw = d.MAIN.continuous_names.index("s.wood_stock")
        self.assertEqual(prep.actions[r, b.KINDS.index("gift_wood")], 50 / 80)
        self.assertAlmostEqual(prep.dtarget[r, cw] * norm.dstd[cw], ep.table[r + 1, wood] - ep.table[r, wood])
        rng = np.random.default_rng(0)
        win = d.sample_windows([prep], 24, 200, rng)
        hits = np.argwhere(win["actions"][:, :24, b.KINDS.index("gift_wood")] > 0)
        self.assertGreater(len(hits), 0)
        sw = d.MAIN.state_names.index("s.wood_stock")
        jump_rows = ep.table[r + 1, wood] - ep.table[r, wood]
        for i, j in hits:
            # Same position j: the action of row t and the change from row t to t+1 (which carries the gift).
            change = float(win["state"][i, j + 1, sw] - win["state"][i, j, sw])
            self.assertAlmostEqual(change, jump_rows, places=3)
            self.assertAlmostEqual(float(win["dtarget"][i, j, cw]) * norm.dstd[cw], change, places=3)
        # Action-blind variants have no action input at all.
        self.assertEqual(d.prepare(ep, d.Variant(False, False), d.Normalizer.fit([ep], d.Variant(False, False))).actions.shape[1], 0)


class NormalizationTests(unittest.TestCase):
    def test_statistics_come_from_train_rows_only(self) -> None:
        train = [sample_episode(0, "train")]
        val = sample_episode(1, "val")
        norm = d.Normalizer.fit(train, d.MAIN)
        state = train[0].state(d.MAIN)
        np.testing.assert_allclose(norm.mean, state.mean(0))
        self.assertEqual(norm.fit_rows, train[0].rows)
        # Changing a validation episode does not change the statistics; fitting on it is refused.
        val.table[:] = 1e6
        np.testing.assert_allclose(d.Normalizer.fit(train, d.MAIN).mean, norm.mean)
        with self.assertRaises(ValueError):
            d.Normalizer.fit(train + [val], d.MAIN)
        self.assertTrue((norm.std > 0).all() and (norm.dstd > 0).all())
        again = d.Normalizer.from_json(norm.to_json())
        np.testing.assert_array_equal(again.dstd, norm.dstd)

    def test_windows_are_contiguous_train_rows(self) -> None:
        ep = sample_episode()
        norm = d.Normalizer.fit([ep], d.MAIN)
        prep = d.prepare(ep, d.MAIN, norm)
        win = d.sample_windows([prep], 48, 16, np.random.default_rng(1))
        self.assertEqual(win["x_state"].shape, (16, 49, len(d.MAIN.state_names)))
        self.assertEqual(win["dtarget"].shape, (16, 48, len(d.MAIN.continuous_names)))
        self.assertEqual(win["fire_next"].shape, (16, 48))
        # The fire target of position j is the fire state of the window's row j+1.
        np.testing.assert_array_equal(win["fire_next"], win["state"][:, 1:, d.MAIN.state_names.index("s.fire_lit")])


if __name__ == "__main__":
    unittest.main()
