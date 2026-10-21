import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw

from .models import PipelineError
from .run_candidate import ROOT, fit_size, process_candidate


class CandidateTests(unittest.TestCase):
    def test_aspect_fit_and_no_enlargement(self) -> None:
        self.assertEqual(fit_size((100, 50), (32, 32)), (30, 15))
        self.assertEqual(fit_size((12, 20), (32, 32)), (12, 20))

    def test_candidate_roundtrip_does_not_imply_acceptance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            source = Image.new("RGBA", (64, 64), (255, 0, 255, 255))
            ImageDraw.Draw(source).ellipse((8, 8, 55, 55), fill=(154, 154, 154, 255))
            source.save(path / "source.png")
            result = process_candidate(path / "source.png", (32, 32), path / "candidate")
            self.assertTrue(result.check.passed, result.check.errors)
            self.assertFalse(result.accepted)
            self.assertEqual(result.crop, (8, 8, 56, 56))
            with Image.open(path / "candidate.png") as output:
                self.assertEqual(output.size, (32, 32))
            self.assertTrue((path / "candidate.json").is_file())

    def test_repository_intermediates_rejected(self) -> None:
        with self.assertRaises(PipelineError):
            _ = process_candidate(ROOT / "missing.png", (32, 32), ROOT / "art/trial/raw")


if __name__ == "__main__":
    _ = unittest.main()
