"""Evidence boundaries for the Art Pass 3 measurements."""
from dataclasses import replace
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
import numpy as np
from PIL import Image
import ingame_qa
from ingame_qa_pixels import Check
from test_ingame_qa import capture


class ScopeTests(unittest.TestCase):
    def test_plain_calibration_is_recorded_but_excluded_from_gameplay(self) -> None:
        with TemporaryDirectory() as temp:
            root = Path(temp)
            records = []
            for name in ['a_d2_1200_F2', 'a_lineup_z4', 'l_d24_1200_F2']:
                pixels = np.tile(np.arange(256, dtype=np.uint8), (256, 1))
                if 'lineup' in name:
                    pixels[:] = 0
                path = root / (name + '.png')
                Image.fromarray(np.repeat(pixels[..., None], 3, axis=2)).save(path)
                records.append(capture(path, ()))
            check = ingame_qa.degeneracy(records)
            self.assertEqual(check.status, 'PASS')
            detail = json.loads(check.detail)
            self.assertEqual(detail['old_all_frames']['status'], 'FAIL')
            self.assertEqual(detail['gameplay_only']['count'], 2)
            self.assertEqual(detail['excluded'], ['a_lineup_z4'])

    def test_degenerate_long_gameplay_still_fails(self) -> None:
        with TemporaryDirectory() as temp:
            path = Path(temp) / 'l_d24_1200_F2.png'
            Image.new('RGB', (8, 8)).save(path)
            self.assertEqual(ingame_qa.degeneracy([capture(path, ())]).status, 'FAIL')

    def test_both_contrast_frames_are_required(self) -> None:
        noon = capture(Path('a_d2_1200_F2.png'), ())
        winter = replace(noon, stem='l_d24_1200_F2')
        with patch('ingame_qa.contrast', side_effect=[Check('A5','PASS','median=.2'),Check('A5','FAIL','min=.09')]):
            check = ingame_qa.contrast_pair(noon, winter, Path('.'))
        self.assertEqual(check.status, 'FAIL')
        self.assertIn('min=.09', check.detail)
        self.assertEqual(ingame_qa.contrast_pair(None, None, Path('.')).status, 'N/A')

class CampTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root/'assets/art/atlas').mkdir(parents=True)
        Image.new('RGBA',(8,8),(1,2,3,255)).save(self.root/'native.png')
        (self.root/'assets/art/atlas/world_atlas.json').write_text(json.dumps({'sprites':{'N3_01':{}}}))
        (self.root/'assets/art/manifest.json').write_text(json.dumps({'assets':[{'sprite_id':'N3_01','native_size':[8,8],'native_path':'native.png'}]}))
        self.data = {'tick':1680,'terrain':{'width':100,'height':4,'terrain':[0]*100+[1]*100+[2]*100+[3]*100},
            'building':{'xs':[90],'ys':[3]},'construction':{'xs':[],'ys':[]},'resource':{'xs':[],'ys':[],'kinds':[]},
            'settlement':{'formation_xs':[],'formation_ys':[]}, 'entities':[]}
        for row,count,kind in [(0,4,'decal'),(1,3,'decal'),(2,45,'rock'),(3,6,'rock')]:
            for x in range(count):
                self.data['entities'].append({'id':row*100+x,'kind':kind,'tile_x':x,'tile_y':row,'shown':True})

    def tearDown(self) -> None:
        self.temp.cleanup()

    def measure(self) -> Check:
        import ingame_qa_camp_p3
        (self.root/'forest_evidence.json').write_text(json.dumps(self.data))
        return ingame_qa_camp_p3.camp_ground(self.root,self.root)

    def test_full_terrain_denominators_pass(self) -> None:
        self.assertEqual(self.measure().status,'PASS')

    def test_offscreen_rock_inside_clearing_fails(self) -> None:
        self.data['entities'].append({'id':999,'kind':'rock','tile_x':89,'tile_y':3,'shown':True,'visible':False})
        self.assertEqual(self.measure().status,'FAIL')

    def test_decal_radius_three_fails(self) -> None:
        self.data['entities'][0]['tile_x'] = 87
        self.assertEqual(self.measure().status,'FAIL')

    def test_duplicate_rock_tile_fails(self) -> None:
        self.data['entities'].append(dict(self.data['entities'][-1]))
        self.assertEqual(self.measure().status,'FAIL')

    def test_actual_native_png_exceeding_metadata_fails(self) -> None:
        Image.new('RGBA',(9,8)).save(self.root/'native.png')
        self.assertEqual(self.measure().status,'FAIL')


if __name__ == '__main__':
    unittest.main()
