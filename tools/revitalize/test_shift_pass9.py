"""Run: python3 -m unittest discover -s tools/revitalize -p test_shift_pass9.py."""
import json
import tempfile
import unittest
from pathlib import Path

import test_evaluate_pass6 as p6
import test_evaluate_pass8 as p8t
from shift_pass9 import bootstrap, main, median, x3_statistics


class ShiftTests(unittest.TestCase):
    def test_paired_bootstrap_and_summary_level(self) -> None:
        values = {s: (float(s), float(s) + 1) for s in range(20)}
        r = bootstrap(values, median)
        self.assertEqual((r["shift"], r["lo"], r["hi"]), (1.0, 1.0, 1.0))
        self.assertTrue(r["excludes_zero"])
        same = bootstrap({s: (float(s), float(s)) for s in range(20)}, median)
        self.assertEqual((same["shift"], same["excludes_zero"]), (0.0, False))
        pooled = bootstrap({s: (3.0, 4.0) for s in range(20)}, median, one_seed=True)
        self.assertTrue(pooled["summary_level"])
        self.assertIsNone(pooled["lo"])
        self.assertFalse(pooled["excludes_zero"])
        constant = bootstrap({s: (3.0, 4.0) for s in range(20)}, median, one_seed=False)
        self.assertFalse(constant["summary_level"])

    def test_x3_statistics(self) -> None:
        winters = [{"seed": 0, "nights_out": n, "first_night_out": f, "severity": s}
                   for n, f, s in ((0, 0, 0), (2, 5, 1), (1, 7, 1), (3, 4, 2))]
        stats = x3_statistics(winters)
        self.assertEqual(stats["c_no_dark_night_share"], .25)
        self.assertEqual(stats["d_two_or_more_share"], .5)
        self.assertEqual(stats["e_distinct_first_nights"], 3.0)
        self.assertAlmostEqual(stats["e_seventh_share"], 1 / 3)
        self.assertEqual(stats["b_mode_share_normal"], .5)

    def test_identical_versions_shift_nothing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, seeds, days, clocks in (("a10", 8, 10, False), ("a120", 4, 120, True)):
                p8t.write_round(root / name, seeds, days, clocks)
            (root / "v11.json").write_text(json.dumps({"measures": p6.V11}))
            args = ["--v14-10", str(root / "a10"), "--v14-120", str(root / "a120"), "--v15-10", str(root / "a10"),
                    "--v15-120", str(root / "a120"), "--v11-targets", str(root / "v11.json"), "--out", str(root / "out")]
            self.assertEqual(main(args), 0)
            report = json.loads((root / "out" / "shifts.json").read_text())
            self.assertEqual(report["flagged"], [])
            self.assertTrue(all(r["shift"] in (0, None) for r in report["x3"].values()))
            self.assertTrue(all(r["shift"] == 0 for r in report["medians"].values()))
            self.assertIn("Y8_winter_food", report["medians"])


if __name__ == "__main__":
    unittest.main()
