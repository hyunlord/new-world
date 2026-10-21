"""Ground seam and crop contracts; synthetic fixtures never become art."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import numpy as np
from PIL import Image

from . import ground
from .models import PipelineError


class GroundTests(unittest.TestCase):
    def test_uniform_ground_passes(self) -> None:
        # Given a constant opaque texture, when measured, then no seam exists.
        image = Image.new('RGBA', (32, 32), (45, 55, 35, 255))
        result = ground.measure_seams(image)
        self.assertTrue(result.passed)
        self.assertEqual(result.x.edge_mean, 0)

    def test_ramp_wrap_fails_even_with_smooth_interior(self) -> None:
        # Given a smooth ramp, when wrapped, then the discontinuity is rejected.
        values = np.zeros((32, 32, 3), dtype=np.uint8)
        values[:] = np.arange(32, dtype=np.uint8)[None, :, None] * 7
        result = ground.measure_seams(Image.fromarray(values))
        self.assertFalse(result.passed)
        self.assertGreater(result.x.edge_mean, 6)

    def test_repeat_preserves_pixels(self) -> None:
        # Given real source pixels, when repeated, then every copy is exact.
        image = Image.new('RGBA', (2, 3), (12, 34, 56, 255))
        image.putpixel((1, 2), (90, 80, 70, 255))
        result = ground.repeat_preview(image)
        self.assertEqual(result.size, (8, 12))
        self.assertEqual(result.crop((6, 9, 8, 12)).tobytes(), image.tobytes())

    def test_prescribed_crops_keep_exact_coordinates(self) -> None:
        # Given a 256 texture, when cut, then positions cannot be cherry-picked.
        image = Image.new('RGBA', (256, 256), (12, 34, 56, 255))
        image.putpixel((64, 64), (90, 80, 70, 255))
        crops = ground.cut_tiles(image, 4)
        self.assertEqual(crops[1].getpixel((0, 0)), (90, 80, 70, 255))
        self.assertEqual(len(crops), 4)

    def test_bad_texture_size_rejected(self) -> None:
        # Given an undersized source, when cropped, then silent padding is forbidden.
        with self.assertRaises(PipelineError):
            _ = ground.cut_tiles(Image.new('RGBA', (32, 32)), 2)

    def test_source_processing_preserves_full_opaque_square(self) -> None:
        # Given a solid ground file, when processed, then no object outline or trim is added.
        with TemporaryDirectory() as temporary:
            source = Path(temporary) / 'source.png'
            Image.new('RGBA', (256, 256), (48, 62, 40, 255)).save(source)
            result = ground.process_ground(source, Path(temporary) / 'output', 2)
            with Image.open(result.image) as image:
                self.assertEqual(image.size, (256, 256))
                self.assertEqual(image.getpixel((0, 0)), (48, 62, 40, 255))
            self.assertFalse(result.distinct_tiles)


if __name__ == '__main__':
    _ = unittest.main()
