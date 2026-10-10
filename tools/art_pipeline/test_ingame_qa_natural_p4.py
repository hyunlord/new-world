"""Natural shader regression fixtures with explicit source and expected texels."""
from dataclasses import replace
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

import numpy as np
from PIL import Image

from ingame_qa import variety
from ingame_qa_types import load_capture, sprite_pixels
from test_ingame_qa import capture, entity


class NaturalColourTests(unittest.TestCase):
    def test_tone_rounding_dark_outline_and_alpha(self) -> None:
        with TemporaryDirectory() as folder:
            root = Path(folder)
            Image.fromarray(np.array([[[25,25,25,255],[26,26,26,127],[105,205,255,255]]], dtype=np.uint8)).save(root/'atlas.png')
            Image.new('RGBA',(3,1),(0,0,0,255)).save(root/'atlas_regions.png')
            (root/'assets/art').mkdir(parents=True)
            Image.new('RGB',(3,15),(240,230,220)).save(root/'assets/art/villager_ramps.png')
            actor = replace(entity(),texture='res://atlas.png',region=(0,0,3,1),variant=(0,4))
            actual = sprite_pixels(actor,root,variants=True)
            np.testing.assert_array_equal(actual,[[[25,25,25,255],[23,23,23,127],[95,185,230,255]]])
            bright = sprite_pixels(replace(actor,variant=(2,4)),root,variants=True)
            np.testing.assert_array_equal(bright,[[[25,25,25,255],[28,28,28,127],[113,221,255,255]]])

    def test_hair_uses_shade_not_tone_and_keeps_alpha(self) -> None:
        with TemporaryDirectory() as folder:
            root = Path(folder)
            Image.new('RGBA',(2,1),(10,10,10,81)).save(root/'atlas.png')
            Image.fromarray(np.array([[[3,0,0,255],[3,2,0,255]]],dtype=np.uint8)).save(root/'atlas_regions.png')
            (root/'assets/art').mkdir(parents=True)
            ramps = Image.new('RGB',(3,15))
            ramps.putpixel((0,8),(250,240,230))
            ramps.putpixel((2,8),(150,140,130))
            ramps.save(root/'assets/art/villager_ramps.png')
            actor = replace(entity(),texture='res://atlas.png',region=(0,0,2,1),variant=(0,4))
            np.testing.assert_array_equal(sprite_pixels(actor,root,variants=True),[[[250,240,230,81],[150,140,130,81]]])

    def test_metadata_keeps_old_values_without_inventing_tone(self) -> None:
        with TemporaryDirectory() as folder:
            path = Path(folder)/'frame.png'
            path.with_suffix('.entities.json').write_text(json.dumps({'zoom':[4,4],'entities':[
                {'kind':'agent','rect':[0,0,1,1],'variant':{'tone':2,'hair':4}},
                {'kind':'agent','rect':[0,0,1,1],'variant':{'skin':3,'hair':1,'clothing':5}}]}))
            actors = load_capture(path).entities
            self.assertEqual(actors[0].variant,(2,4))
            self.assertIsNone(actors[0].legacy_variant)
            self.assertEqual(actors[1].variant,(-1,1))
            self.assertEqual(actors[1].legacy_variant,(3,1,5))

    def test_child_zero_map_keeps_own_colour_at_neutral_tone(self) -> None:
        with TemporaryDirectory() as folder:
            root = Path(folder)
            Image.new('RGBA',(1,1),(120,80,60,255)).save(root/'atlas.png')
            Image.new('RGBA',(1,1),(0,0,0,0)).save(root/'atlas_regions.png')
            (root/'assets/art').mkdir(parents=True)
            Image.new('RGB',(3,15),(255,255,255)).save(root/'assets/art/villager_ramps.png')
            actor = replace(entity(),body='child',texture='res://atlas.png',region=(0,0,1,1),variant=(1,4))
            np.testing.assert_array_equal(sprite_pixels(actor,root,variants=True),[[[120,80,60,255]]])

    def test_old_capture_diagnostic_does_not_pass_new_tuple(self) -> None:
        actors = tuple(replace(entity(),ident=i,variant=(-1,i%5),legacy_variant=(i%4,i%5,i%6),tile=(i,0)) for i in range(12))
        checked = variety(capture(Path('unused'),actors))
        self.assertEqual(checked.status,'FAIL')
        self.assertIn('A12-old (diagnostic): 12 tuples',checked.detail)
        self.assertIn('pass=True',checked.detail)

    def test_child_hair_cannot_manufacture_visible_variety(self) -> None:
        actors = tuple(replace(entity(),ident=i,body='child',variant=(1,i),tile=(i,0)) for i in range(3))
        checked = variety(capture(Path('unused'),actors))
        self.assertEqual(checked.status,'FAIL')
        self.assertIn('1 tuples/3',checked.detail)

    def test_legacy_emulation_is_explicit_and_does_not_change_new_tuple(self) -> None:
        actors = tuple(replace(entity(),ident=i,variant=(1,0),tile=(i,0)) for i in range(12))
        checked = variety(capture(Path('unused'),actors))
        self.assertEqual(checked.status,'FAIL')
        self.assertIn('1 tuples/12',checked.detail)
        self.assertIn('12 emulated from frozen P3',checked.detail)
        self.assertIn('not captured skin/clothing metadata',checked.detail)


if __name__ == '__main__':
    unittest.main()
