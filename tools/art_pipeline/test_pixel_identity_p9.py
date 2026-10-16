"""Small synthetic images lock the identity rule, especially alpha and both bounds."""
from pathlib import Path

import unittest
from tempfile import TemporaryDirectory
import typer
from PIL import Image

from pixel_identity_p9 import Capture, Difference, accepts, compare, difference


class PixelIdentityTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.tmp_path = Path(temporary.name)

    def test_alpha_only_change_counts(self) -> None:
        a = Image.new("RGBA", (2, 2), (10, 20, 30, 255))
        b = a.copy()
        b.putpixel((0, 0), (10, 20, 30, 254))
        metric, _ = difference(a, b)
        assert metric == Difference(1, 1)


    def test_rgb_change_counts_with_unchanged_alpha(self) -> None:
        a = Image.new("RGBA", (2, 2), (10, 20, 30, 255))
        b = a.copy()
        b.putpixel((0, 0), (40, 20, 30, 255))
        metric, _ = difference(a, b)
        assert metric == Difference(1, 30)


    def test_both_bounds(self) -> None:
        cases = [
            (Difference(0, 0), Difference(0, 0), True),
            (Difference(0, 0), Difference(1, 1), False),
            (Difference(2, 5), Difference(2, 5), True),
            (Difference(2, 5), Difference(1, 6), False),
            (Difference(2, 5), Difference(3, 1), False),
        ]
        for control, candidate, expected in cases:
            with self.subTest(control=control, candidate=candidate):
                self.assertEqual(accepts(control, candidate), expected)


    def test_size_mismatch_is_not_a_pixel_pass(self) -> None:
        with self.assertRaises(typer.BadParameter):
            difference(Image.new("RGBA", (1, 1)), Image.new("RGBA", (2, 2)))


    def test_schedule_mismatch_rejected(self) -> None:
        tmp_path = self.tmp_path
        capture = Capture(tmp_path / "P1.png", 130, 10, 40560, 20261010, 60)
        wrong = Capture(capture.path, 131, 11, capture.tick, capture.seed, 60)
        with self.assertRaises(typer.BadParameter):
            compare((capture, capture, wrong), tmp_path)


    def test_compare_writes_visible_alpha_diff(self) -> None:
        tmp_path = self.tmp_path
        (tmp_path / "previews").mkdir()
        captures = []
        for name in ("A", "Aprime", "B"):
            folder = tmp_path / name
            folder.mkdir()
            image = Image.new("RGBA", (2, 2), (10, 20, 30, 255))
            if name == "B":
                image.putpixel((0, 0), (10, 20, 30, 254))
            image.save(folder / "P1.png")
            captures.append(Capture(folder / "P1.png", 130, 10, 40560, 20261010, 60))
        result = compare((captures[0], captures[1], captures[2]), tmp_path)
        assert not result.passed
        assert result.bit_reproducible
        with Image.open(tmp_path / "P1-candidate-diff.png") as diff:
            assert diff.getpixel((0, 0)) == 1


    def test_cli_requires_all_six_and_writes_complete_table(self) -> None:
        tmp_path = self.tmp_path
        import json

        from typer.testing import CliRunner

        from pixel_identity_p9 import FRAMINGS, main

        app = typer.Typer()
        app.command()(main)
        folders = [tmp_path / name for name in ("A", "Aprime", "B")]
        for folder in folders:
            folder.mkdir()
            for framing in FRAMINGS:
                Image.new("RGBA", (2, 2), (10, 20, 30, 255)).save(folder / f"{framing}.png")
                (folder / f"{framing}.json").write_text(json.dumps({
                    "capture_frame": 130, "state_change_frame": 10,
                    "tick": 1, "seed": 1, "fixed_fps": 60,
                }))
        output = tmp_path / "output"
        result = CliRunner().invoke(app, [str(folder) for folder in folders] + [str(output)])
        assert result.exit_code == 0, result.output
        report = json.loads((output / "comparison.json").read_text())
        assert report["passed"]
        assert len(report["framings"]) == 6
        assert len(list((output / "previews").glob("*.png"))) == 30
        (folders[2] / "P6.png").unlink()
        incomplete = CliRunner().invoke(app, [str(folder) for folder in folders] + [str(tmp_path / "incomplete")])
        assert incomplete.exit_code != 0


if __name__ == "__main__":
    unittest.main()
