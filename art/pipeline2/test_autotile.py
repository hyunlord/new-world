"""Connectivity tests exercise real composition with synthetic source pixels."""
import unittest
from collections import deque
from pathlib import Path

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
            _ = autotile.recipe(16)

    def test_v2_real_assets_are_single_connected_components_with_exact_edges(self) -> None:
        root = Path(__file__).resolve().parents[1] / "style05" / "autotile_v2"
        for mask in range(16):
            with self.subTest(mask=mask), Image.open(root / f"wall_{mask:02d}.png") as tile:
                self.assertEqual(tile.size, (64, 64))
                occupied: set[tuple[int, int]] = set()
                for y in range(64):
                    for x in range(64):
                        pixel = tile.getpixel((x, y))
                        self.assertIsInstance(pixel, tuple)
                        if isinstance(pixel, tuple) and pixel[3] > 128:
                            occupied.add((x, y))
                self.assertTrue(occupied)
                pending = deque([occupied.pop()])
                component = set(pending)
                while pending:
                    x, y = pending.popleft()
                    for point in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                        if point in occupied:
                            occupied.remove(point)
                            component.add(point)
                            pending.append(point)
                self.assertFalse(occupied, "Disconnected generated wall geometry")
                observed = (int(any(y == 0 for _, y in component))
                            + 2 * int(any(x == 63 for x, _ in component))
                            + 4 * int(any(y == 63 for _, y in component))
                            + 8 * int(any(x == 0 for x, _ in component)))
                self.assertEqual(observed, mask)


if __name__ == '__main__':
    _ = unittest.main()
