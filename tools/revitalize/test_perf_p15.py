"""Tests of the Pass 15 scaling analysis (perf_p15.py)."""
import json
import math
import os
import random
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import perf_p15 as p  # noqa: E402


def fake_run(stem, mode, seed, agents, alive, tick_ms, systems, feed="fed", per_day=None, side=64, snapshot=None):
    """A minimal worldsim_perf run document."""
    steady = 1000.0
    rows = [{"name": n, "interval": 1, "priority": i, "calls": 1000, "total_ms": us * 1000 / 1000.0,
             "mean_us_per_call": us, "share_of_tick_time": us / 1000.0 / tick_ms, "max_us": us} for i, (n, us) in enumerate(systems)]
    return {"_stem": stem, "_dir": "/nonexistent", "mode": mode, "seed": seed, "agents_requested": agents, "feed": feed, "side": side,
            "ticks_per_day": 1440, "mean_alive_0600": alive, "mean_settlements_0600": 1.0, "contended": False,
            "steady_tick_ms_total": tick_ms * steady, "systems": rows, "per_day": per_day or [],
            "tick_summary": {"mean_ms": tick_ms, "p999_ms": tick_ms * 2, "max_ms": tick_ms * 3, "n_over_8ms": 0, "n_over_16_67ms": 0},
            "snapshot_timings": snapshot}


class Statistics(unittest.TestCase):
    def test_exact_power_law_gives_the_slope(self):
        xs = [10, 20, 50, 100, 200, 500, 1000]
        for b in (0.5, 1.0, 1.37, 2.0):
            fit = p.loglog(xs, [3.0 * x ** b for x in xs])
            self.assertAlmostEqual(fit["b"], b, delta=1e-9)
            self.assertAlmostEqual(math.exp(fit["a"]), 3.0, delta=1e-9)
            self.assertAlmostEqual(fit["ci"][0], b, delta=1e-9)
            self.assertAlmostEqual(fit["ci"][1], b, delta=1e-9)

    def test_noisy_power_law_lies_inside_the_interval(self):
        rng = random.Random(15)
        xs = [25, 50, 100, 200, 500, 1000, 2000] * 8
        hits = 0
        for trial in range(200):
            ys = [x ** 1.5 * math.exp(rng.gauss(0, 0.1)) for x in xs]
            fit = p.loglog(xs, ys)
            hits += fit["ci"][0] <= 1.5 <= fit["ci"][1]
        self.assertGreaterEqual(hits, 180)  # 95 % nominal coverage

    def test_t_quantiles(self):
        self.assertAlmostEqual(p.t_quantile(0.975, 10), 2.228138852, delta=1e-6)
        self.assertAlmostEqual(p.t_quantile(0.975, 1), 12.70620474, delta=1e-5)
        self.assertAlmostEqual(p.t_quantile(0.975, 1000), 1.962339, delta=1e-5)
        self.assertAlmostEqual(p.t_cdf(0.0, 7), 0.5, delta=1e-12)

    def test_classes(self):
        self.assertEqual([p.classify(b) for b in (0.3, 1.19, 1.2, 1.69, 1.7, 2.5)],
                         ["linear or better", "linear or better", "superlinear", "superlinear", "near-quadratic", "near-quadratic"])
        self.assertEqual(p.classify(None), "—")

    def test_nearest_rank_matches_worldsim_perf(self):
        values = list(range(1, 1001))
        self.assertEqual([p.percentile(values, q) for q in (50, 90, 99.9, 100)], [500, 900, 999, 1000])
        self.assertEqual(p.nearest_rank(99.9, 2820), 2818)
        self.assertEqual(p.percentile([7], 99.9), 7)

    def test_sidecar_round_trip(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "x.tick_us.bin")
            ticks = [5000] * 60 + list(range(1, 1001))
            with open(path, "wb") as stream:
                stream.write(struct.pack(f"<{len(ticks)}I", *ticks))
            s = p.summary_from_tick_us(p.read_tick_us(path))
            self.assertEqual(s["ticks"], 1000)
            self.assertAlmostEqual(s["mean_ms"], 0.5005)
            self.assertAlmostEqual(s["p999_ms"], 0.999)


