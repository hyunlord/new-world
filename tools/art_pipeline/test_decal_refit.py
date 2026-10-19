# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_decal_refit.py
import unittest
import numpy as np
from decal_refit import fit_decal, native_limit
from test_normalize_v3 import enlarged


class DecalRefitTest(unittest.TestCase):
    def test_leaf_limit_is_six_and_other_decals_eight(self) -> None:
        self.assertEqual(native_limit('leaf_1'), 6)
        self.assertEqual(native_limit('flower_white'), 8)
        self.assertEqual(native_limit('lily'), 8)

    def test_raw_grid_is_refitted_below_limit_with_binary_palette_pixels(self) -> None:
        palette = [(72, 64, 44), (131, 110, 80), (212, 182, 142)]
        native = np.asarray(palette, dtype=np.uint8)[np.random.default_rng(91).integers(0, 3, (23, 17))]
        raw = enlarged(native, 5.9, 1.7)
        original = raw.tobytes()
        fit = fit_decal(raw, palette, 6)
        self.assertLessEqual(max(fit.native.size), 6)
        self.assertEqual(fit.metrics['palette_match_percent'], 100)
        self.assertTrue(fit.metrics['binary_alpha'])
        self.assertLessEqual(fit.metrics['factor'], 1)
        self.assertEqual(raw.tobytes(), original)


if __name__ == '__main__':
    unittest.main()
