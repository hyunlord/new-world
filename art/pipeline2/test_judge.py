# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with scikit-image python -m unittest art.pipeline2.test_judge
"""Behavioral boundaries of the family-conditioned TRIAL2b numeric judge."""
import json
import tempfile
import unittest
from dataclasses import asdict, replace
from pathlib import Path

from PIL import Image

from art.pipeline2.judge import ModelScores, calibrate, judge, load_calibration, measure


class JudgeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.anchors = Path(__file__).resolve().parents[1] / "style05" / "anchors"
        cls.calibration = calibrate(cls.anchors)
        cls.candidate = cls.anchors / "crops" / "building_hut.png"

    def test_unknown_family_rejected(self) -> None:
        with self.assertRaises(ValueError):
            judge(self.candidate, self.calibration, family="unknown")

    def test_actual_leave_one_out(self) -> None:
        self.assertEqual(len(self.calibration.leave_one_out), len(self.calibration.anchors))
        self.assertTrue(all(f.passed for f in self.calibration.leave_one_out))
        for anchor in self.calibration.anchors:
            self.assertTrue(judge(self.anchors / anchor.file, self.calibration,
                                  family=anchor.family).numeric_pass)

    def test_other_family_cannot_change_verdict(self) -> None:
        reduced = replace(self.calibration, ranges=[r for r in self.calibration.ranges
                                                    if r.family == "buildings"])
        a = judge(self.candidate, self.calibration, family="buildings")
        b = judge(self.candidate, reduced, family="buildings")
        self.assertEqual(a.numeric_checks, b.numeric_checks)
        self.assertNotIn("lab_histogram", a.numeric_checks)

    def test_missing_and_legacy_model_fail_closed(self) -> None:
        scores = ModelScores(4, 4, 4, 4, "Synthetic test only.")
        for model in (None, scores):
            verdict = judge(self.candidate, self.calibration, model, family="buildings")
            self.assertTrue(verdict.numeric_pass)
            self.assertFalse(verdict.passed)
        self.assertTrue(judge(self.candidate, self.calibration, scores, family="buildings",
                              model_family_verified=True).passed)
        for field in ("medium", "line", "light", "palette"):
            failing = replace(scores, **{field: 3})
            self.assertFalse(judge(self.candidate, self.calibration, failing,
                                   family="buildings", model_family_verified=True).passed)

    def test_failed_loo_cannot_certify_candidate(self) -> None:
        broken = replace(self.calibration, leave_one_out=[replace(self.calibration.leave_one_out[0], passed=False)])
        verdict = judge(self.candidate, broken, ModelScores(5, 5, 5, 5, "Synthetic only."),
                        family="buildings", model_family_verified=True)
        self.assertTrue(verdict.numeric_pass)
        self.assertFalse(verdict.passed)

    def test_invalid_model_scores_rejected(self) -> None:
        for value in (0, 6, True, 4.5):
            with self.assertRaises(ValueError):
                ModelScores(value, 4, 4, 4, "Invalid synthetic test.")
        with self.assertRaises(ValueError):
            ModelScores(4, 4, 4, 4, " ")

    def test_blank_foreground_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "blank.png"
            for colour in ((255, 0, 255, 255), (0, 0, 0, 0)):
                Image.new("RGBA", (64, 64), colour).save(path)
                with self.assertRaises(ValueError):
                    measure(path)

    def test_calibration_roundtrip_and_schema_guard(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "calibration.json"
            path.write_text(json.dumps(asdict(self.calibration)))
            self.assertEqual(load_calibration(path), self.calibration)
            path.write_text(json.dumps({"version": 1, "comparison_size": 256}))
            with self.assertRaises(ValueError):
                load_calibration(path)

    def test_exact_boundary_and_one_feature_failure(self) -> None:
        from art.pipeline2.judge import FEATURES, FamilyRange, compare
        metrics = measure(self.candidate)
        values = [getattr(metrics, key) for key in FEATURES]
        self.assertTrue(all(compare(metrics, FamilyRange("buildings", values, values)).values()))
        lower = values.copy()
        lower[0] += 0.001
        checks = compare(metrics, FamilyRange("buildings", lower, values))
        self.assertFalse(checks["edge_density"])
        self.assertEqual(sum(checks.values()), len(FEATURES) - 1)

if __name__ == "__main__":
    unittest.main()
