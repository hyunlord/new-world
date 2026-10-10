"""Hair-only masks preserve connected anatomy without recolouring other texels."""
import unittest

import numpy as np
from PIL import Image

from hair_maps import HairPalette, build_palette, classify

HAIR = (50, 30, 20)
SKIN = (180, 120, 80)


def standing() -> Image.Image:
    image = Image.new('RGBA', (9, 20))
    for y in range(20):
        for x in range(2, 7):
            image.putpixel((x, y), (*SKIN, 255))
    return image


class HairMapTests(unittest.TestCase):
    def setUp(self) -> None:
        self.palette = HairPalette('adult_b', frozenset((HAIR, SKIN)), frozenset((HAIR,)))

    def test_north_long_hair_reaches_back(self) -> None:
        image = standing()
        for y in range(12):
            image.putpixel((3, y), (*HAIR, 255))
        result = np.asarray(classify(image, self.palette))
        self.assertTrue(np.all(result[:12, 3, 0] == 3))
        self.assertEqual(int(result[12, 3, 0]), 0)

    def test_disconnected_hair_coloured_heel_is_excluded(self) -> None:
        image = standing()
        image.putpixel((3, 0), (*HAIR, 255))
        image.putpixel((3, 10), (*HAIR, 255))
        image.putpixel((3, 19), (*HAIR, 255))
        result = np.asarray(classify(image, self.palette))
        self.assertEqual(int(result[0, 3, 0]), 3)
        self.assertEqual(int(result[10, 3, 0]), 0)
        self.assertEqual(int(result[19, 3, 0]), 0)

    def test_highlight_inside_hair_is_filled(self) -> None:
        image = standing()
        for y in range(3):
            for x in range(2, 5):
                image.putpixel((x, y), (*HAIR, 255))
        image.putpixel((3, 1), (240, 220, 190, 255))
        self.assertEqual(classify(image, self.palette).getpixel((3, 1)), (3, 0, 0, 255))

    def test_asleep_lying_child_and_wide_frames_have_no_mask(self) -> None:
        image = Image.new('RGBA', (3, 8), (*HAIR, 255))
        child = HairPalette('child', self.palette.seen_rgb, self.palette.hair_rgb)
        for result in (classify(image, self.palette, 'asleep'),
                       classify(image, self.palette, 'lying'), classify(image, child),
                       classify(Image.new('RGBA', (8, 3), (*HAIR, 255)), self.palette)):
            self.assertFalse(np.asarray(result)[..., :3].any())

    def test_training_minimum_count_and_forty_percent_share(self) -> None:
        colors = (HAIR, SKIN, (80, 50, 30))
        image = Image.new('RGBA', (5, 3))
        reviewed = Image.new('RGBA', image.size)
        for y, color in enumerate(colors):
            for x in range(5 if y < 2 else 2):
                image.putpixel((x, y), (*color, 255))
                reviewed.putpixel((x, y), (3 if x < 2 else 4, 0, 0, 255))
        reviewed.putpixel((1, 1), (4, 0, 0, 255))
        palette = build_palette([(image, reviewed)], 'adult_a')
        self.assertEqual(palette.seen_rgb, frozenset(colors))
        self.assertEqual(palette.hair_rgb, frozenset((HAIR,)))

    def test_known_nonhair_does_not_use_unseen_colour_distance(self) -> None:
        known = (51, 30, 20)
        unseen = (50, 31, 20)
        palette = HairPalette('adult_a', frozenset((HAIR, known)), frozenset((HAIR,)))
        image = Image.new('RGBA', (3, 8), (*SKIN, 255))
        image.putpixel((0, 0), (*known, 255))
        image.putpixel((2, 0), (*unseen, 255))
        result = np.asarray(classify(image, palette))
        self.assertEqual(int(result[0, 0, 0]), 0)
        self.assertEqual(int(result[0, 2, 0]), 3)

    def test_tercile_shades_and_exact_source_alpha(self) -> None:
        colors = ((20, 20, 20), (80, 80, 80), (140, 140, 140))
        palette = HairPalette('adult_a', frozenset(colors), frozenset(colors))
        image = Image.new('RGBA', (3, 8), (*SKIN, 255))
        for x, color in enumerate(colors):
            image.putpixel((x, 0), (*color, 128 + x))
        image.putpixel((0, 7), (123, 45, 67, 0))
        result = np.asarray(classify(image, palette))
        self.assertEqual(result[0, :, 1].tolist(), [2, 1, 0])
        np.testing.assert_array_equal(result[..., 3], np.asarray(image)[..., 3])
        self.assertEqual(result[7, 0].tolist(), [0, 0, 0, 0])


if __name__ == '__main__':
    unittest.main()
