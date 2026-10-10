"""Pixel and topology contracts for deterministic dual-grid terrain."""
import unittest
import numpy as np
from terrain_tiles import coverage, make_tile, read_ramps, LAYERS
from pathlib import Path

class TerrainTilesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.ramps = read_ramps(Path(__file__).with_name('production_palette.json'))

    def test_corner_bit_order(self) -> None:
        for bit, (x, y) in enumerate(((0, 0), (15, 0), (0, 15), (15, 15))):
            mask = coverage(1 << bit, 0)
            self.assertTrue(mask[y, x])
            self.assertEqual(sum(bool(mask[cy, cx]) for cx, cy in ((0, 0), (15, 0), (0, 15), (15, 15))), 1)

    def test_shared_edges_are_seamless(self) -> None:
        for left in range(16):
            for right in range(16):
                if bool(left & 2) == bool(right & 1) and bool(left & 8) == bool(right & 4):
                    self.assertTrue(np.array_equal(coverage(left, 0)[:, -1], coverage(right, 1)[:, 0]))
                if bool(left & 4) == bool(right & 1) and bool(left & 8) == bool(right & 2):
                    self.assertTrue(np.array_equal(coverage(left, 0)[-1, :], coverage(right, 1)[0, :]))

    def test_empty_full_and_binary_palette(self) -> None:
        allowed = {tuple(c) for ramp in self.ramps.values() for c in ramp}
        for layer in LAYERS:
            self.assertFalse(np.asarray(make_tile(layer, 0, 0, 0, self.ramps))[:, :, 3].any())
            for mask in range(1, 16):
                image = np.asarray(make_tile(layer, mask, 0, 0, self.ramps))
                self.assertTrue(set(np.unique(image[:, :, 3])).issubset({0, 255}))
                self.assertTrue(all(tuple(c) in allowed for c in image[:, :, :3][image[:, :, 3] > 0]))
            self.assertTrue((np.asarray(make_tile(layer, 15, 0, 0, self.ramps))[:, :, 3] == 255).all())

    def test_full_features_never_touch_border(self) -> None:
        for layer in LAYERS:
            for variant in range(8 if layer != 'water' else 4):
                a = np.asarray(make_tile(layer, 15, variant, 0, self.ramps))
                border = np.concatenate((a[0], a[-1], a[:, 0], a[:, -1]))
                self.assertEqual(len(np.unique(border, axis=0)), 1)

    def test_determinism_and_water_animation(self) -> None:
        for layer in LAYERS:
            self.assertEqual(make_tile(layer, 7, 1, 0, self.ramps).tobytes(), make_tile(layer, 7, 1, 0, self.ramps).tobytes())
        frames = {make_tile('water', 15, 0, frame, self.ramps).tobytes() for frame in range(4)}
        self.assertEqual(len(frames), 4)

if __name__ == '__main__':
    unittest.main()
