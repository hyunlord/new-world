"""Unit tests for evaluate_pass16.py (Pass 16)."""
import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import evaluate_pass16 as ev  # noqa: E402


def person(pid, t, founder=True, parents=None, alive=True):
    return {
        "id": pid,
        "founder": founder,
        "born_tick": None if founder else 100,
        "temperament": t,
        "parent_temperaments": parents or [None, None],
        "alive_end": alive,
        "ever_adult": True,
    }


def agent(pid, t, idle, bed, social, food, wood=(0, 0), days=20.0):
    return {
        "id": pid,
        "temperament": t,
        "adult_days": days,
        "idle_dist_sum": idle[0],
        "idle_samples": idle[1],
        "bed_minutes": bed,
        "social_completed": social,
        "deposit_food_sum": food[0],
        "deposit_food_count": food[1],
        "deposit_wood_sum": wood[0],
        "deposit_wood_count": wood[1],
        "departed": False,
    }


class Pass16Statistics(unittest.TestCase):
    def test_average_ranks_and_spearman(self):
        self.assertEqual(list(ev.rank_avg([10, 20, 20, 30])), [1.0, 2.5, 2.5, 4.0])
        self.assertAlmostEqual(ev.spearman([1, 2, 3, 4], [10, 20, 30, 40]), 1.0)
        self.assertAlmostEqual(ev.spearman([1, 2, 3, 4], [4, 3, 2, 1]), -1.0)
        # Ties: rho on average ranks equals Pearson on them.
        self.assertAlmostEqual(ev.spearman([1, 2, 2, 3], [1, 2, 3, 4]), 0.9486832980505138)
        self.assertTrue(ev.spearman([1, 2], [1, 2]) != ev.spearman([1, 2], [1, 2]))

    def test_slope_and_bootstrap_are_deterministic(self):
        self.assertAlmostEqual(ev.ols_slope([0, 1, 2, 3], [1, 1.5, 2, 2.5]), 0.5)
        payloads = [[1.0, 2.0], [3.0], [5.0, 7.0], [2.0]]
        stat = lambda ps: sum(v for p in ps for v in p) / sum(len(p) for p in ps)
        first = ev.bootstrap(payloads, stat, b=300)
        self.assertEqual(first, ev.bootstrap(payloads, stat, b=300))
        self.assertAlmostEqual(first[0], 20.0 / 6.0)
        self.assertLessEqual(first[1], first[0])
        self.assertGreaterEqual(first[2], first[0])

    def test_t1_pairs_apply_the_minimums(self):
        t = {"ns": 700, "ha": 200, "rd": 900, "p": 600}
        world = {"agents": [
            agent(1, t, (500, 100), [1200] * 12, 40, (50, 12), (20, 10)),
            agent(2, t, (500, 49), [1200] * 9, 40, (50, 9), (20, 9)),
            agent(3, t, (500, 100), [1200] * 12, 40, (50, 12), days=9.9),
        ]}
        self.assertEqual(ev.t1_pairs(world, "ns"), [(700, 5.0)])
        self.assertEqual(ev.t1_pairs(world, "ha"), [(200, 1200.0)])
        self.assertEqual(ev.t1_pairs(world, "rd"), [(900, 2.0), (900, 2.0)])
        self.assertEqual(ev.t1_pairs(world, "p_food"), [(600, 50 / 12)])
        self.assertEqual(ev.t1_pairs(world, "p_wood"), [(600, 2.0)])

    def test_departure_difference_uses_completed_fissions_only(self):
        people = {i: person(i, {"ns": ns, "ha": 0, "rd": 0, "p": 0}) for i, ns in [(1, 900), (2, 800), (3, 100), (4, 200), (5, 999)]}
        world = {"people": people, "fissions": [
            {"status": "completed", "members": [1, 2], "eligible": [1, 2, 3, 4]},
            {"status": "cancelled", "members": [5], "eligible": [5, 3]},
        ]}
        dep, stay = ev.departure_payload(world)
        self.assertEqual((sorted(dep), sorted(stay)), ([800, 900], [100, 200]))
        self.assertEqual(ev.dep_diff([(dep, stay)]), 700.0)

    def test_inheritance_block(self):
        parents = [{"ns": 800, "ha": 800, "rd": 800, "p": 800}, {"ns": 600, "ha": 600, "rd": 600, "p": 600}]
        people = {
            1: person(1, {"ns": 500, "ha": 500, "rd": 500, "p": 500}),
            2: person(2, {"ns": 300, "ha": 300, "rd": 300, "p": 300}),
            3: person(3, {"ns": 600, "ha": 600, "rd": 600, "p": 600}, founder=False, parents=parents),
            4: person(4, {"ns": 450, "ha": 450, "rd": 450, "p": 450}, founder=False, parents=[parents[1], parents[1]]),
            5: person(5, {"ns": 400, "ha": 400, "rd": 400, "p": 400}, founder=False, parents=[{"ns": 400, "ha": 400, "rd": 400, "p": 400}] * 2),
        }
        t4 = ev.t4_block({0: {"people": people}})
        self.assertEqual(t4["births_two_parents"], 3)
        # mid-parents 700, 600, 400 → children 600, 450, 400: slope 2/3 + …
        self.assertAlmostEqual(t4["slope"]["ns"]["value"], ev.ols_slope([700, 600, 400], [600, 450, 400]))

    def test_end_to_end_on_files(self):
        with tempfile.TemporaryDirectory() as d:
            for scene in ("game_v20", "game_v20_inert"):
                os.makedirs(os.path.join(d, scene))
                runs = []
                for seed in (0, 1, 2):
                    base = os.path.join(d, scene, f"{scene}_s{seed}")
                    agents, people = [], []
                    for i in range(12):
                        v = 80 * i + 10 * seed
                        t = {"ns": v, "ha": v, "rd": v, "p": v}
                        lean = 1.0 if scene == "game_v20" else 0.0
                        agents.append(agent(i, t, (int(100 * (5 + lean * v / 100)), 100), [int(1300 - lean * v / 10)] * 12, int(20 + lean * v / 10), (int(40 + lean * v / 10), 10)))
                        people.append(person(i, t))
                    for name, rows in (("agents", agents), ("people", people), ("fissions", [])):
                        with open(f"{base}_{name}.jsonl", "w") as f:
                            f.write("".join(json.dumps(r) + "\n" for r in rows))
                    with open(f"{base}_trace_meta.json", "w") as f:
                        json.dump({"born_event_misses": 0}, f)
                    runs.append({"seed": seed, "alive_daily": [30, 31], "death_census": {"counts_by_cause": {"old_age": 3}}, "pass11": {"fissions": 0, "first_fission_day": None}})
                with open(os.path.join(d, scene, "summary.json"), "w") as f:
                    json.dump({"runs": runs}, f)
            result = ev.evaluate(os.path.join(d, "game_v20"), os.path.join(d, "game_v20_inert"), tuning=True)
            self.assertEqual(result["verdicts"]["T1-ns"], "PASS")
            self.assertEqual(result["verdicts"]["T1-ha"], "PASS")
            self.assertEqual(result["verdicts"]["T1-rd"], "PASS")
            self.assertEqual(result["verdicts"]["T3"], "PASS")
            self.assertNotIn("T1-dep", result["verdicts"])
            self.assertIn("T1 / T2", ev.render(result))


if __name__ == "__main__":
    unittest.main()
