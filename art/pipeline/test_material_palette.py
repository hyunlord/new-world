"""Material remapping preserves source geometry and every other material."""
import unittest

import numpy as np
from PIL import Image

from .material_palette import MaterialSubset, reduce_material
from .models import PipelineError


class MaterialPaletteTests(unittest.TestCase):
    def test_frequency_selection_and_other_material_preservation(self) -> None:
        greens = ((48,62,40), (55,83,43), (58,74,46), (75,91,55), (113,147,64))
        pixels = np.array([(*color,255) for count,color in enumerate(greens,1)
                           for _ in range(count)] + [(90,58,34,255),(20,18,16,255),(99,88,77,0)],dtype=np.uint8).reshape(3,6,4)
        image = Image.fromarray(pixels)
        result = reduce_material(image, MaterialSubset('foliage',greens))
        self.assertEqual(len(result.retained),4)
        self.assertEqual(result.changed_pixels,1)
        np.testing.assert_array_equal(np.array(result.image)[:, :,3],pixels[:,:,3])
        np.testing.assert_array_equal(np.array(result.image).reshape(-1,4)[-3:],pixels.reshape(-1,4)[-3:])
        self.assertNotIn(greens[0],result.retained)

    def test_ties_are_lexicographic_and_idempotent(self) -> None:
        colors=((10,20,10),(20,30,20),(30,40,30),(40,50,40),(50,60,50))
        image=Image.fromarray(np.array([[(*color,255) for color in colors]],dtype=np.uint8))
        subset=MaterialSubset('foliage',colors)
        first=reduce_material(image,subset)
        self.assertEqual(first.retained,colors[:4])
        self.assertEqual(reduce_material(first.image,subset).image.tobytes(),first.image.tobytes())

    def test_explicit_retained_existing_material_colors(self) -> None:
        colors=((20,20,20),(60,60,60),(100,100,100),(150,150,150),(220,220,220))
        image=Image.fromarray(np.array([[(*color,255) for color in colors]],dtype=np.uint8))
        retained=(colors[0],colors[1],colors[3],colors[4])
        result=reduce_material(image,MaterialSubset('wood',colors,retained=retained))
        self.assertEqual(result.retained,retained)
        self.assertEqual(result.changed_pixels,1)
        with self.assertRaises(PipelineError):
            _ = reduce_material(image,MaterialSubset('wood',colors,retained=((255,0,0),)))

    def test_region_keeps_same_rgb_in_other_material_unchanged(self) -> None:
        image=Image.new('RGBA',(3,1),(10,20,10,255))
        image.putpixel((0,0),(30,40,30,255))
        region=np.array([[True,True,False]],dtype=np.bool_)
        result=reduce_material(image,MaterialSubset('foliage',((10,20,10),(30,40,30)),1),region)
        self.assertEqual(result.image.getpixel((2,0)),image.getpixel((2,0)))
        with self.assertRaises(PipelineError):
            _ = reduce_material(image,MaterialSubset('foliage',((10,20,10),)),np.zeros((2,2),dtype=np.bool_))

    def test_rejects_empty_membership_and_missing_material(self) -> None:
        with self.assertRaises(PipelineError):
            _ = reduce_material(Image.new('RGBA',(2,2)),MaterialSubset('foliage',()))
        with self.assertRaises(PipelineError):
            _ = reduce_material(Image.new('RGBA',(2,2)),MaterialSubset('foliage',((1,2,3),)))

if __name__ == '__main__':
    _ = unittest.main()
