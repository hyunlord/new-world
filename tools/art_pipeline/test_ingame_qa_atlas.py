"""Atlas and captured clearing checks reject recolours and boundary intrusions."""
from pathlib import Path
from tempfile import TemporaryDirectory
import json
import unittest
from unittest.mock import patch
from ingame_qa_atlas import Frame, distinct_activities, opaque_difference, forest

class AtlasChecks(unittest.TestCase):
    def test_opaque_difference_requires_geometry(self) -> None:
        a = frozenset({(0,0),(0,1),(1,1)})
        self.assertEqual(opaque_difference(a,a),0)
        self.assertEqual(opaque_difference(a,frozenset({(0,0),(0,1)})),1/3)

    def test_activity_check_requires_every_body_and_rejects_baskets(self) -> None:
        idle = frozenset({(0,0),(0,1),(1,1)})
        pose = frozenset({(0,0),(0,1)})
        frames = [Frame(body+'/'+activity,body,activity,'S',idle if activity=='idle' else pose,0)
                  for body in ['adult_a','adult_b','child']
                  for activity in ['idle','walk','carry','gather','build','eat','drink','sit','talk']]
        with patch('ingame_qa_atlas.activity_frames',return_value=frames):
            self.assertEqual(distinct_activities(Path('.')).status,'PASS')
        with patch('ingame_qa_atlas.activity_frames',return_value=frames[:-1]):
            self.assertEqual(distinct_activities(Path('.')).status,'FAIL')
        copied = [Frame(f.ident,f.body,f.activity,f.view,idle,0) for f in frames]
        with patch('ingame_qa_atlas.activity_frames',return_value=copied):
            self.assertEqual(distinct_activities(Path('.')).status,'FAIL')
        frames.append(Frame('basket_idle','adult_a','idle','N',idle,6))
        with patch('ingame_qa_atlas.activity_frames',return_value=frames):
            self.assertEqual(distinct_activities(Path('.')).status,'FAIL')

    def test_forest_density_and_clearing_boundary(self) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            empty = {'xs':[],'ys':[]}
            data = {'tick':1680,'terrain':{'width':4,'height':1,'terrain':[1,1,1,1]},
                    'building':{'xs':[5],'ys':[0]},'construction':empty,
                    'settlement':{'formation_xs':[],'formation_ys':[]},'resource':{'xs':[],'ys':[],'kinds':[]},
                    'entities':[{'kind':'tree','id':1,'tile_x':0,'tile_y':0,'shown':True}]}
            path = root/'forest_evidence.json'
            path.write_text(json.dumps(data))
            self.assertEqual(forest(root).status,'PASS')
            data['building']['xs'] = [4]
            path.write_text(json.dumps(data))
            self.assertEqual(forest(root).status,'FAIL')
            data['building']['xs'] = [9]
            data['entities'].append({'kind':'tree','id':2,'tile_x':1,'tile_y':0,'shown':True})
            path.write_text(json.dumps(data))
            self.assertEqual(forest(root).status,'FAIL')

if __name__ == '__main__':
    unittest.main()
