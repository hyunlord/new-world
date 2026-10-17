"""WM-1.2 Z rules on synthetic result files (METRICS_1_2.md 3). numpy only.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import tempfile
import unittest
from pathlib import Path

import wm1  # noqa: F401
from wm1 import calcore as cc
from wm1 import judge_1_2 as j


def node(raw: float, br: float) -> dict:
    return {"mean": raw, "per_var": [raw] * 8, "bias_removed": {"mean": br, "per_var": [br] * 8}}


def result(raw: float, br: float) -> dict:
    return {k: {h: {"test": node(raw * (1 if h == "168" else 0.5), br * (1 if h == "168" else 0.5))} for h in ("24", "168")}
            for k in ("T2", "T2_any")}


def write(root: Path, rel: str, obj) -> None:
    (root / rel).parent.mkdir(parents=True, exist_ok=True)
    (root / rel).write_text(json.dumps(obj))


def selection(best: str, anom: str) -> dict:
    out = {}
    for raw, br in j.CELLS.values():
        out[raw] = {"cal_best": best, "cal_best_fair": "apfit_year"}
        out[br] = {"cal_best": anom, "cal_best_fair": "cal_year", "cal_best_anom": anom}
    return {"selection": out}


class ZTests(unittest.TestCase):
    def build(self, root: Path, v16_models: dict, v15_model: tuple) -> None:
        members = {m: result(1.0, 1.0) for m in cc.MEMBERS}
        members["an5"] = result(0.5, 0.4)          # cal_best raw
        members["cal_abs"] = result(0.9, 0.3)      # cal_best_anom
        for ds in ("p10_v16", "p10_v15"):
            write(root, f"calendar_val_{ds}_vvv/calendar_val.json", selection("an5", "cal_abs"))
            write(root, f"calendar_{ds}_ttt/calendar.json",
                  {"test": members, "references": {"persistence": result(2.0, 2.0), "ridge": result(0.7, 0.6)}})
        for run, (raw, br) in v16_models.items():
            write(root, f"{run}/metrics.json", result(raw, br))
        write(root, "m1tf_p10v15_r3_s0/metrics.json", result(*v15_model))

    def test_keep_on_raw_and_none_on_bias_removed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.build(root, {"m1tf_p10v16_r3_s0": (0.35, 0.35), "m1tf_p10v16_r3_s1": (0.38, 0.32),
                              "m1tf_p10v16_r3_s2": (0.39, 0.33)}, (0.6, 0.5))
            r = j.run(root, "vvv", "ttt")
            self.assertEqual(r["Z"]["Z1"]["verdict"], "KEEP")       # mean 0.373 <= 0.8 x 0.5, all < 0.5
            self.assertEqual(r["Z"]["Z1"]["against"], "an5")
            self.assertEqual(r["Z"]["Z2"]["verdict"], "NONE")       # bias-removed 0.333 vs cal_abs 0.3
            self.assertEqual(r["Z"]["Z2"]["against"], "cal_abs")
            self.assertEqual(r["Z"]["Z3"]["verdict"], "NONE")       # v15 0.6 vs 0.5
            self.assertEqual(r["Z"]["Z1b"]["verdict"], "KEEP")
            z4 = r["Z"]["Z4"]["raw"]["per_variable"]["alive"]
            self.assertAlmostEqual(z4["skill_vs_cal_best"], 1 - (0.35 + 0.38 + 0.39) / 3 / 0.5)
            self.assertAlmostEqual(z4["skill_vs_apfit_abs"], 1 - (0.35 + 0.38 + 0.39) / 3 / 1.0)
            self.assertAlmostEqual(r["Z"]["Z4"]["bias_removed"]["per_variable"]["food"]["skill_vs_cal_best_anom"],
                                   1 - (0.35 + 0.32 + 0.33) / 3 / 0.3)
            t = r["tables"]["p10_v16"]["T2_any_168_br"]["rows"]
            self.assertEqual(t["cal_best"]["mean"], 0.3)
            self.assertEqual(t["persistence"]["mean"], 2.0)
            self.assertEqual(t["models_mean"]["seeds"], 3)
            self.assertIn("Z1", j.markdown(r))
            with self.assertRaises(SystemExit):              # the two run directories carry different commits
                j.run(root, "ttt", "vvv")

    def test_weak_and_missing_seed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.build(root, {"m1tf_p10v16_r3_s0": (0.3, 0.2), "m1tf_p10v16_r3_s1": (0.55, 0.25)}, (0.45, 0.3))
            r = j.run(root, "vvv", "ttt")
            self.assertEqual(r["Z"]["Z1"]["n_seeds"], 2)
            self.assertEqual(r["Z"]["Z1"]["verdict"], "WEAK")        # mean 0.425 < 0.5, above 0.4; one seed >= 0.5
            self.assertEqual(r["Z"]["Z2"]["verdict"], "KEEP")
            self.assertEqual(r["Z"]["Z3"]["verdict"], "WEAK")        # 0.45 < 0.5 but > 0.8 x 0.5
            self.assertEqual(r["Z"]["Z5"], {"verdict": "n/a (dropped or missing)"})


if __name__ == "__main__":
    unittest.main()
