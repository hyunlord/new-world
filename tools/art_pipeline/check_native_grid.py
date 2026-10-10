"""Native recovery regression tests; no generated image calls required."""
from __future__ import annotations

import unittest

import numpy as np
from PIL import Image

from native_grid import key_mask, recover_cell, remove_spill
from pixel_normalize import outline


class NativeGridTests(unittest.TestCase):
    def test_fractional_grid_and_phase_recovers_exact_native_colors_and_size(self) -> None:
        rng = np.random.default_rng(15)
        colors = np.array([[30, 80, 40], [180, 50, 30], [230, 170, 90], [50, 50, 90]], dtype=np.uint8)
        native = colors[rng.integers(0, 4, (23, 17))]
        for phase, periods in (((7.3, 8.6), (5.9, 5.9)), ((9.7, 11.2), (5.9, 5.9)),
                               ((6.1, 3.4), (5.9, 5.9)), ((7.3, 8.6), (5.9, 7.2))):
            with self.subTest(phase=phase, periods=periods):
                y, x = np.indices((190, 130))
                nx = np.floor((x + 0.5 - phase[0]) / periods[0]).astype(int)
                ny = np.floor((y + 0.5 - phase[1]) / periods[1]).astype(int)
                raw = np.full((190, 130, 3), [255, 0, 255], dtype=np.uint8)
                foreground = (nx >= 0) & (nx < 17) & (ny >= 0) & (ny < 23)
                raw[foreground] = native[ny[foreground], nx[foreground]]
                recovered, metadata = recover_cell(Image.fromarray(raw))
                self.assertEqual(recovered.size, (17, 23))
                np.testing.assert_array_equal(np.asarray(recovered)[..., :3], native)
                self.assertEqual(metadata["detected_period_xy"], list(periods))
                self.assertEqual(metadata["axis_periods_separate"], periods[0] != periods[1])
                self.assertTrue(np.all(np.asarray(recovered)[..., 3] == 255))

    def test_key_rule_preserves_red_and_respects_existing_transparency(self) -> None:
        rgba = np.array([[[255, 0, 255, 255], [220, 30, 40, 255], [40, 80, 40, 0]]], dtype=np.uint8)
        self.assertEqual(key_mask(rgba).tolist(), [[False, True, False]])

    def test_native_spill_removed_and_boundary_cast_uses_four_neighbors(self) -> None:
        rgba = np.full((3, 3, 4), [100, 120, 60, 255], dtype=np.uint8)
        rgba[0, 0] = [220, 30, 220, 255]
        # Hue outside the hard-key hue band, but red/blue both exceed green by40.
        rgba[0, 2] = [200, 145, 220, 255]
        cleaned, metadata = remove_spill(Image.fromarray(rgba))
        self.assertEqual(np.asarray(cleaned)[0, 0, 3], 0)
        np.testing.assert_array_equal(np.asarray(cleaned)[0, 2], [100, 120, 60, 255])
        self.assertEqual(metadata, {"spill_removed": 1, "fringe_repaired": 1})

    def test_outline_pads_without_changing_native_texels_or_enclosed_holes(self) -> None:
        rgba = np.full((5, 5, 4), [180, 60, 30, 255], dtype=np.uint8)
        rgba[2, 2] = 0
        result = np.asarray(outline(Image.fromarray(rgba), (10, 10, 10)))
        self.assertEqual(result.shape, (7, 7, 4))
        np.testing.assert_array_equal(result[1:-1, 1:-1], rgba)
        self.assertTrue(np.all(result[0, :, 3] == 255))

    def test_empty_cell_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            recover_cell(Image.new("RGBA", (20, 20), (255, 0, 255, 255)))


if __name__ == "__main__":
    unittest.main()