class Analysis(unittest.TestCase):
    def build(self):
        natural, stress = {}, {}
        # Natural: A grows linearly with alive per day, B quadratically (both per day).
        for seed in range(3):
            days = []
            for d, alive in enumerate((20, 40, 80, 120)):
                days.append({"day": d, "ticks": 1440, "alive_0600": alive, "tick_ms": 1440 * 0.1 * alive / 20,
                             "systems_ms": [0.01 * alive, 1e-5 * alive ** 2, 0.001]})
            natural[f"n{seed}"] = fake_run(f"n{seed}", "natural", seed, None, 60, 0.2, [("A", 100), ("B", 50), ("C", 0.1)], feed="none", per_day=days)
        snap = {g: {"median_us": 1.0, "p90_us": 1.0} for g in set(p.PER_FRAME_WIP) | set(p.PER_FRAME_ART) | {"get_tile_causal_history_all_agent_tiles"}}
        snap["context"] = {"agent_tiles": 10, "hearths": 2}
        for seed in range(2):
            for n in (100, 200, 500, 1000):
                # Stress: A per call ~ n^1.0, B ~ n^2.2 (crowding), C tiny.
                a, b = 0.1 * n, 1e-4 * n ** 2.2
                stress[f"s{seed}_{n}"] = fake_run(f"s{seed}_{n}", "stress", 15100 + seed, n, n, (a + b) / 1000.0 + 0.01,
                                                  [("A", a), ("B", b), ("C", 0.001)], snapshot=snap)
        return natural, stress

    def test_slopes_classes_and_verdicts(self):
        natural, stress = self.build()
        out = p.analyse(natural, stress, {})
        s = out["slopes"]
        self.assertAlmostEqual(s["A"]["natural"]["b"], 1.0, delta=1e-9)
        self.assertAlmostEqual(s["B"]["natural"]["b"], 2.0, delta=1e-9)
        self.assertAlmostEqual(s["A"]["stress"]["b"], 1.0, delta=1e-9)
        self.assertAlmostEqual(s["B"]["stress"]["b"], 2.2, delta=1e-9)
        self.assertEqual(s["B"]["natural_class"], "near-quadratic")
        self.assertFalse(s["B"]["crowding_driven"])  # 2.2 < 2.0 + 0.5
        self.assertTrue(s["C"]["minor"])
        self.assertIsNone(s["C"]["natural_class"])
        self.assertTrue(out["p_table"]["P1"]["pass"])
        self.assertIn("B", out["p_table"]["P5"]["natural_superlinear_major"])
        self.assertFalse(out["p_table"]["P5"]["clean"])
        # Pooled p99.9 needs sidecars; without them P2 fails safe.
        self.assertIsNone(out["fed_sizes"]["500"]["pooled_p999_ms"])
        self.assertFalse(out["p_table"]["P2"]["pass"])
        # B3: every timed getter costs 1 µs.
        self.assertAlmostEqual(out["snapshots_500"]["wip"]["single_sum_us"], len(p.PER_FRAME_WIP))
        self.assertAlmostEqual(out["snapshots_500"]["wip"]["callers_sum_us"], 1 + 1 + 2 + 2 + 2 + 6 + 2 + 1 + 1 + 3)
        self.assertAlmostEqual(out["snapshots_500"]["art"]["callers_sum_us"], 2 + 1 + 4 + (13 + 10) + 2 + 2 + 9 + 4 + 5 + 3 + (10 + 4))
        md = p.render(out)
        self.assertIn("| P1 |", md)
        self.assertIn("near-quadratic", md)

    def test_n_max_and_interpolation(self):
        natural, stress = self.build()
        out = p.analyse(natural, stress, {})
        means = {k: v["median_mean_ms"] for k, v in out["fed_sizes"].items()}
        passing = [int(k) for k, v in means.items() if v <= p.B1_MS]
        self.assertEqual(out["n_max"]["largest_passing_size"], max(passing))
        self.assertIsNotNone(out["n_max"]["interpolated_alive"])

    def test_crowding_mark(self):
        natural, stress = self.build()
        for run in stress.values():
            for row in run["systems"]:
                if row["name"] == "A":
                    row["mean_us_per_call"] = 1e-3 * run["mean_alive_0600"] ** 1.8
        out = p.analyse(natural, stress, {})
        self.assertTrue(out["slopes"]["A"]["crowding_driven"])  # 1.8 >= 1.0 + 0.5


class CommandLine(unittest.TestCase):
    def test_end_to_end_on_files(self):
        natural, stress = Analysis().build()
        with tempfile.TemporaryDirectory() as tmp:
            for name, group in (("nat", natural), ("str", stress)):
                os.makedirs(os.path.join(tmp, name))
                for stem, run in group.items():
                    doc = {k: v for k, v in run.items() if not k.startswith("_")}
                    with open(os.path.join(tmp, name, stem + ".json"), "w") as stream:
                        json.dump(doc, stream)
                    with open(os.path.join(tmp, name, stem + ".tick_us.bin"), "wb") as stream:
                        stream.write(struct.pack("<100I", *([100] * 100)))
            out_dir = os.path.join(tmp, "out")
            os.makedirs(out_dir)
            with open(os.path.join(out_dir, "audit.md"), "w") as stream:
                stream.write("## Static audit\n")
            stdout = sys.stdout
            sys.stdout = open(os.devnull, "w")
            try:
                code = p.main(["--natural", os.path.join(tmp, "nat"), "--stress", os.path.join(tmp, "str"), "--out", out_dir])
            finally:
                sys.stdout.close()
                sys.stdout = stdout
            self.assertEqual(code, 0)
            with open(os.path.join(out_dir, "scaling.json")) as stream:
                result = json.load(stream)
            self.assertAlmostEqual(result["fed_sizes"]["500"]["pooled_p999_ms"], 0.1)
            self.assertTrue(result["p_table"]["P2"]["pass"])
            with open(os.path.join(out_dir, "scaling.md")) as stream:
                self.assertIn("## Static audit", stream.read())


if __name__ == "__main__":
    unittest.main()
