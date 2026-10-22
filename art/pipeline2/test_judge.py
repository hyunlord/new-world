# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: python -m unittest discover -s art/pipeline2 -p test_judge.py
"""Behavioral boundaries for the art-only judge."""
import json
import subprocess
import sys
import tempfile
import unittest
from dataclasses import asdict, replace
from pathlib import Path

from PIL import Image, ImageDraw

from art.pipeline2.judge import ModelScores, calibrate, chi_square, judge, measure


class JudgeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.anchors = Path(__file__).resolve().parents[1] / "style05" / "anchors"
        cls.calibration = calibrate(cls.anchors)

    def test_chi_square_known_extremes(self) -> None:
        self.assertEqual(chi_square([1.0, 0.0], [1.0, 0.0]), 0.0)
        self.assertAlmostEqual(chi_square([1.0, 0.0], [0.0, 1.0]), 1.0)

    def test_every_anchor_passes_numeric_and_loo_threshold(self) -> None:
        self.assertEqual(len(self.calibration.loo_distances), 6)
        for anchor in self.calibration.anchors:
            verdict = judge(self.anchors / f"{anchor.name}.png", self.calibration)
            self.assertTrue(verdict.numeric_pass, (anchor.name, verdict.numeric_checks))
        self.assertEqual(max(self.calibration.loo_distances), self.calibration.histogram_limit)

    def test_missing_model_fails_closed_even_for_anchor(self) -> None:
        verdict = judge(self.anchors / "person.png", self.calibration)
        self.assertTrue(verdict.numeric_pass)
        self.assertFalse(verdict.passed)
        self.assertEqual(verdict.model_status, "MISSING")

    def test_model_all_dimensions_must_pass(self) -> None:
        for dimension in ("medium", "line", "light", "palette"):
            scores = replace(ModelScores(4, 4, 4, 4, "Synthetic unit test only."), **{dimension: 3})
            self.assertFalse(judge(self.anchors / "person.png", self.calibration, scores).passed)
        self.assertTrue(judge(self.anchors / "person.png", self.calibration,
                              ModelScores(4, 4, 4, 4, "Synthetic unit test only.")).passed)

    def test_invalid_model_scores_rejected(self) -> None:
        # Given invalid external JSON scores, including a non-integer and a blank reason.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            calibration = root / "calibration.json"
            _ = calibration.write_text(json.dumps(asdict(self.calibration)))
            scores = root / "scores.json"
            output = root / "verdict.json"
            for score, sentence in ((0, "Invalid"), (6, "Invalid"), (True, "Invalid"),
                                    (4.5, "Invalid"), (4, " ")):
                with self.subTest(score=score, sentence=sentence):
                    _ = scores.write_text(json.dumps({"medium": score, "line": 4,
                        "light": 4, "palette": 4, "sentence": sentence}))
                    # When the actual CLI reads the model response.
                    result = subprocess.run(
                        [sys.executable, str(Path(__file__).with_name("judge.py")), "judge",
                         str(self.anchors / "person.png"), "--calibration", str(calibration),
                         "--model-scores", str(scores), "--output", str(output)],
                        capture_output=True, text=True, check=False,
                    )
                    # Then it rejects before publishing a verdict.
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("ValueError", result.stderr)
                    self.assertFalse(output.exists())

    def test_exact_numeric_boundaries(self) -> None:
        path = self.anchors / "person.png"
        metrics = measure(path, self.calibration.palette_lab)
        exact = replace(self.calibration, histogram=metrics.histogram, histogram_limit=0,
                        edge_range=(metrics.edge_density, metrics.edge_density),
                        wash_range=(metrics.wash, metrics.wash))
        self.assertTrue(judge(path, exact).numeric_pass)
        outside = replace(exact, edge_range=(metrics.edge_density + 0.001, 1.0))
        self.assertFalse(judge(path, outside).numeric_checks["line_density"])
        outside = replace(exact, wash_range=(metrics.wash + 0.001, 1.0))
        self.assertFalse(judge(path, outside).numeric_checks["local_saturation"])

    def test_blank_foreground_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "blank.png"
            for colour in ((255, 0, 255, 255), (20, 20, 20, 0)):
                Image.new("RGBA", (128, 128), colour).save(path)
                with self.assertRaises(ValueError):
                    measure(path)

    def test_normalization_is_canvas_and_resolution_stable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            small = Path(directory) / "small.png"
            large = Path(directory) / "large.png"
            image = Image.new("RGBA", (256, 256), (190, 170, 140, 255))
            draw = ImageDraw.Draw(image)
            for x in range(16, 256, 24):
                draw.line((x, 0, x, 255), fill=(40, 35, 30, 255), width=4)
            image.save(small)
            image.resize((1024, 1024), Image.Resampling.NEAREST).save(large)
            a, b = measure(small), measure(large)
            self.assertLess(abs(a.edge_density - b.edge_density), 0.01)
            self.assertLess(abs(a.wash - b.wash), 0.01)
            self.assertLess(chi_square(a.histogram, b.histogram), 0.02)
            self.assertGreater(a.stroke_width_at_1024, 0)


if __name__ == "__main__":
    unittest.main()
