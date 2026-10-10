"""Run with python3 -m unittest discover -s tools/art_pipeline."""
import unittest
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import numpy as np
from PIL import Image
from pixel_normalize import block_mode, key_out, normalize, run
from palette_tools import rgb_to_lab


class NormalizeTests(unittest.TestCase):
    def test_key_removes_rgb_and_hsv_spill(self) -> None:
        raw = np.array([[[255, 0, 255, 255], [180, 30, 170, 255], [20, 180, 30, 255]]], dtype=np.uint8)
        result = key_out(Image.fromarray(raw))
        self.assertEqual(np.asarray(result)[0, :, 3].tolist(), [0, 0, 255])

    def test_mode_is_not_average_and_requires_half_coverage(self) -> None:
        raw = np.zeros((2, 4, 4), dtype=np.uint8)
        raw[0, 0] = [201, 90, 20, 255]
        raw[1, 0] = [200, 91, 21, 255]
        raw[0, 1] = [20, 90, 200, 255]
        raw[0, 2] = [200, 90, 20, 255]
        result = block_mode(Image.fromarray(raw), (2, 1))
        self.assertEqual(np.asarray(result)[0, 0].tolist(), [204, 92, 20, 255])
        self.assertEqual(np.asarray(result)[0, 1, 3], 0)

    def test_outline_reserves_top_sides_and_touches_baseline(self) -> None:
        raw = Image.new("RGBA", (20, 40), (200, 100, 40, 255))
        palette = [(10, 10, 10), (200, 100, 40)]
        result = np.asarray(normalize(raw, (16, 24), palette))
        self.assertFalse(result[0, :, 3].any())
        self.assertFalse(result[:, 0, 3].any())
        self.assertFalse(result[:, -1, 3].any())
        self.assertTrue(result[-1, :, 3].any())
        self.assertTrue(set(map(tuple, result[result[..., 3] == 255, :3])).issubset(set(palette)))

    def test_outline_leaves_enclosed_hole_unchanged(self) -> None:
        raw = np.full((8, 8, 4), [200, 100, 40, 255], dtype=np.uint8)
        raw[3:5, 3:5] = [0, 0, 0, 0]
        result = np.asarray(normalize(Image.fromarray(raw), (12, 11), [(10, 10, 10), (200, 100, 40)]))
        self.assertEqual(result[5:7, 5:7, 3].tolist(), [[0, 0], [0, 0]])

    def test_opaque_49_percent_discarded_50_percent_retained(self) -> None:
        for count, alpha in [(49, 0), (50, 255)]:
            with self.subTest(coverage=count):
                raw = np.zeros((10, 10, 4), dtype=np.uint8)
                raw.reshape(-1, 4)[:count] = [200, 100, 40, 255]
                self.assertEqual(np.asarray(block_mode(Image.fromarray(raw), (1, 1)))[0, 0, 3], alpha)

    def test_non_divisible_grid_keeps_all_pixels(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            raw = np.full((15, 31, 3), [255, 0, 255], dtype=np.uint8)
            raw[3:12, 3:12] = [200, 100, 40]
            raw[3:12, 18:28] = [200, 100, 40]
            Image.fromarray(raw).save(root / "sheet.png")
            (root / "palette.hex").write_text("#0A0A0A\n#C86428\n")
            (root / "manifest.json").write_text(json.dumps({"columns": 2, "rows": 1, "cells": [
                {"index": i, "name": f"cell_{i}", "width": 16, "height": 16} for i in (1, 2)]}))
            self.assertTrue(run(root / "sheet.png", root / "manifest.json", root / "palette.hex", root / "out"))
            report = json.loads((root / "out/normalize_report.json").read_text())
            self.assertEqual([r["source_opaque"] for r in report], [81, 90])

    def test_erased_thin_tail_still_bottom_aligns_remaining_shape(self) -> None:
        raw = np.full((104, 104, 3), [255, 0, 255], dtype=np.uint8)
        raw[2:92, 2:102] = [200, 100, 40]
        raw[92:102, 51:52] = [200, 100, 40]
        result = np.asarray(normalize(Image.fromarray(raw), (16, 24), [(10, 10, 10), (200, 100, 40)]))
        self.assertTrue(result[-1, :, 3].any())

    def test_lab_known_white(self) -> None:
        np.testing.assert_allclose(rgb_to_lab(np.array([255, 255, 255])), [100, 0, 0], atol=0.001)

    def test_empty_cell_rejected(self) -> None:
        with self.assertRaises(ValueError):
            normalize(Image.new("RGBA", (8, 8), (255, 0, 255, 255)), (16, 16), [(0, 0, 0)])


if __name__ == "__main__":
    unittest.main()
