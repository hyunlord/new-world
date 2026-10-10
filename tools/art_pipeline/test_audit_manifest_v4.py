# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
"""Manifest audit exercises only disposable synthetic assets."""
import json
from pathlib import Path
import tempfile
import unittest
from PIL import Image
from audit_manifest_v4 import audit_manifest

class ManifestAuditTest(unittest.TestCase):
    def test_registration_is_explicit_and_restricted(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);atlas=root/'assets/art/atlas';atlas.mkdir(parents=True)
            for name in ('world_0','world_0_regions','unknown'):
                Image.new('RGBA',(4,4),(1,2,3,255)).save(atlas/f'{name}.png')
            (atlas/'region_atlas.json').write_text(json.dumps({'pages':{'res://assets/art/atlas/world_0.png':'res://assets/art/atlas/world_0_regions.png'}}))
            (atlas/'world_atlas.json').write_text(json.dumps({'sprites':{},'suppressed':{}}))
            manifest={'assets':[{'path':'assets/art/atlas/world_0.png','native_size':[4,4]}]}
            updated,report=audit_manifest(root,manifest)
            self.assertEqual(len(report['unregistered_pngs']),2)
            self.assertEqual(len(manifest['assets']),1)
            updated,report=audit_manifest(root,manifest,register_region_pages=True)
            self.assertEqual(report['unregistered_pngs'],['assets/art/atlas/unknown.png'])
            self.assertEqual(len(updated['assets']),2)
            self.assertFalse(report['passed'])
            again,second=audit_manifest(root,updated,register_region_pages=True)
            self.assertEqual(len(again['assets']),2)
            self.assertEqual(second['registered_region_pages'],[])
            clean={**again,'assets':again['assets']+[{'path':'assets/art/atlas/unknown.png','native_size':[4,4]}]}
            _,good=audit_manifest(root,clean)
            self.assertTrue(good['passed'])
            self.assertEqual(good['png_count'],3)
            (atlas/'region_atlas.json').write_text(json.dumps({'pages':{'x':'res://assets/art/atlas/unknown.png'}}))
            with self.assertRaises(ValueError):audit_manifest(root,manifest,register_region_pages=True)

    def test_duplicates_missing_alpha_dimensions_and_suppression(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);art=root/'assets/art';(art/'atlas').mkdir(parents=True)
            image=Image.new('RGBA',(3,5),(1,2,3,128));image.save(art/'a.png')
            (art/'atlas/world_atlas.json').write_text(json.dumps({'sprites':{'hidden':{'drawable':True}},'suppressed':{'hidden':{'drawable':False}}}))
            manifest={'assets':[{'path':'assets/art/a.png','canvas_size':[2,5],'native_size':[1,3],'sprite_id':'hidden','drawable':False},
                {'path':'assets/art/./a.png','native_size':[3,5]}, {'path':'assets/art/missing.png','native_size':[1,1]}]}
            updated,report=audit_manifest(root,manifest)
            self.assertEqual(report['duplicate_paths'],{'assets/art/a.png':2})
            self.assertEqual(report['missing_pngs'],['assets/art/missing.png'])
            self.assertEqual(report['nonbinary_alpha'],['assets/art/a.png'])
            self.assertEqual(report['suppressed_drawable'],['hidden'])
            self.assertEqual(updated['assets'][0]['canvas_size'],[2,5])
            repaired,report=audit_manifest(root,manifest,record_dimensions=True)
            self.assertEqual(repaired['assets'][0]['canvas_size'],[3,5])
            self.assertEqual(repaired['assets'][0]['native_size'],[1,3])
            self.assertFalse(report['passed'])
            self.assertEqual(repaired['audits']['manifest_v4'],report)

if __name__=='__main__':unittest.main()
