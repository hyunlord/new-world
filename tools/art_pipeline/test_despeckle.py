"""Rare isolated texel correction and rejection-cap regression tests."""
import unittest

from PIL import Image

from despeckle import despeckle


class DespeckleTests(unittest.TestCase):
    def test_stray_blue_texel_replaced_without_changing_source(self) -> None:
        # Given a single blue texel surrounded by abundant brown.
        image = Image.new('RGBA', (10, 10), (100, 60, 20, 255))
        image.putpixel((5, 5), (0, 0, 255, 255))
        original = image.tobytes()
        # When applying the native-image filter.
        result = despeckle(image)
        # Then only the isolated colour changes, under the rejection cap.
        self.assertEqual(result.image.getpixel((5, 5)), (100, 60, 20, 255))
        self.assertEqual((result.changed_count, result.rejected), (1, False))
        self.assertEqual(image.tobytes(), original)

    def test_intentional_single_texel_eye_has_near_dark_neighbor(self) -> None:
        image = Image.new('RGBA', (10, 10), (150, 100, 70, 255))
        image.putpixel((5, 5), (20, 15, 10, 255))
        image.putpixel((4, 4), (25, 20, 15, 255))
        result = despeckle(image)
        self.assertEqual(result.changed_count, 0)
        self.assertEqual(result.image.tobytes(), image.tobytes())

    def test_more_than_three_percent_rejected_but_evidence_retained(self) -> None:
        image = Image.new('RGBA', (5, 5), (100, 60, 20, 255))
        image.putpixel((2, 2), (0, 0, 255, 255))
        result = despeckle(image)
        self.assertTrue(result.rejected)
        self.assertEqual(result.change_fraction, 0.04)
        self.assertEqual(result.image.getpixel((2, 2)), (100, 60, 20, 255))

    def test_exactly_three_percent_is_not_rejected(self) -> None:
        image = Image.new('RGBA', (10, 10), (100, 60, 20, 255))
        for point, color in (((2, 2), (0, 0, 255, 255)), ((5, 5), (0, 255, 0, 255)), ((8, 8), (255, 0, 255, 255))):
            image.putpixel(point, color)
        result = despeckle(image)
        self.assertEqual((result.changed_count, result.rejected), (3, False))

    def test_transparent_majority_removes_isolated_edge(self) -> None:
        image = Image.new('RGBA', (10, 10))
        image.putpixel((0, 0), (0, 0, 255, 255))
        result = despeckle(image)
        self.assertEqual(result.image.getpixel((0, 0)), (0, 0, 0, 0))
        self.assertTrue(result.rejected)

    def test_three_occurrences_are_not_rare(self) -> None:
        image = Image.new('RGBA', (10, 10), (100, 60, 20, 255))
        for point in ((2, 2), (5, 5), (8, 8)):
            image.putpixel(point, (0, 0, 255, 255))
        result = despeckle(image)
        self.assertEqual(result.changed_count, 0)

    def test_empty_image_is_unchanged(self) -> None:
        result = despeckle(Image.new('RGBA', (2, 2)))
        self.assertEqual((result.changed_count, result.change_fraction, result.rejected), (0, 0.0, False))


if __name__ == '__main__':
    unittest.main()
