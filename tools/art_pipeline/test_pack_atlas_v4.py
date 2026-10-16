"""Atlas payload preserves fallback provenance and intentional suppression."""
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from PIL import Image
from pack_atlas import pack


class PackAtlasV4Tests(unittest.TestCase):
    def test_pack_derived_and_suppressed_without_losing_metadata(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'assets/art').mkdir(parents=True)
            Image.new('RGBA',(3,4),(12,30,40,255)).save(root/'assets/art/frame.png')
            entry = {'sprite_id':'V_a1_01','path':'assets/art/frame.png','anchor':[1,3],
                     'qa_status':'derived','source':'derived:mirror','source_sprite':'V_a1_02',
                     'body':'adult_a','view':'E','activity':'idle','frame':0,
                     'contains_basket':False,'fallback_kind':'mirror','activity_icon':'','drawable':True}
            suppressed = {'N3_01':{'drawable':False,'qa_status':'suppressed',
                                  'fallback_kind':'unaccepted_decorative'}}
            (root/'assets/art/manifest.json').write_text(json.dumps({'assets':[entry], 'suppressed':suppressed}))
            atlas = pack(root)
            self.assertEqual(atlas['suppressed'], suppressed)
            packed = atlas['sprites']['V_a1_01']
            for field in ('source','qa_status','source_sprite','contains_basket','fallback_kind','activity_icon','drawable'):
                self.assertEqual(packed[field], entry[field])
            self.assertNotIn('N3_01', atlas['sprites'])
            with Image.open(root/packed['page'].removeprefix('res://')) as image:
                self.assertEqual(image.getpixel(tuple(packed['rect'][:2])), (12,30,40,255))

    def test_explicit_false_beats_legacy_accepted(self) -> None:
        with TemporaryDirectory() as directory:
            root=Path(directory); (root/'assets/art').mkdir(parents=True)
            row={'sprite_id':'N3_01','path':'absent.png','qa_status':'accepted','drawable':False}
            (root/'assets/art/manifest.json').write_text(json.dumps({'assets':[row]}))
            atlas=pack(root)
            self.assertNotIn('N3_01',atlas['sprites'])
            self.assertFalse(atlas['suppressed']['N3_01']['drawable'])
            self.assertEqual(atlas['suppressed']['N3_01']['qa_status'],'suppressed')


if __name__ == '__main__':
    unittest.main()
