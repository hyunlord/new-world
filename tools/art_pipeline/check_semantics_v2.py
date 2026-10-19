"""Semantic regression fixtures exercise success and actual failure boundaries."""
from __future__ import annotations

import unittest

from PIL import Image

from palette_v2 import PaletteMetadata, build_master, quantize
from semantics_v2 import evaluate

RED = (180, 30, 35)
GREEN = (40, 110, 45)
SKIN = (190, 140, 100)
FIRE = (245, 130, 30)
PALETTE = [RED, GREEN, SKIN, FIRE]
META: PaletteMetadata = {'scene_count': 4, 'sprite_median_cut_count': 4, 'final_count': 4,
                        'added_colors': [], 'accent_ramps': {'skin': ['#BE8C64']},
                        'missing_accents': [], 'classification': 'Fixture skin ramp'}


def fixture() -> dict[str, Image.Image]:
    images = {'01_campfire': Image.new('RGBA', (5, 5), FIRE+(255,)),
              '09_villager_gatherer': Image.new('RGBA', (6, 10), SKIN+(255,)),
              '10_villager_builder': Image.new('RGBA', (6, 11), SKIN+(255,)),
              '11_villager_elder': Image.new('RGBA', (6, 12), SKIN+(255,)),
              '12_villager_child': Image.new('RGBA', (4, 7), SKIN+(255,)),
              '16_berry_basket': Image.new('RGBA', (3, 3), GREEN+(255,))}
    for index, (stage, red_count) in enumerate([('full', 8), ('two', 4), ('one', 1), ('bare', 0)], 5):
        image = Image.new('RGBA', (6, 6), GREEN+(255,))
        for pixel in range(red_count):
            image.putpixel((pixel % 6, pixel // 6), RED+(255,))
        images[f'{index:02}_berry_bush_{stage}'] = image
    return images


class SemanticTests(unittest.TestCase):
    def test_all_semantics_pass_on_preserved_fixture(self) -> None:
        images = fixture()
        self.assertTrue(evaluate(images, images, PALETTE, META)['passed'])

    def test_berry_order_and_bare_are_required(self) -> None:
        images = fixture()
        images['08_berry_bush_bare'].putpixel((0, 0), RED+(255,))
        self.assertFalse(evaluate(images, images, PALETTE, META)['checks']['berry_stages'])
        images = fixture()
        images['06_berry_bush_two'] = images['05_berry_bush_full'].copy()
        self.assertFalse(evaluate(images, images, PALETTE, META)['checks']['berry_stages'])

    def test_fire_requires_twenty_texels(self) -> None:
        images = fixture()
        images['01_campfire'] = Image.new('RGBA', (19, 1), FIRE+(255,))
        self.assertFalse(evaluate(images, images, PALETTE, META)['checks']['fire'])
        images['01_campfire'] = Image.new('RGBA', (20, 1), FIRE+(255,))
        self.assertTrue(evaluate(images, images, PALETTE, META)['checks']['fire'])

    def test_skin_is_required_for_every_adult(self) -> None:
        images = fixture()
        images['09_villager_gatherer'] = Image.new('RGBA', (10, 10), GREEN+(255,))
        for x in range(11):
            images['09_villager_gatherer'].putpixel((x % 10, x // 10), SKIN+(255,))
        self.assertFalse(evaluate(images, images, PALETTE, META)['checks']['skin'])
        images['09_villager_gatherer'].putpixel((1, 1), SKIN+(255,))
        self.assertTrue(evaluate(images, images, PALETTE, META)['checks']['skin'])

    def test_child_and_basket_cannot_fill_adult_object_box(self) -> None:
        images = fixture()
        images['12_villager_child'] = images['09_villager_gatherer'].copy()
        images['16_berry_basket'] = images['05_berry_bush_full'].copy()
        report = evaluate(images, images, PALETTE, META)
        self.assertFalse(report['checks']['child_smaller'])
        self.assertFalse(report['checks']['basket_smaller'])

    def test_color_damage_fails_fidelity(self) -> None:
        images = fixture()
        changed = {name: image.copy() for name, image in images.items()}
        changed['01_campfire'] = Image.new('RGBA', (5, 5), GREEN+(255,))
        self.assertFalse(evaluate(images, changed, PALETTE, META)['checks']['fidelity'])

    def test_partial_alpha_and_foreign_color_fail_format(self) -> None:
        images = fixture()
        images['01_campfire'].putpixel((0, 0), (1, 2, 3, 128))
        checks = evaluate(images, images, PALETTE, META)['checks']
        self.assertFalse(checks['binary_alpha'])
        self.assertFalse(checks['palette_membership'])

    def test_missing_accent_is_reported_as_failure(self) -> None:
        images = fixture()
        metadata: PaletteMetadata = {**META, 'missing_accents': ['fire_hot_white']}
        self.assertFalse(evaluate(images, images, PALETTE, metadata)['checks']['accent_ramps'])

    def test_palette_additions_are_native_and_quantization_keeps_shape(self) -> None:
        images = fixture()
        palette, metadata = build_master([GREEN], images)
        native_colors = {image.getpixel((x, y))[:3] for image in images.values()
                         for x in range(image.width) for y in range(image.height)}
        for addition in metadata['added_colors']:
            color = tuple(int(addition['color'][i:i+2], 16) for i in (1, 3, 5))
            self.assertIn(color, native_colors)
        image = quantize(images['01_campfire'], palette)
        self.assertEqual(image.size, images['01_campfire'].size)
        self.assertEqual(image.getchannel('A').tobytes(), images['01_campfire'].getchannel('A').tobytes())


if __name__ == '__main__':
    unittest.main()
