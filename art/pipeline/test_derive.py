"""Deterministic derivative invariants; temporary fixtures are not trial assets."""
import unittest

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .derive import bank_tiles, canopy_shadow, hut_ghost
from .models import PipelineError
from .outline import silhouette_boundary


class DerivativeTests(unittest.TestCase):
    def test_ghost_dashed_boundary_preserves_interior(self) -> None:
        source = Image.new('RGBA', (16, 16))
        pixels: NDArray[np.uint8] = np.array(source)
        pixels[2:14, 2:14] = (120, 92, 60, 255)
        source = Image.fromarray(pixels)
        ghost: NDArray[np.uint8] = np.array(hut_ghost(source))
        mask = pixels[:, :, 3] > 0
        boundary = silhouette_boundary(mask)
        self.assertTrue(np.any(np.equal(ghost[boundary, 3], 0)))
        self.assertTrue(np.any(np.equal(ghost[boundary, 3], 80)))
        moat = silhouette_boundary(mask & ~boundary)
        self.assertTrue(np.all(np.equal(ghost[moat, 3], 0)))
        self.assertTrue(np.all(np.equal(ghost[mask & ~boundary & ~moat, 3], 80)))
        self.assertTrue(np.all(np.equal(ghost[ghost[:, :, 3] > 0, :3], (111, 176, 232))))
        self.assertTrue(np.all(np.equal(ghost[~mask], 0)))

    def test_shadow_has_sixty_percent_ellipse_area_and_clean_alpha(self) -> None:
        image = Image.new('RGBA', (100, 100), (113, 147, 64, 255))
        shadow: NDArray[np.uint8] = np.array(canopy_shadow(image))
        area = int(np.count_nonzero(shadow[:, :, 3] > 0))
        self.assertLess(abs(area / (np.pi * 50 * 50) - 0.6), 0.02)
        self.assertEqual(set(np.unique(shadow[:, :, 3])), {0, 90})
        self.assertTrue(np.all(np.equal(shadow[shadow[:, :, 3] > 0, :3], (20, 18, 16))))
        with self.assertRaises(PipelineError):
            _ = canopy_shadow(Image.new('RGBA', (32, 32)))

    def test_eight_banks_are_rotational_and_preserve_water_rgb(self) -> None:
        water = Image.new('RGBA', (32, 32), (38, 60, 76, 255))
        sand = (165, 143, 76)
        tiles = bank_tiles(water, sand).images
        self.assertEqual(len(tiles), 8)
        for image in tiles.values():
            self.assertEqual(image.size, (32, 32))
            colors = np.unique(np.array(image).reshape(-1, 4), axis=0)
            np.testing.assert_array_equal(colors, np.array([(38, 60, 76, 255), (*sand, 255)]))
        np.testing.assert_array_equal(np.rot90(np.array(tiles['bank_n'])), np.array(tiles['bank_w']))
        np.testing.assert_array_equal(np.rot90(np.array(tiles['bank_in_nw'])), np.array(tiles['bank_in_sw']))
        with self.assertRaises(PipelineError):
            _ = bank_tiles(Image.new('RGBA', (16, 16)), sand)

if __name__ == '__main__':
    _ = unittest.main()
