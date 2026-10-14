"""Tests of dose_map_p14.py on tiny synthetic schema-6 episodes and synthetic pairs."""
import argparse
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dose_map_p14 as dm  # noqa: E402

D = 1440
ROWS = 120 * D // 60 + 1
GLOBAL = ["tick", "alive"] + [f"deaths_{c}" for c in dm.CAUSES]


def tick(day: int) -> int:
    return (day - 1) * D - 120


def write_episode(root: Path, name: str, seed: int, k: int, departs: list[int], alive118: float = 40.0,
                  exposure_at: int | None = None, schedule=None, interventions=None, extra_settlement: bool = False):
    path = root / name
    path.mkdir(parents=True)
    g = np.zeros((ROWS, len(GLOBAL)), dtype=np.float32)
    g[:, 0] = np.arange(ROWS) * 60
    g[:, 1] = alive118
    if exposure_at is not None:
        g[exposure_at // 60:, GLOBAL.index("deaths_exposure")] = 1.0
    slots = 1 + len(departs)
    s = np.full((ROWS, slots, 2), np.nan, dtype=np.float32)
    s[:, 0, 0] = 30
    s[:, 0, 1] = -1
    for i, t in enumerate(departs):
        s[t // 60 + 100:, i + 1, 0] = 10
        s[t // 60 + 100:, i + 1, 1] = t
    arrays = {}
    for fname, arr in (("global.npy", g), ("settlements.npy", s)):
        np.save(path / fname, arr)
        arrays[fname] = {"dtype": "float32", "shape": list(arr.shape),
                         "sha256": hashlib.sha256((path / fname).read_bytes()).hexdigest()}
    meta = {"schema_version": 6, "seed": seed, "scene": "game_v19", "episode_id": name.split("_")[3],
            "noise_fork": {"tick": 14_280, "replicate": k}, "schedule": schedule, "arrays": arrays,
            "fields": {"global.npy": [{"name": n} for n in GLOBAL],
                       "settlements.npy": [{"name": "members"}, {"name": "depart_tick"}]}}
    (path / "meta.json").write_text(json.dumps(meta))
    (path / "interventions.json").write_text(json.dumps(interventions or []))


def p(delta=None, direction="tie", settle=False, alive=0.0, d35=0.0, d120=0.0):
    return {"delta": delta, "direction": direction, "settle_diff": {60: settle, 90: settle, 118: settle},
            "alive_signed": alive, "deaths35": [0, 0, 0, 0, d35], "deaths120": [0, 0, 0, 0, d120]}


class Pairs(unittest.TestCase):
    def facts(self, departs, alive=40.0):
        deaths = np.zeros((ROWS, 5))
        return {"departs": departs, "settle": {60: 1, 90: 1, 118: 1 + len(departs)}, "alive118": alive, "deaths": deaths}

    def test_direction_rules(self):
        c = tick(20)
        both = dm.pair(self.facts([tick(25)]), self.facts([tick(30)]), c)
        self.assertEqual((both["delta"], both["direction"]), (-5, "earlier"))
        only_arm = dm.pair(self.facts([tick(25)]), self.facts([]), c)
        self.assertEqual((only_arm["delta"], only_arm["direction"]), (None, "earlier"))
        only_fact = dm.pair(self.facts([]), self.facts([tick(25)]), c)
        self.assertEqual(only_fact["direction"], "later")
        neither = dm.pair(self.facts([]), self.facts([]), c)
        self.assertEqual(neither["direction"], "tie")
        tie = dm.pair(self.facts([tick(25)]), self.facts([tick(25)]), c)
        self.assertEqual((tie["delta"], tie["direction"]), (0, "tie"))

    def test_departures_before_the_cast_are_ignored_and_the_cast_tick_counts(self):
        c = tick(20)
        r = dm.pair(self.facts([tick(15), tick(30)]), self.facts([tick(15), tick(28)]), c)
        self.assertEqual(r["delta"], 2)
        r = dm.pair(self.facts([c]), self.facts([tick(28)]), c)
        self.assertEqual(r["day_a"], 20)

    def test_deaths_windows(self):
        a, f = self.facts([]), self.facts([])
        a["deaths"][tick(30) // 60:, 4] = 2.0   # inside 35 days of day 20
        a["deaths"][tick(80) // 60:, 0] = 1.0   # after the window
        r = dm.pair(a, f, tick(20))
        self.assertEqual(r["deaths35"], [0, 0, 0, 0, 2.0])
        self.assertEqual(r["deaths120"], [1.0, 0, 0, 0, 2.0])

    def test_stats_right_share_enr_and_noise_floor(self):
        pairs = [p(-4, "earlier"), p(None, "earlier"), p(2, "later"), p(0, "tie")]
        noise = [p(1, "later")] * 8
        st = dm.stats(pairs, noise, "spoil_food")
        self.assertAlmostEqual(st["right_share"], 0.5)
        self.assertEqual(st["n_both"], 3)
        self.assertAlmostEqual(st["mean_abs"], 2.0)
        self.assertAlmostEqual(st["share_gt3"], 1 / 3)
        self.assertAlmostEqual(st["enr"], 2.0)
        st = dm.stats(pairs, [p(0, "tie")] * 8, "gift_food")
        self.assertAlmostEqual(st["enr"], 4.0)  # noise mean 0 -> floor 0.5
        self.assertAlmostEqual(st["right_share"], 0.25)
        st = dm.stats(pairs, [p(1, "later")] * 7, "gift_food")
        self.assertTrue(np.isnan(st["enr"]))  # fewer than 8 noise pairs where both depart
        st = dm.stats(pairs, noise, "rain")
        self.assertTrue(np.isnan(st["right_share"]))

    def test_summarize_bootstrap_is_reproducible_and_brackets_point(self):
        seeds = list(range(10))
        pb = {s: [p(-2 - s % 3, "earlier"), p(None, "later" if s % 2 else "earlier")] for s in seeds}
        nb = {s: p(1, "later") for s in seeds}
        a = dm.summarize(seeds, pb, nb, "spoil_food", b=200)
        b = dm.summarize(seeds, pb, nb, "spoil_food", b=200)
        self.assertEqual(a["ci90"], b["ci90"])
        lo, hi = a["ci90"]["right_share"]
        self.assertLessEqual(lo, a["point"]["right_share"])
        self.assertGreaterEqual(hi, a["point"]["right_share"])

    def test_paired_right_diff(self):
        seeds = list(range(6))
        low = {s: [p(-1, "earlier"), p(-1, "earlier")] for s in seeds}
        eve = {s: [p(0, "tie"), p(-1, "earlier")] for s in seeds}
        r = dm.paired_right_diff(seeds, low, eve, "spoil_food", b=100)
        self.assertAlmostEqual(r["diff"], 0.5)
        self.assertTrue(r["passed"])


class EndToEnd(unittest.TestCase):
    def test_run_scores_checks_and_unforked(self):
        with tempfile.TemporaryDirectory() as d:
            root, sched = Path(d) / "rec", Path(d) / "sched"
            root.mkdir()
            sched.mkdir()
            cast = tick(20)
            files = []
            for s in (1, 2):
                rows = [{"tick": cast, "kind": "spoil_food", "settlement": "home", "amount": 60}]
                text = json.dumps(rows) + "\n"
                (sched / f"s{s}_sp60L.json").write_text(text)
                files.append({"file": f"s{s}_sp60L.json", "seed": s, "arm": "sp60L", "cast_tick": cast,
                              "sha256": hashlib.sha256(text.encode()).hexdigest()})
                for k in (0, 1):
                    write_episode(root, f"game_v19_s{s}_none_r{k}", s, k, [tick(30 + k)])
                    recs = [{"tick": cast, "kind": "spoil_food", "settlement": 1, "amount": 60, "applied": 50, "soaked_wood": 0}]
                    write_episode(root, f"game_v19_s{s}_sp60L_r{k}", s, k, [tick(26)], alive118=38.0,
                                  exposure_at=tick(25), schedule=rows, interventions=recs)
            (sched / "manifest.json").write_text(json.dumps({"arms": {"sp60L": ["spoil_food", 60, "L", 1]}, "files": files}))
            unf = Path(d) / "unforked"
            unf.mkdir()
            write_episode(unf, "game_v19_s1_none", 1, 0, [tick(30)])
            a = argparse.Namespace(root=str(root), scene="game_v19", seeds="1..3", replicates=2, fork_tick=14_280,
                                   manifest=[str(sched / "manifest.json")], arms=None, prereg="sp60L", probe_summary=None,
                                   unforked=str(unf), label="Stage T")
            r = dm.run(a)
            self.assertEqual(r["checks"]["errors"], [])
            arm = r["arms"]["sp60L"]["point"]
            self.assertEqual(arm["right_share"], 1.0)
            self.assertEqual(arm["n_both"], 4)
            self.assertAlmostEqual(arm["mean_abs"], 4.5)
            self.assertEqual(arm["noise_n_both"], 2)
            self.assertIsNone(arm["enr"])  # 2 < 8 noise pairs
            self.assertAlmostEqual(arm["alive_signed"], -2.0)
            self.assertAlmostEqual(arm["deaths35_exposure"], 1.0)
            self.assertEqual(r["arms"]["sp60L"]["applied"]["short_rows"], 4)
            self.assertFalse(r["k"]["K2"]["sp60L"]["passed"])
            u = r["unforked"]["1"]
            self.assertTrue(u["arrays_equal"])
            self.assertEqual(u["meta_keys_differing"], [])
            self.assertIn("| `sp60L` |", dm.markdown(r))
            # A schedule row without its record is an error.
            bad = root / "game_v19_s2_sp60L_r1" / "interventions.json"
            bad.write_text("[]")
            self.assertTrue(dm.run(a)["checks"]["errors"])


if __name__ == "__main__":
    unittest.main()
