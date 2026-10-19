"""Colour-region classification contract regressions."""
import json
import unittest

import numpy as np
from PIL import Image

from regions_v2 import build_table, classify, table_from_data, table_to_data, RegionTableError


class RegionV2Tests(unittest.TestCase):
    def setUp(self) -> None:
        self.reference = Image.new('RGBA', (3, 3))
        self.approved = Image.new('RGBA', (3, 3))
        for x, (color, region) in enumerate((((30, 20, 10), 3), ((210, 150, 100), 2), ((240, 220, 160), 4))):
            self.reference.putpixel((x, 0), (*color, 255))
            self.approved.putpixel((x, 0), (region, 0, 0, 255))
        self.table = build_table(self.reference, self.approved, 'adult_a')

    def test_bending_and_kneeling_keep_reference_regions(self) -> None:
        # Given reference colours moved below the legacy hair cap.
        for row in (1, 2):
            pose = Image.new('RGBA', (3, 3))
            for x in range(3):
                pose.putpixel((x, row), self.reference.getpixel((x, 0)))
            # When classifying either pose.
            encoded = classify(pose, self.table)
            # Then anatomical meaning follows colour rather than height.
            self.assertEqual([encoded.getpixel((x, row))[0] for x in range(3)], [3, 2, 4])

    def test_unique_distant_color_remains_other(self) -> None:
        image = Image.new('RGBA', (1, 1), (0, 0, 255, 255))
        encoded = classify(image, self.table)
        self.assertEqual(encoded.getpixel((0, 0))[0], 5)

    def test_near_color_uses_reference_region(self) -> None:
        image = Image.new('RGBA', (1, 1), (211, 150, 100, 255))
        encoded = classify(image, self.table)
        self.assertEqual(encoded.getpixel((0, 0))[0], 2)

    def test_fallback_only_unknown_standing_hair_rows(self) -> None:
        image = Image.new('RGBA', (1, 4), (0, 0, 50, 255))
        posed, standing = classify(image, self.table), classify(image, self.table, standing=True)
        self.assertEqual([posed.getpixel((0, y))[0] for y in range(4)], [5, 5, 5, 5])
        self.assertEqual([standing.getpixel((0, y))[0] for y in range(4)], [3, 5, 5, 5])

    def test_ambiguity_counts_and_tie_survive_json_roundtrip(self) -> None:
        reference = Image.new('RGBA', (2, 1), (100, 90, 80, 255))
        approved = Image.new('RGBA', (2, 1), (4, 0, 0, 255))
        approved.putpixel((0, 0), (2, 0, 0, 255))
        table = build_table(reference, approved, 'child')
        restored = table_from_data(json.loads(json.dumps(table_to_data(table))))
        self.assertEqual(restored, table)
        self.assertEqual(restored.colors[0].region, 2)
        self.assertEqual(restored.colors[0].counts, (0, 0, 1, 0, 1, 0))

    def test_alpha_and_shade_preserve_encoding(self) -> None:
        reference = Image.new('RGBA', (3, 1))
        approved = Image.new('RGBA', (3, 1))
        for x, value in enumerate((40, 80, 120)):
            reference.putpixel((x, 0), (value, value, value, 255))
            approved.putpixel((x, 0), (3, 0, 0, 255))
        table = build_table(reference, approved, 'adult_b')
        result = classify(reference, table)
        self.assertEqual([result.getpixel((x, 0))[1] for x in range(3)], [2, 1, 0])
        np.testing.assert_array_equal(np.asarray(result)[..., 3], np.asarray(reference)[..., 3])

    def test_bad_table_majority_is_rejected(self) -> None:
        data = table_to_data(self.table)
        data['colors'][0]['region'] = 4
        with self.assertRaises(RegionTableError):
            table_from_data(data)


if __name__ == '__main__':
    unittest.main()
