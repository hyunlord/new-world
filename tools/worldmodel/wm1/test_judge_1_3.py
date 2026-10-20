"""judge_1_3.py on synthetic result files (METRICS_1_3.md 5): verdicts, Y3 differences, missing files as n/a.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
from wm1 import calcore as cc
from wm1 import judge_1_3 as j

SEEDS = list(range(30480, 30500))


def t2_node(per_world: np.ndarray) -> dict:
    """A T2 test node whose pooled numbers are the means of its per-world numbers (equal start counts)."""
    pw = {str(s): {"per_var": row.tolist(), "mean": float(row.mean()), "n": 384,
                   "bias_removed": {"per_var": (0.8 * row).tolist(), "mean": float(0.8 * row.mean())}}
          for s, row in zip(SEEDS, per_world)}
    m = per_world.mean(axis=0)
    return {"mean": float(m.mean()), "per_var": m.tolist(), "n": 384 * len(SEEDS),
            "bias_removed": {"mean": float(0.8 * m.mean()), "per_var": (0.8 * m).tolist()}, "per_world": pw}


def result(per_world: np.ndarray) -> dict:
    return {key: {h: {"test": t2_node(per_world)} for h in ("24", "168")} for key in ("T2", "T2_any")}


def write(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj))


class JudgeTests(unittest.TestCase):
    def test_verdicts_and_y3_on_synthetic_files(self) -> None:
        rng = np.random.default_rng(0)
        base = 0.2 + 0.05 * rng.random((len(SEEDS), 8))
        with tempfile.TemporaryDirectory() as tmp:
            runs = Path(tmp)
            sel = {t: {"cal_best": "an5", "cal_best_fair": "apfit_year"} for t in ("T2any_168", "T2any_24", "T2_168", "T2_24")}
            sel |= {t: {"cal_best": "cal_abs", "cal_best_fair": "cal_year", "cal_best_anom": "cal_abs"}
                    for t in ("T2anybr_168", "T2anybr_24", "T2br_168", "T2br_24")}
            for ds in j.DATASETS:
                write(runs / f"calendar_val_{ds}_aaaa" / "calendar_val.json", {"selection": sel, "dataset": ds})
                scale = 1.0 if ds != "w13_v16_n80" else 1.1
                write(runs / f"calendar_{ds}_bbbb" / "calendar.json",
                      {"test": {m: result(scale * base * (1.0 if m == "an5" else 1.2)) for m in cc.MEMBERS},
                       "references": {"persistence": result(2 * base), "ridge": result(1.5 * base)}})
            for r in j.RUNS["w13_v16_n400"]:
                write(runs / r / "metrics.json", result(0.7 * base))
            for r in j.RUNS["w13_v16_n80"]:
                write(runs / r / "metrics.json", result(0.9 * base))
                write(runs / f"{r}_short" / "metrics.json", result(0.95 * base))
            write(runs / j.RUNS["w13_v17_n400"][0] / "metrics.json", result(base + 0.05))
            out = j.run(runs, "aaaa", "bbbb")
            md = j.markdown(out)
        y = out["Y"]
        self.assertEqual(y["Y1"]["verdict"], "KEEP")
        self.assertEqual(y["Y1"]["seeds_present"], 3)
        self.assertAlmostEqual(y["Y1"]["ratio"], 0.7, delta=1e-12)
        # Bias-removed: model 0.8 * 0.7 * base vs cal_abs's 0.8 * 1.2 * base.
        self.assertEqual(y["Y2"]["against"], "cal_best_anom = cal_abs")
        self.assertEqual(y["Y2"]["verdict"], "KEEP")
        self.assertEqual(y["Y4"]["verdict"], "NONE")
        self.assertEqual(y["Y4"]["seeds_present"], 1)
        mean = float(base.mean())
        self.assertAlmostEqual(y["Y3"]["delta_steps"]["point"], -0.05 * mean, delta=1e-12)
        self.assertAlmostEqual(y["Y3"]["delta_data"]["point"], -0.2 * mean, delta=1e-12)
        self.assertAlmostEqual(y["Y3"]["delta_cal"]["point"], -0.1 * mean, delta=1e-12)
        self.assertAlmostEqual(y["Y3"]["delta_data_minus_delta_cal"]["point"], -0.1 * mean, delta=1e-12)
        self.assertTrue(y["Y3"]["scale_helps_model_more"])
        self.assertNotIn("delta_steps_xworld", y["Y3"])                  # WM-1.2 runs not present
        self.assertEqual(y["Y1_n200"]["verdict"], "n/a")
        self.assertEqual(y["Y6"]["verdict"], "n/a (dropped or missing)")
        self.assertEqual(y["Y5"]["T2_any_168"]["80"]["cal_best"]["member"], "an5")
        self.assertIn("| Y1 |", md)
        sk = out["skills"]["w13_v16_n400"]["raw"]["food"]["cal_best"]
        self.assertAlmostEqual(sk["point"], 0.3, delta=1e-12)


if __name__ == "__main__":
    unittest.main()
