"""Connectivity tests exercise real composition with synthetic source pixels."""
import unittest

from PIL import Image

from art.pipeline2 import autotile


class AutotileTests(unittest.TestCase):
    def test_masks_have_exact_requested_edge_connections(self) -> None:
        # Given three visibly different generated-material stand-ins.
        sources = autotile.Sources(Image.new('RGBA', (64, 14), (80, 60, 40, 255)),
                                   Image.new('RGBA', (16, 16), (90, 70, 50, 255)),
                                   Image.new('RGBA', (16, 16), (100, 80, 60, 255)))
        for mask in range(16):
            with self.subTest(mask=mask):
                # When composing the actual recipe.
                image = autotile.compose(mask, sources)
                # Then dimensions and edge connectivity match the requested mask.
                self.assertEqual(image.size, (64, 64))
                observed = 0
                for bit, point in [(1, (32, 0)), (2, (63, 32)),
                                   (4, (32, 63)), (8, (0, 32))]:
                    pixel = image.getpixel(point)
                    assert isinstance(pixel, tuple), "Expected an RGBA pixel"
                    if pixel[3] > 0:
                        observed += bit
                self.assertEqual(observed, mask)

    def test_bad_mask_is_rejected(self) -> None:
        # Given an out-of-range mask, when requesting a recipe, then reject it.
        with self.assertRaises(ValueError):
            autotile.recipe(16)


if __name__ == '__main__':
    unittest.main()
