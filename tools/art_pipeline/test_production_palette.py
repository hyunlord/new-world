"""Numerical and artifact checks for the shared production palette."""
import json
import unittest
from pathlib import Path

import numpy as np
from PIL import Image

from build_production_palette import Registry, lab_to_rgb, transform
from palette_tools import rgb_to_lab

ROOT = Path(__file__).resolve().parents[2]


class ProductionPaletteTests(unittest.TestCase):
    def test_lab_inverse_round_trip(self) -> None:
        colors = np.random.default_rng(172).integers(0,256,(1000,3),dtype=np.uint8)
        np.testing.assert_array_equal(lab_to_rgb(rgb_to_lab(colors)),colors)

    def test_ground_transform_formula(self) -> None:
        color = (130,110,75)
        before = rgb_to_lab(np.array(color))
        after = rgb_to_lab(np.array(transform(color,-3,.72,4)))
        self.assertAlmostEqual(float(after[0]-before[0]),-3,delta=.3)
        self.assertAlmostEqual(float(np.linalg.norm(after[1:])/np.linalg.norm(before[1:])),.72,delta=.02)
        hue_delta = np.degrees(np.arctan2(after[2],after[1])-np.arctan2(before[2],before[1]))
        self.assertAlmostEqual(float(hue_delta),4,delta=1)

    def test_merge_preserves_roles_and_ids(self) -> None:
        registry = Registry()
        first = registry.add((100,100,100),'actor','a',0)
        second = registry.add((101,100,100),'skin','skin_0',1)
        self.assertEqual(first,second)
        self.assertEqual(registry.entries[0]['roles'],['actor','skin'])
        self.assertEqual(len(registry.entries[0]['ramp_positions']),2)

    def test_lookup_membership_and_order(self) -> None:
        data = json.loads((ROOT/'tools/art_pipeline/production_palette.json').read_text())
        rgb = data['rgb_ramps']['villager']
        expected = np.array([row for family in ('skin','hair','clothing') for row in rgb[family]],dtype=np.uint8)
        actual = np.asarray(Image.open(ROOT/'assets/art/villager_ramps.png'))
        self.assertEqual(actual.shape,(15,3,3))
        np.testing.assert_array_equal(actual,expected)
        palette = {tuple(entry['rgb']) for entry in data['colors']}
        self.assertTrue(all(tuple(color) in palette for color in actual.reshape(-1,3)))
        for rows in rgb.values():
            for ramp in rows:
                self.assertTrue(np.all(np.diff(rgb_to_lab(np.array(ramp))[:,0]) < 0))

    def test_every_ground_ramp_has_transform(self) -> None:
        data = json.loads((ROOT/'tools/art_pipeline/production_palette.json').read_text())
        self.assertEqual(set(data['ground_ramps']),{'grass','earth','forest_floor','hill','rock','camp_earth','shore','water'})
        for name,colors in data['source_ground_rgb'].items():
            expected = [list(transform(tuple(c),0,.85)) if name=='water' else list(transform(tuple(c),-3,.72,4)) for c in colors]
            self.assertEqual(data['unmerged_ground_rgb'][name],expected)


if __name__ == '__main__':
    unittest.main()
