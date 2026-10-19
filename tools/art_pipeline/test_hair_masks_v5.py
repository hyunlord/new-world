"""Position templates must isolate hair from similarly coloured props."""
import unittest
import numpy as np
from PIL import Image
import hair_masks_v5 as masks


class PositionMaskTests(unittest.TestCase):
    def fixture(self):
        rgba = np.zeros((24, 20, 4), dtype=np.uint8)
        rgba[2:22, 5:13] = (160, 130, 100, 255)
        rgba[2:13, 5:9] = (72, 45, 25, 255)
        template = np.zeros_like(rgba)
        template[..., 3] = rgba[..., 3]
        template[2:13, 5:9, 0] = 3
        image = Image.fromarray(rgba.copy())
        ref = masks.Reference('r', 'adult_b', 'S', 'idle', image, Image.fromarray(template))
        return rgba, ref

    def result(self, rgba, ref):
        image = Image.fromarray(rgba)
        return masks.remask(image, image, 'adult_b', 'S', 'idle', [ref])

    def test_hair_coloured_wood_outside_template_is_excluded(self):
        rgba, ref = self.fixture()
        rgba[16:20, 14:17] = (72, 45, 25, 255)
        result = self.result(rgba, ref)
        self.assertIsNotNone(result.mask)
        self.assertFalse(np.any(np.asarray(result.mask)[16:20, 14:17, 0] == 3))

    def test_long_hair_inside_template_is_covered(self):
        rgba, ref = self.fixture()
        result = self.result(rgba, ref)
        self.assertTrue(np.all(np.asarray(result.mask)[2:13, 5:9, 0] == 3))

    def test_dark_inner_patch_inside_template_is_filled(self):
        rgba, ref = self.fixture()
        rgba[7, 7, :3] = (9, 8, 7)
        result = self.result(rgba, ref)
        self.assertEqual(np.asarray(result.mask)[7, 7, 0], 3)

    def test_outline_ring_is_never_hair(self):
        rgba, ref = self.fixture()
        native = rgba.copy()
        native[2, 5:9] = 0
        result = masks.remask(Image.fromarray(rgba), Image.fromarray(native), 'adult_b', 'S', 'idle', [ref])
        self.assertFalse(np.any(np.asarray(result.mask)[2, 5:9, 0] == 3))

    def test_low_iou_returns_no_mask(self):
        rgba, ref = self.fixture()
        rgba[:] = 0
        rgba[2:22, 0:2] = (72, 45, 25, 255)
        result = self.result(rgba, ref)
        self.assertIsNone(result.mask)
        self.assertLess(result.iou, .60)

    def test_same_input_same_mask(self):
        rgba, ref = self.fixture()
        self.assertEqual(self.result(rgba, ref).mask.tobytes(), self.result(rgba, ref).mask.tobytes())


if __name__ == '__main__':
    unittest.main()
